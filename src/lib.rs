pub mod contract;
mod error;
pub mod integration_tests; 
pub mod msg;
pub mod state;


pub use crate::error::ContractError;

#[cfg(test)] // Ensures this module is only compiled for tests
pub mod tests;  // This tells Rust to look for src/tests.rs or src/tests/mod.rs