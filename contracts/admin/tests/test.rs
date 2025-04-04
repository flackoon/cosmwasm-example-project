use cosmwasm_std::coins;
use cosmwasm_std::Addr;
use cw_multi_test::{App, ContractWrapper, Executor};

use admin::contract::{
    execute as admin_execute,
    instantiate as admin_instantiate,
    migrate as admin_migrate,
    query as admin_query,
    reply as admin_reply
};
use admin::contract::{NEW_VERSION, OLD_VERSION};
use admin::error::ContractError;
use admin::msg::{AdminsListResp, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use verifier::{
    execute as verifier_execute,
    instantiate as verifier_instantiate,
    query as verifier_query,
};
#[test]
fn instantiation() {
    let mut app = App::default();

    let code =
        ContractWrapper::new(admin_execute, admin_instantiate, admin_query);
    let code_id = app.store_code(Box::new(code));

    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec![],
                donation_denom: "usdc".to_owned(),
                verifier: "verifier".to_owned(),
            },
            &[],
            "Contract",
            None,
        )
        .unwrap();

    let resp: AdminsListResp = app
        .wrap()
        .query_wasm_smart(addr, &QueryMsg::AdminsList {})
        .unwrap();

    assert_eq!(resp, AdminsListResp { admins: vec![] });

    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                donation_denom: "usdc".to_owned(),
                verifier: "verifier".to_owned(),
            },
            &[],
            "Contract 2",
            None,
        )
        .unwrap();

    let resp: AdminsListResp = app
        .wrap()
        .query_wasm_smart(addr.clone(), &QueryMsg::AdminsList {})
        .unwrap();

    assert_eq!(
        resp,
        AdminsListResp {
            admins: vec![Addr::unchecked("admin1"), Addr::unchecked("admin2")]
        }
    );

    // Assert version is 1
    let resp: u32 = app
        .wrap()
        .query_wasm_smart(addr, &QueryMsg::GetVersion {})
        .unwrap();

    assert_eq!(resp, 1)
}

#[test]
fn unauthorized() {
    let mut app = App::default();

    let code =
        ContractWrapper::new(admin_execute, admin_instantiate, admin_query);
    let code_id = app.store_code(Box::new(code));

    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec![],
                donation_denom: "usdc".to_owned(),
                verifier: "verifier".to_owned(),
            },
            &[],
            "Contract",
            None,
        )
        .unwrap();

    let err = app
        .execute_contract(
            Addr::unchecked("user"),
            addr,
            &ExecuteMsg::AddMembers {
                admins: vec!["user".to_owned()],
            },
            &[],
        )
        .unwrap_err();

    assert_eq!(
        ContractError::Unauthorized {
            sender: Addr::unchecked("user")
        },
        err.downcast().unwrap()
    );
}

#[test]
fn add_members() {
    let mut app = App::default();

    let code =
        ContractWrapper::new(admin_execute, admin_instantiate, admin_query);
    let code_id = app.store_code(Box::new(code));

    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec!["owner".to_owned()],
                donation_denom: "usdc".to_owned(),
                verifier: "verifier".to_owned(),
            },
            &[],
            "Contract",
            None,
        )
        .unwrap();

    let resp = app
        .execute_contract(
            Addr::unchecked("owner"),
            addr,
            &ExecuteMsg::AddMembers {
                admins: vec!["user".to_owned()],
            },
            &[],
        )
        .unwrap();

    let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "action")
            .unwrap()
            .value,
        "add_members"
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "added_count")
            .unwrap()
            .value,
        "1"
    );

    let admin_added: Vec<_> = resp
        .events
        .iter()
        .filter(|ev| ev.ty == "wasm-admin_added")
        .collect();

    assert_eq!(admin_added.len(), 1);

    assert_eq!(
        admin_added[0]
            .attributes
            .iter()
            .find(|attr| attr.key == "addr")
            .unwrap()
            .value,
        "user"
    )
}

