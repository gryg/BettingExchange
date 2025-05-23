
// --- Enums (mirroring Rust enums) ---
export enum Outcome { Yes = "Yes", No = "No" }
export enum EventStatus { Open = "Open", Resolved = "Resolved", Cancelled = "Cancelled" }
export enum OrderType { Back = "Back", Lay = "Lay" }
export enum OrderStatus { Open = "Open", PartiallyFilled = "PartiallyFilled", Filled = "Filled", Cancelled = "Cancelled" }

// --- Basic Types ---
export interface Coin {
    denom: string;
    amount: string; // u128 is typically represented as a string in JSON
}

// --- State Structs (mirroring Rust state.rs structs) ---
export interface Config {
    admin: string; // Addr
    betting_denom: string;
}

export interface Event {
    id: string; // u64
    creator: string; // Addr
    description: string;
    oracle: string; // Addr
    status: EventStatus;
    winning_outcome?: Outcome | null; // Option<Outcome>
    resolution_deadline?: string | null; // Option<Timestamp> (u64 seconds)
    creation_time: string; // Timestamp (u64 seconds)
}

export interface Order {
    id: string; // u64
    event_id: string; // u64
    owner: string; // Addr
    order_type: OrderType;
    outcome: Outcome;
    initial_backer_stake: Coin;
    remaining_backer_stake: Coin;
    odds: string; // Decimal as string
    creation_time: string; // Timestamp (u64 seconds)
    status: OrderStatus;
}

export interface MatchedBet {
    id: string; // u64
    event_id: string; // u64
    backer_addr: string; // Addr
    lay_addr: string; // Addr
    backer_stake: Coin;
    layer_liability: Coin;
    outcome_backed: Outcome;
    odds: string; // Decimal as string
    creation_time: string; // Timestamp (u64 seconds)
}

// --- Query Message Response Types (mirroring Rust msg.rs response structs) ---
export interface ConfigResponse extends Config {
    next_event_id: string; // u64
    next_order_id: string; // u64
    next_bet_id: string; // u64
}

export interface EventResponse {
    event: Event;
}

export interface EventsResponse {
    events: Event[];
}

export interface OrderResponse {
    order: Order;
}

export interface OrdersResponse {
    orders: Order[];
}

export interface MatchedBetsResponse {
    matched_bets: MatchedBet[];
}

// --- Execute Message Parameter Types (for frontend forms) ---
export interface ExecuteCreateEventParams {
    description: string;
    oracleAddr?: string;         // Corresponds to oracle_addr: Option<String>
    resolutionDeadline?: string; // Corresponds to resolution_deadline: Option<Timestamp> (string of u64 seconds)
}

export interface ExecutePlaceOrderParams {
    eventId: string;    // u64
    orderType: OrderType;
    outcome: Outcome;
    stake: string;      // Uint128 as string
    odds: string;       // Decimal as string
}

export interface ExecuteCancelOrderParams {
    orderId: string;    // u64
}

export interface ExecuteResolveEventParams {
    eventId: string;    // u64
    winningOutcome: Outcome;
}