// src/tests.rs
#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query, migrate};
    use crate::msg::{
        InstantiateMsg, ExecuteMsg, QueryMsg, Outcome, OrderType,
        ConfigResponse, EventResponse,
        OrderResponse,
    };
    use crate::msg::OrderStatus;

    use cosmwasm_std::{coins, Addr, Decimal, Timestamp, Uint128, Api as StdApi}; 
    use cw_multi_test::{App, ContractWrapper, Executor, AppResponse, BasicAppBuilder};
    use anyhow;

    const ADMIN_ID_STR: &str = "admin0001";
    const USER1_ID_STR: &str = "user0001";
    const USER2_ID_STR: &str = "user0002";
    const USER3_ID_STR: &str = "user0003";
    const ORACLE_ID_STR: &str = "oracle0001";
    const BETTING_DENOM: &str = "uinj";

    fn decimal_times_uint128_trunc_for_test(decimal_val: Decimal, uint128_val: Uint128) -> Uint128 {
        if decimal_val < Decimal::zero() {
            panic!("Cannot multiply by negative decimal");
        }
        let product_of_atomics_and_uint = decimal_val.atomics() * uint128_val;
        let denominator = Uint128::new(10u128.pow(Decimal::DECIMAL_PLACES));
        if denominator.is_zero() {
            panic!("Decimal denominator is zero");
        }
        product_of_atomics_and_uint / denominator
    }

    // setup_contract now also returns the generated admin Addr for convenience
    fn setup_contract(app: &mut App, betting_denom: String) -> (Addr, Addr) {
        let contract_wrapper = ContractWrapper::new(execute, instantiate, query).with_migrate(migrate);
        let code_id = app.store_code(Box::new(contract_wrapper));

        let admin_addr = app.api().addr_make(ADMIN_ID_STR);

        let instantiate_msg = InstantiateMsg {
            admin: Some(admin_addr.to_string()), 
            betting_denom,
        };
        
        let contract_addr = app.instantiate_contract(
            code_id,
            admin_addr.clone(), // Sender is the bech32 admin Addr
            &instantiate_msg,
            &[],
            "Injective Betting Contract",
            Some(admin_addr.to_string()), 
        )
        .unwrap();
        (contract_addr, admin_addr)
    }

    fn default_app() -> App {
        // Create a new App and initialize balances with bech32 addresses
        let mut app = BasicAppBuilder::new_custom().build(|router, api, storage| {
            let user1_addr = api.addr_make(USER1_ID_STR);
            let user2_addr = api.addr_make(USER2_ID_STR);
            let user3_addr = api.addr_make(USER3_ID_STR);
            let oracle_addr = api.addr_make(ORACLE_ID_STR);
            let admin_addr = api.addr_make(ADMIN_ID_STR);

            router.bank.init_balance(storage, &user1_addr, coins(1_000_000_000, BETTING_DENOM)).unwrap();
            router.bank.init_balance(storage, &user2_addr, coins(1_000_000_000, BETTING_DENOM)).unwrap();
            router.bank.init_balance(storage, &user3_addr, coins(1_000_000_000, BETTING_DENOM)).unwrap();
            router.bank.init_balance(storage, &oracle_addr, coins(100_000, BETTING_DENOM)).unwrap();
            router.bank.init_balance(storage, &admin_addr, coins(100_000, BETTING_DENOM)).unwrap();
        });
        app
    }

    #[test]
    fn proper_initialization() {
        let mut app = default_app();
        let (contract_addr, admin_addr) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let config_res: ConfigResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetConfig {}).unwrap();
        assert_eq!(config_res.admin, admin_addr); // admin_addr is now the bech32 Addr
        assert_eq!(config_res.betting_denom, BETTING_DENOM);
        assert_eq!(config_res.next_event_id, 0);
        assert_eq!(config_res.next_order_id, 0);
        assert_eq!(config_res.next_bet_id, 0);
    }

    #[test]
    fn create_event() {
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let res = app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Will it rain tomorrow?".to_string(), oracle_addr: None, resolution_deadline: Some(Timestamp::from_seconds(app.block_info().time.seconds() + 10000))}, &[]).unwrap();
        let event_id_1: u64 = res.custom_attrs(1).iter().find(|attr| attr.key == "event_id").unwrap().value.parse().unwrap();
        assert_eq!(event_id_1, 1);
        let event_res: EventResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetEvent { event_id: 1 }).unwrap();
        assert_eq!(event_res.event.id, 1);
        assert_eq!(event_res.event.oracle, user1_addr);

        let user2_addr = app.api().addr_make(USER2_ID_STR);
        let oracle_addr = app.api().addr_make(ORACLE_ID_STR);
        let res2 = app.execute_contract(user2_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Price of ATOM > $10 by EOY?".to_string(), oracle_addr: Some(oracle_addr.to_string()), resolution_deadline: None}, &[]).unwrap();
        let event_id_2: u64 = res2.custom_attrs(1).iter().find(|attr| attr.key == "event_id").unwrap().value.parse().unwrap();
        assert_eq!(event_id_2, 2);
        let event_res_2: EventResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetEvent { event_id: 2 }).unwrap();
        assert_eq!(event_res_2.event.oracle, oracle_addr);
    }

    #[test]
    fn place_back_order_no_match() {
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user2_addr = app.api().addr_make(USER2_ID_STR);

        app.execute_contract(user1_addr.clone(),contract_addr.clone(),&ExecuteMsg::CreateEvent { description: "Test Event 1".to_string(), oracle_addr: None, resolution_deadline: None },&[],).unwrap();
        let stake_amount = Uint128::new(100_000);
        let odds = Decimal::from_atomics(Uint128::new(250), 2).unwrap();
        let res = app.execute_contract(user2_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::Yes, stake: stake_amount, odds}, &coins(stake_amount.u128(), BETTING_DENOM)).unwrap();
        let order_id: u64 = res.custom_attrs(1).iter().find(|attr| attr.key == "order_id").unwrap().value.parse().unwrap();
        assert_eq!(order_id, 1);
        let order_res: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 1 }).unwrap();
        assert_eq!(order_res.order.status, OrderStatus::Open);
        let contract_balance = app.wrap().query_balance(contract_addr.as_str(), BETTING_DENOM).unwrap();
        assert_eq!(contract_balance.amount, stake_amount);
    }

    #[test]
    fn place_lay_order_no_match() {
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user3_addr = app.api().addr_make(USER3_ID_STR);

        app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Test".to_string(), oracle_addr: None, resolution_deadline: None }, &[]).unwrap();
        let backer_stake_to_match = Uint128::new(50_000);
        let odds = Decimal::from_atomics(Uint128::new(200), 2).unwrap();
        let odds_factor = odds.checked_sub(Decimal::one()).unwrap();
        let layer_liability = decimal_times_uint128_trunc_for_test(odds_factor, backer_stake_to_match);
        let res = app.execute_contract(user3_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Lay, outcome: Outcome::Yes, stake: backer_stake_to_match, odds}, &coins(layer_liability.u128(), BETTING_DENOM)).unwrap();
        let order_id: u64 = res.custom_attrs(1).iter().find(|attr| attr.key == "order_id").unwrap().value.parse().unwrap();
        assert_eq!(order_id, 1);
        let order_res: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 1 }).unwrap();
        assert_eq!(order_res.order.status, OrderStatus::Open);
        let contract_balance = app.wrap().query_balance(contract_addr.as_str(), BETTING_DENOM).unwrap();
        assert_eq!(contract_balance.amount, layer_liability);
    }

    #[test]
    fn place_orders_and_match_full() {
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user2_addr = app.api().addr_make(USER2_ID_STR);
        let oracle_addr = app.api().addr_make(ORACLE_ID_STR);

        app.execute_contract( user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Event X".to_string(), oracle_addr: Some(oracle_addr.to_string()), resolution_deadline: None }, &[]).unwrap();
        let back_stake = Uint128::new(100_000);
        let odds = Decimal::from_atomics(Uint128::new(300), 2).unwrap();
        app.execute_contract(user1_addr.clone(),contract_addr.clone(),&ExecuteMsg::PlaceOrder {event_id: 1, order_type: OrderType::Back, outcome: Outcome::Yes, stake: back_stake, odds,},&coins(back_stake.u128(), BETTING_DENOM),).unwrap();
        let lay_backer_stake_to_match = back_stake;
        let odds_factor = odds.checked_sub(Decimal::one()).unwrap();
        let layer_liability = decimal_times_uint128_trunc_for_test(odds_factor, lay_backer_stake_to_match);
        let res_lay = app.execute_contract(user2_addr.clone(),contract_addr.clone(),&ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Lay, outcome: Outcome::Yes, stake: lay_backer_stake_to_match, odds,},&coins(layer_liability.u128(), BETTING_DENOM),).unwrap();
        assert!(res_lay.custom_attrs(1).into_iter().any(|attr| attr.key == "matched_bet_id"));
        let order1_res: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 1 }).unwrap();
        let order2_res: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 2 }).unwrap();
        assert_eq!(order1_res.order.status, OrderStatus::Filled);
        assert_eq!(order2_res.order.status, OrderStatus::Filled);
        let contract_balance = app.wrap().query_balance(contract_addr.as_str(), BETTING_DENOM).unwrap();
        assert_eq!(contract_balance.amount, back_stake + layer_liability);
    }
    
    #[test]
    fn place_orders_and_match_partial() {
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let admin_addr = app.api().addr_make(ADMIN_ID_STR);
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user2_addr = app.api().addr_make(USER2_ID_STR);
        let oracle_addr = app.api().addr_make(ORACLE_ID_STR);

        app.execute_contract( admin_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Event Partial".to_string(), oracle_addr: Some(oracle_addr.to_string()), resolution_deadline: None }, &[]).unwrap();
        let odds = Decimal::from_atomics(Uint128::new(200), 2).unwrap();
        let user1_back_stake = Uint128::new(100_000);
        app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::No, stake: user1_back_stake, odds, }, &coins(user1_back_stake.u128(), BETTING_DENOM)).unwrap();
        let user2_lay_backer_stake_to_match = Uint128::new(50_000);
        let odds_factor = odds.checked_sub(Decimal::one()).unwrap();
        let user2_layer_liability = decimal_times_uint128_trunc_for_test(odds_factor, user2_lay_backer_stake_to_match);
        app.execute_contract(user2_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Lay, outcome: Outcome::No, stake: user2_lay_backer_stake_to_match, odds, }, &coins(user2_layer_liability.u128(), BETTING_DENOM)).unwrap();
        let order1: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 1 }).unwrap();
        let order2: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 2 }).unwrap();
        assert_eq!(order1.order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order1.order.remaining_backer_stake.amount, Uint128::new(100_000 - 50_000));
        assert_eq!(order2.order.status, OrderStatus::Filled);
        assert_eq!(order2.order.remaining_backer_stake.amount, Uint128::zero());
    }

    #[test]
    fn cancel_open_order() { 
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let admin_addr = app.api().addr_make(ADMIN_ID_STR);
        let user1_addr = app.api().addr_make(USER1_ID_STR);

        app.execute_contract(admin_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Cancel Event".to_string(), oracle_addr: None, resolution_deadline: None }, &[]).unwrap();
        let stake = Uint128::new(70_000);
        app.execute_contract( user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::Yes, stake, odds: Decimal::percent(200) }, &coins(stake.u128(), BETTING_DENOM)).unwrap();
        
        let balance_before_cancel = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        
        app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::CancelOrder { order_id: 1 }, &[]).unwrap();
        
        let order: OrderResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &QueryMsg::GetOrder { order_id: 1 }).unwrap();
        assert_eq!(order.order.status, OrderStatus::Cancelled);

        let balance_after_cancel = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        assert_eq!(balance_after_cancel, balance_before_cancel + stake);
    }

    #[test]
    fn resolve_event_backer_wins() { 
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let admin_addr = app.api().addr_make(ADMIN_ID_STR);
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user2_addr = app.api().addr_make(USER2_ID_STR);
        let oracle_addr = app.api().addr_make(ORACLE_ID_STR);

        app.execute_contract(admin_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Resolve Backer Win".to_string(), oracle_addr: Some(oracle_addr.to_string()), resolution_deadline: None }, &[]).unwrap();
        let back_stake = Uint128::new(100_000);
        let odds = Decimal::from_atomics(Uint128::new(200), 2).unwrap();
        let odds_factor = odds.checked_sub(Decimal::one()).unwrap();
        let layer_liability = decimal_times_uint128_trunc_for_test(odds_factor, back_stake);

        app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::Yes, stake: back_stake, odds }, &coins(back_stake.u128(), BETTING_DENOM)).unwrap();
        app.execute_contract(user2_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Lay, outcome: Outcome::Yes, stake: back_stake, odds }, &coins(layer_liability.u128(), BETTING_DENOM)).unwrap();
        
        let user1_bal_before_resolve = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        
        app.execute_contract(oracle_addr.clone(), contract_addr.clone(), &ExecuteMsg::ResolveEvent { event_id: 1, winning_outcome: Outcome::Yes }, &[]).unwrap();

        let user1_bal_after_resolve = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        assert_eq!(user1_bal_after_resolve, user1_bal_before_resolve + back_stake + layer_liability);
    }
    
    #[test]
    fn resolve_event_layer_wins() { 
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());
        let admin_addr = app.api().addr_make(ADMIN_ID_STR);
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user2_addr = app.api().addr_make(USER2_ID_STR);
        let oracle_addr = app.api().addr_make(ORACLE_ID_STR);

        app.execute_contract(admin_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Resolve Layer Win".to_string(), oracle_addr: Some(oracle_addr.to_string()), resolution_deadline: None }, &[]).unwrap();
        let back_stake = Uint128::new(100_000);
        let odds = Decimal::from_atomics(Uint128::new(200), 2).unwrap();
        let odds_factor = odds.checked_sub(Decimal::one()).unwrap();
        let layer_liability = decimal_times_uint128_trunc_for_test(odds_factor, back_stake);

        app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::Yes, stake: back_stake, odds }, &coins(back_stake.u128(), BETTING_DENOM)).unwrap();
        app.execute_contract(user2_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Lay, outcome: Outcome::Yes, stake: back_stake, odds }, &coins(layer_liability.u128(), BETTING_DENOM)).unwrap();
        
        let user2_bal_before_resolve = app.wrap().query_balance(user2_addr.to_string(), BETTING_DENOM).unwrap().amount;

        app.execute_contract(oracle_addr.clone(), contract_addr.clone(), &ExecuteMsg::ResolveEvent { event_id: 1, winning_outcome: Outcome::No }, &[]).unwrap();

        let user2_bal_after_resolve = app.wrap().query_balance(user2_addr.to_string(), BETTING_DENOM).unwrap().amount;
        assert_eq!(user2_bal_after_resolve, user2_bal_before_resolve + back_stake + layer_liability);
    }

    #[test]
    fn resolve_event_with_open_orders_refund() { 
        let mut app = default_app();
        let (contract_addr, _) = setup_contract(&mut app, BETTING_DENOM.to_string());

        let admin_addr = app.api().addr_make(ADMIN_ID_STR);
        let user1_addr = app.api().addr_make(USER1_ID_STR);
        let user2_addr = app.api().addr_make(USER2_ID_STR);
        let user3_addr = app.api().addr_make(USER3_ID_STR);
        let oracle_addr = app.api().addr_make(ORACLE_ID_STR);

        app.execute_contract(admin_addr.clone(), contract_addr.clone(), &ExecuteMsg::CreateEvent { description: "Resolve with Open Orders".to_string(), oracle_addr: Some(oracle_addr.to_string()), resolution_deadline: None }, &[]).unwrap();
        
        let back_stake_matched = Uint128::new(100_000);
        let odds_matched = Decimal::percent(200);
        let odds_factor_matched = odds_matched.checked_sub(Decimal::one()).unwrap();
        let liability_matched = decimal_times_uint128_trunc_for_test(odds_factor_matched, back_stake_matched);
        
        let user1_bal_before_placing_order = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        app.execute_contract(user1_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::Yes, stake: back_stake_matched, odds: odds_matched }, &coins(back_stake_matched.u128(), BETTING_DENOM)).unwrap();
        let user1_bal_after_placing_order = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        assert_eq!(user1_bal_after_placing_order, user1_bal_before_placing_order - back_stake_matched);

        let user2_bal_before_placing_order = app.wrap().query_balance(user2_addr.to_string(), BETTING_DENOM).unwrap().amount;
        app.execute_contract(user2_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Lay, outcome: Outcome::Yes, stake: back_stake_matched, odds: odds_matched }, &coins(liability_matched.u128(), BETTING_DENOM)).unwrap();
        let user2_bal_after_placing_order = app.wrap().query_balance(user2_addr.to_string(), BETTING_DENOM).unwrap().amount;
        assert_eq!(user2_bal_after_placing_order, user2_bal_before_placing_order - liability_matched);

        let open_back_stake_user3 = Uint128::new(50_000);
        app.execute_contract(user3_addr.clone(), contract_addr.clone(), &ExecuteMsg::PlaceOrder { event_id: 1, order_type: OrderType::Back, outcome: Outcome::No, stake: open_back_stake_user3, odds: Decimal::percent(300) }, &coins(open_back_stake_user3.u128(), BETTING_DENOM)).unwrap();
        
        let user1_bal_before_resolve = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        let user3_bal_before_resolve = app.wrap().query_balance(user3_addr.to_string(), BETTING_DENOM).unwrap().amount;

        let res_resolve: Result<AppResponse, anyhow::Error> = app.execute_contract(oracle_addr.clone(), contract_addr.clone(), &ExecuteMsg::ResolveEvent { event_id: 1, winning_outcome: Outcome::Yes }, &[]);
        assert!(res_resolve.is_ok());
        
        let user1_bal_after_resolve = app.wrap().query_balance(user1_addr.to_string(), BETTING_DENOM).unwrap().amount;
        let expected_user1_payout = back_stake_matched + liability_matched;
        assert_eq!(user1_bal_after_resolve, user1_bal_before_resolve + expected_user1_payout);

        let user3_bal_after_resolve = app.wrap().query_balance(user3_addr.to_string(), BETTING_DENOM).unwrap().amount;
        assert_eq!(user3_bal_after_resolve, user3_bal_before_resolve + open_back_stake_user3);

        if let Ok(response) = res_resolve {
            let wasm_event_attributes: Vec<_> = response.events.iter()
                .filter(|event| event.ty == "wasm") 
                .flat_map(|event| event.attributes.iter())
                .collect();
            
            // payout_winner attribute comes from Addr::to_string(), which will be the bech32 string
            assert!(wasm_event_attributes.iter().any(|attr| attr.key == "payout_winner" && attr.value == user1_addr.to_string()));
            assert!(wasm_event_attributes.iter().any(|attr| attr.key == "payout_amount" && attr.value.starts_with(&expected_user1_payout.to_string())));
            assert!(wasm_event_attributes.iter().any(|attr| attr.key == "refunded_open_order_id" && attr.value == "3"));
        }
    }
}