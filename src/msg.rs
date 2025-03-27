use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{Donation, FundDetails};

#[cw_serde]
pub struct InstantiateMsg {
    pub title: String,
    pub description: String,
    pub email: String,
    pub fullname: String
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
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},

    #[returns(Vec<Donation>)]
    GetDonations {}
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}
