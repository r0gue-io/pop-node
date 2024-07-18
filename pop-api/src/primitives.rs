use ink::env::{DefaultEnvironment, Environment};

pub use pop_primitives::*;

pub(crate) type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub(crate) type Balance = <DefaultEnvironment as Environment>::Balance;
#[cfg(any(feature = "nfts", feature = "cross-chain"))]
type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
