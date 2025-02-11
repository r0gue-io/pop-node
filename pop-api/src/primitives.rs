use ink::env::{DefaultEnvironment, Environment};
pub use pop_primitives::*;

// Public due to integration tests crate.
pub type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub type Balance = <DefaultEnvironment as Environment>::Balance;
pub type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
