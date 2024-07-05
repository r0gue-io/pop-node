use codec::{Compact, Decode, Encode};
use cumulus_pallet_parachain_system::RelaychainDataProvider;
use frame_support::traits::{Contains, OriginTrait};
use frame_support::{
	dispatch::{GetDispatchInfo, RawOrigin},
	pallet_prelude::*,
	traits::{
		fungibles::{approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect},
		nonfungibles_v2::Inspect as NonFungiblesInspect,
	},
};
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
	traits::{BlockNumberProvider, Dispatchable},
	DispatchError, MultiAddress,
};
use sp_std::{boxed::Box, vec::Vec};
use xcm::{
	latest::{prelude::*, OriginKind::SovereignAccount},
	VersionedXcm,
};

use crate::{
	config::assets::TrustBackedAssetsInstance, AccountId, AllowedApiCalls, Balance, Runtime,
	RuntimeCall, RuntimeOrigin, UNIT,
};
use pop_primitives::{
	cross_chain::CrossChainMessage,
	nfts::{CollectionId, ItemId},
	storage_keys::{
		AssetsKeys::{self, *},
		NftsKeys, ParachainSystemKeys, RuntimeStateKeys,
	},
	AssetId,
};

mod v0;

const LOG_TARGET: &str = "pop-api::extension";
// Versions:
const V0: u8 = 0;

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

#[derive(Default)]
pub struct PopApiExtension;

// TODO: check removal or simplification of trait bounds.
impl<T> ChainExtension<T> for PopApiExtension
where
	T: pallet_contracts::Config
		+ pallet_xcm::Config
		+ pallet_assets::Config<TrustBackedAssetsInstance, AssetId = AssetId>
		+ pallet_nfts::Config<CollectionId = CollectionId, ItemId = ItemId>
		+ cumulus_pallet_parachain_system::Config
		+ frame_system::Config<
			RuntimeOrigin = RuntimeOrigin,
			AccountId = AccountId,
			RuntimeCall = RuntimeCall,
		>,
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
	{
		log::debug!(target:LOG_TARGET, " extension called ");
		let mut env = env.buf_in_buf_out();
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going
		// from contract layer to substrate layer.
		// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
		let contract_host_weight = ContractSchedule::<T>::get().host_fn_weights;
		env.charge_weight(contract_host_weight.debug_message)?;

		// Extract version and function_id from first two bytes.
		let (version, function_id) = {
			let bytes = env.func_id().to_le_bytes();
			(bytes[0], bytes[1])
		};
		// Extract pallet index and call / key index from last two bytes.
		let (pallet_index, call_index) = {
			let bytes = env.ext_id().to_le_bytes();
			(bytes[0], bytes[1])
		};

		let result = match FuncId::try_from(function_id) {
			Ok(function_id) => {
				// Read encoded parameters from buffer and calculate weight for reading `len` bytes`.
				// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				let params = env.read(len)?;
				log::debug!(target: LOG_TARGET, "Read input successfully");
				match function_id {
					FuncId::Dispatch => {
						dispatch::<T, E>(&mut env, version, pallet_index, call_index, params)
					},
					FuncId::ReadState => {
						read_state::<T, E>(&mut env, version, pallet_index, call_index, params)
					},
					// TODO
					FuncId::SendXcm => send_xcm::<T, E>(&mut env),
				}
			},
			Err(e) => Err(e),
		};

		match result {
			Ok(_) => Ok(RetVal::Converging(0)),
			Err(e) => Ok(RetVal::Converging(convert_to_status_code(e, version))),
		}
	}
}

fn dispatch<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	params: Vec<u8>,
) -> Result<(), DispatchError>
where
	T: frame_system::Config<RuntimeOrigin = RuntimeOrigin, RuntimeCall = RuntimeCall>,
	RuntimeOrigin: From<RawOrigin<T::AccountId>>,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " dispatch |";
	let call = construct_call(version, pallet_index, call_index, params)?;
	// Contract is the origin by default.
	let origin: RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();
	dispatch_call::<T, E>(env, call, origin, LOG_PREFIX)
}

