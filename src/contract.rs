use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order as CwOrder, Response, StdResult, Uint128, Decimal, Storage, StdError, Addr, Timestamp, OverflowError, DivideByZeroError
}; // Removed Rounding, Added DivideByZeroError
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, Outcome, OrderType, EventStatus, OrderStatus,
    ConfigResponse, EventResponse, EventsResponse, OrderResponse, OrdersResponse,
    MatchedBetsResponse
};
use crate::state::{
    Config, Event, Order, MatchedBet, CONFIG, NEXT_EVENT_ID, NEXT_ORDER_ID, NEXT_BET_ID,
    EVENTS, orders, MATCHED_BETS, EVENT_TO_MATCHED_BETS
};

const CONTRACT_NAME: &str = "crates.io:injective-betting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

// Helper function for Decimal * Uint128 -> Uint128 (with truncation)
fn decimal_times_uint128_trunc(decimal_val: Decimal, uint128_val: Uint128) -> Result<Uint128, ContractError> {
    if decimal_val < Decimal::zero() {
        return Err(ContractError::CalculationError { msg: "Cannot multiply by negative decimal".to_string() });
    }

    // Explicitly implement (decimal.atomics * uint_val) / 10^DECIMAL_PLACES
    let product_of_atomics_and_uint = decimal_val.atomics().checked_mul(uint128_val)
        .map_err(|e: OverflowError| ContractError::CalculationError { 
            msg: format!("Intermediate multiplication overflow: {} * {}: {}", decimal_val.atomics(), uint128_val, e) 
        })?;
    
    let denominator = Uint128::new(10u128.pow(Decimal::DECIMAL_PLACES));
    if denominator.is_zero() { 
        return Err(ContractError::CalculationError { msg: "Decimal denominator is zero".to_string() });
    }

    let result = product_of_atomics_and_uint.checked_div(denominator)
        .map_err(|e: DivideByZeroError| ContractError::CalculationError { 
            msg: format!("Division by denominator failed: {} / {}: {}", product_of_atomics_and_uint, denominator, e)
        })?;
        
    Ok(result)
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.map_or(Ok(info.sender.clone()), |addr_str| deps.api.addr_validate(&addr_str))?;
    
    if msg.betting_denom.is_empty() {
        return Err(ContractError::InvalidDenom { expected_denom: "any non-empty string".to_string(), received_denom: msg.betting_denom });
    }

    let config = Config {
        admin,
        betting_denom: msg.betting_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    NEXT_EVENT_ID.save(deps.storage, &0u64)?;
    NEXT_ORDER_ID.save(deps.storage, &0u64)?;
    NEXT_BET_ID.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin.to_string())
        .add_attribute("betting_denom", config.betting_denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateEvent { description, oracle_addr, resolution_deadline } => 
            execute_create_event(deps, env, info, description, oracle_addr, resolution_deadline),
        ExecuteMsg::PlaceOrder { event_id, order_type, outcome, stake, odds } => 
            execute_place_order(deps, env, info, event_id, order_type, outcome, stake, odds),
        ExecuteMsg::CancelOrder { order_id } => 
            execute_cancel_order(deps, env, info, order_id),
        ExecuteMsg::ResolveEvent { event_id, winning_outcome } => 
            execute_resolve_event(deps, env, info, event_id, winning_outcome),
    }
}

fn execute_create_event(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    description: String,
    oracle_addr: Option<String>,
    resolution_deadline: Option<Timestamp>,
) -> Result<Response, ContractError> {
    if description.is_empty() {
        return Err(ContractError::InvalidDescription {});
    }

    let oracle = match oracle_addr {
        Some(addr_str) => deps.api.addr_validate(&addr_str)?,
        None => info.sender.clone(),
    };
    let event_id = NEXT_EVENT_ID.update(deps.storage, |id| -> StdResult<_> { Ok(id + 1) })?;
    
    let event = Event {
        id: event_id,
        creator: info.sender.clone(),
        description,
        oracle,
        status: EventStatus::Open,
        winning_outcome: None,
        resolution_deadline,
        creation_time: env.block.time,
    };
    EVENTS.save(deps.storage, event_id, &event)?;
    EVENT_TO_MATCHED_BETS.save(deps.storage, event_id, &Vec::new())?;

    Ok(Response::new()
        .add_attribute("method", "create_event")
        .add_attribute("event_id", event_id.to_string())
        .add_attribute("creator", info.sender.to_string())
        .add_attribute("oracle", event.oracle.to_string()))
}

