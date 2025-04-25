use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Fundraiser Closed")]
    FundraiserClosed {},

    #[error("Fundraiser Pending, please wait.")]
    FundraiserPending {},

    #[error("This fundraiser has being cancelled")]
    FundraiserCanceled {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