fn dispatch_call<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	call: RuntimeCall,
	mut origin: RuntimeOrigin,
	log_prefix: &str,
) -> Result<(), DispatchError>
where
	T: frame_system::Config<RuntimeOrigin = RuntimeOrigin, RuntimeCall = RuntimeCall>,
	RuntimeOrigin: From<RawOrigin<T::AccountId>>,
	E: Ext<T = T>,
{
	let charged_dispatch_weight = env.charge_weight(call.get_dispatch_info().weight)?;
	log::debug!(target:LOG_TARGET, "{} Inputted RuntimeCall: {:?}", log_prefix, call);
	origin.add_filter(AllowedApiCalls::contains);
	match call.dispatch(origin) {
		Ok(info) => {
			log::debug!(target:LOG_TARGET, "{} success, actual weight: {:?}", log_prefix, info.actual_weight);
			// Refund weight if the actual weight is less than the charged weight.
			if let Some(actual_weight) = info.actual_weight {
				env.adjust_weight(charged_dispatch_weight, actual_weight);
			}
			Ok(())
		},
		Err(err) => {
			log::debug!(target:LOG_TARGET, "{} failed: error: {:?}", log_prefix, err.error);
			Err(err.error)
		},
	}
}

fn construct_call(
	version: u8,
	pallet_index: u8,
	call_index: u8,
	params: Vec<u8>,
) -> Result<RuntimeCall, DispatchError> {
	match pallet_index {
		52 => {
			let call = versioned_construct_assets_call(version, call_index, params)?;
			Ok(RuntimeCall::Assets(call))
		},
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}

fn construct_key(
	version: u8,
	pallet_index: u8,
	call_index: u8,
	params: Vec<u8>,
) -> Result<RuntimeStateKeys, DispatchError> {
	match pallet_index {
		52 => {
			let key = versioned_construct_assets_key(version, call_index, params)?;
			Ok(RuntimeStateKeys::Assets(key))
		},
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}

fn versioned_construct_assets_call(
	version: u8,
	call_index: u8,
	params: Vec<u8>,
) -> Result<pallet_assets::Call<Runtime, TrustBackedAssetsInstance>, DispatchError> {
	match version {
		V0 => v0::assets::construct_assets_call(call_index, params),
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}

fn versioned_construct_assets_key(
	version: u8,
	call_index: u8,
	params: Vec<u8>,
) -> Result<AssetsKeys, DispatchError> {
	match version {
		V0 => v0::assets::construct_assets_key(call_index, params),
		_ => Err(DispatchError::Other("UnknownFunctionId")),
	}
}

fn read_state<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	params: Vec<u8>,
) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config
		+ pallet_assets::Config<TrustBackedAssetsInstance, AssetId = AssetId>
		+ pallet_nfts::Config<CollectionId = CollectionId, ItemId = ItemId>
		+ cumulus_pallet_parachain_system::Config
		+ frame_system::Config<AccountId = sp_runtime::AccountId32>,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " read_state |";
	let key = construct_key(version, pallet_index, call_index, params)?;
	let result = match key {
		RuntimeStateKeys::Nfts(key) => read_nfts_state::<T, E>(key, env),
		RuntimeStateKeys::ParachainSystem(key) => read_parachain_system_state::<T, E>(key, env),
		RuntimeStateKeys::Assets(key) => read_assets_state::<T, E>(key, env),
	}?
	.encode();
	log::trace!(
		target:LOG_TARGET,
		"{} result: {:?}.", LOG_PREFIX, result
	);
	env.write(&result, false, None)
}

fn send_xcm<T, E>(env: &mut Environment<E, BufInBufOutState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config
		+ frame_system::Config<
			RuntimeOrigin = RuntimeOrigin,
			AccountId = AccountId,
			RuntimeCall = RuntimeCall,
		>,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " send_xcm |";
	// Read the input as CrossChainMessage.
	let xc_call: CrossChainMessage = env.read_as::<CrossChainMessage>()?;
	// Determine the call to dispatch.
	let (dest, message) = match xc_call {
		CrossChainMessage::Relay(message) => {
			let dest = Location::parent().into_versioned();
			let assets: Asset = (Here, 10 * UNIT).into();
			let beneficiary: Location =
				AccountId32 { id: (env.ext().address().clone()).into(), network: None }.into();
			let message = Xcm::builder()
				.withdraw_asset(assets.clone().into())
				.buy_execution(assets.clone(), Unlimited)
				.transact(
					SovereignAccount,
					Weight::from_parts(250_000_000, 10_000),
					message.encode().into(),
				)
				.refund_surplus()
				.deposit_asset(assets.into(), beneficiary)
				.build();
			(dest, message)
		},
	};
	// TODO: revisit to replace with signed contract origin
	let origin: RuntimeOrigin = RawOrigin::Root.into();
	// Generate runtime call to dispatch.
	let call = RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
		dest: Box::new(dest),
		message: Box::new(VersionedXcm::V4(message)),
	});
	dispatch_call::<T, E>(env, call, origin, LOG_PREFIX)
}

