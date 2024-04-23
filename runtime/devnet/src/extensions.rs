use cumulus_pallet_parachain_system::RelaychainDataProvider;
use frame_support::traits::{Contains, OriginTrait};
use frame_support::{
	dispatch::{GetDispatchInfo, RawOrigin},
	pallet_prelude::*,
	traits::nonfungibles_v2::Inspect,
};
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, ChargedAmount, Environment, Ext, InitState, RetVal,
};
use pop_primitives::{
	cross_chain::CrossChainMessage,
	storage_keys::{NftsKeys, ParachainSystemKeys, RuntimeStateKeys},
	CollectionId, ItemId,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
	traits::{BlockNumberProvider, Dispatchable},
	DispatchError,
};
use sp_std::{boxed::Box, vec::Vec};
use xcm::{
	latest::{prelude::*, OriginKind::SovereignAccount},
	VersionedXcm,
};

use crate::{AccountId, AllowedApiCalls, RuntimeCall, RuntimeOrigin, UNIT};

const LOG_TARGET: &str = "pop-api::extension";

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

#[derive(Default)]
pub struct PopApiExtension;

impl<T> ChainExtension<T> for PopApiExtension
where
	T: pallet_contracts::Config
		+ pallet_xcm::Config
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
		T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	{
		log::debug!(target:LOG_TARGET, " extension called ");
		match v0::FuncId::try_from(env.func_id())? {
			v0::FuncId::Dispatch => {
				match dispatch::<T, E>(env) {
					Ok(()) => Ok(RetVal::Converging(0)),
					Err(DispatchError::Module(error)) => {
						// encode status code = pallet index in runtime + error index, allowing for
						// 999 errors
						Ok(RetVal::Converging(
							(error.index as u32 * 1_000) + u32::from_le_bytes(error.error),
						))
					},
					Err(e) => Err(e),
				}
			},
			v0::FuncId::ReadState => {
				read_state::<T, E>(env)?;
				Ok(RetVal::Converging(0))
			},
			v0::FuncId::SendXcm => {
				send_xcm::<T, E>(env)?;
				Ok(RetVal::Converging(0))
			},
		}
	}
}

pub mod v0 {
	#[derive(Debug)]
	pub enum FuncId {
		Dispatch,
		ReadState,
		SendXcm,
	}
}

impl TryFrom<u16> for v0::FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			0x0 => Self::Dispatch,
			0x1 => Self::ReadState,
			0x2 => Self::SendXcm,
			_ => {
				log::error!("called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("unimplemented func_id"));
			},
		};

		Ok(id)
	}
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

	log::debug!(target:LOG_TARGET, "{} inputted RuntimeCall: {:?}", log_prefix, call);

	origin.add_filter(AllowedApiCalls::contains);

	match call.dispatch(origin) {
		Ok(info) => {
			log::debug!(target:LOG_TARGET, "{} success, actual weight: {:?}", log_prefix, info.actual_weight);

			// refund weight if the actual weight is less than the charged weight
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

fn charge_overhead_weight<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	len: u32,
	log_prefix: &str,
) -> Result<ChargedAmount, DispatchError>
where
	T: pallet_contracts::Config,
	E: Ext<T = T>,
{
	let contract_host_weight = ContractSchedule::<T>::get().host_fn_weights;

	// calculate weight for reading bytes of `len`
	// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
	let base_weight: Weight = contract_host_weight.return_per_byte.saturating_mul(len.into());

	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
	let overhead = contract_host_weight.debug_message;

	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
	log::debug!(target: LOG_TARGET, "{} charged weight: {:?}", log_prefix, charged_weight);

	Ok(charged_weight)
}

fn dispatch<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config
		+ frame_system::Config<RuntimeCall = RuntimeCall, RuntimeOrigin = RuntimeOrigin>,
	RuntimeOrigin: From<RawOrigin<T::AccountId>>,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " dispatch |";

	let mut env = env.buf_in_buf_out();
	let len = env.in_len();

	charge_overhead_weight::<T, E>(&mut env, len, LOG_PREFIX)?;

	// read the input as RuntimeCall
	let call: RuntimeCall = env.read_as_unbounded(len)?;

	// contract is the origin by default
	let origin: RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();

	dispatch_call::<T, E>(&mut env, call, origin, LOG_PREFIX)
}

