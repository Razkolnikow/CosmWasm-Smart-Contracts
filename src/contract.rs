use cosmwasm_std::{DepsMut, StdResult, Response, MessageInfo, Coin};
use cw2::{set_contract_version, get_contract_version};
use cw_storage_plus::Item;

use crate::error::ContractError;
use crate::{msg::InstantiateMsg, state::{OWNER, STATE, State}};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(deps: DepsMut, info:MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(deps.storage, &State { counter: 0, minimal_donation: msg.minimal_donation })?;
    OWNER.save(deps.storage, &info.sender)?;

    Ok(Response::new())
}

pub fn migrate_0_1_0(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");

    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

    STATE.save(deps.storage, 
        &State {
            counter,
            minimal_donation,
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
    use cosmwasm_std::{ DepsMut, MessageInfo, Response, StdResult, Env, BankMsg, Coin, Uint128,};

    use crate::{state::{STATE, OWNER}, error::ContractError};

    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        // COUNTER.update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;
        // Ok(Response::new())

        let mut state = STATE.load(deps.storage)?;

        if info.funds.iter().any(|coin| {
            coin.denom == state.minimal_donation.denom && coin.amount >= state.minimal_donation.amount
        }) {
            state.counter += 1;
            STATE.save(deps.storage, &state)?;
        }

        let resp = Response::new()
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