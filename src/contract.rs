#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{coins, to_json_binary, BankMsg, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{Donation, FundDetails, Owner, State, DETAILS, DONATIONS, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:crowdfund-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DONATIONS.save(deps.storage, &vec![])?;
    DETAILS.save(deps.storage, &FundDetails {
        owner: Owner {
            addr: info.sender,
            email: msg.email,
            fullname: msg.fullname
        },
        title: msg.title,
        description: msg.description,
        amount_to_be_raised: msg.amount_to_be_raised.parse::<Uint128>().unwrap(),
        denom: msg.denom,
        image_url: msg.image_url
    })?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let details = DETAILS.load(deps.storage).unwrap();
    match msg {
        ExecuteMsg::Donate { message } => {
            let amount = cw_utils::may_pay(&info, &details.denom).unwrap();

            let donation = Donation {
                participant: info.sender,
                amount,
                message: message.clone(),
            };
            if donation.amount.u128() > 0 {
                let mut donations = DONATIONS.load(deps.storage)?;
                donations.push(donation.clone());

                DONATIONS.save(deps.storage, &donations)?;
            }

            Ok(Response::new()
                .add_attribute("message", message)
                .add_attribute("participant", &donation.participant.to_string())
                .add_attribute("amount", &donation.amount.to_string()))
        },
        ExecuteMsg::Withdraw {  } => {
            let owner = &details.owner.addr;

            if info.sender != owner && deps.querier.query_balance(&env.contract.address, &details.denom).unwrap().amount < details.amount_to_be_raised {
                Ok(Response::new())
            } else {
                let msg = BankMsg::Send { to_address: (owner).to_string(), amount: vec![deps.querier.query_balance(env.contract.address, details.denom)?] };
                Ok(Response::new()
                    .add_message(msg))
            }
        }
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
        QueryMsg::GetDonations {  } => {
            to_json_binary(&DONATIONS.load(deps.storage).unwrap_or(vec![]))
        },
        QueryMsg::GetDetails { } => {
            to_json_binary(&DETAILS.load(deps.storage).unwrap())
        }
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            email: String::from("example@email.com"),
            fullname: String::from("John Doe"),
            title: String::from("Example crowdfund"),
            description: String::from("need some funds"),
            amount_to_be_raised: 500.to_string(),
            denom: "usdc".into(),
            image_url: String::new()
        };

        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

    }


}
