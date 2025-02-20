#[cfg(not(feature = "std"))]
use alloc::format;
use alloc::vec::Vec;

use parachains_common::AccountId;
use polkadot_parachain_primitives::primitives::Sibling;
pub use serde_json::{json, to_string, Value};
pub use sp_keyring::sr25519::Keyring;
use sp_runtime::traits::AccountIdConversion;

/// Sovereign account of AssetHub on Pop.
pub fn asset_hub_sa_on_pop() -> AccountId {
	Sibling::from(1_000).into_account_truncating()
}

/// A set of dev accounts, typically used for endowments at genesis for development chains.
pub fn dev_accounts() -> Vec<AccountId> {
	Keyring::well_known().map(|k| k.to_account_id()).collect()
}

/// Derive a multisig key from a given set of `accounts` and a `threshold`.
pub fn derive_multisig<T: pallet_multisig::Config>(
	mut signatories: Vec<T::AccountId>,
	threshold: u16,
) -> T::AccountId {
	assert!(!signatories.is_empty(), "Signatories set cannot be empty");
	assert!(threshold > 0, "Threshold for multisig cannot be 0");
	assert!(
		signatories.len() >= threshold.into(),
		"Threshold must be less than or equal to the number of signatories"
	);
	// Sorting is done to deterministically order the multisig set
	// So that a single authority set (A, B, C) may generate only a single unique multisig key
	// Otherwise, (B, A, C) or (C, A, B) could produce different keys and cause chaos
	signatories.sort();

	// Derive a multisig with `threshold / signatories.len()` threshold
	pallet_multisig::Pallet::<T>::multi_account_id(&signatories[..], threshold)
}
