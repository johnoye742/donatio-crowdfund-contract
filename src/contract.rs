use std::u128;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Uint128, WasmMsg};
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

                    // close contract when the donations are more than 100% needed

                    let donations = &DONATIONS.load(deps.storage).unwrap_or(vec![]);
                    let mut total: Uint128 = Uint128::new(0);
                    for donation in donations {
                        total += donation.amount;
                    }
                    if (total + Uint128::new(amount.u128())) >= Uint128::new(details.amount_to_be_raised.into()) {
                        STATE.save(deps.storage, &State::Closed {  })?;
                    }

                    dbg!(amount.u128() / 1_000_000);
                    let actual_amount = amount.u128() / 1_000_000;

                    let (maybe_reward, maybe_uri) = match actual_amount {
                        11..=49 => (Some("d-3"), Some("https://ipfs.io/ipfs/bafkreiburzi2iphi5jf2rl2rzqvae6qrxgmpezguwbi65s5spzvgg5xyzu")),
                        50..=69 => (Some("d-2"), Some("https://ipfs.io/ipfs/bafkreibiorot22hrse3qjsaomb7fcizujfnp7rte3koqboszgrjeh54ubu")),
                        70..=99 => (Some("d-1"), Some("https://ipfs.io/ipfs/bafkreibeo3detwdrksydm7jrrbwdak6u53gjby5wcrcpky4mcehw4zb7hi")),
                        100..=499 => (Some("s"), Some("https://ipfs.io/ipfs/bafkreievs65xrrdzckzm2rsqvpw3htjolygji5nosudcgma7lfyifuvvmm")),
                        500..=u128::MAX => (Some("elite"), Some("https://ipfs.io/ipfs/bafkreiaceflv7edhh7wudylmoqsue7ggy2kmkvmjwlp57fah2syzcml7cq")),
                        _ => (None, None),
                    };

                    let mut res = Response::new();

                    if let (Some(reward), Some(token_uri)) =  (maybe_reward, maybe_uri)  {
                        let token_id = format!("{}-{}-{}", reward, env.block.height, info.sender);

                        let nft_msg = WasmMsg::Execute {
                            contract_addr: "xion1jgve7p9sx5wmm9x7dk6fmwawed7eekt2n7vj8jvhnhjcczfepmasf7p8vq".into(),
                            msg: to_json_binary(&GovernanceExecuteMsg::IssueNFT {
                                user_addr: info.sender.clone(),
                                token_id,
                                token_uri: token_uri.to_string(),
                                nft_addr: "xion1rtp30fh4pltea8h8fkxalqeuztddaxgxpnjxam2d786axxthe0tqq5knek".into(),
                            })?,
                            funds: vec![],
                        };

                        if details.denom == "uxion" || details.denom == "uusdc" {
                            res = res.add_message(nft_msg);

                        }
                    }

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

                    Ok(res
                        .add_attribute("participant", &donation.participant.to_string())
                        .add_attribute("amount", &donation.amount.to_string()))

                }
                State::Closed {  } => {
                    Err(crate::error::ContractError::FundraiserClosed {  })
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
            let current_balance = deps.querier.query_balance(&env.contract.address, &details.denom).unwrap().amount;

            if info.sender != owner {
                Err(crate::error::ContractError::Unauthorized {  })
            } else if current_balance < details.amount_to_be_raised {
                Err(crate::error::ContractError::WithdrawalNotExpectedError { expected_amount: details.amount_to_be_raised.into(), current_balance: current_balance.into(), denom: details.denom })
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
            let mut total: Uint128 = Uint128::new(0);
            for donation in donations {
                total += donation.amount;
            }
            to_json_binary(&total)
        }
    }
}

