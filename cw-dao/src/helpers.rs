use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, DepsMut, Env, StdError, StdResult, WasmMsg};

use crate::{
    msg::ExecuteMsg,
    state::{BLOCK_HEIGHTS, BLOCK_INDEX},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DaoContract(pub Addr);

impl DaoContract {
    pub fn increase_block(deps: DepsMut, env: &Env) -> StdResult<()> {
        let idx = BLOCK_INDEX.load(deps.storage)?;
        let block_height = env.block.height;
        let prev_idx = idx - 1;
        let prev_block_height = BLOCK_HEIGHTS.load(deps.storage, prev_idx)?;
        if prev_block_height >= block_height {
            return Err(StdError::generic_err("block height is not increased"));
        }
        BLOCK_HEIGHTS.save(deps.storage, idx, &block_height)?;
        BLOCK_INDEX.save(deps.storage, &(idx + 1))?;

        Ok(())
    }

    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}
