#![cfg(test)]

use std::vec;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Addr, Api, DepsMut, Env, MessageInfo, Response};

use crate::execute::{execute, instantiate};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PropsalResponse, PropsalResultResponse,
    PropsalsResponse, QueryMsg,
};
use crate::query::query;
use crate::state::{Candidate, Config, Propsal, PropsalStatus, Vote};
use crate::ContractError;

fn setup_contract(
    deps: DepsMut,
    mut env: Env,
    info: MessageInfo,
) -> Result<(Response, Env), ContractError> {
    let msg = InstantiateMsg {
        owner: info.sender.to_string(),
        max_duration_seconds: 2629743,
    };
    let res = instantiate(deps, env.clone(), info, msg)?;
    env.block.height += 1;

    Ok((res, env))
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let owner = "owner";
    let info = mock_info(&owner, &[]);
    let (_, env) = setup_contract(deps.as_mut(), env.clone(), info).unwrap();

    let max_duration_seconds = 2629743;

    let query_config = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let response_config: ConfigResponse = from_binary(&query_config).unwrap();
    let config = response_config.config;

    assert_eq!(owner, config.owner);
    assert_eq!(max_duration_seconds, config.max_duration_seconds);
}

#[test]
fn test_update_config() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let owner = "owner";
    let info = mock_info(&owner, &[]);
    let (_, env) = setup_contract(deps.as_mut(), env.clone(), info.clone()).unwrap();

    let owner = String::from("update-owner");
    let max_duration_seconds = 2629745;

    let config = Config {
        start_blocknumber: env.block.height - 1,
        owner: Addr::unchecked(owner),
        max_duration_seconds,
    };

    let msg = ExecuteMsg::UpdateConfig {
        max_duration_seconds: Some(config.max_duration_seconds),
        owner: Some(config.owner.to_string()),
    };
    _ = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();

    assert_eq!(config.clone(), config_response.config.clone());
}

#[test]
fn test_add_propsal() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let owner = "owner";
    let info = mock_info(&owner, &[]);
    _ = setup_contract(deps.as_mut(), env.clone(), info.clone()).unwrap();

    let title = String::from("test-propsal");
    let status = PropsalStatus::Enabled;
    let nft_address = String::from("nft");
    let expiration = env.block.time.seconds() + 100;
    let candidates = vec![Candidate {
        id: 1,
        name: ("candiate1").to_string(),
    }];

    let msg = ExecuteMsg::AddPropsal {
        title: title.clone(),
        status: status.clone(),
        nft_address: nft_address.clone(),
        expiration,
        candidates: candidates.clone(),
    };

    env.block.height += 1;
    _ = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Propsals {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();

    let propsals_response: PropsalsResponse = from_binary(&res).unwrap();
    let valid_nft_address = deps.api.addr_validate(&nft_address).unwrap();
    let votes = vec![];

    let propsal = Propsal {
        id: 1,
        title,
        status,
        nft_address: valid_nft_address,
        expiration,
        candidates,
        votes,
    };

    assert_eq!(Some(vec![propsal]), Some(propsals_response.propsals));
}

#[test]
fn test_update_propsal() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let owner = "owner";
    let info: MessageInfo = mock_info(&owner, &[]);
    _ = setup_contract(deps.as_mut(), env.clone(), info.clone()).unwrap();
    _ = add_propsal(deps.as_mut(), env.clone(), info.clone());

    let id = 1u64;
    let title = String::from("update-propsal");
    let status = PropsalStatus::Disabled;
    let nft_address = String::from("nft");
    let expiration = env.block.time.seconds() + 100;
    let candidates = vec![Candidate {
        id: 1,
        name: ("u-candiate1").to_string(),
    }];

    let msg = ExecuteMsg::UpdatePropsal {
        id,
        title: Some(title.clone()),
        status: Some(status.clone()),
        nft_address: Some(nft_address.clone()),
        expiration: Some(expiration.clone()),
        candidates: Some(candidates.clone()),
    };

    env.block.height += 1;
    _ = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Propsal { id }).unwrap();
    let propsal_response: PropsalResponse = from_binary(&res).unwrap();
    let valid_nft_address = deps.api.addr_validate(&nft_address).unwrap();
    let votes = vec![];

    let propsal = Propsal {
        id: 1,
        title,
        status,
        nft_address: valid_nft_address,
        expiration,
        candidates,
        votes,
    };

    assert_eq!(propsal, propsal_response.propsal);
}

