#![cfg_attr(not(feature = "std"), no_std)]

pub use extension::Extension;
use frame_support::pallet_prelude::Weight;

pub mod extension;
pub mod fungibles;
pub mod messaging;
#[cfg(test)]
mod mock;
pub mod nonfungibles;

/// Trait for performing reads of runtime state.
pub trait Read {
	/// The type of read requested.
	type Read;
	/// The type or result returned.
	type Result;

	/// Determines the weight of the requested read, used to charge the appropriate weight before
	/// the read is performed.
	///
	/// # Parameters
	/// - `request` - The read request.
	fn weight(read: &Self::Read) -> Weight;

	/// Performs the requested read and returns the result.
	///
	/// # Parameters
	/// - `request` - The read request.
	fn read(request: Self::Read) -> Self::Result;
}
