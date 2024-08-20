#![cfg(test)]

use crate::chains::{
	asset_hub_paseo::{genesis::ED as ASSET_HUB_PASEO_ED, AssetHubPaseoParaPallet},
	paseo::{genesis::ED as PASEO_ED, PaseoRelayPallet},
	pop_network::PopNetworkParaPallet,
};
use asset_hub_paseo_runtime::xcm_config::XcmConfig as AssetHubPaseoXcmConfig;
use asset_test_utils::xcm_helpers;
use chains::{asset_hub_paseo::AssetHubPaseo, paseo::Paseo, pop_network::PopNetwork};
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	xcm_emulator::{
		assert_expected_events, bx, decl_test_networks,
		decl_test_sender_receiver_accounts_parameter_types, Chain, Parachain as Para,
		RelayChain as Relay, Test, TestArgs, TestContext, TestExt,
	},
};
use frame_support::{pallet_prelude::Weight, sp_runtime::DispatchResult};
use paseo_runtime::xcm_config::XcmConfig as PaseoXcmConfig;
use pop_runtime_common::Balance;
use pop_runtime_devnet::config::xcm::XcmConfig as PopNetworkXcmConfig;
use xcm::prelude::*;

mod chains;

decl_test_networks! {
	// `pub` mandatory for the macro
	pub struct PaseoMockNet {
		relay_chain = Paseo,
		parachains = vec![
			AssetHubPaseo,
			PopNetwork,
		],
		bridge = ()
	},
}

decl_test_sender_receiver_accounts_parameter_types! {
	PaseoRelay { sender: ALICE, receiver: BOB },
	AssetHubPaseoPara { sender: ALICE, receiver: BOB },
	PopNetworkPara { sender: ALICE, receiver: BOB}
}

type RelayToParaTest = Test<PaseoRelay, PopNetworkPara>;
type SystemParaToParaTest = Test<AssetHubPaseoPara, PopNetworkPara>;
type ParaToSystemParaTest = Test<PopNetworkPara, AssetHubPaseoPara>;
type ParaToRelayTest = Test<PopNetworkPara, PaseoRelay>;