#[test]
fn donations() {
    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked("user"), coins(5, "usdc"))
            .unwrap()
    });

    let code =
        ContractWrapper::new(admin_execute, admin_instantiate, admin_query);
    let code_id = app.store_code(Box::new(code));

    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                donation_denom: "usdc".to_owned(),
                verifier: "verifier".to_owned(),
            },
            &[],
            "Contract",
            None,
        )
        .unwrap();

    app.execute_contract(
        Addr::unchecked("user"),
        addr.clone(),
        &ExecuteMsg::Donate {},
        &coins(5, "usdc"),
    )
    .unwrap();

    assert_eq!(
        app.wrap()
            .query_balance("user", "usdc")
            .unwrap()
            .amount
            .u128(),
        0
    );
    assert_eq!(
        app.wrap()
            .query_balance(&addr, "usdc")
            .unwrap()
            .amount
            .u128(),
        1
    );
    assert_eq!(
        app.wrap()
            .query_balance("admin1", "usdc")
            .unwrap()
            .amount
            .u128(),
        2
    );
    assert_eq!(
        app.wrap()
            .query_balance("admin2", "usdc")
            .unwrap()
            .amount
            .u128(),
        2
    );
}

#[test]
fn migration() {
    let mut app = App::default();

    let verifier_code = ContractWrapper::new(
        verifier_execute,
        verifier_instantiate,
        verifier_query
    );
    let verifier_code_id = app.store_code(Box::new(verifier_code));

    let verifier_addr = app
        .instantiate_contract(
            verifier_code_id,
            Addr::unchecked("owner"),
            &verifier::msg::InstantiateMsg {},
            &[],
            "Verifier",
            Some("contract_admin".to_owned())
        )
        .unwrap();

    let admin_code = ContractWrapper::new(
            admin_execute,
            admin_instantiate,
            admin_query
        );
        
    let admin_code_id = app.store_code(Box::new(admin_code));

    let admin_addr = app
        .instantiate_contract(
            admin_code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                donation_denom: "usdc".to_owned(),
                verifier: verifier_addr.to_string(),
            },
            &[],
            "Contract",
            Some("contract_admin".to_owned()),
        )
        .unwrap();

    let new_admin_code = ContractWrapper::new(
            admin_execute,
            admin_instantiate,
            admin_query
        )
        .with_migrate(admin_migrate)
        .with_reply(admin_reply);
    let new_admin_code_id = app.store_code(Box::new(new_admin_code));

    app.migrate_contract(
        Addr::unchecked("contract_admin"),
        admin_addr.clone(),
        &MigrateMsg {
            reason: "bug_fix".to_owned(),
        },
        new_admin_code_id,
    ).unwrap();

    let version: u32 = app
        .wrap()
        .query_wasm_smart(admin_addr, &QueryMsg::GetVersion {})
        .unwrap();

    assert_eq!(version, NEW_VERSION)
}

#[test]
fn failing_migration() {
    let mut app = App::default();

    let verifier_code = ContractWrapper::new(
        verifier_execute,
        verifier_instantiate,
        verifier_query
    );
    let verifier_code_id = app.store_code(Box::new(verifier_code));

    let verifier_addr = app
        .instantiate_contract(
            verifier_code_id,
            Addr::unchecked("owner"),
            &verifier::msg::InstantiateMsg {},
            &[],
            "Verifier",
            Some("contract_admin".to_owned())
        )
        .unwrap();

    let admin_code = ContractWrapper::new(
            admin_execute,
            admin_instantiate,
            admin_query
        );
        
    let admin_code_id = app.store_code(Box::new(admin_code));

    let admin_addr = app
        .instantiate_contract(
            admin_code_id,
            Addr::unchecked("owner"),
            &InstantiateMsg {
                admins: vec!["admin1".to_owned(), "admin2".to_owned()],
                donation_denom: "usdc".to_owned(),
                verifier: verifier_addr.to_string(),
            },
            &[],
            "Contract",
            Some("contract_admin".to_owned()),
        )
        .unwrap();

    let new_admin_code = ContractWrapper::new(
            admin_execute,
            admin_instantiate,
            admin_query
        )
        .with_migrate(admin_migrate)
        .with_reply(admin_reply);
    let new_admin_code_id = app.store_code(Box::new(new_admin_code));

    // We expect this call to succeed as it dispatches an `execute` call to
    // the Verifier contract, which will only update the contract version on success.
    app.migrate_contract(
        Addr::unchecked("contract_admin"),
        admin_addr.clone(),
        &MigrateMsg {
            reason: "not_bug_fix".to_owned(),
        },
        new_admin_code_id,
    ).unwrap();

    let version: u32 = app
        .wrap()
        .query_wasm_smart(admin_addr, &QueryMsg::GetVersion {})
        .unwrap();

    assert_eq!(version, OLD_VERSION)
}