fn execute_place_order(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    event_id: u64,
    order_type: OrderType,
    outcome: Outcome,
    backer_stake_amount_msg: Uint128,
    odds: Decimal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if backer_stake_amount_msg.is_zero() {
        return Err(ContractError::InvalidStakeAmount {});
    }
    if odds <= Decimal::one() {
        return Err(ContractError::InvalidOdds {});
    }

    let event = EVENTS.load(deps.storage, event_id)
        .map_err(|_| ContractError::EventNotFound { event_id })?;
    if event.status != EventStatus::Open {
        return Err(ContractError::EventNotOpen { event_id });
    }
    if let Some(deadline) = event.resolution_deadline {
        if env.block.time > deadline {
            return Err(ContractError::DeadlinePassed {});
        }
    }

    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::MultipleCoinsSent {});
    }
    let sent_coin = info.funds[0].clone();
    if sent_coin.denom != config.betting_denom {
        return Err(ContractError::InvalidDenom { expected_denom: config.betting_denom.clone(), received_denom: sent_coin.denom });
    }

    let required_deposit: Uint128;
    let order_backer_stake = Coin { denom: config.betting_denom.clone(), amount: backer_stake_amount_msg };

    match order_type {
        OrderType::Back => {
            required_deposit = order_backer_stake.amount;
        }
        OrderType::Lay => {
            let odds_factor = odds.checked_sub(Decimal::one())
                .map_err(|e: OverflowError| ContractError::CalculationError { msg: format!("Odds factor calculation error: {}", e) })?;
            if odds_factor.is_zero() { 
                 return Err(ContractError::InvalidOdds {});
            }
            required_deposit = decimal_times_uint128_trunc(odds_factor, order_backer_stake.amount)?;
        }
    }

    if sent_coin.amount != required_deposit {
        return Err(ContractError::InsufficientFundsSent { 
            required: required_deposit.to_string() + &config.betting_denom, 
            sent: sent_coin.amount.to_string() + &sent_coin.denom 
        });
    }
    
    let order_id = NEXT_ORDER_ID.update(deps.storage, |id| -> StdResult<_> { Ok(id + 1) })?;
    let order = Order {
        id: order_id,
        event_id,
        owner: info.sender.clone(),
        order_type,
        outcome,
        initial_backer_stake: order_backer_stake.clone(),
        remaining_backer_stake: order_backer_stake.clone(),
        odds,
        creation_time: env.block.time,
        status: OrderStatus::Open,
    };
    orders().save(deps.storage, order_id, &order)?;

    let match_results = try_match_order(deps.storage, env.clone(), order_id)?;
    
    let mut res = Response::new()
        .add_attribute("method", "place_order")
        .add_attribute("order_id", order_id.to_string())
        .add_attribute("event_id", event_id.to_string())
        .add_attribute("owner", info.sender.to_string())
        .add_attribute("order_type", format!("{:?}", order_type))
        .add_attribute("outcome", format!("{:?}", outcome))
        .add_attribute("backer_stake", order_backer_stake.amount.to_string())
        .add_attribute("odds", odds.to_string());

    for matched_bet_id in &match_results.newly_matched_bet_ids { // Iterate by reference
        res = res.add_attribute("matched_bet_id", matched_bet_id.to_string());
    }
    if match_results.order_fully_filled {
         res = res.add_attribute("order_status_after_match", "Filled");
    } else if !match_results.newly_matched_bet_ids.is_empty() { 
         res = res.add_attribute("order_status_after_match", "PartiallyFilled");
    } else {
         res = res.add_attribute("order_status_after_match", "Open");
    }

    Ok(res)
}

