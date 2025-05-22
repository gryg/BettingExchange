use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Event, MatchedBet, Order};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>, // Contract administrator
    pub betting_denom: String, // The denomination used for betting (e.g., "inj" or a sub-unit)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateEvent {
        description: String,
        oracle_addr: Option<String>, // If None, defaults to sender
        resolution_deadline: Option<Timestamp>, // Optional deadline for resolution
    },
    PlaceOrder {
        event_id: u64,
        order_type: OrderType,
        outcome: Outcome,
        stake: Uint128, // For Back: amount betting. For Lay: amount of liability willing to cover.
        odds: Decimal,  // e.g., 2.5
    },
    CancelOrder {
        order_id: u64,
    },
    ResolveEvent {
        event_id: u64,
        winning_outcome: Outcome,
    },
    // Future: UpdateConfig { admin: Option<String>, betting_denom: Option<String> }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetEvent {
        event_id: u64,
    },
    ListEvents {
        start_after: Option<u64>, // Event ID to start after
        limit: Option<u32>,
        filter_status: Option<EventStatus>,
    },
    GetOrder {
        order_id: u64,
    },
    ListOrdersByEvent {
        event_id: u64,
        start_after: Option<u64>, // Order ID to start after
        limit: Option<u32>,
        filter_order_type: Option<OrderType>,
        filter_outcome: Option<Outcome>,
    },
    ListMatchedBetsByEvent {
        event_id: u64,
        start_after: Option<u64>, // Bet ID to start after
        limit: Option<u32>,
    },
    // Query user's open orders, matched bets etc. could be added
}

// Responses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: Addr,
    pub betting_denom: String,
    pub next_event_id: u64,
    pub next_order_id: u64,
    pub next_bet_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EventResponse {
    pub event: Event,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EventsResponse {
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderResponse {
    pub order: Order,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrdersResponse {
    pub orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MatchedBetResponse {
    pub matched_bet: MatchedBet,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MatchedBetsResponse {
    pub matched_bets: Vec<MatchedBet>,
}


// Shared Enums and Structs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum Outcome {
    Yes,
    No,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum EventStatus {
    Open,      // Accepting bets
    Resolved,  // Outcome declared, bets settled or being settled
    Cancelled, // Event cancelled, funds returned (Future improvement)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum OrderType {
    Back, // Betting that an outcome WILL happen
    Lay,  // Betting that an outcome WILL NOT happen (acting as bookie for that specific outcome)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum OrderStatus {
    Open,            // Available for matching
    PartiallyFilled, // Partially matched
    Filled,          // Fully matched
    Cancelled,       // Cancelled by user
}