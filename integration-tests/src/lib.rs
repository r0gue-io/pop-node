#![cfg(test)]

use asset_test_utils::xcm_helpers;
use chains::{
	asset_hub::{
		self,
		genesis::{ED as ASSET_HUB_ED, USDC},
		runtime::xcm_config::XcmConfig as AssetHubXcmConfig,
		AssetHub, AssetHubParaPallet,
	},
	pop_network::{genesis, runtime, PopNetwork, PopNetworkParaPallet},
	relay::{
		genesis::ED as RELAY_ED, runtime::xcm_config::XcmConfig as RelayXcmConfig, Relay,
		RelayRelayPallet as RelayPallet,
	},
};
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	xcm_emulator::{
		assert_expected_events, bx, decl_test_networks,
		decl_test_sender_receiver_accounts_parameter_types, Chain, Parachain as Para, RelayChain,
		Test, TestArgs, TestContext, TestExt,
	},
};
use frame_support::{
	pallet_prelude::Weight, sp_runtime::DispatchResult, traits::fungibles::Inspect,
};
use polkadot_primitives::AccountId;
use pop_runtime_common::Balance;
use pop_runtime_devnet::config::xcm::XcmConfig as PopNetworkXcmConfig;
use xcm::prelude::*;

mod chains;

decl_test_networks! {
	// `pub` mandatory for the macro
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			AssetHub,
			PopNetwork,
		],
		bridge = ()
	},
}

decl_test_sender_receiver_accounts_parameter_types! {
	RelayRelay { sender: ALICE, receiver: BOB },
	AssetHubPara { sender: ALICE, receiver: BOB },
	PopNetworkPara { sender: ALICE, receiver: BOB}
}

type RelayToParaTest = Test<RelayRelay, PopNetworkPara>;
type SystemParaToParaTest = Test<AssetHubPara, PopNetworkPara>;
type ParaToSystemParaTest = Test<PopNetworkPara, AssetHubPara>;

