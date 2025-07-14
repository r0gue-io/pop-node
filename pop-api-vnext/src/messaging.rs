pub use ink::primitives::Weight;
use ink::{
	contract_ref,
	env::hash::{Blake2x256, CryptoHash},
	prelude::vec::Vec,
	primitives::AccountId,
	scale::{Compact, Encode},
	U256,
};
use sol::{Sol, SolDecode, SolEncode};
pub use v0::*;

use super::*;

/// APIs for messaging using the Interoperable State Machine Protocol (ISMP).
pub mod ismp;
/// APIs for messaging using Polkadot's Cross-Consensus Messaging (XCM).
pub mod xcm;

/// The first version of the Messaging API.
pub mod v0;

pub type Bytes = Vec<u8>;
pub type MessageId = u64;

// todo: docs
pub fn hashed_account(para_id: u32, account_id: AccountId) -> AccountId {
	let location = (
		b"SiblingChain",
		Compact::<u32>::from(para_id),
		Encode::encode(&(b"AccountId32", account_id.0)),
	)
		.encode();
	let mut output = [0u8; 32];
	Blake2x256::hash(&location, &mut output);
	AccountId::from(output)
}

#[test]
fn hashed_account_works() {
	let account_id: [u8; 32] = [
		27, 2, 24, 17, 104, 5, 173, 98, 25, 32, 36, 0, 82, 159, 11, 212, 178, 11, 39, 219, 14, 178,
		226, 179, 216, 62, 19, 85, 226, 17, 80, 179,
	];
	let location = (
		b"SiblingChain",
		Compact::<u32>::from(4001),
		Encode::encode(&(b"AccountId32", account_id)),
	)
		.encode();
	let mut output = [0u8; 32];
	Blake2x256::hash(&location, &mut output);
	println!("{output:?}")
}