struct MatchResult {
    newly_matched_bet_ids: Vec<u64>,
    order_fully_filled: bool,
}

fn try_match_order(
    storage: &mut dyn Storage,
    env: Env,
    new_order_id: u64,
) -> Result<MatchResult, ContractError> {
    let mut newly_matched_bet_ids = Vec::new();
    let mut new_order = orders().load(storage, new_order_id)?;
    let config = CONFIG.load(storage)?;

    if new_order.status == OrderStatus::Filled || new_order.remaining_backer_stake.amount.is_zero() { 
        return Ok(MatchResult { newly_matched_bet_ids, order_fully_filled: true });
    }

    let counter_order_type = match new_order.order_type {
        OrderType::Back => OrderType::Lay,
        OrderType::Lay => OrderType::Back,
    };
    let outcome_u8 = match new_order.outcome { Outcome::Yes => 0, Outcome::No => 1 };
    
    // Prefix for sub_prefix is event_id (first part of the (u64, u8) index key)
    let sub_prefix_key_for_index = new_order.event_id; 
    
    let mut potential_matches_data: Vec<(u64, Order)> = Vec::new();
    for item_result in orders()
        .idx
        .event_outcome_params 
        .sub_prefix(sub_prefix_key_for_index) 
        .range(storage, None, None, CwOrder::Ascending) 
    {
        let (order_primary_key_u64, order_from_iterator) = item_result?; 
        
        let current_order_outcome_u8 = match order_from_iterator.outcome { Outcome::Yes => 0, Outcome::No => 1 };

        // Manual filtering for outcome (second part of index key), type, and odds
        if current_order_outcome_u8 == outcome_u8 && 
           order_from_iterator.order_type == counter_order_type &&
           order_from_iterator.odds == new_order.odds {
            
            if order_primary_key_u64 != new_order.id && 
               (order_from_iterator.status == OrderStatus::Open || order_from_iterator.status == OrderStatus::PartiallyFilled) &&
               order_from_iterator.owner != new_order.owner { 
                 potential_matches_data.push((order_primary_key_u64, order_from_iterator));
            }
        }
    }

    let mut matched_any_this_call = false;

    for (_existing_order_id, mut existing_order) in potential_matches_data { 
        if new_order.remaining_backer_stake.amount.is_zero() { break; } 
        
        let matchable_backer_stake_amount = new_order.remaining_backer_stake.amount.min(existing_order.remaining_backer_stake.amount);

        if matchable_backer_stake_amount.is_zero() {
            continue;
        }
        matched_any_this_call = true;

        new_order.remaining_backer_stake.amount = new_order.remaining_backer_stake.amount.checked_sub(matchable_backer_stake_amount)
            .map_err(|e: OverflowError| ContractError::CalculationError { msg: format!("New order stake sub overflow: {}", e) })?;
        existing_order.remaining_backer_stake.amount = existing_order.remaining_backer_stake.amount.checked_sub(matchable_backer_stake_amount)
            .map_err(|e: OverflowError| ContractError::CalculationError { msg: format!("Existing order stake sub overflow: {}", e) })?;

        new_order.status = if new_order.remaining_backer_stake.amount.is_zero() { OrderStatus::Filled } else { OrderStatus::PartiallyFilled };
        existing_order.status = if existing_order.remaining_backer_stake.amount.is_zero() { OrderStatus::Filled } else { OrderStatus::PartiallyFilled };
        
        orders().save(storage, existing_order.id, &existing_order)?;

        let bet_id = NEXT_BET_ID.update(storage, |id| -> StdResult<_> { Ok(id + 1) })?;
        
        let odds_factor_for_liability = new_order.odds.checked_sub(Decimal::one())
            .map_err(|e: OverflowError| ContractError::CalculationError {msg: format!("Liability odds factor error: {}", e)})?;

        let layer_liability_amount = decimal_times_uint128_trunc(odds_factor_for_liability, matchable_backer_stake_amount)?;

        let (backer_addr, lay_addr, backer_stake_coin, layer_liability_coin) = 
            if new_order.order_type == OrderType::Back { 
                (
                    new_order.owner.clone(),
                    existing_order.owner.clone(),
                    Coin { denom: config.betting_denom.clone(), amount: matchable_backer_stake_amount },
                    Coin { denom: config.betting_denom.clone(), amount: layer_liability_amount } 
                )
            } else { 
                (
                    existing_order.owner.clone(),
                    new_order.owner.clone(),
                    Coin { denom: config.betting_denom.clone(), amount: matchable_backer_stake_amount },
                    Coin { denom: config.betting_denom.clone(), amount: layer_liability_amount }
                )
            };

        let matched_bet = MatchedBet {
            id: bet_id,
            event_id: new_order.event_id,
            backer_addr,
            lay_addr,
            backer_stake: backer_stake_coin,
            layer_liability: layer_liability_coin,
            outcome_backed: new_order.outcome, 
            odds: new_order.odds,
            creation_time: env.block.time,
        };
        MATCHED_BETS.save(storage, bet_id, &matched_bet)?;
        
        EVENT_TO_MATCHED_BETS.update(storage, new_order.event_id, |bet_ids_opt| -> StdResult<_> {
            let mut ids = bet_ids_opt.unwrap_or_default();
            ids.push(bet_id);
            Ok(ids)
        })?;
        newly_matched_bet_ids.push(bet_id);
    }
    
    if matched_any_this_call { 
         orders().save(storage, new_order.id, &new_order)?;
    }
    
    let order_fully_filled = new_order.remaining_backer_stake.amount.is_zero();
    Ok(MatchResult { newly_matched_bet_ids, order_fully_filled })
}


