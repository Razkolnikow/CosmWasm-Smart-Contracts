use std::ops::Add;

use cosmwasm_std::{coin, coins, Addr, Empty, Decimal};
use cw_multi_test::{App, Executor};

use crate::add;
use crate::msg::{ValueResp, Parent};

use crate::error::ContractError;
use crate::state::{STATE, State};

use super::contract::CountingContract;

use counting_contract_0_1_0::multitest::contract::CountingContract as CountingContract_0_1_0;

const ATOM: &str = "atom";

#[test]
fn donate_with_funds() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, "atom"),
        None,
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });
}

#[test]
fn query_value() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();
    
    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, "atom"),
        None,
    )
    .unwrap();

    let resp: ValueResp = contract.query_value(&app).unwrap();

    assert_eq!(resp, ValueResp { value: 0 });
}

#[test]
fn donate() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract_addr = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, "atom"),
        None,
    )
    .unwrap();

    contract_addr.donate(&mut app, &sender, &[]).unwrap();

    let resp: ValueResp = contract_addr.query_value(&app).unwrap();
    
    assert_eq!(resp, ValueResp {value: 0});
}

#[test]
fn withdraw() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, "atom"),
        None,
    )
    .unwrap();

    contract.donate(&mut app, &sender, &coins(10, ATOM)).unwrap();

    contract.withdraw(&mut app, &owner).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(10, ATOM)
    );

    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);

    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![]
    );

}

#[test]
fn withdraw_to() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
    let receiver = Addr::unchecked("receiver");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, "atom"),
        None,
    )
    .unwrap();

    contract.donate(&mut app, &sender, &coins(10, ATOM)).unwrap();

    contract.withdraw_to(&mut app, &owner, &receiver, coins(5, ATOM)).unwrap();

    assert_eq!(
        app.wrap().query_all_balances(receiver).unwrap(),
        coins(5, ATOM)
    );

    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);

    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(5, ATOM)
    );

}

#[test]
fn unauthorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let member = Addr::unchecked("member");

    let mut app = App::default();

    let contract_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, "atom"),
        None,
    )
    .unwrap();

    let err = contract
        .withdraw(&mut app, &member)
        .unwrap_err();

    assert_eq!(
        ContractError::Unauthorized {
            owner: owner.into()
        },
        err
    );
}

#[test]
fn migration() {
    let owner = Addr::unchecked("owner");
    let admin = Addr::unchecked("admin");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let old_code_id = CountingContract_0_1_0::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract_0_1_0::instantiate(&mut app, old_code_id, &owner, Some(&admin), "Counting Contract", None, coin(10, ATOM)).unwrap();
    contract.donate(&mut app, &sender, &coins(10, ATOM)).unwrap();

    let contract = CountingContract::migrate(
        &mut app, 
        &admin, 
        contract.addr(), 
        new_code_id).unwrap();

    let resp = contract.query_value(&app).unwrap();

    assert_eq!(resp.value, 1);

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {counter:1,minimal_donation:coin(10,ATOM),donating_parent:None, owner: owner, }
    )
}

#[test]
fn donating_parent() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(20, "atom"))
            .unwrap();
    });

    let code_id = CountingContract::store_code(&mut app);

    let parent_contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        None,
        "Parent contract",
        None,
        coin(0, ATOM),
        None,
    )
    .unwrap();

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        None,
        "Counting contract",
        None,
        coin(10, ATOM),
        Parent {
            addr: parent_contract.addr().to_string(),
            donating_period: 2,
            part: Decimal::percent(10),
        },
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let resp = parent_contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 2 });

    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(18, ATOM)
    );
    assert_eq!(
        app.wrap()
            .query_all_balances(parent_contract.addr())
            .unwrap(),
        coins(2, ATOM)
    );
}