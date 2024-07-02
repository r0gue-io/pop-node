#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{scale::{Decode, Encode}, env::{chain_extension::FromStatusCode, DefaultEnvironment, Environment}};
pub use sp_runtime::MultiAddress;

#[cfg(feature = "assets")]
pub use v0::assets;
#[cfg(feature = "balances")]
pub use v0::balances;
#[cfg(feature = "cross-chain")]
pub use v0::cross_chain;
#[cfg(feature = "nfts")]
pub use v0::nfts;

pub mod error;
pub mod primitives;
pub mod v0;

type AccountId = <DefaultEnvironment as Environment>::AccountId;
type Balance = <DefaultEnvironment as Environment>::Balance;
// #[cfg(any(feature = "nfts", feature = "cross-chain"))]
// type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[derive(Debug, PartialEq, Eq, Clone)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct StatusCode(pub u32);

impl From<u32> for StatusCode {
	fn from(value: u32) -> Self {
		StatusCode(value)
	}
}
impl FromStatusCode for StatusCode {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		match status_code {
            0 => Ok(()),
            _ => Err(StatusCode(status_code))
        }
	}
}

impl From<ink::scale::Error> for StatusCode {
	fn from(_: ink::scale::Error) -> Self {
		StatusCode(255u32)
	}
}
