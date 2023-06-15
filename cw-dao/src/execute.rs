#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::DaoContract;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{
    Candidate, Config, Propsal, PropsalStatus, Vote, BLOCK_HEIGHTS, BLOCK_INDEX, CONFIG, PROPSALS,
    PROPSAL_INDEX,
};

const CONTRACT_NAME: &str = "nft-dao";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        start_blocknumber: env.block.height,
        max_duration_seconds: msg.max_duration_seconds,
        owner: deps.api.addr_validate(msg.owner.as_str())?,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    PROPSAL_INDEX.save(deps.storage, &1u64)?;
    BLOCK_HEIGHTS.save(deps.storage, 0u64, &env.block.height)?;
    BLOCK_INDEX.save(deps.storage, &1u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("sender", info.sender)
        .add_attribute("owner", config.owner)
        .add_attribute(
            "max_duration_seconds",
            config.max_duration_seconds.to_string(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _: MigrateMsg) -> Result<Response, ContractError> {
    let ver = cw2::get_contract_version(deps.storage)?;

    if ver.contract != CONTRACT_NAME {
        return Err(ContractError::InvalidContractName {});
    }
    #[allow(clippy::cmp_owned)]
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(ContractError::InvalidContractVersion {});
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("name", CONTRACT_NAME)
        .add_attribute("version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            max_duration_seconds,
        } => update_config(deps, env, info, owner, max_duration_seconds),
        ExecuteMsg::AddPropsal {
            title,
            status,
            nft_address,
            expiration,
            candidates,
        } => add_propsal(
            deps,
            env,
            info,
            title,
            status,
            nft_address,
            expiration,
            candidates,
        ),
        ExecuteMsg::UpdatePropsal {
            id,
            title,
            status,
            nft_address,
            expiration,
            candidates,
        } => update_propsal(
            deps,
            env,
            info,
            id,
            title,
            status,
            nft_address,
            expiration,
            candidates,
        ),
        ExecuteMsg::RemovePropsal { id } => remove_propsal(deps, env, info, id),
        ExecuteMsg::ExecuteVote {
            propsal_id,
            candidate_id,
        } => execute_vote(deps, env, info, propsal_id, candidate_id),
    }
}

/// 컨트랙트 설정 업데이트
fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    max_duration_seconds: Option<u64>,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    let mut config: Config = CONFIG.load(deps.storage)?;

    if let Some(owner) = owner.clone() {
        config.owner = deps.api.addr_validate(&owner)?;
    }
    if let Some(max_duration_seconds) = max_duration_seconds.clone() {
        config.max_duration_seconds = max_duration_seconds;
    }

    CONFIG.save(deps.storage, &config)?;
    DaoContract::increase_block(deps, &env)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("sender", info.sender)
        .add_attribute("owner", config.owner)
        .add_attribute(
            "max_duration_seconds",
            config.max_duration_seconds.to_string(),
        ))
}

/// 투표 제안 추가
fn add_propsal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    status: PropsalStatus,
    nft_address: String,
    expiration: u64,
    candidates: Vec<Candidate>,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let valid_nft_address = deps.api.addr_validate(&nft_address)?;

    let id = PROPSAL_INDEX.load(deps.storage)?;
    let propsal = Propsal {
        id,
        title,
        status,
        nft_address: valid_nft_address,
        expiration,
        votes: vec![],
        candidates,
    };

    let config = CONFIG.load(deps.storage)?;
    if expiration < env.block.time.seconds() {
        return Err(ContractError::LowerTime {});
    }
    if (expiration - env.block.time.seconds()) > config.max_duration_seconds {
        return Err(ContractError::MaxDuration {});
    }

    PROPSALS.save(deps.storage, id, &propsal)?;
    DaoContract::increase_block(deps, &env)?;

    Ok(Response::new()
        .add_attribute("action", "add_propsal")
        .add_attribute("sender", info.sender)
        .add_attribute("id", propsal.id.to_string())
        .add_attribute("title", propsal.title)
        .add_attribute("status", format!("{:?}", &propsal.status))
        .add_attribute("nft_address", nft_address)
        .add_attribute("expiration", expiration.to_string())
        .add_attribute("candidates", format!("{:?}", &propsal.candidates)))
}

/// 투표 제안 업데이트
fn update_propsal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    title: Option<String>,
    status: Option<PropsalStatus>,
    nft_address: Option<String>,
    expiration: Option<u64>,
    candidates: Option<Vec<Candidate>>,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let mut propsal: Propsal = PROPSALS.load(deps.storage, id)?;
    if let Some(title) = title.clone() {
        propsal.title = title.clone();
    }
    if let Some(status) = status.clone() {
        propsal.status = status.clone();
    }
    if let Some(nft_address) = nft_address.clone() {
        let valid_nft_address = deps.api.addr_validate(&nft_address)?;
        propsal.nft_address = valid_nft_address.clone();
    }

    if let Some(expiration) = expiration.clone() {
        // 만료시간 유효성 검사
        let config = CONFIG.load(deps.storage)?;
        if expiration < env.block.time.seconds() {
            return Err(ContractError::LowerTime {});
        }
        if (expiration - env.block.time.seconds()) > config.max_duration_seconds {
            return Err(ContractError::MaxDuration {});
        }
        propsal.expiration = expiration.clone();
    }

    if let Some(candidates) = candidates.clone() {
        propsal.candidates = candidates.clone();
    }

    PROPSALS.save(deps.storage, id.clone(), &propsal)?;
    DaoContract::increase_block(deps, &env)?;

    Ok(Response::new()
        .add_attribute("action", "update_propsal")
        .add_attribute("sender", info.sender)
        .add_attribute("id", propsal.id.to_string())
        .add_attribute("title", propsal.title)
        .add_attribute("status", format!("{:?}", &propsal.status))
        .add_attribute("nft_address", propsal.nft_address.clone().to_string())
        .add_attribute("expiration", propsal.expiration.to_string())
        .add_attribute("candidates", format!("{:?}", &propsal.candidates)))
}

/// 투표 제안 삭제
fn remove_propsal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    PROPSALS.remove(deps.storage, id.clone());
    DaoContract::increase_block(deps, &env)?;

    Ok(Response::new()
        .add_attribute("action", "remove_propsal")
        .add_attribute("sender", info.sender)
        .add_attribute("id", id.to_string()))
}

/// 투표 실행
fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    propsal_id: u64,
    candidate_id: u64,
) -> Result<Response, ContractError> {
    let mut propsal = PROPSALS.load(deps.storage, propsal_id)?;

    let vote = Vote {
        /// 1 NFT = 1 power
        power: 1,
        voter: info.clone().sender,
        propsal_id,
        candidate_id,
    };

    propsal.votes.append(&mut vec![vote.clone()]);

    PROPSALS.save(deps.storage, propsal_id.clone(), &propsal)?;
    DaoContract::increase_block(deps, &env)?;

    Ok(Response::new()
        .add_attribute("action", "execute_vote")
        .add_attribute("sender", info.clone().sender)
        .add_attribute("propsal_id", vote.clone().propsal_id.to_string())
        .add_attribute("candidate_id", format!("{:?}", &propsal.candidates)))
}

/// 오너 체크
fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}
