#[cfg(test)]
mod chains;

#[cfg(test)]
mod tests {
	use crate::chains::{
		asset_hub_rococo::{
			genesis::ED as ASSET_HUB_ROCOCO_ED, AssetHubRococo, AssetHubRococoParaPallet,
		},
		pop_network::{PopNetwork, PopNetworkParaPallet},
		rococo::{genesis::ED as ROCOCO_ED, Rococo, RococoRelayPallet},
	};
	use asset_hub_rococo_runtime::xcm_config::XcmConfig as AssetHubRococoXcmConfig;
	use asset_test_utils::xcm_helpers;
	use emulated_integration_tests_common::{
		accounts::{ALICE, BOB},
		xcm_emulator::{
			assert_expected_events, bx, decl_test_networks,
			decl_test_sender_receiver_accounts_parameter_types, Chain, Parachain as Para,
			RelayChain as Relay, Test, TestArgs, TestContext, TestExt,
		},
	};
	use frame_support::{
		assert_err,
		pallet_prelude::Weight,
		sp_runtime::{DispatchError, DispatchResult},
		traits::fungibles::Inspect,
	};
	use pop_runtime::{xcm_config::XcmConfig as PopNetworkXcmConfig, Balance};
	use rococo_runtime::xcm_config::XcmConfig as RococoXcmConfig;
	use xcm::prelude::{AccountId32 as AccountId32Junction, *};

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

	decl_test_sender_receiver_accounts_parameter_types! {
		RococoRelay { sender: ALICE, receiver: BOB },
		AssetHubRococoPara { sender: ALICE, receiver: BOB },
		PopNetworkPara { sender: ALICE, receiver: BOB}
	}

	const ASSET_ID: u32 = 1;
	const ASSET_MIN_BALANCE: u128 = 1000;
	const ASSETS_PALLET_ID: u8 = 50;

	type RelayToParaTest = Test<RococoRelay, PopNetworkPara>;
	type SystemParaToParaTest = Test<AssetHubRococoPara, PopNetworkPara>;
	type ParaToSystemParaTest = Test<PopNetworkPara, AssetHubRococoPara>;

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

