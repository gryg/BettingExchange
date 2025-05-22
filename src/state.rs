use cosmwasm_std::{Addr, Coin, Decimal, Timestamp, Uint128};
use cw_storage_plus::{Item, Map, IndexedMap, MultiIndex, IndexList, Index};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::{EventStatus, OrderStatus, OrderType, Outcome};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub betting_denom: String, // e.g., "inj"
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Event {
    pub id: u64,
    pub creator: Addr,
    pub description: String,
    pub oracle: Addr,
    pub status: EventStatus,
    pub winning_outcome: Option<Outcome>,
    pub resolution_deadline: Option<Timestamp>, // Optional deadline
    pub creation_time: Timestamp,
    // To track total volume or matched amounts if needed
    // pub total_volume: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Order {
    pub id: u64,
    pub event_id: u64,
    pub owner: Addr,
    pub order_type: OrderType, // Back or Lay
    pub outcome: Outcome,      // The outcome this order pertains to (e.g., YES)
    
    // If Back: initial_stake_backing is the amount user wants to bet.
    // If Lay: initial_stake_backing is the equivalent backer's stake the layer is willing to match.
    //          The actual funds deposited by layer is initial_stake_backing * (odds - 1).
    pub initial_backer_stake: Coin, 
    pub remaining_backer_stake: Coin, // How much of the initial_backer_stake is available to be matched

    pub odds: Decimal, // e.g., 2.5 implies win 1.5x stake on top of getting stake back.
    pub creation_time: Timestamp,
    pub status: OrderStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MatchedBet {
    pub id: u64,
    pub event_id: u64,
    
    pub backer_addr: Addr,
    pub lay_addr: Addr,
    
    pub backer_stake: Coin, // Amount the backer staked
    pub layer_liability: Coin, // Amount the layer risked (backer_stake * (odds - 1))
    
    pub outcome_backed: Outcome, // The outcome the backer supported
    pub odds: Decimal, // Matched odds
    pub creation_time: Timestamp,
}

// Storage Items
pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_EVENT_ID: Item<u64> = Item::new("next_event_id");
pub const NEXT_ORDER_ID: Item<u64> = Item::new("next_order_id");
pub const NEXT_BET_ID: Item<u64> = Item::new("next_bet_id");

pub const EVENTS: Map<u64, Event> = Map::new("events");

// Orders are indexed for efficient querying during matching.
// Primary key: order_id (u64)
// Indexes:
// - event_id, outcome, order_type, odds (as string for exact match)
// - owner
pub struct OrderIndexes<'a> {
    // (event_id, outcome (0 for Yes, 1 for No), order_type (0 for Back, 1 for Lay), odds_string, order_id)
    pub event_match_params: MultiIndex<'a, (u64, u8, u8, String, u64), Order, u64>,
    pub owner: MultiIndex<'a, (Addr, u64), Order, u64>,
}

impl<'a> IndexList<Order> for OrderIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Order>>> {
        let v: Vec<&dyn Index<Order>> = vec![&self.event_match_params, &self.owner];
        Box::new(v.into_iter())
    }
}

pub fn orders<'a>() -> IndexedMap<'a, u64, Order, OrderIndexes<'a>> {
    let indexes = OrderIndexes {
        event_match_params: MultiIndex::new(
            |o: &Order, pk: Vec<u8>| {
                let outcome_u8 = match o.outcome { Outcome::Yes => 0, Outcome::No => 1 };
                let order_type_u8 = match o.order_type { OrderType::Back => 0, OrderType::Lay => 1 };
                // Odds to string for exact matching. Consider precision.
                (o.event_id, outcome_u8, order_type_u8, o.odds.to_string(), pk_to_u64(&pk))
            },
            "orders",
            "orders__event_match_params",
        ),
        owner: MultiIndex::new(
            |o: &Order, pk: Vec<u8>| (o.owner.clone(), pk_to_u64(&pk)),
            "orders",
            "orders__owner",
        ),
    };
    IndexedMap::new("orders", indexes)
}

// Helper to convert primary key bytes to u64 for multi-index
fn pk_to_u64(pk: &[u8]) -> u64 {
    pk[0..8].try_into().map(u64::from_be_bytes).unwrap_or_default()
}


// Matched bets, could be indexed by event_id if queried frequently standalone
// Or simply list all and filter in contract logic / client-side for MVP
pub const MATCHED_BETS: Map<u64, MatchedBet> = Map::new("matched_bets");

// For resolving, it might be useful to store which bets belong to an event
// Map<event_id, Vec<bet_id>>
pub const EVENT_TO_MATCHED_BETS: Map<u64, Vec<u64>> = Map::new("event_to_matched_bets");
// And which orders belong to an event
// pub const EVENT_TO_ORDERS: Map<u64, Vec<u64>> = Map::new("event_to_orders");
// However, iterating with `prefix` on IndexedMap is better for orders.