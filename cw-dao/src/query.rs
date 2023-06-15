use std::collections::HashMap;

use cosmwasm_std::{entry_point, to_binary, Binary, Deps, Env, Order::Ascending, StdResult};

use cw_storage_plus::Bounder;

use crate::msg::{
    BlockHeightResponse, BlockIndexResponse, ConfigResponse, PropsalResponse,
    PropsalResultResponse, PropsalsResponse, QueryMsg,
};
use crate::state::{Candidate, PropsalResult, BLOCK_HEIGHTS, BLOCK_INDEX, CONFIG, PROPSALS};

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Propsals { start_after, limit } => {
            to_binary(&query_propsals(deps, start_after, limit)?)
        }
        QueryMsg::Propsal { id } => to_binary(&query_propsal(deps, id)?),
        QueryMsg::PropsalResult { id } => to_binary(&query_propsal_result(deps, id)?),
        QueryMsg::BlockHeight { num } => to_binary(&query_block_height(deps, num.clone())?),
        QueryMsg::BlockIndex {} => to_binary(&query_block_index(deps)?),
    }
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse { config })
}

fn query_propsals(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<PropsalsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = if let Some(start_after) = start_after {
        Some(start_after.exclusive_bound().unwrap())
    } else {
        None
    };

    let result: StdResult<Vec<_>> = PROPSALS
        .range(deps.storage, start, None, Ascending)
        .take(limit)
        .collect();

    let propsals = result?.into_iter().map(|(_, v)| v).collect();

    Ok(PropsalsResponse { propsals })
}

fn query_propsal(deps: Deps, id: u64) -> StdResult<PropsalResponse> {
    let propsal = PROPSALS.load(deps.storage, id)?;

    Ok(PropsalResponse { propsal })
}

fn query_propsal_result(deps: Deps, id: u64) -> StdResult<PropsalResultResponse> {
    let propsal = PROPSALS.load(deps.storage, id)?;

    let mut candidate_counts: HashMap<u64, u32> = HashMap::new();

    // 투표수 세기
    for vote in propsal.clone().votes {
        let count = candidate_counts.entry(vote.candidate_id).or_insert(0);
        *count += 1;
    }

    // TODO: power 계산 필요, 가장 많은 투표를 받은 후보 찾기
    let (max_candidate_id, _) = candidate_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&candidate_id, &count)| (candidate_id, count))
        .unwrap();

    let winner = Candidate {
        id: max_candidate_id,
        name: String::from(""),
    };

    let propsal_result = PropsalResult {
        propsal: propsal.clone(),
        winner,
    };

    Ok(PropsalResultResponse { propsal_result })
}

fn query_block_height(deps: Deps, key: u64) -> StdResult<BlockHeightResponse> {
    let block_height = BLOCK_HEIGHTS.load(deps.storage, key)?;

    Ok(BlockHeightResponse { block_height })
}

fn query_block_index(deps: Deps) -> StdResult<BlockIndexResponse> {
    let block_index = BLOCK_INDEX.load(deps.storage)?;

    Ok(BlockIndexResponse { block_index })
}