fn relay_to_para_sender_assertions(t: RelayToParaTest) {
	type RuntimeEvent = <RelayRelay as Chain>::RuntimeEvent;
	RelayRelay::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(864_610_000, 8_799)));
	assert_expected_events!(
		RelayRelay,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == t.sender.account_id,
				to: *to == RelayRelay::sovereign_account_id_of(
					t.args.dest.clone()
				),
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn system_para_to_para_sender_assertions(t: SystemParaToParaTest) {
	type RuntimeEvent = <AssetHubPara as Chain>::RuntimeEvent;
	AssetHubPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
		864_610_000,
		8_799,
	)));
	assert_expected_events!(
		AssetHubPara,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == t.sender.account_id,
				to: *to == AssetHubPara::sovereign_account_id_of(
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
			RuntimeEvent::Balances(pallet_balances::Event::Minted { .. }) => {},
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
			RuntimeEvent::Balances(pallet_balances::Event::Burned { who, amount }) => {
				who: *who == t.sender.account_id,
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn para_to_system_para_receiver_assertions(t: ParaToSystemParaTest) {
	type RuntimeEvent = <AssetHubPara as Chain>::RuntimeEvent;
	let sov_pop_net_on_ahr = AssetHubPara::sovereign_account_id_of(
		AssetHubPara::sibling_location_of(PopNetworkPara::para_id()),
	);
	assert_expected_events!(
		AssetHubPara,
		vec![
			// Amount to reserve transfer is withdrawn from Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Burned { who, amount }
			) => {
				who: *who == sov_pop_net_on_ahr.clone().into(),
				amount: *amount == t.args.amount,
			},
			RuntimeEvent::Balances(pallet_balances::Event::Minted { .. }) => {},
			RuntimeEvent::MessageQueue(
				pallet_message_queue::Event::Processed { success: true, .. }
			) => {},
		]
	);
}

fn relay_to_para_reserve_transfer_assets(t: RelayToParaTest) -> DispatchResult {
	<RelayRelay as RelayPallet>::XcmPallet::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

fn system_para_to_para_reserve_transfer_assets(t: SystemParaToParaTest) -> DispatchResult {
	<AssetHubPara as AssetHubParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
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

// Funds Pop with relay tokens from system para
fn fund_pop_from_system_para(
	sender: AccountId,
	amount_to_send: Balance,
	beneficiary: AccountId,
	assets: Assets,
) {
	let destination = AssetHubPara::sibling_location_of(PopNetworkPara::para_id());
	let test_args = TestContext {
		sender,
		receiver: beneficiary.clone(),
		args: TestArgs::new_para(destination, beneficiary, amount_to_send, assets, None, 0),
	};

	let mut test = SystemParaToParaTest::new(test_args);
	test.set_dispatchable::<AssetHubPara>(system_para_to_para_reserve_transfer_assets);
	test.assert();
}

use pop_runtime_devnet::{config::assets::AssetRegistrarMetadata, RuntimeOrigin};

/// Asset Hub.
pub const ASSET_HUB: Junction = Parachain(1000);
/// Assets on Asset Hub.
pub const ASSET_HUB_ASSETS: Junction = PalletInstance(50);
pub const USDC_POP_ASSET_ID: u32 = 1;

#[cfg(feature = "devnet")]
#[test]
fn receive_usdc_on_pop() {
	use codec::Encode;
	use pallet_contracts::{Code, CollectEvents, Determinism};
	const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);
	const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;
	init_tracing();

	// Provide bob DOT.
	let amount: Balance = ASSET_HUB_ED * 1000;
	let bob_on_pop = PopNetworkParaReceiver::get();
	fund_pop_from_system_para(
		AssetHubParaSender::get(),
		amount * 20,
		bob_on_pop.clone(),
		(Parent, amount * 20).into(),
	);
	// Fund Pop Network's SA on AH with DOT.
	let pop_net_location_as_seen_by_ahr =
		AssetHubPara::sibling_location_of(PopNetworkPara::para_id());
	let sov_pop_net_on_ahr = AssetHubPara::sovereign_account_id_of(pop_net_location_as_seen_by_ahr);
	AssetHubPara::fund_accounts(vec![(sov_pop_net_on_ahr.into(), amount * 2)]);
	// Amounts used for doing transfers.
	let transfer_amount = amount / 2;

	// Create a foreign asset for usdc on Pop.
	// ------
	let usdc = AssetId([ASSET_HUB_ASSETS, GeneralIndex(USDC.into())].into());
	let usdc_location = Location::new(1, [ASSET_HUB, ASSET_HUB_ASSETS, GeneralIndex(USDC.into())]);
	let metadata = AssetRegistrarMetadata {
		name: "USDC".encode(),
		symbol: "USDC".encode(),
		decimals: 10,
		is_frozen: false,
	};
	let mut contract = AccountId::from([0u8; 32]);
	PopNetwork::<MockNet>::execute_with(|| {
		<PopNetwork<MockNet> as PopNetworkParaPallet>::AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			usdc_location,
			metadata,
			1,
			true,
		);

		let path = "../pop-api/examples/foreign_fungibles/target/ink/foreign_fungibles.wasm";
		let wasm_binary = std::fs::read(path).expect("could not read .wasm file");
		let instantiate_result =
			<PopNetwork<MockNet> as PopNetworkParaPallet>::Contracts::bare_instantiate(
				bob_on_pop.clone(),
				amount * 5,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				CollectEvents::Skip,
			)
			.result
			.expect("Contract instantiation failed");
		assert!(!instantiate_result.result.did_revert());
		contract = instantiate_result.account_id;
	});
	// ------

	// Provide sender on AH with usdc to transfer.
	// ------
	let sender = AssetHubParaSender::get();
	let usdc_owner = asset_hub::AssetOwner::get();
	let usdc_owner_signer = <AssetHubPara as Chain>::RuntimeOrigin::signed(usdc_owner.clone());
	let usdc_amount_to_send = asset_hub::USD_MIN_BALANCE * 100_000;
	AssetHubPara::mint_asset(
		usdc_owner_signer.clone(),
		USDC,
		sender.clone(),
		usdc_amount_to_send + asset_hub::USD_MIN_BALANCE,
	);
	let usdc_asset = (usdc, usdc_amount_to_send).into();
	// ------

	// Transfer usdc to contract.
	let fee_amount_to_send = runtime::UNIT / 2;
	let fee_asset = (Parent, fee_amount_to_send).into();
	let assets: Assets = vec![usdc_asset, fee_asset].into();
	let destination = AssetHubPara::sibling_location_of(PopNetworkPara::para_id());
	let receiver = contract.clone();
	let location_usdc = Location::new(1, [ASSET_HUB, ASSET_HUB_ASSETS, GeneralIndex(USDC.into())]); // Init Test
	let test_args = TestContext {
		sender: sender.clone(),
		receiver: receiver.clone(),
		args: TestArgs::new_para(
			destination.clone(),
			receiver.clone(),
			usdc_amount_to_send,
			assets,
			Some(USDC),
			1,
		),
	};
	let mut test = Test::<AssetHubPara, PopNetworkPara>::new(test_args);
	// ---- Assertions
	let sender_usdc_balance_before = AssetHubPara::execute_with(|| {
		<<AssetHubPara as AssetHubParaPallet>::Assets as Inspect<_>>::balance(USDC, &sender)
	});
	let receiver_usdc_balance_before = PopNetworkPara::execute_with(|| {
		<<PopNetworkPara as PopNetworkParaPallet>::Assets as Inspect<_>>::balance(
			USDC_POP_ASSET_ID,
			&receiver,
		)
	});
	test.set_assertion::<AssetHubPara>(asset_hub_to_para_assets_sender_assertions);
	test.set_assertion::<PopNetworkPara>(asset_hub_to_para_assets_receiver_assertions);
	test.set_dispatchable::<AssetHubPara>(asset_hub_to_para_transfer_assets);
	test.assert();
	let sender_usdc_balance_after = AssetHubPara::execute_with(|| {
		<<AssetHubPara as AssetHubParaPallet>::Assets as Inspect<_>>::balance(USDC, &sender)
	});
	let receiver_usdc_balance_after = PopNetworkPara::execute_with(|| {
		<<PopNetworkPara as PopNetworkParaPallet>::Assets as Inspect<_>>::balance(
			USDC_POP_ASSET_ID,
			&receiver,
		)
	});

	assert_eq!(sender_usdc_balance_after, sender_usdc_balance_before - usdc_amount_to_send);
	assert_eq!(receiver_usdc_balance_after, receiver_usdc_balance_before + usdc_amount_to_send);

	// ------

	let ah_transfer_amount = usdc_amount_to_send / 2;
	// Step 4: Interact with the contract on Pop Network
	PopNetworkPara::execute_with(|| {
		// Call contract to transfer half USDC to another account on Pop.
		let local_transfer_amount = usdc_amount_to_send / 2;
		let local_receiver = AccountId::from([2u8; 32]);
		let function = function_selector("transfer");
		let params = [local_receiver.encode(), local_transfer_amount.encode()].concat();
		let input = [function, params].concat();
		let call_result = <PopNetwork<MockNet> as PopNetworkParaPallet>::Contracts::bare_call(
			bob_on_pop.clone(),
			contract.clone(),
			0,
			GAS_LIMIT,
			None,
			input,
			DEBUG_OUTPUT,
			CollectEvents::Skip,
			Determinism::Enforced,
		);
		let result = decoded::<Result<(), ()>>(call_result.result.unwrap()).unwrap();
		assert!(result.is_ok());
		// Verify local transfer
		let local_receiver_balance =
			<<PopNetwork<MockNet> as PopNetworkParaPallet>::Assets as Inspect<_>>::balance(
				USDC_POP_ASSET_ID,
				&local_receiver,
			);
		assert_eq!(local_receiver_balance, local_transfer_amount);
		let contract_usdc_balance_after_local =
			<<PopNetwork<MockNet> as PopNetworkParaPallet>::Assets as Inspect<_>>::balance(
				USDC_POP_ASSET_ID,
				&contract,
			);
		assert_eq!(contract_usdc_balance_after_local, usdc_amount_to_send - local_transfer_amount);

		// Call contract to transfer remaining USDC back to AH
		let ah_receiver = AssetHubParaReceiver::get();
		let function = function_selector("ah_transfer");
		let params =
			[ah_receiver.encode(), ah_transfer_amount.encode(), 2500000000000u128.encode()]
				.concat();
		let input = [function, params].concat();
		let call_result = <PopNetwork<MockNet> as PopNetworkParaPallet>::Contracts::bare_call(
			bob_on_pop.clone(),
			contract.clone(),
			0,
			GAS_LIMIT,
			None,
			input,
			DEBUG_OUTPUT,
			CollectEvents::Skip,
			Determinism::Enforced,
		);
		let result = decoded::<Result<(), ()>>(call_result.result.unwrap()).unwrap();
		assert!(result.is_ok());

		// Check Pop Network events for USDC burn
		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
		let events = PopNetworkPara::events();
		println!("\nPopNetwork events after contract calls: {:?}", events);
		assert_event_matches!(
					events,
					RuntimeEvent::Assets(pallet_assets::Event::Burned { asset_id, owner, balance })
					if *asset_id == USDC_POP_ASSET_ID && *owner == contract && *balance ==
		ah_transfer_amount 		);
	});

	// Step 5: Verify the transfer back to Asset Hub
	AssetHubPara::execute_with(|| {
		// Check AH events for USDC mint
		type RuntimeEventAH = <AssetHubPara as Chain>::RuntimeEvent;
		let sov_pop_net_on_ahr = AssetHubPara::sovereign_account_id_of(destination);
		let ah_receiver = AssetHubParaReceiver::get();
		let events = AssetHubPara::events();
		println!("\nAssetHub events after contract call: {:?}", events);
		assert_event_matches!(
			events,
			RuntimeEventAH::Assets(pallet_assets::Event::Burned { asset_id, owner, balance })
			if *asset_id == USDC && *owner == sov_pop_net_on_ahr && *balance == ah_transfer_amount
		);
		assert_event_matches!(
			events,
			RuntimeEventAH::Assets(pallet_assets::Event::Issued { asset_id, owner, amount })
			if *asset_id == USDC && *owner == ah_receiver && *amount <= ah_transfer_amount &&
		*amount > 0 );
		assert_event_matches!(
			events,
			RuntimeEventAH::MessageQueue(pallet_message_queue::Event::Processed { success, .. })
			if *success == true
		);

		// Verify receiver’s USDC balance on AH
		let receiver_usdc_balance =
			<<AssetHubPara as AssetHubParaPallet>::Assets as Inspect<_>>::balance(
				USDC,
				&ah_receiver,
			);
		assert!(receiver_usdc_balance == ah_transfer_amount);
	});
}

/// Reserve Transfers of native asset from Relay to Parachain should work
#[test]
#[should_panic]
fn reserve_transfer_native_asset_from_relay_to_para_should_fail() {
	init_tracing();

	// Init values for Relay
	let destination = RelayRelay::child_location_of(PopNetworkPara::para_id());
	let beneficiary_id = PopNetworkParaReceiver::get();
	let amount_to_send: Balance = RELAY_ED * 1000;

	let test_args = TestContext {
		sender: RelayRelaySender::get(),
		receiver: PopNetworkParaReceiver::get(),
		args: TestArgs::new_relay(destination, beneficiary_id, amount_to_send),
	};

	let mut test = RelayToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<RelayRelay>(relay_to_para_sender_assertions);
	test.set_assertion::<PopNetworkPara>(para_receiver_assertions);
	test.set_dispatchable::<RelayRelay>(relay_to_para_reserve_transfer_assets);
	test.assert();

	let delivery_fees = RelayRelay::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<RelayXcmConfig as xcm_executor::Config>::XcmSender,
		>(
			test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest
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
	init_tracing();

	// Init values for System Parachain
	let destination = AssetHubPara::sibling_location_of(PopNetworkPara::para_id());
	let beneficiary_id = PopNetworkParaReceiver::get();
	let amount_to_send: Balance = ASSET_HUB_ED * 1000;
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: AssetHubParaSender::get(),
		receiver: PopNetworkParaReceiver::get(),
		args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
	};

	let mut test = SystemParaToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<AssetHubPara>(system_para_to_para_sender_assertions);
	test.set_assertion::<PopNetworkPara>(para_receiver_assertions);
	test.set_dispatchable::<AssetHubPara>(system_para_to_para_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	let delivery_fees = AssetHubPara::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<AssetHubXcmConfig as xcm_executor::Config>::XcmSender,
		>(
			test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest
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
	init_tracing();

	// Setup: reserve transfer from AH to Pop, so that sovereign account accurate for return
	// transfer
	let amount_to_send: Balance = ASSET_HUB_ED * 1000;
	fund_pop_from_system_para(
		AssetHubParaSender::get(),
		amount_to_send,
		PopNetworkParaReceiver::get(),
		(Parent, amount_to_send).into(),
	); // alice on asset hub > bob on pop

	// Init values for Pop Network Parachain
	let destination = PopNetworkPara::sibling_location_of(AssetHubPara::para_id());
	let beneficiary_id = AssetHubParaReceiver::get(); // bob on asset hub
	let amount_to_send = PopNetworkPara::account_data_of(PopNetworkParaReceiver::get()).free; // bob on pop balance
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: PopNetworkParaReceiver::get(), // bob on pop
		receiver: AssetHubParaReceiver::get(), // bob on asset hub
		args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
	};

	let mut test = ParaToSystemParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let pop_net_location_as_seen_by_ahr =
		AssetHubPara::sibling_location_of(PopNetworkPara::para_id());
	let sov_pop_net_on_ahr = AssetHubPara::sovereign_account_id_of(pop_net_location_as_seen_by_ahr);

	// fund Pop Network's SA on AHR with the native tokens held in reserve
	AssetHubPara::fund_accounts(vec![(sov_pop_net_on_ahr.into(), amount_to_send * 2)]);

	test.set_assertion::<PopNetworkPara>(para_to_system_para_sender_assertions);
	test.set_assertion::<AssetHubPara>(para_to_system_para_receiver_assertions);
	test.set_dispatchable::<PopNetworkPara>(para_to_system_para_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	let delivery_fees = PopNetworkPara::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<PopNetworkXcmConfig as xcm_executor::Config>::XcmSender,
		>(
			test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest
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

// Note: commented out until coretime added to Paseo
// #[test]
// fn place_coretime_spot_order_from_para_to_relay() {
// 	init_tracing();
//
// 	let beneficiary: AccountId = [1u8; 32].into();
//
// 	// Setup: reserve transfer from relay to Pop, so that sovereign account accurate for return
// transfer 	let amount_to_send: Balance = pop_runtime::UNIT * 1000;
// 	fund_pop_from_relay(RelayRelaySender::get(), amount_to_send, beneficiary.clone());
//
// 	let message = {
// 		let assets: Asset = (Here, 10 * pop_runtime::UNIT).into();
// 		let beneficiary = AccountId { id: beneficiary.clone().into(), network: None }.into();
// 		let spot_order = <RelayRelay as Chain>::RuntimeCall::OnDemandAssignmentProvider(
// 			assigner_on_demand::Call::<<RelayRelay as Chain>::Runtime>::place_order_keep_alive {
// 				max_amount: 1 * pop_runtime::UNIT,
// 				para_id: AssetHubPara::para_id().into(),
// 			},
// 		);
//
// 		// Set up transact status response handler
// 		let query_id = PopNetworkPara::execute_with(|| {
// 			<PopNetwork<PaseoMockNet> as PopNetworkParaPallet>::PolkadotXcm::new_query(
// 				PopNetworkPara::parent_location(),
// 				// timeout in blocks
// 				10u32.into(),
// 				Location::here(),
// 			)
// 		});
//
// 		let message = Xcm::builder()
// 			.withdraw_asset(assets.clone().into())
// 			.buy_execution(assets.clone().into(), Unlimited)
// 			.transact(
// 				OriginKind::SovereignAccount,
// 				Weight::from_parts(220_000_000, 15_000),
// 				spot_order.encode().into(),
// 			)
// 			.report_transact_status(QueryResponseInfo {
// 				destination: RelayRelay::child_location_of(PopNetworkPara::para_id()),
// 				query_id,
// 				max_weight: Weight::from_parts(250_000_000, 10_000),
// 			})
// 			.refund_surplus()
// 			.deposit_asset(assets.into(), beneficiary)
// 			.build();
// 		message
// 	};
//
// 	let destination = PopNetworkPara::parent_location().into_versioned();
// 	PopNetworkPara::execute_with(|| {
// 		let res = <PopNetworkPara as Chain>::RuntimeCall::PolkadotXcm(pallet_xcm::Call::<
// 			<PopNetworkPara as Chain>::Runtime,
// 		>::send {
// 			dest: bx!(destination),
// 			message: bx!(VersionedXcm::V4(message)),
// 		})
// 		// TODO: replace root with signed, currently prohibited by HashedDescription<AccountId, DescribeFamily<DescribeBodyTerminal>> (https://github.com/paritytech/polkadot-sdk/blob/a6713c55fd5082d333518c3ca13f2a4294726fcc/polkadot/runtime/rococo/src/xcm_config.rs#L67) rather than HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>> (https://github.com/polkadot-fellows/runtimes/blob/e42821da8d85f721d0dd1670dfb23f4dd91bd3e8/relay/kusama/src/xcm_config.rs#L76)
// 		//.dispatch(RawOrigin::Signed(beneficiary).into());
// 		.dispatch(RawOrigin::Root.into());
//
// 		assert!(res.is_ok());
// 		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
// 		// Check that the message was sent
// 		assert_expected_events!(
// 			PopNetworkPara,
// 			vec![
// 				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
// 			]
// 		);
// 	});
//
// 	RelayRelay::execute_with(|| {
// 		type RuntimeEvent = <RelayRelay as Chain>::RuntimeEvent;
// 		assert_expected_events!(
// 			RelayRelay,
// 			vec![
// 				// We currently only check that the message was processed successfully
// 				RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) =>
// {}, 				// TODO: check order placed once we can have on-demand para id registered (probably via
// setting raw storage as a workaround) 				//
// RuntimeEvent::OnDemandAssignmentProvider(assigner_on_demand::Event::OnDemandOrderPlaced { 				//
// .. 				// }) => {},
// 			]
// 		);
// 	});
//
// 	PopNetworkPara::execute_with(|| {
// 		type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
// 		// Check that the reporting of the transact status message was sent
// 		assert_expected_events!(
// 			PopNetworkPara,
// 			vec![
// 				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::ResponseReady { query_id: 0, .. }) => {},
// 				RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) =>
// {}, 			]
// 		);
// 	});
// }

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

/// Asserts that an event has been emitted using pattern matching.
macro_rules! assert_event_matches {
    ($expression:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
		assert!($expression.iter().any(|e| matches!(e, $pattern $(if $guard)?)), "expected event not found in {:#?}", $expression);
    };
}
pub(crate) use assert_event_matches;

fn asset_hub_to_para_transfer_assets(t: Test<AssetHubPara, PopNetworkPara>) -> DispatchResult {
	<AssetHubPara as AssetHubParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

fn asset_hub_to_para_assets_sender_assertions(t: Test<AssetHubPara, PopNetworkPara>) {
	type RuntimeEvent = <AssetHubPara as Chain>::RuntimeEvent;
	AssetHubPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
		864_610_000,
		8_799,
	)));
	let sovereign_account = AssetHubPara::sovereign_account_id_of(t.args.dest.clone());
	assert_event_matches!(
		AssetHubPara::events(),
		RuntimeEvent::Assets(pallet_assets::Event::Transferred { asset_id, from, to, amount })
		if *asset_id == USDC && *from == t.sender.account_id && *to == sovereign_account && *amount == t.args.amount
	);
}

fn asset_hub_to_para_assets_receiver_assertions(t: Test<AssetHubPara, PopNetworkPara>) {
	type RuntimeEvent = <PopNetworkPara as Chain>::RuntimeEvent;
	let events = PopNetworkPara::events();
	println!("PopNetwork events: {:?}", events); // Debug output
	assert_event_matches!(
		events,
		RuntimeEvent::Assets(pallet_assets::Event::Issued { asset_id, owner, amount })
		if *asset_id == USDC_POP_ASSET_ID && *owner == t.receiver.account_id && *amount == t.args.amount
	);
	assert_event_matches!(
		events,
		RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success, .. })
		if *success == true
	);
}

use codec::Decode;
use pallet_contracts::{Code, CollectEvents};

fn function_selector(name: &str) -> Vec<u8> {
	let hash = sp_io::hashing::blake2_256(name.as_bytes());
	[hash[0..4].to_vec()].concat()
}

fn decoded<T: Decode>(
	result: pallet_contracts::ExecReturnValue,
) -> Result<T, pallet_contracts::ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}
