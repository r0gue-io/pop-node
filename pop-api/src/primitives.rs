use ink::env::{DefaultEnvironment, Environment};
pub use primitives::*;

pub(crate) type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub(crate) type Balance = <DefaultEnvironment as Environment>::Balance;
