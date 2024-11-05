#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::hash::{Blake2x128, CryptoHash},
	prelude::vec::Vec,
	scale::{Decode, Encode},
	storage::Mapping,
};
use pop_api::{
	messaging::{
		self as api,
		ismp::{self, Get},
		ParaId, RequestId, Status,
	},
	nonfungibles::{CollectionId, ItemId},
	StatusCode,
};

pub use self::nft_verifier::{NftVerifier, NftVerifierRef};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
	StatusCode(u32),
	NotReady,
	Unknown,
	DecodingFailed,
}

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		Error::StatusCode(value.0)
	}
}

#[ink::contract]
mod nft_verifier {
	use super::{Error::*, *};

	#[ink(storage)]
	pub struct NftVerifier {
		parachain: ParaId,
		collection: CollectionId,
		requests: Mapping<(AccountId, ItemId), RequestId>,
		next_request: RequestId,
	}

	impl NftVerifier {
		#[ink(constructor, payable)]
		pub fn new(parachain: ParaId, collection: CollectionId) -> Self {
			Self { parachain, collection, requests: Mapping::default(), next_request: 0 }
			// TODO: verify that collection exists.
		}

		// TODO: can we get the height for the developer?
		#[ink(message, payable)]
		pub fn verify(&mut self, height: u32, item: ItemId) -> Result<()> {
			self.next_request = self.next_request.saturating_add(1);
			let account = self.env().caller();
			let key: Vec<u8> = generate_key(account.clone(), self.collection, item);
			ismp::get(
				self.next_request,
				Get::new(self.parachain, height, 0, Vec::default(), Vec::from([key])),
				0,
			)?;
			self.requests.insert((account, item), &self.next_request);
			self.env().emit_event(NftVerificationEnacted { account, item });
			Ok(())
		}

		#[ink(message, payable)]
		pub fn complete(&mut self, item: ItemId) -> Result<bool> {
			let account = self.env().caller();
			let request = self.requests.get((&account, item)).ok_or(Unknown)?;
			if let Ok(Some(status)) = api::poll((self.env().account_id(), request)) {
				if status == Status::Complete {
					if let Some(result) = api::get((self.env().account_id(), request))? {
						api::remove([request].to_vec())?;
						let result = match Option::<()>::decode(&mut &result[..])
							.map_err(|_| DecodingFailed)?
						{
							Some(()) => true,
							None => false,
						};
						self.env().emit_event(NftVerificationCompleted { account, item, result });
						return Ok(result)
					}
				}
			}
			Err(NotReady)
		}
	}

	// This function returns the complete storage key for the NFTs pallet's `Account` storage map.
	pub fn generate_key(account: AccountId, collection_id: u32, item_id: u32) -> Vec<u8> {
		// The storage map prefix.
		let storage_map_prefix: [u8; 32] = [
			232, 212, 147, 137, 194, 226, 62, 21, 47, 221, 99, 100, 218, 173, 210, 204, 185, 157,
			136, 14, 198, 129, 121, 156, 12, 243, 14, 136, 134, 55, 29, 169,
		];
		// Hash and concatenate each component using blake2_128_concat logic.
		let hashed_account = blake2_128_concat(&account.encode());
		let hashed_collection = blake2_128_concat(&collection_id.to_le_bytes());
		let hashed_item = blake2_128_concat(&item_id.to_le_bytes());
		// Concatenate the storage map prefix with the hashed key components
		let mut complete_key = Vec::new();
		complete_key.extend_from_slice(&storage_map_prefix);
		complete_key.extend_from_slice(&hashed_account);
		complete_key.extend_from_slice(&hashed_collection);
		complete_key.extend_from_slice(&hashed_item);
		complete_key
	}

	// A helper function to perform the `blake2_128_concat` logic.
	// This will hash the input and concatenate the result with the original input.
	fn blake2_128_concat(input: &[u8]) -> Vec<u8> {
		let mut output = [0u8; 16]; // blake2_128 produces a 128-bit (16 bytes) hash
		Blake2x128::hash(input, &mut output);
		// Concatenate the hash with the original input
		let mut result = Vec::new();
		result.extend_from_slice(&output);
		result.extend_from_slice(input);
		result
	}

	#[ink::event]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct NftVerificationEnacted {
		pub account: AccountId,
		pub item: ItemId,
	}

	#[ink::event]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct NftVerificationCompleted {
		pub account: AccountId,
		pub item: ItemId,
		pub result: bool,
	}

	#[cfg(test)]
	mod tests {
		use hex::FromHex;

		use super::*;

		// #[ink::test]
		// fn generate_key_works() {
		// 	let alice_pop = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
		// 	let accountid_32: [u8; 32] =
		// 		sp_runtime::AccountId32::from_string(alice_pop).unwrap().into();
		// 	let key = generate_key(accountid_32.into(), 0, 0);
		// 	println!("Complete storage key (hex): {}", HexDisplay::from(&key));
		// }

		// #[ink::test]
		// fn decode_completed_event() {
		// 	let result_bytes =
		// hex::decode("
		// 020000000000000001f10104e101e8d49389c2e23e152fdd6364daadd2ccb99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d11d2df4e979aa105cf552e9544ebd2b500000000d82c12285b5d4551f88e8f6e7eb52b810100000000"
		// ).expect("come one"); 	// println!("Result: {:?}", result_bytes);
		// 	let key_bytes =
		// hex::decode("
		// e8d49389c2e23e152fdd6364daadd2ccb99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d11d2df4e979aa105cf552e9544ebd2b500000000d82c12285b5d4551f88e8f6e7eb52b8101000000"
		// ).expect("come one"); 	// println!("key: {:?}", key_bytes);
		// 	let mut result_slice = &result_bytes[..];
		// 	let Completed { id, result } = Completed::decode(&mut result_slice)?;
		// 	println!("Result 1: {:?}", result);
		// 	// let result = Completed::decode(&mut result_slice)?;
		// 	let result = find_extra_bytes_at_end(result.unwrap(), key_bytes);
		// 	println!("Result: {:?}", result);
		// 	let result = Option::<()>::decode(&mut &result[..]).expect("should work");
		// 	println!("Result: {:?}", result);
		// }

		#[ink::test]
		fn decode_completed_event() {
			let result_bytes = hex::decode(
				"d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0a00000000",
			)
			.expect("come one");
			let result = NftVerificationCompleted::decode(&mut &result_bytes[..])?;
			println!("Result: {:?}", result);
			// let result = Option::<()>::decode(&mut &(result.unwrap())[..]).expect("should work");
			// println!("Decoded Result: {:?}", result);
		}

		fn find_extra_bytes_at_end(v1: Vec<u8>, v2: Vec<u8>) -> Vec<u8> {
			// Ensure `v1` is the longer vector.
			let (longer, shorter) = if v1.len() > v2.len() { (v1, v2) } else { (v2, v1) };

			// Find where the common section starts.
			let start_offset = longer
				.windows(shorter.len())
				.position(|window| window == shorter)
				.expect("Shorter vector not found in longer vector");

			// Calculate where the common section ends in `longer`.
			let common_end_index = start_offset + shorter.len();

			// Return a new Vec containing the extra bytes at the end.
			longer[common_end_index..].to_vec()
		}

		// #[ink::test]
		// fn it_works() {
		// 	let mut nft_verifier = NftVerifier::new(false);
		// 	assert_eq!(nft_verifier.get(), false);
		// 	nft_verifier.flip();
		// 	assert_eq!(nft_verifier.get(), true);
		// }
	}
}
