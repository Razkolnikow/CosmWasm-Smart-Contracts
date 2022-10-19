use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::App;

use crate::msg::{ValueResp};

use crate::error::ContractError;

use super::contract::CountingContract;

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