#[test]
fn test_remove_propsal() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let owner = "owner";
    let info = mock_info(&owner, &[]);
    _ = setup_contract(deps.as_mut(), env.clone(), info.clone()).unwrap();
    _ = add_propsal(deps.as_mut(), env.clone(), info.clone());

    let msg = ExecuteMsg::RemovePropsal { id: 1 };

    env.block.height += 1;
    _ = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Propsals {
            start_after: None,
            limit: None,
        },
    )
    .unwrap();

    let propsals_response: PropsalsResponse = from_binary(&res).unwrap();

    assert_eq!(Some(vec![]), Some(propsals_response.propsals));
}

#[test]
fn test_execute_vote() {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let owner = "owner";
    let info = mock_info(&owner, &[]);
    _ = setup_contract(deps.as_mut(), env.clone(), info.clone()).unwrap();
    _ = add_propsal(deps.as_mut(), env.clone(), info.clone());

    let title = String::from("test-propsal");
    let status = PropsalStatus::Enabled;
    let nft_address = String::from("nft");
    let expiration = env.block.time.seconds() + 100;
    let candidates = vec![Candidate {
        id: 1,
        name: ("candiate1").to_string(),
    }];

    let msg = ExecuteMsg::ExecuteVote {
        propsal_id: 1,
        candidate_id: 1,
    };

    env.block.height += 1;
    _ = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

    let res = query(deps.as_ref(), env.clone(), QueryMsg::Propsal { id: 1 }).unwrap();

    let propsal_response: PropsalResponse = from_binary(&res).unwrap();

    let valid_nft_address = deps.api.addr_validate(&nft_address).unwrap();
    let vote = Vote {
        power: 1,
        voter: Addr::unchecked(owner),
        propsal_id: 1,
        candidate_id: 1,
    };
    let votes = vec![vote];

    let propsal = Propsal {
        id: 1,
        title,
        status,
        nft_address: valid_nft_address,
        expiration,
        candidates,
        votes,
    };

    assert_eq!(propsal, propsal_response.propsal);
}

#[test]
fn test_propsal_result() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let owner = "owner";
    let info = mock_info(&owner, &[]);
    _ = setup_contract(deps.as_mut(), env.clone(), info.clone()).unwrap();
    _ = add_propsal(deps.as_mut(), env.clone(), info.clone());
    _ = execute_vote(deps.as_mut(), env.clone(), info.clone(), 1, 1);

    let res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::PropsalResult { id: 1 },
    )
    .unwrap();

    let propsal_result_response: PropsalResultResponse = from_binary(&res).unwrap();

    let candidate = Candidate {
        id: 1,
        name: String::from(""),
    };

    assert_eq!(candidate, propsal_result_response.propsal_result.winner);
}

fn add_propsal(deps: DepsMut, mut env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let title = String::from("test-propsal");
    let status = PropsalStatus::Enabled;
    let nft_address = String::from("nft");
    let expiration = env.block.time.seconds() + 100;
    let candidates = vec![Candidate {
        id: 1,
        name: ("candiate1").to_string(),
    }];

    let msg = ExecuteMsg::AddPropsal {
        title: title.clone(),
        status: status.clone(),
        nft_address: nft_address.clone(),
        expiration,
        candidates: candidates.clone(),
    };

    let res = execute(deps, env.clone(), info.clone(), msg.clone())?;

    env.block.height += 1;

    Ok(res)
}

fn execute_vote(
    deps: DepsMut,
    mut env: Env,
    info: MessageInfo,
    propsal_id: u64,
    candidate_id: u64,
) -> Result<Response, ContractError> {
    let msg = ExecuteMsg::ExecuteVote {
        propsal_id,
        candidate_id,
    };

    let res = execute(deps, env.clone(), info.clone(), msg.clone())?;

    env.block.height += 1;

    Ok(res)
}
