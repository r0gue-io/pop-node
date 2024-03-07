pub use crate::chains::{asset_hub_rococo, pop_network, rococo};
use crate::chains::{asset_hub_rococo::AssetHubRococo, pop_network::PopNetwork, rococo::Rococo};

// Cumulus
use emulated_integration_tests_common::{
    accounts::{ALICE, BOB},
    xcm_emulator::{decl_test_networks, decl_test_sender_receiver_accounts_parameter_types},
};

decl_test_networks! {
    pub struct RococoMockNet {
        relay_chain = Rococo,
        parachains = vec![
            AssetHubRococo,
            PopNetwork,
        ],
        bridge = ()
    },
}

decl_test_sender_receiver_accounts_parameter_types! {
    RococoRelay { sender: ALICE, receiver: BOB },
    AssetHubRococoPara { sender: ALICE, receiver: BOB },
    PopNetworkPara { sender: ALICE, receiver: BOB}
}
