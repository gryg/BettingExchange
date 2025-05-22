use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("EventNotFound: Event with ID {event_id} not found")]
    EventNotFound { event_id: u64 },

    #[error("OrderNotFound: Order with ID {order_id} not found")]
    OrderNotFound { order_id: u64 },

    #[error("EventNotOpen: Event {event_id} is not open for betting")]
    EventNotOpen { event_id: u64 },

    #[error("EventAlreadyResolved: Event {event_id} has already been resolved")]
    EventAlreadyResolved { event_id: u64 },

    #[error("OracleMismatch: Sender is not the designated oracle for event {event_id}")]
    OracleMismatch { event_id: u64 },

    #[error("InvalidStakeAmount: Stake amount must be positive")]
    InvalidStakeAmount {},

    #[error("InsufficientFundsSent: Required {required}, Sent {sent}")]
    InsufficientFundsSent { required: String, sent: String },

    #[error("InvalidOdds: Odds must be greater than 1.0")]
    InvalidOdds {},

    #[error("CannotCancelFilledOrder: Order {order_id} has been fully matched")]
    CannotCancelFilledOrder { order_id: u64 },

    #[error("DeadlinePassed: The resolution deadline has passed")]
    DeadlinePassed {},

    #[error("InvalidDescription: Description cannot be empty")]
    InvalidDescription {},

    #[error("IdenticalOracleAndCreator: Oracle cannot be the same as the creator if explicitly set to a different address (for future external oracle integrations)")]
    IdenticalOracleAndCreator {},

    #[error("InvalidOrderTypeForMatching: Order types are not compatible for matching")]
    InvalidOrderTypeForMatching {},

    #[error("OddsMismatchForMatching: Odds must be identical for matching in this MVP")]
    OddsMismatchForMatching {},

    #[error("OutcomeMismatchForMatching: Outcomes must be identical for matching")]
    OutcomeMismatchForMatching {},

    #[error("NoFundsSent: You must send funds with this operation")]
    NoFundsSent {},

    #[error("MultipleCoinsSent: Only one type of coin is supported for staking")]
    MultipleCoinsSent {},

    #[error("InvalidDenom: Invalid currency denom received. Expected: {expected_denom}, Got: {received_denom}")]
    InvalidDenom { expected_denom: String, received_denom: String },

    #[error("CalculationError: {msg}")]
    CalculationError { msg: String },

    #[error("MigrationError: {msg}")]
    MigrationError { msg: String },
}