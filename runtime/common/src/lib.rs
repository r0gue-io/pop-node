#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime::Perbill;

use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight};

// Cumulus types re-export
// These types are shared between the devnet and testnet runtimes
pub use parachains_common::{AccountId, AuraId, Balance, Block, BlockNumber, Hash, Signature};
pub use polkadot_primitives::MAX_POV_SIZE;

/// Nonce for an account
pub type Nonce = u32;

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
// Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

/// Relay chain slot duration, in milliseconds.
// Value is 6000 millisecs. If `MILLISECS_PER_BLOCK` changes this needs addressing.
pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6000;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 2 seconds of compute with a 6-second average block.
pub const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), MAX_POV_SIZE as u64);

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 10_000_000_000; // 10 decimals

pub const MILLIUNIT: Balance = UNIT / 1_000;
pub const MICROUNIT: Balance = UNIT / 1_000_000;

// Deposits
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance * UNIT + (bytes as Balance) * (5 * MILLIUNIT / 100)) / 10
}
/// The existential deposit. Set to 1/1_000 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

// Async backing
/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;

/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
pub const BLOCK_PROCESSING_VELOCITY: u32 = 1;

/// Proxy commons for Pop runtimes
pub mod proxy {

	use super::{deposit, Balance};
	use codec::{Decode, Encode, MaxEncodedLen};
	use frame_support::parameter_types;
	use sp_runtime::RuntimeDebug;

	parameter_types! {
		// One storage item; key size 32, value size 8; .
		pub const ProxyDepositBase: Balance = deposit(1, 40);
		// Additional storage item size of 33 bytes.
		pub const ProxyDepositFactor: Balance = deposit(0, 33);
		pub const MaxProxies: u16 = 32;
		// One storage item; key size 32, value size 16
		pub const AnnouncementDepositBase: Balance = deposit(1, 48);
		pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
		pub const MaxPending: u16 = 32;
	}

	/// The type used to represent the kinds of proxying allowed.
	#[derive(
		Copy,
		Clone,
		Eq,
		PartialEq,
		Ord,
		PartialOrd,
		Encode,
		Decode,
		RuntimeDebug,
		MaxEncodedLen,
		scale_info::TypeInfo,
	)]
	pub enum ProxyType {
		/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
		Any,
		/// Can execute any call that does not transfer funds or assets.
		NonTransfer,
		/// Proxy with the ability to reject time-delay proxy announcements.
		CancelProxy,
		/// Assets proxy. Can execute any call from `assets`, **including asset transfers**.
		Assets,
		/// Owner proxy. Can execute calls related to asset ownership.
		AssetOwner,
		/// Asset manager. Can execute calls related to asset management.
		AssetManager,
		/// Collator selection proxy. Can execute calls related to collator selection mechanism.
		Collator,
	}
	impl Default for ProxyType {
		fn default() -> Self {
			Self::Any
		}
	}

	impl ProxyType {
		pub fn is_superset(s: &ProxyType, o: &ProxyType) -> bool {
			match (s, o) {
				(x, y) if x == y => true,
				(ProxyType::Any, _) => true,
				(_, ProxyType::Any) => false,
				(ProxyType::Assets, ProxyType::AssetOwner) => true,
				(ProxyType::Assets, ProxyType::AssetManager) => true,
				(ProxyType::NonTransfer, ProxyType::Collator) => true,
				_ => false,
			}
		}
	}
}
