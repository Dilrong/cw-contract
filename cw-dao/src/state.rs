use cosmwasm_schema::cw_serde;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// 컨트랙트 설정
#[cw_serde]
pub struct Config {
    /// 시작 블록
    pub start_blocknumber: u64,
    /// 최대 등록 기간
    pub max_duration_seconds: u64,
    /// 컨트랙트 오너
    pub owner: Addr,
}

/// 투표 제안서
#[cw_serde]
pub struct Propsal {
    /// 아이디
    pub id: u64,
    /// 제목
    pub title: String,
    /// 상태
    pub status: PropsalStatus,
    /// 투표 가능한 nft 주소
    pub nft_address: Addr,
    /// 투표 마감기한
    pub expiration: u64,
    /// 투표
    pub votes: Vec<Vote>,
    /// 후보
    pub candidates: Vec<Candidate>,
}

/// 제안서 상태
#[cw_serde]
pub enum PropsalStatus {
    /// 진행
    Enabled,
    /// 미진행
    Disabled,
    /// 종료
    Finalized,
}

/// 투표
#[cw_serde]
pub struct Vote {
    /// 보팅 파워
    pub power: u64,
    /// 투표자
    pub voter: Addr,
    /// Propsal 인덱스
    pub propsal_id: u64,
    /// Candidate 인덱스
    pub candidate_id: u64,
}

/// 투표 후보
#[cw_serde]
pub struct Candidate {
    /// 후보 인덱스
    pub id: u64,
    /// 후보 이름
    pub name: String,
}

/// 투표 결과
#[cw_serde]
pub struct PropsalResult {
    /// 제안
    pub propsal: Propsal,
    /// 최다 득표자
    pub winner: Candidate,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BLOCK_HEIGHTS: Map<u64, u64> = Map::new("block_record");
pub const BLOCK_INDEX: Item<u64> = Item::new("block_index");
pub const PROPSAL_INDEX: Item<u64> = Item::new("propsal_index");
pub const PROPSALS: Map<u64, Propsal> = Map::new("propsals");