fn execute_cancel_order(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    order_id: u64,
) -> Result<Response, ContractError> {
    let mut order = orders().load(deps.storage, order_id)
        .map_err(|_| ContractError::OrderNotFound { order_id })?;

    if order.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if order.status == OrderStatus::Filled {
        return Err(ContractError::CannotCancelFilledOrder { order_id });
    }
    if order.status == OrderStatus::Cancelled { 
        return Ok(Response::new().add_attribute("method", "cancel_order").add_attribute("status", "already_cancelled"));
    }
    
    let config = CONFIG.load(deps.storage)?;
    let mut refund_messages: Vec<CosmosMsg> = Vec::new();

    if !order.remaining_backer_stake.amount.is_zero() {
        let amount_to_refund: Uint128;
        match order.order_type {
            OrderType::Back => { 
                amount_to_refund = order.remaining_backer_stake.amount;
            }
            OrderType::Lay => { 
                let odds_factor = order.odds.checked_sub(Decimal::one())
                    .map_err(|e: OverflowError| ContractError::CalculationError { msg: format!("Cancel odds factor error: {}", e) })?;
                amount_to_refund = decimal_times_uint128_trunc(odds_factor, order.remaining_backer_stake.amount)?;
            }
        }
        if !amount_to_refund.is_zero() {
            refund_messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: order.owner.to_string(),
                amount: vec![Coin { denom: config.betting_denom.clone(), amount: amount_to_refund }],
            }));
        }
    }
    
    order.status = OrderStatus::Cancelled;
    orders().save(deps.storage, order_id, &order)?;

    Ok(Response::new()
        .add_messages(refund_messages)
        .add_attribute("method", "cancel_order")
        .add_attribute("order_id", order_id.to_string())
        .add_attribute("refunded_to", order.owner.to_string()))
}

