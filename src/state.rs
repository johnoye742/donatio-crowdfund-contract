use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum State {
    Open {},
    Closed {},
    Pending {},
    Canceled {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Owner {
    pub addr: Addr,
    pub email: String,
    pub fullname: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct FundDetails {
    pub owner: Owner,
    pub title: String,
    pub description: String,
    pub amount_to_be_raised: Uint128,
    pub denom: String,
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Donation {
    pub participant: Addr,
    pub message: String,
    pub amount: Uint128
}

pub const STATE: Item<State> = Item::new("contract_state");

pub const DETAILS: Item<FundDetails> = Item::new("fund_details");

pub const DONATIONS: Item<Vec<Donation>> = Item::new("fund_donations");

