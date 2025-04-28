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

    #[error("Cannot widthdraw because the fundraiser is still at {current_balance:?} and we need {expected_amount:?}")]
    WithdrawalNotExpectedError {
        expected_amount: u128,
        current_balance: u128,
        denom: String
    }
}