fn execute_resolve_event(
    deps: DepsMut,
    _env: Env, 
    info: MessageInfo,
    event_id: u64,
    winning_outcome: Outcome,
) -> Result<Response, ContractError> {
    let mut event = EVENTS.load(deps.storage, event_id)
        .map_err(|_| ContractError::EventNotFound { event_id })?;

    if event.oracle != info.sender {
        return Err(ContractError::OracleMismatch { event_id });
    }
    if event.status == EventStatus::Resolved {
        return Err(ContractError::EventAlreadyResolved { event_id });
    }

    event.status = EventStatus::Resolved;
    event.winning_outcome = Some(winning_outcome);
    EVENTS.save(deps.storage, event_id, &event)?;

    let config = CONFIG.load(deps.storage)?;
    let mut payout_messages: Vec<CosmosMsg> = Vec::new();
    let mut response_attributes_map: Vec<(String, String)> = vec![
        ("method".to_string(), "resolve_event".to_string()),
        ("event_id".to_string(), event_id.to_string()),
        ("winning_outcome".to_string(), format!("{:?}", winning_outcome)),
    ];

    let bet_ids = EVENT_TO_MATCHED_BETS.load(deps.storage, event_id)?;
    for bet_id in bet_ids {
        let bet = MATCHED_BETS.load(deps.storage, bet_id)?;
        
        let winner_addr: Addr;
        let total_pot_amount = bet.backer_stake.amount.checked_add(bet.layer_liability.amount)
            .map_err(|e: OverflowError| ContractError::CalculationError { msg: format!("Resolve total pot overflow for bet {}: {}", bet_id, e) })?;
        let payout_coin = Coin { denom: config.betting_denom.clone(), amount: total_pot_amount };

        if bet.outcome_backed == winning_outcome { 
            winner_addr = bet.backer_addr;
        } else { 
            winner_addr = bet.lay_addr;
        }
        
        payout_messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: winner_addr.to_string(),
            amount: vec![payout_coin.clone()],
        }));
        response_attributes_map.push(("payout_bet_id".to_string(), bet_id.to_string()));
        response_attributes_map.push(("payout_winner".to_string(), winner_addr.to_string()));
        response_attributes_map.push(("payout_amount".to_string(), payout_coin.amount.to_string() + &payout_coin.denom));
    }

    let orders_to_process: Vec<Order> = orders()
        .range(deps.storage, None, None, CwOrder::Ascending)
        .filter_map(|res| res.ok().map(|(_order_id_u64, order_val)| order_val))
        .collect();
    
    for mut order in orders_to_process {
        if order.event_id == event_id && (order.status == OrderStatus::Open || order.status == OrderStatus::PartiallyFilled) {
            if !order.remaining_backer_stake.amount.is_zero() {
                let amount_to_refund: Uint128;
                match order.order_type {
                    OrderType::Back => amount_to_refund = order.remaining_backer_stake.amount,
                    OrderType::Lay => {
                        let odds_factor = order.odds.checked_sub(Decimal::one())
                            .map_err(|e: OverflowError| ContractError::CalculationError { msg: format!("Refund odds factor error for order {}: {}", order.id, e) })?;
                        amount_to_refund = decimal_times_uint128_trunc(odds_factor, order.remaining_backer_stake.amount)?; 
                    }
                }
                if !amount_to_refund.is_zero() {
                    payout_messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: order.owner.to_string(),
                        amount: vec![Coin { denom: config.betting_denom.clone(), amount: amount_to_refund }],
                    }));
                    response_attributes_map.push(("refunded_open_order_id".to_string(), order.id.to_string()));
                }
            }
            order.status = OrderStatus::Cancelled; 
            orders().save(deps.storage, order.id, &order)?;
        }
    }

    Ok(Response::new()
        .add_messages(payout_messages)
        .add_attributes(response_attributes_map))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GetEvent { event_id } => to_json_binary(&query_event(deps, event_id)?),
        QueryMsg::ListEvents { start_after, limit, filter_status } => 
            to_json_binary(&query_list_events(deps, start_after, limit, filter_status)?),
        QueryMsg::GetOrder { order_id } => to_json_binary(&query_order(deps, order_id)?),
        QueryMsg::ListOrdersByEvent { event_id, start_after, limit, filter_order_type, filter_outcome } => 
            to_json_binary(&query_list_orders_by_event(deps, event_id, start_after, limit, filter_order_type, filter_outcome)?),
        QueryMsg::ListMatchedBetsByEvent { event_id, start_after, limit } =>
            to_json_binary(&query_list_matched_bets_by_event(deps, event_id, start_after, limit)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let next_event_id = NEXT_EVENT_ID.load(deps.storage).unwrap_or(0); 
    let next_order_id = NEXT_ORDER_ID.load(deps.storage).unwrap_or(0);
    let next_bet_id = NEXT_BET_ID.load(deps.storage).unwrap_or(0);
    Ok(ConfigResponse {
        admin: config.admin,
        betting_denom: config.betting_denom,
        next_event_id,
        next_order_id,
        next_bet_id,
    })
}

