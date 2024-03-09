use chains::{asset_hub_rococo::AssetHubRococo, pop_network::PopNetwork, rococo::Rococo};
use emulated_integration_tests_common::xcm_emulator::decl_test_networks;

#[cfg(test)]
mod chains;

decl_test_networks! {
	// `pub` mandatory for the macro
	pub struct RococoMockNet {
		relay_chain = Rococo,
		parachains = vec![
			AssetHubRococo,
			PopNetwork,
		],
		bridge = ()
	},
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::chains::{
		asset_hub_rococo::{genesis::ED as ASSET_HUB_ROCOCO_ED, AssetHubRococoParaPallet},
		pop_network::PopNetworkParaPallet,
		rococo::{genesis::ED as ROCOCO_ED, RococoRelayPallet},
	};
	use asset_hub_rococo_runtime::xcm_config::XcmConfig as AssetHubRococoXcmConfig;
	use asset_test_utils::xcm_helpers;
	use emulated_integration_tests_common::{
		accounts::{ALICE, BOB},
		xcm_emulator::{
			assert_expected_events, bx, decl_test_sender_receiver_accounts_parameter_types, Chain,
			Parachain as Para, RelayChain as Relay, Test, TestArgs, TestContext, TestExt,
		},
	};
	use frame_support::{pallet_prelude::Weight, sp_runtime::DispatchResult};
	use pop_runtime::{xcm_config::XcmConfig as PopNetworkXcmConfig, Balance};
	use rococo_runtime::xcm_config::XcmConfig as RococoXcmConfig;
	use xcm::prelude::*;

	decl_test_sender_receiver_accounts_parameter_types! {
		RococoRelay { sender: ALICE, receiver: BOB },
		AssetHubRococoPara { sender: ALICE, receiver: BOB },
		PopNetworkPara { sender: ALICE, receiver: BOB}
	}

	type RelayToParaTest = Test<RococoRelay, PopNetworkPara>;
	type SystemParaToParaTest = Test<AssetHubRococoPara, PopNetworkPara>;
	type ParaToSystemParaTest = Test<PopNetworkPara, AssetHubRococoPara>;
	type ParaToRelayTest = Test<PopNetworkPara, RococoRelay>;

	fn relay_to_para_sender_assertions(t: RelayToParaTest) {
		type RuntimeEvent = <RococoRelay as Chain>::RuntimeEvent;
		RococoRelay::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
			864_610_000,
			8_799,
		)));
		assert_expected_events!(
			RococoRelay,
			vec![
				// Amount to reserve transfer is transferred to Parachain's Sovereign account
				RuntimeEvent::Balances(
					pallet_balances::Event::Transfer { from, to, amount }
				) => {
					from: *from == t.sender.account_id,
					to: *to == RococoRelay::sovereign_account_id_of(
						t.args.dest.clone()
					),
					amount: *amount == t.args.amount,
				},
			]
		);
	}

	fn system_para_to_para_sender_assertions(t: SystemParaToParaTest) {
		type RuntimeEvent = <AssetHubRococoPara as Chain>::RuntimeEvent;
		AssetHubRococoPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
			864_610_000,
			8_799,
		)));
		assert_expected_events!(
			AssetHubRococoPara,
			vec![
				// Amount to reserve transfer is transferred to Parachain's Sovereign account
				RuntimeEvent::Balances(
					pallet_balances::Event::Transfer { from, to, amount }
				) => {
					from: *from == t.sender.account_id,
					to: *to == AssetHubRococoPara::sovereign_account_id_of(
						t.args.dest.clone()
					),
					amount: *amount == t.args.amount,
				},
			]
		);
	}

	fn para_receiver_assertions<Test>(_: Test) {
		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
		assert_expected_events!(
			PopNetworkPara,
			vec![
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { .. }) => {},
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	}

	fn para_to_system_para_sender_assertions(t: ParaToSystemParaTest) {
		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
		PopNetworkPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
			864_610_000,
			8_799,
		)));
		assert_expected_events!(
			PopNetworkPara,
			vec![
				// Amount to reserve transfer is transferred to Parachain's Sovereign account
				RuntimeEvent::Balances(
					pallet_balances::Event::Withdraw { who, amount }
				) => {
					who: *who == t.sender.account_id,
					amount: *amount == t.args.amount,
				},
			]
		);
	}

	fn para_to_relay_sender_assertions(t: ParaToRelayTest) {
		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
		PopNetworkPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
			864_610_000,
			8_799,
		)));
		assert_expected_events!(
			PopNetworkPara,
			vec![
				// Amount to reserve transfer is transferred to Parachain's Sovereign account
				RuntimeEvent::Balances(
					pallet_balances::Event::Withdraw { who, amount }
				) => {
					who: *who == t.sender.account_id,
					amount: *amount == t.args.amount,
				},
			]
		);
	}

	fn para_to_system_para_receiver_assertions(t: ParaToSystemParaTest) {
		type RuntimeEvent = <AssetHubRococoPara as Chain>::RuntimeEvent;
		let sov_pop_net_on_ahr = AssetHubRococoPara::sovereign_account_id_of(
			AssetHubRococoPara::sibling_location_of(PopNetworkPara::para_id()),
		);
		assert_expected_events!(
			AssetHubRococoPara,
			vec![
				// Amount to reserve transfer is withdrawn from Parachain's Sovereign account
				RuntimeEvent::Balances(
					pallet_balances::Event::Withdraw { who, amount }
				) => {
					who: *who == sov_pop_net_on_ahr.clone().into(),
					amount: *amount == t.args.amount,
				},
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { .. }) => {},
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	}

	fn para_to_relay_receiver_assertions(t: ParaToRelayTest) {
		type RuntimeEvent = <RococoRelay as Chain>::RuntimeEvent;
		let sov_pop_net_on_relay = RococoRelay::sovereign_account_id_of(
			RococoRelay::child_location_of(PopNetworkPara::para_id()),
		);
		assert_expected_events!(
			RococoRelay,
			vec![
				// Amount to reserve transfer is withdrawn from Parachain's Sovereign account
				RuntimeEvent::Balances(
					pallet_balances::Event::Withdraw { who, amount }
				) => {
					who: *who == sov_pop_net_on_relay.clone().into(),
					amount: *amount == t.args.amount,
				},
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { .. }) => {},
				RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				) => {},
			]
		);
	}

	fn relay_to_para_reserve_transfer_assets(t: RelayToParaTest) -> DispatchResult {
		<RococoRelay as RococoRelayPallet>::XcmPallet::limited_reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest.into()),
			bx!(t.args.beneficiary.into()),
			bx!(t.args.assets.into()),
			t.args.fee_asset_item,
			t.args.weight_limit,
		)
	}

	fn system_para_to_para_reserve_transfer_assets(t: SystemParaToParaTest) -> DispatchResult {
		<AssetHubRococoPara as AssetHubRococoParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
            t.signed_origin,
            bx!(t.args.dest.into()),
            bx!(t.args.beneficiary.into()),
            bx!(t.args.assets.into()),
            t.args.fee_asset_item,
            t.args.weight_limit,
        )
	}

	fn para_to_system_para_reserve_transfer_assets(t: ParaToSystemParaTest) -> DispatchResult {
		<PopNetworkPara as PopNetworkParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest.into()),
			bx!(t.args.beneficiary.into()),
			bx!(t.args.assets.into()),
			t.args.fee_asset_item,
			t.args.weight_limit,
		)
	}

	fn para_to_relay_reserve_transfer_assets(t: ParaToRelayTest) -> DispatchResult {
		<PopNetworkPara as PopNetworkParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
			t.signed_origin,
			bx!(t.args.dest.into()),
			bx!(t.args.beneficiary.into()),
			bx!(t.args.assets.into()),
			t.args.fee_asset_item,
			t.args.weight_limit,
		)
	}

	/// Reserve Transfers of native asset from Relay to Parachain should work
	#[test]
	fn reserve_transfer_native_asset_from_relay_to_para() {
		// Init values for Relay
		let destination = RococoRelay::child_location_of(PopNetworkPara::para_id());
		let beneficiary_id = PopNetworkParaReceiver::get();
		let amount_to_send: Balance = ROCOCO_ED * 1000;

		let test_args = TestContext {
			sender: RococoRelaySender::get(),
			receiver: PopNetworkParaReceiver::get(),
			args: TestArgs::new_relay(destination, beneficiary_id, amount_to_send),
		};

		let mut test = RelayToParaTest::new(test_args);

		let sender_balance_before = test.sender.balance;
		let receiver_balance_before = test.receiver.balance;

		test.set_assertion::<RococoRelay>(relay_to_para_sender_assertions);
		test.set_assertion::<PopNetworkPara>(para_receiver_assertions);
		test.set_dispatchable::<RococoRelay>(relay_to_para_reserve_transfer_assets);
		test.assert();

		let delivery_fees = RococoRelay::execute_with(|| {
			xcm_helpers::transfer_assets_delivery_fees::<
				<RococoXcmConfig as xcm_executor::Config>::XcmSender,
			>(
				test.args.assets.clone(),
				0,
				test.args.weight_limit,
				test.args.beneficiary,
				test.args.dest,
			)
		});

		let sender_balance_after = test.sender.balance;
		let receiver_balance_after = test.receiver.balance;

		// Sender's balance is reduced
		assert_eq!(sender_balance_before - amount_to_send - delivery_fees, sender_balance_after);
		// Receiver's balance is increased
		assert!(receiver_balance_after > receiver_balance_before);
		// Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
		// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown
		// but should be non-zero
		assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
	}

	/// Reserve Transfers of native asset from Parachain to Relay should work
	#[test]
	fn reserve_transfer_native_asset_from_para_to_relay() {
		// Setup: reserve transfer from relay to Pop, so that sovereign account accurate for return transfer
		let amount_to_send: Balance = ROCOCO_ED * 1000;
		{
			let destination = RococoRelay::child_location_of(PopNetworkPara::para_id());
			let beneficiary_id = PopNetworkParaReceiver::get();

			let test_args = TestContext {
				sender: RococoRelaySender::get(),
				receiver: PopNetworkParaReceiver::get(),
				args: TestArgs::new_relay(destination, beneficiary_id, amount_to_send),
			};

			let mut test = RelayToParaTest::new(test_args);
			test.set_dispatchable::<RococoRelay>(relay_to_para_reserve_transfer_assets);
			test.assert();
		}

		// Init values for Pop Network Parachain
		let destination = PopNetworkPara::parent_location();
		let beneficiary_id = RococoRelayReceiver::get();
		let assets = (Parent, amount_to_send).into();

		let test_args = TestContext {
			sender: PopNetworkParaSender::get(),
			receiver: RococoRelayReceiver::get(),
			args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
		};

		let mut test = ParaToRelayTest::new(test_args);

		let sender_balance_before = test.sender.balance;
		let receiver_balance_before = test.receiver.balance;

		test.set_assertion::<PopNetworkPara>(para_to_relay_sender_assertions);
		test.set_assertion::<RococoRelay>(para_to_relay_receiver_assertions);
		test.set_dispatchable::<PopNetworkPara>(para_to_relay_reserve_transfer_assets);
		test.assert();

		let sender_balance_after = test.sender.balance;
		let receiver_balance_after = test.receiver.balance;

		let delivery_fees = PopNetworkPara::execute_with(|| {
			xcm_helpers::transfer_assets_delivery_fees::<
				<PopNetworkXcmConfig as xcm_executor::Config>::XcmSender,
			>(
				test.args.assets.clone(),
				0,
				test.args.weight_limit,
				test.args.beneficiary,
				test.args.dest,
			)
		});

		// Sender's balance is reduced
		assert_eq!(sender_balance_before - amount_to_send - delivery_fees, sender_balance_after);
		// Receiver's balance is increased
		assert!(receiver_balance_after > receiver_balance_before);
		// Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
		// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown
		// but should be non-zero
		assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
	}

	/// Reserve Transfers of native asset from System Parachain to Parachain should work
	#[test]
	fn reserve_transfer_native_asset_from_system_para_to_para() {
		// Init values for System Parachain
		let destination = AssetHubRococoPara::sibling_location_of(PopNetworkPara::para_id());
		let beneficiary_id = PopNetworkParaReceiver::get();
		let amount_to_send: Balance = ASSET_HUB_ROCOCO_ED * 1000;
		let assets = (Parent, amount_to_send).into();

		let test_args = TestContext {
			sender: AssetHubRococoParaSender::get(),
			receiver: PopNetworkParaReceiver::get(),
			args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
		};

		let mut test = SystemParaToParaTest::new(test_args);

		let sender_balance_before = test.sender.balance;
		let receiver_balance_before = test.receiver.balance;

		test.set_assertion::<AssetHubRococoPara>(system_para_to_para_sender_assertions);
		test.set_assertion::<PopNetworkPara>(para_receiver_assertions);
		test.set_dispatchable::<AssetHubRococoPara>(system_para_to_para_reserve_transfer_assets);
		test.assert();

		let sender_balance_after = test.sender.balance;
		let receiver_balance_after = test.receiver.balance;

		let delivery_fees = AssetHubRococoPara::execute_with(|| {
			xcm_helpers::transfer_assets_delivery_fees::<
				<AssetHubRococoXcmConfig as xcm_executor::Config>::XcmSender,
			>(
				test.args.assets.clone(),
				0,
				test.args.weight_limit,
				test.args.beneficiary,
				test.args.dest,
			)
		});

		// Sender's balance is reduced
		assert_eq!(sender_balance_before - amount_to_send - delivery_fees, sender_balance_after);
		// Receiver's balance is increased
		assert!(receiver_balance_after > receiver_balance_before);
		// Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
		// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown
		// but should be non-zero
		assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
	}

	/// Reserve Transfers of native asset from Parachain to System Parachain should work
	#[test]
	fn reserve_transfer_native_asset_from_para_to_system_para() {
		// Init values for Pop Network Parachain
		let destination = PopNetworkPara::sibling_location_of(AssetHubRococoPara::para_id());
		let beneficiary_id = AssetHubRococoParaReceiver::get();
		let amount_to_send: Balance = ASSET_HUB_ROCOCO_ED * 1000;
		let assets = (Parent, amount_to_send).into();

		let test_args = TestContext {
			sender: PopNetworkParaSender::get(),
			receiver: AssetHubRococoParaReceiver::get(),
			args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
		};

		let mut test = ParaToSystemParaTest::new(test_args);

		let sender_balance_before = test.sender.balance;
		let receiver_balance_before = test.receiver.balance;

		let pop_net_location_as_seen_by_ahr =
			AssetHubRococoPara::sibling_location_of(PopNetworkPara::para_id());
		let sov_pop_net_on_ahr =
			AssetHubRococoPara::sovereign_account_id_of(pop_net_location_as_seen_by_ahr);

		// fund the Pop Network's SA on AHR with the native tokens held in reserve
		AssetHubRococoPara::fund_accounts(vec![(sov_pop_net_on_ahr.into(), amount_to_send * 2)]);

		test.set_assertion::<PopNetworkPara>(para_to_system_para_sender_assertions);
		test.set_assertion::<AssetHubRococoPara>(para_to_system_para_receiver_assertions);
		test.set_dispatchable::<PopNetworkPara>(para_to_system_para_reserve_transfer_assets);
		test.assert();

		let sender_balance_after = test.sender.balance;
		let receiver_balance_after = test.receiver.balance;

		let delivery_fees = PopNetworkPara::execute_with(|| {
			xcm_helpers::transfer_assets_delivery_fees::<
				<PopNetworkXcmConfig as xcm_executor::Config>::XcmSender,
			>(
				test.args.assets.clone(),
				0,
				test.args.weight_limit,
				test.args.beneficiary,
				test.args.dest,
			)
		});

		// Sender's balance is reduced
		assert_eq!(sender_balance_before - amount_to_send - delivery_fees, sender_balance_after);
		// Receiver's balance is increased
		assert!(receiver_balance_after > receiver_balance_before);
		// Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
		// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown
		// but should be non-zero
		assert!(receiver_balance_after < receiver_balance_before + amount_to_send);
	}

	#[allow(dead_code)]
	static INIT: std::sync::Once = std::sync::Once::new();
	// Used during debugging by calling this function as first statement in test
	#[allow(dead_code)]
	fn init_tracing() {
		INIT.call_once(|| {
			// Add test tracing (from sp_tracing::init_for_tests()) but filtering for xcm logs only
			let _ = tracing_subscriber::fmt()
				.with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
				.with_env_filter("xcm=trace,system::events=trace,evm=trace") // Comment out this line to see all traces
				.with_test_writer()
				.init();
		});
	}
}
