mod chains;
mod rococo_network;

pub use codec::Encode;

// Substrate
pub use frame_support::{
	assert_err, assert_ok,
	pallet_prelude::Weight,
	sp_runtime::{AccountId32, DispatchError, DispatchResult},
	traits::fungibles::Inspect,
};

// Polkadot
pub use xcm::{
	prelude::{AccountId32 as AccountId32Junction, *},
	v3::{self, Error, NetworkId::Rococo as RococoId},
};

// Cumulus
pub use crate::rococo_network::{
	asset_hub_rococo::{
		genesis::ED as ASSET_HUB_ROCOCO_ED, AssetHubRococoParaPallet as AssetHubRococoPallet,
	},
	rococo::{genesis::ED as ROCOCO_ED, RococoRelayPallet as RococoPallet},
	AssetHubRococoPara as AssetHubRococo, AssetHubRococoParaReceiver as AssetHubRococoReceiver,
	AssetHubRococoParaSender as AssetHubRococoSender, RococoRelay as Rococo,
	RococoRelayReceiver as RococoReceiver, RococoRelaySender as RococoSender,
};
pub use asset_test_utils::xcm_helpers;
pub use emulated_integration_tests_common::{
	test_parachain_is_trusted_teleporter,
	xcm_emulator::{
		assert_expected_events, bx, helpers::weight_within_threshold, Chain, Parachain as Para,
		RelayChain as Relay, Test, TestArgs, TestContext, TestExt,
	},
	xcm_helpers::{xcm_transact_paid_execution, xcm_transact_unpaid_execution},
	PROOF_SIZE_THRESHOLD, REF_TIME_THRESHOLD, XCM_V3,
};
pub use parachains_common::{AccountId, Balance};

pub const ASSET_ID: u32 = 1;
pub const ASSET_MIN_BALANCE: u128 = 1000;
// `Assets` pallet index
pub const ASSETS_PALLET_ID: u8 = 50;

pub type RelayToSystemParaTest = Test<Rococo, AssetHubRococo>;
pub type SystemParaToRelayTest = Test<AssetHubRococo, Rococo>;

#[cfg(test)]
mod tests;
