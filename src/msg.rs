use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::QueryResponses; // Added for QueryResponses

use crate::state::{Event, MatchedBet, Order};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>, 
    pub betting_denom: String, 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateEvent {
        description: String,
        oracle_addr: Option<String>, 
        resolution_deadline: Option<Timestamp>, 
    },
    PlaceOrder {
        event_id: u64,
        order_type: OrderType,
        outcome: Outcome,
        stake: Uint128, 
        odds: Decimal,  
    },
    CancelOrder {
        order_id: u64,
    },
    ResolveEvent {
        event_id: u64,
        winning_outcome: Outcome,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)] // Added QueryResponses
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(EventResponse)]
    GetEvent {
        event_id: u64,
    },
    #[returns(EventsResponse)]
    ListEvents {
        start_after: Option<u64>, 
        limit: Option<u32>,
        filter_status: Option<EventStatus>,
    },
    #[returns(OrderResponse)]
    GetOrder {
        order_id: u64,
    },
    #[returns(OrdersResponse)]
    ListOrdersByEvent {
        event_id: u64,
        start_after: Option<u64>, 
        limit: Option<u32>,
        filter_order_type: Option<OrderType>,
        filter_outcome: Option<Outcome>,
    },
    #[returns(MatchedBetsResponse)]
    ListMatchedBetsByEvent {
        event_id: u64,
        start_after: Option<u64>, 
        limit: Option<u32>,
    },
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum Outcome {
    Yes,
    No,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum EventStatus {
    Open,      
    Resolved,  
    Cancelled, 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum OrderType {
    Back, 
    Lay,  
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Copy)]
pub enum OrderStatus {
    Open,            
    PartiallyFilled, 
    Filled,          
    Cancelled,       
}