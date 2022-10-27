use cosmwasm_std::{DepsMut, StdResult, Response, MessageInfo, Coin, Addr};
use cw2::{set_contract_version, get_contract_version};
use cw_storage_plus::Item;
use serde::{Serialize, Deserialize};

use crate::error::ContractError;
use crate::msg::Parent;
use crate::state::{PARENT_DONATION, ParentDonation};
use crate::{msg::InstantiateMsg, state::{OWNER, STATE, State}};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info:MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let parent = &msg.parent;
    let sender = &info.sender;
    STATE.save(deps.storage, &State { 
        counter: 0, 
        minimal_donation: msg.minimal_donation,
        owner: sender.clone(),
        donating_parent: parent.as_ref().map(|p| p.donating_period), 
    })?;
    OWNER.save(deps.storage, &info.sender)?;

    if let Some(parent) = parent {
        PARENT_DONATION.save(deps.storage, &ParentDonation {
            address: deps.api.addr_validate(&parent.addr)?,
            donating_parent_period: parent.donating_period,
            part: parent.part,
        })?;
    }

    Ok(Response::new())
}

pub fn migrate_0_1_0(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");
    const OWNER: Item<Addr> = Item::new("owner");

    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;
    let owner = OWNER.load(deps.storage)?;

    STATE.save(deps.storage, 
        &State {
            counter,
            minimal_donation,
            owner,
            donating_parent: None,
        },
    )?;

    Ok(Response::new())
}

pub fn migrate(mut deps: DepsMut) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    if contract_version.contract != CONTRACT_NAME {
        return  Err(ContractError::InvalidContract {
            contract: contract_version.contract,
        });
    }

    let resp = match contract_version.version.as_str() {
        "0.1.0" => migrate_0_1_0(deps.branch()).map_err(ContractError::from)?,
        "0.2.0" => migrate_0_2_0(deps.branch()).map_err(ContractError::from)?,
        CONTRACT_VERSION => return Ok(Response::default()),
        version => {
            return Err(ContractError::InvalidContractVersion { 
                version: version.into(), 
            })
        }
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(resp)
}

pub fn migrate_0_2_0(deps: DepsMut) -> StdResult<Response> {
    #[derive(Serialize, Deserialize)]
    struct OldState {
        counter: u64,
        minimal_donation: Coin,
        owner: Addr,
    }

    const OLD_STATE: Item<OldState> = Item::new("state");

    let OldState {
        counter,
        minimal_donation,
        owner,
    } = OLD_STATE.load(deps.storage)?;

    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
            owner,
            donating_parent: None,
        },
    )?;

    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{StdResult, Deps};

    use crate::msg::ValueResp;
    use crate::state::{STATE};

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResp {value})
    }
}

pub mod exec {
    use cosmwasm_std::{ DepsMut, MessageInfo, Response, StdResult, Env, BankMsg, Coin, Uint128, WasmMsg, to_binary,};

    use crate::{state::{STATE, OWNER, PARENT_DONATION}, error::ContractError, msg::ExecMsg};

    pub fn donate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        // COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;
        // Ok(Response::new())

        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
            && coin.amount >= state.minimal_donation.amount
            })
        {
            state.counter += 1;
            if let Some(parent) = &mut state.donating_parent {
                *parent -= 1;
    
                if *parent == 0 {
                    let parent_donation = PARENT_DONATION.load(deps.storage)?;
                    *parent = parent_donation.donating_parent_period;
    
                    let funds: Vec<_> = deps
                        .querier
                        .query_all_balances(env.contract.address)?
                        .into_iter()
                        .map(|mut coin| {
                            coin.amount = coin.amount * parent_donation.part;
                            coin
                        })
                        .collect();
    
                    let msg = WasmMsg::Execute {
                        contract_addr: parent_donation.address.to_string(),
                        msg: to_binary(&ExecMsg::Donate {})?,
                        funds,
                    };
    
                    resp = resp
                        .add_message(msg)
                        .add_attribute("donated_to_parent", parent_donation.address.to_string());
                }
            }
    
            STATE.save(deps.storage, &state)?;
        }

        resp = resp
        .add_attribute("action", "poke")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("counter", state.counter.to_string());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn withdraw_to(deps: DepsMut, env: Env, info: MessageInfo, receiver: String, funds: Vec<Coin>) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let mut balance = deps.querier.query_all_balances(&env.contract.address)?;

        if !funds.is_empty() {
            for coin in &mut balance {
                let limit = funds
                    .iter()
                    .find(|c| c.denom == coin.denom)
                    .map(|c| c.amount)
                    .unwrap_or(Uint128::zero());

                coin.amount = std::cmp::min(coin.amount, limit);
            }
        }

        let bank_msg = BankMsg::Send {
            to_address: receiver,
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}