fn query_event(deps: Deps, event_id: u64) -> StdResult<EventResponse> {
    let event = EVENTS.load(deps.storage, event_id)
        .map_err(|_| StdError::not_found(format!("event {}", event_id)))?;
    Ok(EventResponse { event })
}

fn query_list_events(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    filter_status: Option<EventStatus>,
) -> StdResult<EventsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive); 

    let events: Vec<Event> = EVENTS
        .range(deps.storage, start, None, CwOrder::Ascending)
        .filter_map(|item| {
            item.ok().and_then(|(_id_vec, event)| { 
                if filter_status.is_none() || Some(event.status) == filter_status {
                    Some(event)
                } else {
                    None
                }
            })
        })
        .take(limit)
        .collect();
    Ok(EventsResponse { events })
}

fn query_order(deps: Deps, order_id: u64) -> StdResult<OrderResponse> {
    let order = orders().load(deps.storage, order_id)
        .map_err(|_| StdError::not_found(format!("order {}", order_id)))?;
    Ok(OrderResponse { order })
}

fn query_list_orders_by_event(
    deps: Deps,
    event_id: u64,
    start_after: Option<u64>, 
    limit: Option<u32>,
    filter_order_type: Option<OrderType>,
    filter_outcome: Option<Outcome>,
) -> StdResult<OrdersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = start_after.map(Bound::exclusive); 
    
    let orders_list: Vec<Order> = orders()
        .range(deps.storage, start_bound, None, CwOrder::Ascending) 
        .filter_map(|item| {
            item.ok().and_then(|(_pk_vec, order)| { 
                if order.event_id == event_id &&
                   (filter_order_type.is_none() || Some(order.order_type) == filter_order_type) &&
                   (filter_outcome.is_none() || Some(order.outcome) == filter_outcome) &&
                   (order.status == OrderStatus::Open || order.status == OrderStatus::PartiallyFilled)
                {
                    Some(order)
                } else {
                    None
                }
            })
        })
        .take(limit)
        .collect();

    Ok(OrdersResponse { orders: orders_list })
}

fn query_list_matched_bets_by_event(
    deps: Deps,
    event_id: u64,
    start_after: Option<u64>, 
    limit: Option<u32>,
) -> StdResult<MatchedBetsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let bet_ids_for_event = EVENT_TO_MATCHED_BETS.may_load(deps.storage, event_id)?.unwrap_or_default();
    
    let relevant_bet_ids = bet_ids_for_event.into_iter()
        .filter(|&id| start_after.map_or(true, |sa| id > sa)) 
        .take(limit);

    let mut matched_bets_list: Vec<MatchedBet> = Vec::with_capacity(limit);
    for bet_id in relevant_bet_ids {
        if let Ok(bet) = MATCHED_BETS.load(deps.storage, bet_id) {
            matched_bets_list.push(bet);
        }
    }
    Ok(MatchedBetsResponse { matched_bets: matched_bets_list })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: InstantiateMsg) -> Result<Response, ContractError> {
    Ok(Response::default().add_attribute("method", "migrate"))
}