// Converts a `DispatchError` to a `u32` status code based on the version of the API the contract uses.
// The contract calling the chain extension can convert the status code to the descriptive `Error`.
//
// For `Error` see `pop_primitives::<version>::error::Error`.
//
// The error encoding can vary per version, allowing for flexible and backward-compatible error handling.
// As a result, contracts maintain compatibility across different versions of the runtime.
//
// # Parameters
//
// - `error`: The `DispatchError` encountered during contract execution.
// - `version`: The version of the chain extension, used to determine the known errors.
pub(crate) fn convert_to_status_code(error: DispatchError, version: u8) -> u32 {
	// "UnknownFunctionId" and "DecodingFailed" are mapped to specific errors in the API and will
	// never change.
	let mut encoded_error = match error {
		DispatchError::Other("UnknownFunctionId") => vec![254, 0, 0, 0],
		DispatchError::Other("DecodingFailed") => vec![255, 0, 0, 0],
		_ => error.encode(),
	};
	// Resize the encoded value to 4 bytes in order to decode the value in a u32 (4 bytes).
	encoded_error.resize(4, 0);
	let mut encoded_error = encoded_error.try_into().expect("qid, resized to 4 bytes line above");
	match version {
		// If an unknown variant of the `DispatchError` is detected the error needs to be converted
		// into the encoded value of `Error::Other`. This conversion is performed by shifting the bytes one
		// position forward (discarding the last byte as it is not used) and setting the first byte to the
		// encoded value of `Other` (0u8). This ensures the error is correctly categorized as an `Other`
		// variant which provides all the necessary information to debug which error occurred in the runtime.
		//
		// Byte layout explanation:
		// - Byte 0: index of the variant within `Error`
		// - Byte 1:
		//   - Must be zero for `UNIT_ERRORS`.
		//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
		//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
		// - Byte 2:
		//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
		// - Byte 3:
		//   - Unused or represents further nested information.
		0 => v0::error::handle_unknown_error(&mut encoded_error),
		_ => encoded_error = [254, 0, 0, 0],
	}
	u32::from_le_bytes(encoded_error)
}

#[derive(Debug)]
pub enum FuncId {
	Dispatch,
	ReadState,
	SendXcm,
}

impl TryFrom<u8> for FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u8) -> Result<Self, Self::Error> {
		let id = match func_id {
			0 => Self::Dispatch,
			1 => Self::ReadState,
			2 => Self::SendXcm,
			_ => {
				return Err(DispatchError::Other("UnknownFuncId"));
			},
		};
		Ok(id)
	}
}

fn read_parachain_system_state<T, E>(
	key: ParachainSystemKeys,
	env: &mut Environment<E, BufInBufOutState>,
) -> Result<Vec<u8>, DispatchError>
where
	T: pallet_contracts::Config + cumulus_pallet_parachain_system::Config,
	E: Ext<T = T>,
{
	match key {
		ParachainSystemKeys::LastRelayChainBlockNumber => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(RelaychainDataProvider::<T>::current_block_number().encode())
		},
	}
}