fn read_state<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config
		+ pallet_nfts::Config<CollectionId = CollectionId, ItemId = ItemId>
		+ cumulus_pallet_parachain_system::Config
		+ frame_system::Config,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " read_state |";

	let mut env = env.buf_in_buf_out();

	// To be conservative, we charge the weight for reading the input bytes of a fixed-size type.
	let base_weight: Weight = ContractSchedule::<T>::get()
		.host_fn_weights
		.return_per_byte
		.saturating_mul(env.in_len().into());
	let charged_weight = env.charge_weight(base_weight)?;

	log::debug!(target:LOG_TARGET, "{} charged weight: {:?}", LOG_PREFIX, charged_weight);

	let key: RuntimeStateKeys = env.read_as()?;

	let result = match key {
		RuntimeStateKeys::Nfts(key) => read_nfts_state::<T, E>(key, &mut env),
		RuntimeStateKeys::ParachainSystem(key) => {
			read_parachain_system_state::<T, E>(key, &mut env)
		},
	}?
	.encode();

	log::trace!(
		target:LOG_TARGET,
		"{} result: {:?}.", LOG_PREFIX, result
	);
	env.write(&result, false, None).map_err(|e| {
		log::trace!(target: LOG_TARGET, "{:?}", e);
		DispatchError::Other("unable to write results to contract memory")
	})
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

fn send_xcm<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
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

	let mut env = env.buf_in_buf_out();
	let len = env.in_len();

	let _ = charge_overhead_weight::<T, E>(&mut env, len, LOG_PREFIX)?;

	// read the input as CrossChainMessage
	let xc_call: CrossChainMessage = env.read_as::<CrossChainMessage>()?;

	// Determine the call to dispatch
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

	// Generate runtime call to dispatch
	let call = RuntimeCall::PolkadotXcm(pallet_xcm::Call::send {
		dest: Box::new(dest),
		message: Box::new(VersionedXcm::V4(message)),
	});

	dispatch_call::<T, E>(&mut env, call, origin, LOG_PREFIX)
}

#[cfg(test)]
mod tests {
	pub use super::*;
	pub use crate::*;
	use enumflags2::BitFlags;
	pub use pallet_contracts::Code;
	use pallet_nfts::{CollectionConfig, CollectionSetting, CollectionSettings, MintSettings};
	use parachains_common::CollectionId;
	pub use sp_runtime::{traits::Hash, AccountId32};

	const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;

