#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Donation, FundDetails, Owner, State, DETAILS, DONATIONS, STATE};
use donacio_governance::msg::ExecuteMsg as GovernanceExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:crowdfund-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: cosmwasm_std::Reply) -> StdResult<Response> {
    if msg.id != 1 {
        return Err(cosmwasm_std::StdError::generic_err("Invalid submsg id"));
    }

    // We received a message! From the contract we invoked earlier.
    println!("{:?}", msg.result);

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    DONATIONS.save(deps.storage, &vec![])?;
    STATE.save(deps.storage, &State::Open {  });
    DETAILS.save(deps.storage, &FundDetails {
        owner: Owner {
            addr: msg.owner,
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
    let state = STATE.load(deps.storage).unwrap();
    match msg {
        ExecuteMsg::Donate { message } => {
            match state {
                State::Open { } => {
                    let amount = cw_utils::may_pay(&info, &details.denom).unwrap();

                    let reward = if amount.u128() > 10 && amount.u128() < 50 {
                        "d-3"
                    } else if amount.u128() > 50 && amount.u128() < 70 {
                        "d-2"
                    } else if amount.u128() > 70 && amount.u128() < 100 {
                        "d-1"
                    } else if amount.u128() > 100 && amount.u128() < 500 {
                        "s"
                    } else if amount.u128() > 500 {
                        "elite"
                    } else {
                        ""
                    };

                    let token_uri: String = if reward == "elite" {
                        "https://ipfs.io/ipfs/bafkreiaceflv7edhh7wudylmoqsue7ggy2kmkvmjwlp57fah2syzcml7cq".into()
                    } else if reward == "s" {
                        "https://ipfs.io/ipfs/bafkreievs65xrrdzckzm2rsqvpw3htjolygji5nosudcgma7lfyifuvvmm".into()
                    } else if reward == "d-3" {
                        "https://ipfs.io/ipfs/bafkreiburzi2iphi5jf2rl2rzqvae6qrxgmpezguwbi65s5spzvgg5xyzu".into()
                    } else if reward == "d-2" {
                        "https://ipfs.io/ipfs/bafkreibiorot22hrse3qjsaomb7fcizujfnp7rte3koqboszgrjeh54ubu".into()
                    } else if reward == "d-1" {
                        "https://ipfs.io/ipfs/bafkreibeo3detwdrksydm7jrrbwdak6u53gjby5wcrcpky4mcehw4zb7hi".into()
                    } else {
                        "".into()
                    };


                    let nft_msg = WasmMsg::Execute { contract_addr: "xion1jgve7p9sx5wmm9x7dk6fmwawed7eekt2n7vj8jvhnhjcczfepmasf7p8vq".into(), msg: to_json_binary(&GovernanceExecuteMsg::IssueNFT {
                        user_addr: info.sender.clone(),
                        token_id: format!("{reward}-{}-{}", info.sender.clone(), env.block.height).into(),
                        token_uri,
                        nft_addr: "xion1rtp30fh4pltea8h8fkxalqeuztddaxgxpnjxam2d786axxthe0tqq5knek".into()
                    })?, funds: vec![] };


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

                    if amount.u128() > 10 {
                        return Ok(Response::new()
                            .add_attribute("message", message)
                            .add_message(nft_msg)
                            .add_attribute("participant", &donation.participant.to_string())
                            .add_attribute("amount", &donation.amount.to_string()))
                    }
                    Ok(Response::new()
                        .add_attribute("message", message)
                        .add_attribute("participant", &donation.participant.to_string())
                        .add_attribute("amount", &donation.amount.to_string()))

                }
                State::Closed {  } => {
                    Err(crate::error::ContractError::FundraiserCloseed {  })
                }
                State::Pending {  } => {
                    Err(crate::error::ContractError::FundraiserPending {  })
                }
                State::Canceled {  } => {
                    Err(crate::error::ContractError::FundraiserCanceled {  })
                }
            }
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
        QueryMsg::GetDonations {  } => {
            to_json_binary(&DONATIONS.load(deps.storage).unwrap_or(vec![]))
        },
        QueryMsg::GetDetails { } => {
            to_json_binary(&DETAILS.load(deps.storage).unwrap())
        },
        QueryMsg::GetTotal {  } => {
            let donations = &DONATIONS.load(deps.storage).unwrap_or(vec![]);
        }
    }
}

