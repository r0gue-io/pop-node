use ink::env::{DefaultEnvironment, Environment};
use ink::scale::Decode;
pub use pop_primitives::*;

pub(crate) type AccountId = <DefaultEnvironment as Environment>::AccountId;
pub(crate) type Balance = <DefaultEnvironment as Environment>::Balance;

/// Decode slice of bytes to environment associated type AccountId.
pub fn account_id_from_slice(s: &[u8; 32]) -> AccountId {
	AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}