	fn system_para_to_para_assets_sender_assertions(t: SystemParaToParaTest) {
		type RuntimeEvent = <AssetHubRococoPara as Chain>::RuntimeEvent;
		AssetHubRococoPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
			864_610_000,
			8799,
		)));
		assert_expected_events!(
			AssetHubRococoPara,
			vec![
				// Amount to reserve transfer is transferred to Parachain's Sovereign account
				RuntimeEvent::Assets(
					pallet_assets::Event::Transferred { asset_id, from, to, amount }
				) => {
					asset_id: *asset_id == ASSET_ID,
					from: *from == t.sender.account_id,
					to: *to == AssetHubRococoPara::sovereign_account_id_of(
						t.args.dest.clone()
					),
					amount: *amount == t.args.amount,
				},
			]
		);
	}

	fn system_para_to_para_assets_receiver_assertions<Test>(_: Test) {
		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
		assert_expected_events!(
			PopNetworkPara,
			vec![
				RuntimeEvent::Balances(pallet_balances::Event::Deposit { .. }) => {},
				RuntimeEvent::Assets(pallet_assets::Event::Issued { .. }) => {},
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

	/// Reserve Transfers of native asset from Relay Chain to the System Parachain shouldn't work
	#[test]
	fn reserve_transfer_native_asset_from_relay_to_system_para_fails() {
		let signed_origin =
			<RococoRelay as Chain>::RuntimeOrigin::signed(RococoRelaySender::get().into());
		let destination = RococoRelay::child_location_of(AssetHubRococoPara::para_id());
		let beneficiary: Location =
			AccountId32Junction { network: None, id: AssetHubRococoParaReceiver::get().into() }
				.into();
		let amount_to_send: Balance = ROCOCO_ED * 1000;
		let assets: Assets = (Here, amount_to_send).into();
		let fee_asset_item = 0;

		// this should fail
		RococoRelay::execute_with(|| {
			let result =
				<RococoRelay as RococoRelayPallet>::XcmPallet::limited_reserve_transfer_assets(
					signed_origin,
					bx!(destination.into()),
					bx!(beneficiary.into()),
					bx!(assets.into()),
					fee_asset_item,
					WeightLimit::Unlimited,
				);
			assert_err!(
				result,
				DispatchError::Module(sp_runtime::ModuleError {
					index: 99,
					error: [2, 0, 0, 0],
					message: Some("Filtered")
				})
			);
		});
	}

	/// Reserve Transfers of native asset from System Parachain to Relay Chain shouldn't work
	#[test]
	fn reserve_transfer_native_asset_from_system_para_to_relay_fails() {
		// Init values for System Parachain
		let signed_origin = <AssetHubRococoPara as Chain>::RuntimeOrigin::signed(
			AssetHubRococoParaSender::get().into(),
		);
		let destination = AssetHubRococoPara::parent_location();
		let beneficiary_id = RococoRelayReceiver::get();
		let beneficiary: Location =
			AccountId32Junction { network: None, id: beneficiary_id.into() }.into();
		let amount_to_send: Balance = ASSET_HUB_ROCOCO_ED * 1000;

		let assets: Assets = (Parent, amount_to_send).into();
		let fee_asset_item = 0;

		// this should fail
		AssetHubRococoPara::execute_with(|| {
			let result =
                <AssetHubRococoPara as AssetHubRococoParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
                    signed_origin,
                    bx!(destination.into()),
                    bx!(beneficiary.into()),
                    bx!(assets.into()),
                    fee_asset_item,
                    WeightLimit::Unlimited,
                );
			assert_err!(
				result,
				DispatchError::Module(sp_runtime::ModuleError {
					index: 31,
					error: [2, 0, 0, 0],
					message: Some("Filtered")
				})
			);
		});
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

	/// Reserve Transfers of a local asset and native asset from System Parachain to Parachain
	/// should work
	#[test]
	fn reserve_transfer_assets_from_system_para_to_para() {
		// Force create asset on AssetHubRococoPara and PopNetworkPara from Relay Chain
		AssetHubRococoPara::force_create_and_mint_asset(
			ASSET_ID,
			ASSET_MIN_BALANCE,
			false,
			AssetHubRococoParaSender::get(),
			Some(Weight::from_parts(1_019_445_000, 200_000)),
			ASSET_MIN_BALANCE * 1_000_000,
		);
		PopNetworkPara::force_create_and_mint_asset(
			ASSET_ID,
			ASSET_MIN_BALANCE,
			false,
			PopNetworkParaSender::get(),
			None,
			0,
		);

		// Init values for System Parachain
		let destination = AssetHubRococoPara::sibling_location_of(PopNetworkPara::para_id());
		let beneficiary_id = PopNetworkParaReceiver::get();
		let fee_amount_to_send = ASSET_HUB_ROCOCO_ED * 1000;
		let asset_amount_to_send = ASSET_MIN_BALANCE * 1000;
		let assets: Assets = vec![
			(Parent, fee_amount_to_send).into(),
			(
				[PalletInstance(ASSETS_PALLET_ID), GeneralIndex(ASSET_ID.into())],
				asset_amount_to_send,
			)
				.into(),
		]
		.into();
		let fee_asset_index = assets
			.inner()
			.iter()
			.position(|r| r == &(Parent, fee_amount_to_send).into())
			.unwrap() as u32;

		let para_test_args = TestContext {
			sender: AssetHubRococoParaSender::get(),
			receiver: PopNetworkParaReceiver::get(),
			args: TestArgs::new_para(
				destination,
				beneficiary_id,
				asset_amount_to_send,
				assets,
				None,
				fee_asset_index,
			),
		};

		let mut test = SystemParaToParaTest::new(para_test_args);

		// Create SA-of-Pop Network-on-AHR with ED.
		let pop_net_location = AssetHubRococoPara::sibling_location_of(PopNetworkPara::para_id());
		let sov_pop_net_on_ahr = AssetHubRococoPara::sovereign_account_id_of(pop_net_location);
		AssetHubRococoPara::fund_accounts(vec![(sov_pop_net_on_ahr.into(), ROCOCO_ED)]);

		let sender_balance_before = test.sender.balance;
		let receiver_balance_before = test.receiver.balance;

		let sender_assets_before = AssetHubRococoPara::execute_with(|| {
			type Assets = <AssetHubRococoPara as AssetHubRococoParaPallet>::Assets;
			<Assets as Inspect<_>>::balance(ASSET_ID, &AssetHubRococoParaSender::get())
		});
		let receiver_assets_before = PopNetworkPara::execute_with(|| {
			type Assets = <PopNetworkPara as PopNetworkParaPallet>::Assets;
			<Assets as Inspect<_>>::balance(ASSET_ID, &PopNetworkParaReceiver::get())
		});

		test.set_assertion::<AssetHubRococoPara>(system_para_to_para_assets_sender_assertions);
		test.set_assertion::<PopNetworkPara>(system_para_to_para_assets_receiver_assertions);
		test.set_dispatchable::<AssetHubRococoPara>(system_para_to_para_reserve_transfer_assets);
		test.assert();

		let sender_balance_after = test.sender.balance;
		let receiver_balance_after = test.receiver.balance;

		// Sender's balance is reduced
		assert!(sender_balance_after < sender_balance_before);
		// Receiver's balance is increased
		assert!(receiver_balance_after > receiver_balance_before);
		// Receiver's balance increased by `amount_to_send - delivery_fees - bought_execution`;
		// `delivery_fees` might be paid from transfer or JIT, also `bought_execution` is unknown
		// but should be non-zero
		assert!(receiver_balance_after < receiver_balance_before + fee_amount_to_send);

		let sender_assets_after = AssetHubRococoPara::execute_with(|| {
			type Assets = <AssetHubRococoPara as AssetHubRococoParaPallet>::Assets;
			<Assets as Inspect<_>>::balance(ASSET_ID, &AssetHubRococoParaSender::get())
		});
		let receiver_assets_after = PopNetworkPara::execute_with(|| {
			type Assets = <PopNetworkPara as PopNetworkParaPallet>::Assets;
			<Assets as Inspect<_>>::balance(ASSET_ID, &PopNetworkParaReceiver::get())
		});

		// Sender's balance is reduced by exact amount
		assert_eq!(sender_assets_before - asset_amount_to_send, sender_assets_after);
		// Receiver's balance is increased by exact amount
		assert_eq!(receiver_assets_after, receiver_assets_before + asset_amount_to_send);
	}
}
