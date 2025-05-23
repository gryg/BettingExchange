use cosmwasm_std::{Addr, Coin, Decimal, Timestamp}; // Removed Uint128
use cw_storage_plus::{Item, Map, IndexedMap, MultiIndex, IndexList, Index};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::{EventStatus, OrderStatus, OrderType, Outcome};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub betting_denom: String, 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Event {
    pub id: u64,
    pub creator: Addr,
    pub description: String,
    pub oracle: Addr,
    pub status: EventStatus,
    pub winning_outcome: Option<Outcome>,
    pub resolution_deadline: Option<Timestamp>, 
    pub creation_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Order {
    pub id: u64,
    pub event_id: u64,
    pub owner: Addr,
    pub order_type: OrderType, 
    pub outcome: Outcome,      
    pub initial_backer_stake: Coin, 
    pub remaining_backer_stake: Coin, 
    pub odds: Decimal, 
    pub creation_time: Timestamp,
    pub status: OrderStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MatchedBet {
    pub id: u64,
    pub event_id: u64,
    pub backer_addr: Addr,
    pub lay_addr: Addr,
    pub backer_stake: Coin, 
    pub layer_liability: Coin, 
    pub outcome_backed: Outcome, 
    pub odds: Decimal, 
    pub creation_time: Timestamp,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_EVENT_ID: Item<u64> = Item::new("next_event_id");
pub const NEXT_ORDER_ID: Item<u64> = Item::new("next_order_id");
pub const NEXT_BET_ID: Item<u64> = Item::new("next_bet_id");
pub const EVENTS: Map<u64, Event> = Map::new("events");

pub struct OrderIndexes<'a> {
    pub event_outcome_params: MultiIndex<'a, (u64, u8), Order, u64>,
    pub owner: MultiIndex<'a, Addr, Order, u64>,
}

impl<'a> IndexList<Order> for OrderIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Order>> + '_> {
        let v: Vec<&dyn Index<Order>> = vec![&self.event_outcome_params, &self.owner];
        Box::new(v.into_iter())
    }
}

pub fn orders<'a>() -> IndexedMap<u64, Order, OrderIndexes<'a>> {
    let indexes = OrderIndexes {
        event_outcome_params: MultiIndex::new(
            |_pk: &[u8], o: &Order| { 
                let outcome_u8 = match o.outcome { Outcome::Yes => 0, Outcome::No => 1 };
                (o.event_id, outcome_u8)
            },
            "orders", 
            "orders__event_outcome_params",
        ),
        owner: MultiIndex::new(
            |_pk: &[u8], o: &Order| o.owner.clone(),
            "orders",
            "orders__owner",
        ),
    };
    IndexedMap::new("orders", indexes)
}

pub const MATCHED_BETS: Map<u64, MatchedBet> = Map::new("matched_bets");
pub const EVENT_TO_MATCHED_BETS: Map<u64, Vec<u64>> = Map::new("event_to_matched_bets");