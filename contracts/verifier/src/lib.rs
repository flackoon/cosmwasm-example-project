use cosmwasm_std::{entry_point, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdResult};

pub mod error;
pub mod msg;

use msg::*;
use error::*;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: InstantiateMsg) -> StdResult<Response> {
  Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ValidateMigrationMsg {current_version, new_version, reason} => {
            if current_version >= new_version {
                return Err(ContractError::AlreadyMigrated { version: current_version })
            }

            if reason != "bug_fix" {
                return Err(ContractError::InvalidMigrationReason {});
            }

            Ok(Response::new())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    _deps: Deps,
    _env: Env,
    _msg: QueryMsg,
) -> StdResult<QueryResponse> {
  Ok(QueryResponse::default())
}