fn read_nfts_state<T, E>(
	key: NftsKeys,
	env: &mut Environment<E, BufInBufOutState>,
) -> Result<Vec<u8>, DispatchError>
where
	T: pallet_contracts::Config + pallet_nfts::Config<CollectionId = CollectionId, ItemId = ItemId>,
	E: Ext<T = T>,
{
	match key {
		NftsKeys::Collection(collection) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Collection::<T>::get(collection).encode())
		},
		NftsKeys::CollectionOwner(collection) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Pallet::<T>::collection_owner(collection).encode())
		},
		NftsKeys::Item(collection, item) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Item::<T>::get(collection, item).encode())
		},
		NftsKeys::Owner(collection, item) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Pallet::<T>::owner(collection, item).encode())
		},
		NftsKeys::Attribute(collection, item, key) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Pallet::<T>::attribute(&collection, &item, &key).encode())
		},
		// NftsKeys::CustomAttribute(account, collection, item, key) => {
		// 	env.charge_weight(T::DbWeight::get().reads(1_u64))?;
		// 	Ok(pallet_nfts::Pallet::<T>::custom_attribute(&account, &collection, &item, &key)
		// 		.encode())
		// },
		NftsKeys::SystemAttribute(collection, item, key) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Pallet::<T>::system_attribute(&collection, item.as_ref(), &key)
				.encode())
		},
		NftsKeys::CollectionAttribute(collection, key) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_nfts::Pallet::<T>::collection_attribute(&collection, &key).encode())
		},
	}
}

