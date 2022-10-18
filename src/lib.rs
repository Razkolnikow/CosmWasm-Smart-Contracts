
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Empty};
use msg::InstantiateMsg;
use error::ContractError;

mod contract;
pub mod error;
pub mod msg;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;
mod state;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
	deps: DepsMut,
	_env: Env,
	_info: MessageInfo,
	msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, _info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: msg::ExecMsg,
) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match _msg {
        Donate {} => exec::donate(_deps, _info)
            .map_err(ContractError::Std),
        Withdraw {} => exec::withdraw(_deps, _env, _info),
        WithdrawTo {
            receiver,
            funds
        } => exec::withdraw_to(_deps, _env, _info, receiver, funds),
        _ => Ok(Response::new()),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    contract::migrate(deps)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    _deps: Deps,
    _env: Env,
    msg: msg::QueryMsg,
) -> StdResult<Binary> {
    use msg::QueryMsg::*;
    use contract::query;

    match msg {
        Value {} => to_binary(&query::value(_deps)?),
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
