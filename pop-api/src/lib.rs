#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::{chain_extension::FromStatusCode, DefaultEnvironment, Environment};
use primitives::error::Error;

#[cfg(feature = "assets")]
pub use v0::assets;
#[cfg(feature = "balances")]
pub use v0::balances;
#[cfg(feature = "cross-chain")]
pub use v0::cross_chain;
#[cfg(feature = "nfts")]
pub use v0::nfts;

pub mod primitives;
pub mod v0;

type AccountId = <DefaultEnvironment as Environment>::AccountId;
type Balance = <DefaultEnvironment as Environment>::Balance;
#[cfg(any(feature = "nfts", feature = "cross-chain"))]
type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct StatusCode(pub u32);

impl From<u32> for StatusCode {
	fn from(value: u32) -> Self {
		StatusCode(value)
	}
}
impl FromStatusCode for StatusCode {
	fn from_status_code(status_code: u32) -> Result<()> {
		match status_code {
			0 => Ok(()),
			_ => Err(StatusCode(status_code)),
		}
	}
}

impl From<ink::scale::Error> for StatusCode {
	fn from(_: ink::scale::Error) -> Self {
		StatusCode(255u32)
	}
}

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}

impl From<Error> for StatusCode {
	fn from(value: Error) -> Self {
		StatusCode::from(u32::from(value))
	}
}