fn relay_to_para_sender_assertions(t: RelayToParaTest) {
	type RuntimeEvent = <PaseoRelay as Chain>::RuntimeEvent;
	PaseoRelay::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(864_610_000, 8_799)));
	assert_expected_events!(
		PaseoRelay,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == t.sender.account_id,
				to: *to == PaseoRelay::sovereign_account_id_of(
					t.args.dest.clone()
				),
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn system_para_to_para_sender_assertions(t: SystemParaToParaTest) {
	type RuntimeEvent = <AssetHubPaseoPara as Chain>::RuntimeEvent;
	AssetHubPaseoPara::assert_xcm_pallet_attempted_complete(Some(Weight::from_parts(
		864_610_000,
		8_799,
	)));
	assert_expected_events!(
		AssetHubPaseoPara,
		vec![
			// Amount to reserve transfer is transferred to Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Transfer { from, to, amount }
			) => {
				from: *from == t.sender.account_id,
				to: *to == AssetHubPaseoPara::sovereign_account_id_of(
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
			RuntimeEvent::Balances(pallet_balances::Event::Burned { who, amount }) => {
				who: *who == t.sender.account_id,
				amount: *amount == t.args.amount,
			},
		]
	);
}

fn para_to_system_para_receiver_assertions(t: ParaToSystemParaTest) {
	type RuntimeEvent = <AssetHubPaseoPara as Chain>::RuntimeEvent;
	let sov_pop_net_on_ahr = AssetHubPaseoPara::sovereign_account_id_of(
		AssetHubPaseoPara::sibling_location_of(PopNetworkPara::para_id()),
	);
	assert_expected_events!(
		AssetHubPaseoPara,
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

fn para_to_relay_receiver_assertions(t: ParaToRelayTest) {
	type RuntimeEvent = <PaseoRelay as Chain>::RuntimeEvent;
	let sov_pop_net_on_relay = PaseoRelay::sovereign_account_id_of(PaseoRelay::child_location_of(
		PopNetworkPara::para_id(),
	));
	assert_expected_events!(
		PaseoRelay,
		vec![
			// Amount to reserve transfer is withdrawn from Parachain's Sovereign account
			RuntimeEvent::Balances(
				pallet_balances::Event::Burned { who, amount }
			) => {
				who: *who == sov_pop_net_on_relay.clone().into(),
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
	<PaseoRelay as PaseoRelayPallet>::XcmPallet::limited_reserve_transfer_assets(
		t.signed_origin,
		bx!(t.args.dest.into()),
		bx!(t.args.beneficiary.into()),
		bx!(t.args.assets.into()),
		t.args.fee_asset_item,
		t.args.weight_limit,
	)
}

fn system_para_to_para_reserve_transfer_assets(t: SystemParaToParaTest) -> DispatchResult {
	<AssetHubPaseoPara as AssetHubPaseoParaPallet>::PolkadotXcm::limited_reserve_transfer_assets(
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

// Funds Pop with relay tokens
fn fund_pop_from_relay(
	sender: sp_runtime::AccountId32,
	amount_to_send: Balance,
	beneficiary: sp_runtime::AccountId32,
) {
	let destination = PaseoRelay::child_location_of(PopNetworkPara::para_id());
	let test_args = TestContext {
		sender,
		receiver: beneficiary.clone(),
		args: TestArgs::new_relay(destination, beneficiary, amount_to_send),
	};

	let mut test = RelayToParaTest::new(test_args);
	test.set_dispatchable::<PaseoRelay>(relay_to_para_reserve_transfer_assets);
	test.assert();
}

// Funds Pop with relay tokens from system para
fn fund_pop_from_system_para(
	sender: sp_runtime::AccountId32,
	amount_to_send: Balance,
	beneficiary: sp_runtime::AccountId32,
	assets: Assets,
) {
	let destination = AssetHubPaseoPara::sibling_location_of(PopNetworkPara::para_id());
	let test_args = TestContext {
		sender,
		receiver: beneficiary.clone(),
		args: TestArgs::new_para(destination, beneficiary, amount_to_send, assets, None, 0),
	};

	let mut test = SystemParaToParaTest::new(test_args);
	test.set_dispatchable::<AssetHubPaseoPara>(system_para_to_para_reserve_transfer_assets);
	test.assert();
}

/// Reserve Transfers of native asset from Relay to Parachain should work
#[test]
fn reserve_transfer_native_asset_from_relay_to_para() {
	init_tracing();

	// Init values for Relay
	let destination = PaseoRelay::child_location_of(PopNetworkPara::para_id());
	let beneficiary_id = PopNetworkParaReceiver::get();
	let amount_to_send: Balance = PASEO_ED * 1000;

	let test_args = TestContext {
		sender: PaseoRelaySender::get(),
		receiver: PopNetworkParaReceiver::get(),
		args: TestArgs::new_relay(destination, beneficiary_id, amount_to_send),
	};

	let mut test = RelayToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<PaseoRelay>(relay_to_para_sender_assertions);
	test.set_assertion::<PopNetworkPara>(para_receiver_assertions);
	test.set_dispatchable::<PaseoRelay>(relay_to_para_reserve_transfer_assets);
	test.assert();

	let delivery_fees = PaseoRelay::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<PaseoXcmConfig as xcm_executor::Config>::XcmSender,
		>(test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest)
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
	init_tracing();

	// Setup: reserve transfer from relay to Pop, so that sovereign account accurate for return
	// transfer
	let amount_to_send: Balance = PASEO_ED * 1_000;
	fund_pop_from_relay(PaseoRelaySender::get(), amount_to_send, PopNetworkParaReceiver::get()); // alice on relay > bob on pop

	// Init values for Pop Network Parachain
	let destination = PopNetworkPara::parent_location(); // relay
	let beneficiary_id = PaseoRelayReceiver::get(); // bob on relay
	let amount_to_send = PopNetworkPara::account_data_of(PopNetworkParaReceiver::get()).free; // bob on pop balance
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: PopNetworkParaReceiver::get(), // bob on pop
		receiver: PaseoRelayReceiver::get(),   // bob on relay
		args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
	};

	let mut test = ParaToRelayTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<PopNetworkPara>(para_to_relay_sender_assertions);
	test.set_assertion::<PaseoRelay>(para_to_relay_receiver_assertions);
	test.set_dispatchable::<PopNetworkPara>(para_to_relay_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	let delivery_fees = PopNetworkPara::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<PopNetworkXcmConfig as xcm_executor::Config>::XcmSender,
		>(test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest)
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
	init_tracing();

	// Init values for System Parachain
	let destination = AssetHubPaseoPara::sibling_location_of(PopNetworkPara::para_id());
	let beneficiary_id = PopNetworkParaReceiver::get();
	let amount_to_send: Balance = ASSET_HUB_PASEO_ED * 1000;
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: AssetHubPaseoParaSender::get(),
		receiver: PopNetworkParaReceiver::get(),
		args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
	};

	let mut test = SystemParaToParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	test.set_assertion::<AssetHubPaseoPara>(system_para_to_para_sender_assertions);
	test.set_assertion::<PopNetworkPara>(para_receiver_assertions);
	test.set_dispatchable::<AssetHubPaseoPara>(system_para_to_para_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	let delivery_fees = AssetHubPaseoPara::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<AssetHubPaseoXcmConfig as xcm_executor::Config>::XcmSender,
		>(test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest)
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

	// Setup: reserve transfer from AH to Pop, so that sovereign account accurate for return transfer
	let amount_to_send: Balance = ASSET_HUB_PASEO_ED * 1000;
	fund_pop_from_system_para(
		AssetHubPaseoParaSender::get(),
		amount_to_send,
		PopNetworkParaReceiver::get(),
		(Parent, amount_to_send).into(),
	); // alice on asset hub > bob on pop

	// Init values for Pop Network Parachain
	let destination = PopNetworkPara::sibling_location_of(AssetHubPaseoPara::para_id());
	let beneficiary_id = AssetHubPaseoParaReceiver::get(); // bob on asset hub
	let amount_to_send = PopNetworkPara::account_data_of(PopNetworkParaReceiver::get()).free; // bob on pop balance
	let assets = (Parent, amount_to_send).into();

	let test_args = TestContext {
		sender: PopNetworkParaReceiver::get(),      // bob on pop
		receiver: AssetHubPaseoParaReceiver::get(), // bob on asset hub
		args: TestArgs::new_para(destination, beneficiary_id, amount_to_send, assets, None, 0),
	};

	let mut test = ParaToSystemParaTest::new(test_args);

	let sender_balance_before = test.sender.balance;
	let receiver_balance_before = test.receiver.balance;

	let pop_net_location_as_seen_by_ahr =
		AssetHubPaseoPara::sibling_location_of(PopNetworkPara::para_id());
	let sov_pop_net_on_ahr =
		AssetHubPaseoPara::sovereign_account_id_of(pop_net_location_as_seen_by_ahr);

	// fund Pop Network's SA on AHR with the native tokens held in reserve
	AssetHubPaseoPara::fund_accounts(vec![(sov_pop_net_on_ahr.into(), amount_to_send * 2)]);

	test.set_assertion::<PopNetworkPara>(para_to_system_para_sender_assertions);
	test.set_assertion::<AssetHubPaseoPara>(para_to_system_para_receiver_assertions);
	test.set_dispatchable::<PopNetworkPara>(para_to_system_para_reserve_transfer_assets);
	test.assert();

	let sender_balance_after = test.sender.balance;
	let receiver_balance_after = test.receiver.balance;

	let delivery_fees = PopNetworkPara::execute_with(|| {
		xcm_helpers::teleport_assets_delivery_fees::<
			<PopNetworkXcmConfig as xcm_executor::Config>::XcmSender,
		>(test.args.assets.clone(), 0, test.args.weight_limit, test.args.beneficiary, test.args.dest)
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
// 	let beneficiary: sp_runtime::AccountId32 = [1u8; 32].into();
//
// 	// Setup: reserve transfer from relay to Pop, so that sovereign account accurate for return transfer
// 	let amount_to_send: Balance = pop_runtime::UNIT * 1000;
// 	fund_pop_from_relay(PaseoRelaySender::get(), amount_to_send, beneficiary.clone());
//
// 	let message = {
// 		let assets: Asset = (Here, 10 * pop_runtime::UNIT).into();
// 		let beneficiary = AccountId32 { id: beneficiary.clone().into(), network: None }.into();
// 		let spot_order = <PaseoRelay as Chain>::RuntimeCall::OnDemandAssignmentProvider(
// 			assigner_on_demand::Call::<<PaseoRelay as Chain>::Runtime>::place_order_keep_alive {
// 				max_amount: 1 * pop_runtime::UNIT,
// 				para_id: AssetHubPaseoPara::para_id().into(),
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
// 				destination: PaseoRelay::child_location_of(PopNetworkPara::para_id()),
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
// 	PaseoRelay::execute_with(|| {
// 		type RuntimeEvent = <PaseoRelay as Chain>::RuntimeEvent;
// 		assert_expected_events!(
// 			PaseoRelay,
// 			vec![
// 				// We currently only check that the message was processed successfully
// 				RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) => {},
// 				// TODO: check order placed once we can have on-demand para id registered (probably via setting raw storage as a workaround)
// 				// RuntimeEvent::OnDemandAssignmentProvider(assigner_on_demand::Event::OnDemandOrderPlaced {
// 				// 	..
// 				// }) => {},
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
// 				RuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) => {},
// 			]
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
