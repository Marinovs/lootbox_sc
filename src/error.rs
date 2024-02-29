use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")] Std(#[from] StdError),
    #[error("InvalidCw721Token")] InvalidCw721Token {},
    #[error("Unauthorized")] Unauthorized {},
    #[error("Max Odds Reached")] MaxOddsReached {
        msg: String,
    },
    #[error("Box Not Found")] BoxNotFound {},
    #[error("Amount not match")] AmountNotMatch {},
    #[error("Payment Failed")] PaymentFailed {},
    #[error("Conflict ID")] ConflictID {},
    #[error("Reward not found")] RewardNotFound {},
    #[error("Box terminated")] BoxTerminated {},
}
