#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::chain_extension::FromStatusCode;

use constants::DECODING_FAILED;

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

pub type Result<T> = core::result::Result<T, StatusCode>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct StatusCode(pub u32);

mod constants {
	// Errors:
	pub(crate) const DECODING_FAILED: u32 = 255;
	pub(crate) const MODULE_ERROR: u8 = 3;

	// Function IDs:
	pub(crate) const DISPATCH: u8 = 0;
	pub(crate) const READ_STATE: u8 = 1;

	// Modules:
	pub(crate) const ASSETS_MODULE: u8 = 52;
	pub(crate) const BALANCES_MODULE: u8 = 10;
}

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
		StatusCode(DECODING_FAILED)
	}
}
