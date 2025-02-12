#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};
// Cumulus types re-export
// These types are shared between the devnet and testnet runtimes
pub use parachains_common::{AccountId, AuraId, Balance, Block, BlockNumber, Hash, Signature};
pub use polkadot_primitives::MAX_POV_SIZE;
use sp_runtime::Perbill;

pub mod proxy;

/// Nonce for an account
pub type Nonce = u32;

#[docify::export]
mod block_times {
	/// This determines the average expected block time that we are targeting.
	/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
	/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
	/// up by `pallet_aura` to implement `fn slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 6000;
	/// The duration of a slot.
	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	// Attempting to do so will brick block production.
	pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
}
pub use block_times::*;

// Time is measured by number of blocks.
/// A minute, measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
/// An hour, measured by number of blocks.
pub const HOURS: BlockNumber = MINUTES * 60;
/// A day, measured by number of blocks.
pub const DAYS: BlockNumber = HOURS * 24;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 2 seconds of compute with a 6-second average block.
#[docify::export(max_block_weight)]
pub const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), MAX_POV_SIZE as u64);

/// A unit of the native asset.
pub const UNIT: Balance = 10_000_000_000; // 10 decimals
/// A milli-unit of the native asset.
pub const MILLI_UNIT: Balance = UNIT / 1_000; // 10_000_000
/// A micro-unit of the native asset.
pub const MICRO_UNIT: Balance = UNIT / 1_000_000; // 10_000

/// Deposits.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance * UNIT + (bytes as Balance) * (5 * MILLI_UNIT / 100)) / 10
}
/// The existential deposit. Set to 1/1_000 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLI_UNIT;

#[docify::export]
mod async_backing_params {
	/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
	/// into the relay chain.
	pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;
	/// How many parachain blocks are processed by the relay chain per parent. Limits the
	/// number of blocks authored per slot.
	pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;
	/// Relay chain slot duration, in milliseconds.
	// Value is 6000 millisecs. If `MILLISECS_PER_BLOCK` changes this needs addressing.
	pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;
}
pub use async_backing_params::*;
