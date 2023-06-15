use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid Contract Name")]
    InvalidContractName {},

    #[error("Invalid Contract Version")]
    InvalidContractVersion {},

    #[error("Exceed max duration")]
    MaxDuration {},

    #[error("It is lower than the block time")]
    LowerTime {},

    #[error("Propsal Status is Disabled.")]
    StatusDisabled {},
}