fn read_assets_state<T, E>(
	key: AssetsKeys,
	env: &mut Environment<E, BufInBufOutState>,
) -> Result<Vec<u8>, DispatchError>
where
	T: pallet_contracts::Config
		+ pallet_assets::Config<TrustBackedAssetsInstance, AssetId = AssetId>,
	E: Ext<T = T>,
	T: frame_system::Config<AccountId = sp_runtime::AccountId32>,
{
	match key {
		TotalSupply(id) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_assets::Pallet::<T, TrustBackedAssetsInstance>::total_supply(id).encode())
		},
		BalanceOf(id, owner) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_assets::Pallet::<T, TrustBackedAssetsInstance>::balance(id, &owner.0.into())
				.encode())
		},
		Allowance(id, owner, spender) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(pallet_assets::Pallet::<T, TrustBackedAssetsInstance>::allowance(
				id,
				&owner.0.into(),
				&spender.0.into(),
			)
			.encode())
		},
		TokenName(id) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(<pallet_assets::Pallet<T, TrustBackedAssetsInstance> as MetadataInspect<
				AccountId,
			>>::name(id)
			.encode())
		},
		TokenSymbol(id) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(<pallet_assets::Pallet<T, TrustBackedAssetsInstance> as MetadataInspect<
				AccountId,
			>>::symbol(id)
			.encode())
		},
		TokenDecimals(id) => {
			env.charge_weight(T::DbWeight::get().reads(1_u64))?;
			Ok(<pallet_assets::Pallet<T, TrustBackedAssetsInstance> as MetadataInspect<
				AccountId,
			>>::decimals(id)
			.encode())
		},
		// AssetsKeys::AssetExists(id) => {
		// 	env.charge_weight(T::DbWeight::get().reads(1_u64))?;
		// 	Ok(pallet_assets::Pallet::<T, TrustBackedAssetsInstance>::asset_exists(id).encode())
		// },
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{Assets, Runtime, System};
	use sp_runtime::BuildStorage;
	// Test ensuring `func_id()` and `ext_id()` work as expected, i.e. extracting the first two
	// bytes and the last two bytes, respectively, from a 4 byte array.
	#[test]
	fn test_byte_extraction() {
		use rand::Rng;

		// Helper functions
		fn func_id(id: u32) -> u16 {
			(id & 0x0000FFFF) as u16
		}
		fn ext_id(id: u32) -> u16 {
			(id >> 16) as u16
		}

		// Number of test iterations
		let test_iterations = 1_000_000;

		// Create a random number generator
		let mut rng = rand::thread_rng();

		// Run the test for a large number of random 4-byte arrays
		for _ in 0..test_iterations {
			// Generate a random 4-byte array
			let bytes: [u8; 4] = rng.gen();

			// Convert the 4-byte array to a u32 value
			let value = u32::from_le_bytes(bytes);

			// Extract the first two bytes (least significant 2 bytes)
			let first_two_bytes = func_id(value);

			// Extract the last two bytes (most significant 2 bytes)
			let last_two_bytes = ext_id(value);

			// Check if the first two bytes match the expected value
			assert_eq!([bytes[0], bytes[1]], first_two_bytes.to_le_bytes());

			// Check if the last two bytes match the expected value
			assert_eq!([bytes[2], bytes[3]], last_two_bytes.to_le_bytes());
		}
	}

	// Test showing all the different type of variants and its encoding.
	#[test]
	fn encoding_of_enum() {
		#[derive(Debug, PartialEq, Encode, Decode)]
		enum ComprehensiveEnum {
			SimpleVariant,
			DataVariant(u8),
			NamedFields { w: u8 },
			NestedEnum(InnerEnum),
			OptionVariant(Option<u8>),
			VecVariant(Vec<u8>),
			TupleVariant(u8, u8),
			NestedStructVariant(NestedStruct),
			NestedEnumStructVariant(NestedEnumStruct),
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		enum InnerEnum {
			A,
			B { inner_data: u8 },
			C(u8),
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct NestedStruct {
			x: u8,
			y: u8,
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct NestedEnumStruct {
			inner_enum: InnerEnum,
		}

		// Creating each possible variant for an enum.
		let enum_simple = ComprehensiveEnum::SimpleVariant;
		let enum_data = ComprehensiveEnum::DataVariant(42);
		let enum_named = ComprehensiveEnum::NamedFields { w: 42 };
		let enum_nested = ComprehensiveEnum::NestedEnum(InnerEnum::B { inner_data: 42 });
		let enum_option = ComprehensiveEnum::OptionVariant(Some(42));
		let enum_vec = ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4, 5]);
		let enum_tuple = ComprehensiveEnum::TupleVariant(42, 42);
		let enum_nested_struct =
			ComprehensiveEnum::NestedStructVariant(NestedStruct { x: 42, y: 42 });
		let enum_nested_enum_struct =
			ComprehensiveEnum::NestedEnumStructVariant(NestedEnumStruct {
				inner_enum: InnerEnum::C(42),
			});

		// Encode and print each variant individually to see their encoded values.
		println!("{:?} -> {:?}", enum_simple, enum_simple.encode());
		println!("{:?} -> {:?}", enum_data, enum_data.encode());
		println!("{:?} -> {:?}", enum_named, enum_named.encode());
		println!("{:?} -> {:?}", enum_nested, enum_nested.encode());
		println!("{:?} -> {:?}", enum_option, enum_option.encode());
		println!("{:?} -> {:?}", enum_vec, enum_vec.encode());
		println!("{:?} -> {:?}", enum_tuple, enum_tuple.encode());
		println!("{:?} -> {:?}", enum_nested_struct, enum_nested_struct.encode());
		println!("{:?} -> {:?}", enum_nested_enum_struct, enum_nested_enum_struct.encode());
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn encoding_decoding_dispatch_error() {
		use sp_runtime::{ArithmeticError, DispatchError, ModuleError, TokenError};

		new_test_ext().execute_with(|| {
			let error = DispatchError::Module(ModuleError {
				index: 255,
				error: [2, 0, 0, 0],
				message: Some("error message"),
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 255, 2, 0, 0, 0]);
			assert_eq!(
				decoded,
				// `message` is skipped for encoding.
				DispatchError::Module(ModuleError {
					index: 255,
					error: [2, 0, 0, 0],
					message: None
				})
			);

			// Example pallet assets Error into ModuleError.
			let index = <<Runtime as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<
				Assets,
			>()
			.expect("Every active module has an index in the runtime; qed") as u8;
			let mut error =
				pallet_assets::Error::NotFrozen::<Runtime, TrustBackedAssetsInstance>.encode();
			error.resize(MAX_MODULE_ERROR_ENCODED_SIZE, 0);
			let error = DispatchError::Module(ModuleError {
				index,
				error: TryInto::try_into(error).expect("should work"),
				message: None,
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 52, 18, 0, 0, 0]);
			assert_eq!(
				decoded,
				DispatchError::Module(ModuleError {
					index: 52,
					error: [18, 0, 0, 0],
					message: None
				})
			);

			// Example DispatchError::Token
			let error = DispatchError::Token(TokenError::UnknownAsset);
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![7, 4]);
			assert_eq!(decoded, error);

			// Example DispatchError::Arithmetic
			let error = DispatchError::Arithmetic(ArithmeticError::Overflow);
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![8, 1]);
			assert_eq!(decoded, error);
		});
	}
}
// use enumflags2::BitFlags;
// use pallet_nfts::{CollectionConfig, CollectionSetting, CollectionSettings, MintSettings};
// use parachains_common::CollectionId;
// {
//     // NFT helper functions
//     fn collection_config_from_disabled_settings(
//         settings: BitFlags<CollectionSetting>,
//     ) -> CollectionConfig<Balance, crate::BlockNumber, CollectionId> {
//         CollectionConfig {
//             settings: CollectionSettings::from_disabled(settings),
//             max_supply: None,
//             mint_settings: MintSettings::default(),
//         }
//     }
//
//     fn default_collection_config() -> CollectionConfig<Balance, crate::BlockNumber, CollectionId> {
//         collection_config_from_disabled_settings(CollectionSetting::DepositRequired.into())
//     }
//
//     #[test]
//     #[ignore]
//     fn dispatch_balance_transfer_from_contract_works() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) = load_wasm_module::<Runtime>(
//                 "../../pop-api/examples/balance-transfer/target/ink/balance_transfer.wasm",
//             )
//                 .unwrap();
//
//             let init_value = 100 * UNIT;
//
//             let result = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
//
//             let addr = result.account_id;
//
//             let function = function_selector("transfer_through_runtime");
//             let value_to_send: u128 = 10 * UNIT;
//             let params = [function, BOB.encode(), value_to_send.encode()].concat();
//
//             let bob_balance_before = Balances::free_balance(&BOB);
//             assert_eq!(bob_balance_before, INITIAL_AMOUNT);
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // check for revert
//             assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
//
//             let bob_balance_after = Balances::free_balance(&BOB);
//             assert_eq!(bob_balance_before + value_to_send, bob_balance_after);
//         });
//     }
//
//     // Create a test for tesing create_nft_collection
//     #[test]
//     #[ignore]
//     fn dispatch_nfts_create_nft_collection() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) = load_wasm_module::<Runtime>(
//                 "../../pop-api/examples/nfts/target/ink/pop_api_nft_example.wasm",
//             )
//                 .unwrap();
//
//             let init_value = 100 * UNIT;
//
//             let result = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
//
//             let addr = result.account_id;
//
//             let function = function_selector("create_nft_collection");
//
//             let params = [function].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // check that the nft collection was created
//             assert_eq!(Nfts::collection_owner(0), Some(addr.clone().into()));
//
//             // test reading the collection
//             let function = function_selector("read_collection");
//
//             let params = [function, 0.encode()].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // assert that the collection was read successfully
//             assert_eq!(result.result.clone().unwrap().data, vec![1, 1]);
//         });
//     }
//
//     #[test]
//     #[ignore]
//     fn dispatch_nfts_mint_from_contract_works() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) =
//                 load_wasm_module::<Runtime>("../../pop-api/examples/nfts/target/ink/nfts.wasm")
//                     .unwrap();
//
//             let init_value = 100;
//
//             let result = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
//
//             let addr = result.account_id;
//
//             let collection_id: u32 = 0;
//             let item_id: u32 = 1;
//
//             // create nft collection with contract as owner
//             assert_eq!(
//                 Nfts::force_create(
//                     RuntimeOrigin::root(),
//                     addr.clone().into(),
//                     default_collection_config()
//                 ),
//                 Ok(())
//             );
//
//             assert_eq!(Nfts::collection_owner(collection_id), Some(addr.clone().into()));
//             // assert that the item does not exist yet
//             assert_eq!(Nfts::owner(collection_id, item_id), None);
//
//             let function = function_selector("mint_through_runtime");
//
//             let params =
//                 [function, collection_id.encode(), item_id.encode(), BOB.encode()].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // check for revert
//             assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
//
//             assert_eq!(Nfts::owner(collection_id, item_id), Some(BOB.into()));
//         });
//     }
//
//     #[test]
//     #[ignore]
//     fn nfts_mint_surfaces_error() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) =
//                 load_wasm_module::<Runtime>("../../pop-api/examples/nfts/target/ink/nfts.wasm")
//                     .unwrap();
//
//             let init_value = 100;
//
//             let result = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
//
//             let addr = result.account_id;
//
//             let collection_id: u32 = 0;
//             let item_id: u32 = 1;
//
//             let function = function_selector("mint_through_runtime");
//
//             let params =
//                 [function, collection_id.encode(), item_id.encode(), BOB.encode()].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // check for revert with expected error
//             let result = result.result.unwrap();
//             assert!(result.did_revert());
//         });
//     }
//
//     #[test]
//     #[ignore]
//     fn reading_last_relay_chain_block_number_works() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) = load_wasm_module::<Runtime>(
//                 "../../pop-api/examples/read-runtime-state/target/ink/read_relay_blocknumber.wasm",
//             )
//                 .unwrap();
//
//             let init_value = 100;
//
//             let contract = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!contract.result.did_revert(), "deploying contract reverted {:?}", contract);
//
//             let addr = contract.account_id;
//
//             let function = function_selector("read_relay_block_number");
//             let params = [function].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::UnsafeCollect,
//                 pallet_contracts::Determinism::Relaxed,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // check for revert
//             assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
//         });
//     }
//
//     #[test]
//     #[ignore]
//     fn place_spot_order_from_contract_works() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) = load_wasm_module::<Runtime>(
//                 "../../pop-api/examples/place-spot-order/target/ink/spot_order.wasm",
//             )
//                 .unwrap();
//
//             let init_value = 100 * UNIT;
//
//             let result = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
//
//             let addr = result.account_id;
//
//             let function = function_selector("place_spot_order");
//
//             let max_amount = 1 * UNIT;
//             let para_id = 2000;
//
//             let params = [function, max_amount.encode(), para_id.encode()].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("result: {:?}", result);
//             }
//
//             // check for revert
//             assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
//         });
//     }
//
//     #[test]
//     #[ignore]
//     fn allow_call_filter_blocks_call() {
//         new_test_ext().execute_with(|| {
//             let _ = env_logger::try_init();
//
//             let (wasm_binary, _) = load_wasm_module::<Runtime>(
//                 "../../tests/contracts/filtered-call/target/ink/pop_api_filtered_call.wasm",
//             )
//                 .unwrap();
//
//             let init_value = 100 * UNIT;
//
//             let result = Contracts::bare_instantiate(
//                 ALICE,
//                 init_value,
//                 GAS_LIMIT,
//                 None,
//                 Code::Upload(wasm_binary),
//                 function_selector("new"),
//                 vec![],
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//             )
//                 .result
//                 .unwrap();
//
//             assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
//
//             let addr = result.account_id;
//
//             let function = function_selector("get_filtered");
//             let params = [function].concat();
//
//             let result = Contracts::bare_call(
//                 ALICE,
//                 addr.clone(),
//                 0,
//                 Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
//                 None,
//                 params,
//                 DEBUG_OUTPUT,
//                 pallet_contracts::CollectEvents::Skip,
//                 pallet_contracts::Determinism::Enforced,
//             );
//
//             if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
//                 log::debug!(
// 					"Contract debug buffer - {:?}",
// 					String::from_utf8(result.debug_message.clone())
// 				);
//                 log::debug!("filtered result: {:?}", result);
//             }
//
//             // check for revert
//             assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
//         });
//     }
// }
