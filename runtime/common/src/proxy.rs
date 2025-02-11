/// Proxy commons for Pop runtimes
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::parameter_types;
use sp_runtime::RuntimeDebug;

use crate::{deposit, Balance};

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
	/// Smart contract proxy. Can execute calls related to smart contract management.
	SmartContract,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl ProxyType {
	/// Defines proxies permission hierarchy.
	// Example: A proxy that is not superset of another one won't be able to remove
	// that proxy relationship
	// src: https://github.com/paritytech/polkadot-sdk/blob/4cd07c56378291fddb9fceab3b508cf99034126a/substrate/frame/proxy/src/lib.rs#L802
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
