use core::clone::Clone;

use frame_support::{assert_ok, pallet_prelude::TypeInfo};
use pallet_revive::{test_utils::AccountId32, AccountId32Mapper, ExecReturnValue};
use scale::Encode;
use sp_runtime::app_crypto::sp_core::H160;

use super::*;
use crate::{bare_call, function_selector, instantiate, new_test_ext, upload, INIT_VALUE};

const NFT_VERIFIER: &str = "contracts/dao/nft-verifier/target/ink/nft_verifier.riscv";
const DAO: &str = "contracts/dao/target/ink/dao.riscv";

#[test]
fn instantiate_dao() {
	new_test_ext().execute_with(|| {
		let expected_collection =
			pallet_nfts::NextCollectionId::<Runtime>::get().unwrap_or_default();
		let contract = Dao::new();
		let Dao((h160, id32)) = contract.clone();
		let collection = contract.collection_id();
		assert_eq!(collection, expected_collection);
		assert_eq!(
			pallet_nfts::Collection::<Runtime>::get(collection),
			Some(pallet_nfts::CollectionDetails {
				owner: id32.clone(),
				owner_deposit: 100000000000,
				items: 0,
				item_metadatas: 0,
				item_configs: 0,
				attributes: 0,
			})
		);
		assert!(pallet_nfts::CollectionConfigOf::<Runtime>::get(collection).is_some());
		let item = 0;
		assert_ok!(contract.register(0, item));
		let dao_nft = contract.complete(item, true).expect("Registration failed");
		// Test some errors.
		{
			assert_eq!(contract.complete(10, true), Err(Error::Unknown));
			assert_eq!(contract.complete(0, false), Err(Error::Rejected));
		}
		assert_eq!(pallet_nfts::Account::<Runtime>::get((ALICE, collection, dao_nft)), Some(()));
		assert_eq!(ALICE, pallet_nfts::Pallet::<Runtime>::owner(collection, dao_nft).unwrap());
	});
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum Error {
	StatusCode(u32),
	NotReady,
	Unknown,
	DecodingFailed,
	Rejected,
}

#[derive(Clone, Debug, PartialEq)]
struct Dao((H160, AccountId32));
impl Dao {
	fn new() -> Self {
		let function = function_selector("new");
		Self(instantiate(DAO, INIT_VALUE, function, vec![]))
	}

	fn register(&self, height: u32, item: u32) -> Result<(), Error> {
		let result = self.call("register", (height, item).encode(), 0);
		Result::<(), Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn complete(&self, item: u32, outcome: bool) -> Result<u32, Error> {
		let result = self.call("complete", (item, outcome).encode(), 0);
		Result::<u32, Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn collection_id(&self) -> u32 {
		// let function = function_selector("collection_id");
		// let result = bare_call(self.0.clone().0, params, value).expect("should work");
		let result = self.call("collection_id", vec![], 0);
		u32::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		bare_call(self.0.clone().0, params, value).expect("should work")
	}
}

// #[test]
// fn verify_works() {
//     let mut ext = new_test_ext();
//     let contract = ext.execute_with(|| {
//         let contract = NftVerifier::new();
//         assert_ok!(contract.verify());
//         let (h160, id32) = contract.clone().0;
//         let result = pallet_api::messaging::pallet::Messages::<Runtime>::get(id32, 0u64);
//         println!("result: {:?}", result);
//         // assert_eq!(contract.poll(id).unwrap(), Some(Status::Pending));
//
//         // TODO: assert events from messaging and ismp pallets emitted
//         println!("{:#?}", System::events());
//
//         contract
//     });
// }

// #[derive(Clone)]
// struct NftVerifier((H160, AccountId32));
// impl NftVerifier {
// 	fn new() -> Self {
// 		let function = function_selector("new");
// 		let input = [function, 1000u32.encode(), 0u32.encode()].concat();
// 		Self(instantiate(NFT_VERIFIER, INIT_VALUE, input, vec![]))
// 	}
//
// 	fn upload() -> sp_core::H256 {
// 		upload(NFT_VERIFIER, INIT_VALUE / 2)
// 	}
//
// 	fn verify(&self) -> Result<(), Error> {
// 		let result = self.call("verify", 0);
// 		Result::<(), Error>::decode(&mut &result.data[1..])
// 			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
// 	}
//
// 	fn call(&self, function: &str, value: u128) -> ExecReturnValue {
// 		let function = function_selector(function);
// 		let params = [function, 0u32.encode(), 0u32.encode()].concat();
// 		bare_call(self.0.clone().0, params, value).expect("should work")
// 	}
// }
//
