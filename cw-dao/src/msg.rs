use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{Candidate, Config, Propsal, PropsalResult, PropsalStatus};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub max_duration_seconds: u64,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        max_duration_seconds: Option<u64>,
    },
    AddPropsal {
        title: String,
        status: PropsalStatus,
        nft_address: String,
        expiration: u64,
        candidates: Vec<Candidate>,
    },
    UpdatePropsal {
        id: u64,
        title: Option<String>,
        status: Option<PropsalStatus>,
        nft_address: Option<String>,
        expiration: Option<u64>,
        candidates: Option<Vec<Candidate>>,
    },
    RemovePropsal {
        id: u64,
    },
    ExecuteVote {
        propsal_id: u64,
        candidate_id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(PropsalsResponse)]
    Propsals {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(PropsalsResponse)]
    Propsal { id: u64 },
    #[returns(PropsalResultResponse)]
    PropsalResult { id: u64 },
    #[returns(BlockHeightResponse)]
    BlockHeight { num: u64 },
    #[returns(BlockIndexResponse)]
    BlockIndex {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct PropsalsResponse {
    pub propsals: Vec<Propsal>,
}

#[cw_serde]
pub struct PropsalResponse {
    pub propsal: Propsal,
}

#[cw_serde]
pub struct PropsalResultResponse {
    pub propsal_result: PropsalResult,
}

#[cw_serde]
pub struct BlockHeightResponse {
    pub block_height: u64,
}

#[cw_serde]
pub struct BlockIndexResponse {
    pub block_index: u64,
}
