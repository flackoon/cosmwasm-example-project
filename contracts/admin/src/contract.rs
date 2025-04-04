use crate::error::ContractError;
use crate::msg::{AdminsListResp, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{ADMINS, DONATION_DENOM, VERIFIER, VERSION};
use cosmwasm_std::{
    coins, to_binary, Addr, BankMsg, Binary, Deps, DepsMut, Empty, Env, Event, MessageInfo, Order, Reply, Response, StdResult, SubMsg, WasmMsg
};
use query::get_version;

pub const OLD_VERSION: u32 = 1;
pub const NEW_VERSION: u32 = 2;

const VALIDATE_MIGRATION_REPLY_ID: u64 = 1;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    for addr in msg.admins {
        let admin = deps.api.addr_validate(&addr)?;
        ADMINS.save(deps.storage, &admin, &Empty {})?;
    }

    DONATION_DENOM.save(deps.storage, &msg.donation_denom)?;
    VERSION.save(deps.storage, &OLD_VERSION)?;
    VERIFIER.save(deps.storage, &deps.api.addr_validate(&msg.verifier)?)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        AdminsList {} => to_binary(&query::admins_list(deps)?),
        GetVersion {} => to_binary(&query::get_version(deps)?)
    }
}

mod query {
    use super::*;

    pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
        let admins: Result<Vec<_>, _> = ADMINS
            .keys(deps.storage, None, None, Order::Ascending)
            .collect();
        let admins = admins?;
        let resp = AdminsListResp { admins };
        Ok(resp)
    }

    pub fn get_version(deps: Deps) -> StdResult<u32> {
        let resp = VERSION.load(deps.storage)?;
        Ok(resp)
    }
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        AddMembers { admins } => exec::add_members(deps, info, admins),
        Leave {} => exec::leave(deps, info).map_err(Into::into),
        Donate {} => exec::donate(deps, info),
    }
}

mod exec {
    use super::*;

    pub fn add_members(
        deps: DepsMut,
        info: MessageInfo,
        admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        if !ADMINS.has(deps.storage, &info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }

        // Prevents admins front-running donations and stealing a bigger % of the
        // donations by duplicating their address.
        for new_admin in &admins {
            if ADMINS.has(deps.storage, &Addr::unchecked(new_admin)) {
                return Err(ContractError::AdminExists { admin: Addr::unchecked(new_admin) });
            }
            ADMINS.save(deps.storage, &Addr::unchecked(new_admin), &Empty {})?;
        }

        let events = admins
            .iter()
            .map(|admin| Event::new("admin_added").add_attribute("addr", admin));
        let resp = Response::new()
            .add_events(events)
            .add_attribute("action", "add_members")
            .add_attribute("added_count", admins.len().to_string());

        Ok(resp)
    }

    pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        ADMINS.remove(deps.storage, &info.sender);

        let resp = Response::new()
            .add_attribute("action", "leave")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let denom = DONATION_DENOM.load(deps.storage)?;
        let admins: Result<Vec<_>, _> = ADMINS
            .keys(deps.storage, None, None, Order::Ascending)
            .collect();
        let admins = admins?;

        // ensures DONATION_DENOM coins were sent with this call and returns the sent amount
        let donation = cw_utils::must_pay(&info, &denom)?.u128();

        // rounds down because of unsigned integers division
        let donation_per_admin = donation / (admins.len() as u128);

        let messages = admins.into_iter().map(|admin| BankMsg::Send {
            to_address: admin.to_string(),
            amount: coins(donation_per_admin, &denom),
        });

        let resp = Response::new()
            .add_messages(messages)
            .add_attribute("action", "donate")
            .add_attribute("amount", donation.to_string())
            .add_attribute("per_admin", donation_per_admin.to_string());

        Ok(resp)
    }
}

pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let current_version = get_version(deps.as_ref())?;
    let verifier_addr = VERIFIER.load(deps.storage)?;

    let execute_msg = WasmMsg::Execute {
        contract_addr: verifier_addr.to_string(),
        msg: to_binary(&verifier::msg::ExecuteMsg::ValidateMigrationMsg {
           current_version,
           new_version: NEW_VERSION,
           reason: msg.reason, 
        })?,
        funds: vec![],
    };

    let sub_msg = SubMsg::reply_always(execute_msg, VALIDATE_MIGRATION_REPLY_ID);

    Ok(Response::new().add_submessage(sub_msg))
}

pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        VALIDATE_MIGRATION_REPLY_ID => {
            if msg.result.is_ok() {
                VERSION.save(deps.storage, &NEW_VERSION)?;
            }
            Ok(Response::new())
        }
        _ => Err(cosmwasm_std::StdError::generic_err("Unknown reply ID")),
    }
}
