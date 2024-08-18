use crate::{
	fungibles::{utils::*, *},
	new_test_ext, ALICE, BOB,
};
use frame_support::assert_ok;
use pop_runtime_devnet::{Assets, Contracts, Runtime, System};
use scale::Encode;
use sp_runtime::{traits::Hash, AccountId32, BuildStorage, DispatchError};

fn create_event(id: AssetId, contract: AccountId32) -> Vec<u8> {
	pop_api::assets::fungibles::events::Create { id, creator: contract.clone(), admin: contract }
		.encode()
}

#[test]
fn instantiate_and_create_fungible_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let contract =
			"contracts/create_token_in_constructor/target/ink/create_token_in_constructor.wasm";
		// Asset already exists.
		create_asset(ALICE, 0, 1);
		assert_eq!(
			instantiate_and_create_fungible(contract, 0, 1),
			Err(Module { index: 52, error: 5 })
		);
		// Successfully create an asset when instantiating the contract.
		let contract = instantiate_and_create_fungible(contract, ASSET_ID, 1).expect("Should work");
		assert!(Assets::asset_exists(ASSET_ID));

		// Test events.
		// let events = System::read_events_for_pallet::<pallet_contracts::Event<Runtime>>();
		let events = frame_system::Pallet::<Runtime>::read_events_for_pallet::<
			pallet_contracts::Event<Runtime>,
		>();
		let event: Vec<&[u8]> = events
			.iter()
			.filter_map(|event| match event {
				pallet_contracts::Event::<Runtime>::ContractEmitted { data, .. } => {
					Some(data.as_slice())
				},
				_ => None,
			})
			.collect();
		let expected = create_event(ASSET_ID, contract.clone());
		assert_eq!(event.last().unwrap(), &expected.as_slice());
	});
}
