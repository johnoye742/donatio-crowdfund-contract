use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{Donation, FundDetails};

#[cw_serde]
pub struct InstantiateMsg {
    pub title: String,
    pub description: String,
    pub email: String,
    pub fullname: String,
    pub amount_to_be_raised: String,
    pub denom: String,
    pub image_url: String,
    pub owner: Addr
}


#[cw_serde]
pub enum ExecuteMsg {
    Donate {
        message: String
    },
    Withdraw {

    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<Donation>)]
    GetDonations {},

    #[returns(FundDetails)]
    GetDetails {},
}