	const ALICE: AccountId32 = AccountId32::new([1_u8; 32]);
	const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
	const INITIAL_AMOUNT: u128 = 100_000 * UNIT;
	const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(ALICE, INITIAL_AMOUNT), (BOB, INITIAL_AMOUNT)],
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	fn load_wasm_module<T>(path: &str) -> std::io::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
	where
		T: frame_system::Config,
	{
		let wasm_binary = std::fs::read(path)?;
		let code_hash = T::Hashing::hash(&wasm_binary);
		Ok((wasm_binary, code_hash))
	}

	fn function_selector(name: &str) -> Vec<u8> {
		let hash = sp_io::hashing::blake2_256(name.as_bytes());
		[hash[0..4].to_vec()].concat()
	}

	// NFT helper functions
	fn collection_config_from_disabled_settings(
		settings: BitFlags<CollectionSetting>,
	) -> CollectionConfig<Balance, crate::BlockNumber, CollectionId> {
		CollectionConfig {
			settings: CollectionSettings::from_disabled(settings),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}

	fn default_collection_config() -> CollectionConfig<Balance, crate::BlockNumber, CollectionId> {
		collection_config_from_disabled_settings(CollectionSetting::DepositRequired.into())
	}

	#[test]
	#[ignore]
	fn dispatch_balance_transfer_from_contract_works() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../pop-api/examples/balance-transfer/target/ink/pop_api_extension_demo.wasm",
			)
			.unwrap();

			let init_value = 100 * UNIT;

			let result = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);

			let addr = result.account_id;

			let function = function_selector("transfer_through_runtime");
			let value_to_send: u128 = 10 * UNIT;
			let params = [function, BOB.encode(), value_to_send.encode()].concat();

			let bob_balance_before = Balances::free_balance(&BOB);
			assert_eq!(bob_balance_before, INITIAL_AMOUNT);

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
				pallet_contracts::Determinism::Enforced,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("result: {:?}", result);
			}

			// check for revert
			assert!(!result.result.unwrap().did_revert(), "Contract reverted!");

			let bob_balance_after = Balances::free_balance(&BOB);
			assert_eq!(bob_balance_before + value_to_send, bob_balance_after);
		});
	}

	// Create a test for tesing create_nft_collection
	#[test]
	#[ignore]
	fn dispatch_nfts_create_nft_collection() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../pop-api/examples/nfts/target/ink/pop_api_nft_example.wasm",
			)
			.unwrap();

			let init_value = 100 * UNIT;

			let result = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);

			let addr = result.account_id;

			let function = function_selector("create_nft_collection");

			let params = [function].concat();

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
				pallet_contracts::Determinism::Enforced,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("result: {:?}", result);
			}

			// // check for revert
			assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
		});
	}

	#[test]
	#[ignore]
	fn dispatch_nfts_mint_from_contract_works() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../pop-api/examples/nfts/target/ink/pop_api_nft_example.wasm",
			)
			.unwrap();

			let init_value = 100;

			let result = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);

			let addr = result.account_id;

			let collection_id: u32 = 0;
			let item_id: u32 = 1;

			// create nft collection with contract as owner
			assert_eq!(
				Nfts::force_create(
					RuntimeOrigin::root(),
					addr.clone().into(),
					default_collection_config()
				),
				Ok(())
			);

			assert_eq!(Nfts::collection_owner(collection_id), Some(addr.clone().into()));
			// assert that the item does not exist yet
			assert_eq!(Nfts::owner(collection_id, item_id), None);

			let function = function_selector("mint_through_runtime");

			let params =
				[function, collection_id.encode(), item_id.encode(), BOB.encode()].concat();

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
				pallet_contracts::Determinism::Enforced,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("result: {:?}", result);
			}

			// check for revert
			assert!(!result.result.unwrap().did_revert(), "Contract reverted!");

			assert_eq!(Nfts::owner(collection_id, item_id), Some(BOB.into()));
		});
	}

	#[test]
	#[ignore]
	fn nfts_mint_surfaces_error() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../pop-api/examples/nfts/target/ink/pop_api_nft_example.wasm",
			)
			.unwrap();

			let init_value = 100;

			let result = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);

			let addr = result.account_id;

			let collection_id: u32 = 0;
			let item_id: u32 = 1;

			let function = function_selector("mint_through_runtime");

			let params =
				[function, collection_id.encode(), item_id.encode(), BOB.encode()].concat();

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
				pallet_contracts::Determinism::Enforced,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("result: {:?}", result);
			}

			// check for revert with expected error
			let result = result.result.unwrap();
			assert!(result.did_revert());
		});
	}

	#[test]
	#[ignore]
	fn reading_last_relay_chain_block_number_works() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../pop-api/examples/read-runtime-state/target/ink/pop_api_extension_demo.wasm",
			)
			.unwrap();

			let init_value = 100;

			let contract = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!contract.result.did_revert(), "deploying contract reverted {:?}", contract);

			let addr = contract.account_id;

			let function = function_selector("read_relay_block_number");
			let params = [function].concat();

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::UnsafeCollect,
				pallet_contracts::Determinism::Relaxed,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("result: {:?}", result);
			}

			// check for revert
			assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
		});
	}

	#[test]
	#[ignore]
	fn place_spot_order_from_contract_works() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../pop-api/examples/place-spot-order/target/ink/pop_api_spot_order_example.wasm",
			)
			.unwrap();

			let init_value = 100 * UNIT;

			let result = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);

			let addr = result.account_id;

			let function = function_selector("place_spot_order");

			let max_amount = 1 * UNIT;
			let para_id = 2000;

			let params = [function, max_amount.encode(), para_id.encode()].concat();

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
				pallet_contracts::Determinism::Enforced,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("result: {:?}", result);
			}

			// check for revert
			assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
		});
	}

	#[test]
	#[ignore]
	fn allow_call_filter_blocks_call() {
		new_test_ext().execute_with(|| {
			let _ = env_logger::try_init();

			let (wasm_binary, _) = load_wasm_module::<Runtime>(
				"../../tests/contracts/filtered-call/target/ink/pop_api_filtered_call.wasm",
			)
			.unwrap();

			let init_value = 100 * UNIT;

			let result = Contracts::bare_instantiate(
				ALICE,
				init_value,
				GAS_LIMIT,
				None,
				Code::Upload(wasm_binary),
				function_selector("new"),
				vec![],
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
			)
			.result
			.unwrap();

			assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);

			let addr = result.account_id;

			let function = function_selector("get_filtered");
			let params = [function].concat();

			let result = Contracts::bare_call(
				ALICE,
				addr.clone(),
				0,
				Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
				None,
				params,
				DEBUG_OUTPUT,
				pallet_contracts::CollectEvents::Skip,
				pallet_contracts::Determinism::Enforced,
			);

			if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
				log::debug!(
					"Contract debug buffer - {:?}",
					String::from_utf8(result.debug_message.clone())
				);
				log::debug!("filtered result: {:?}", result);
			}

			// check for revert
			assert!(!result.result.unwrap().did_revert(), "Contract reverted!");
		});
	}
}
