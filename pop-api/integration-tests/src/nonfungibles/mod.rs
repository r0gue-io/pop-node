use pallet_nfts::CollectionConfig;
use pop_api::{
	nonfungibles::{
		events::{Approval, AttributeSet, Transfer},
		types::*,
	},
	primitives::BlockNumber,
};
use pop_primitives::{ArithmeticError::*, Error, Error::*, TokenError::*};
use utils::*;

use super::*;

mod utils;

const COLLECTION_ID: CollectionId = 0;
const ITEM_ID: ItemId = 1;
const CONTRACT: &str = "contracts/nonfungibles/target/ink/nonfungibles.wasm";

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			total_supply(&addr, COLLECTION_ID),
			Ok(Nfts::collection_items(COLLECTION_ID).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(0));

		// Tokens in circulation.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(
			total_supply(&addr, COLLECTION_ID),
			Ok(Nfts::collection_items(COLLECTION_ID).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(1));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			balance_of(&addr, COLLECTION_ID, ALICE),
			Ok(nfts::balance_of(COLLECTION_ID, ALICE)),
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(0));

		// Tokens in circulation.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(
			balance_of(&addr, COLLECTION_ID, ALICE),
			Ok(nfts::balance_of(COLLECTION_ID, ALICE)),
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(1));
	});
}

// Testing a contract that creates a token in the constructor.
#[test]
fn instantiate_and_create_fungible_works() {
	new_test_ext().execute_with(|| {
		// let contract =
		// 	"contracts/create_token_in_constructor/target/ink/create_token_in_constructor.wasm";
		// // Token already exists.
		// assets::create(&ALICE, 0, 1);
		// assert_eq!(
		// 	instantiate_and_create_fungible(contract, 0, 1),
		// 	Err(Module { index: 52, error: [5, 0] })
		// );
		// // Successfully create a token when instantiating the contract.
		// let result_with_address = instantiate_and_create_fungible(contract, TOKEN_ID, 1);
		// let instantiator = result_with_address.clone().ok();
		// assert_ok!(result_with_address);
		// assert_eq!(&Assets::owner(TOKEN_ID), &instantiator);
		// assert!(Assets::asset_exists(TOKEN_ID));
		// // Successfully emit event.
		// let instantiator = account_id_from_slice(instantiator.unwrap().as_ref());
		// let expected =
		// 	Created { id: TOKEN_ID, creator: instantiator.clone(), admin: instantiator }.encode();
		// assert_eq!(last_contract_event(), expected.as_slice());
	});
}
