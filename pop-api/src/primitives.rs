use ink::env::{DefaultEnvironment, Environment};
pub use pop_primitives::*;

/// Alias for contract environment account ID.
pub type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub(crate) type Balance = <DefaultEnvironment as Environment>::Balance;
