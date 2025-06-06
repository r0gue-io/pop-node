use core::marker::PhantomData;

use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, Nothing, Randomness},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureSigned};
use pop_runtime_common::{DepositPerByte, DepositPerItem, UNIT};

use super::api::{self, Config};
use crate::{
    config::assets::TrustBackedAssetsInstance, deposit, Balance, Balances, Perbill, Runtime,
    RuntimeCall, RuntimeEvent, RuntimeHoldReason, Timestamp, TransactionPayment,
};

type Erc20<const PREFIX: u16, I> = pallet_api_vnext::Erc20<PREFIX, Runtime, I>;
type Fungibles<const FIXED: u16, I> = pallet_api_vnext::Fungibles<FIXED, Runtime, I>;

fn schedule<T: pallet_contracts::Config>() -> pallet_contracts::Schedule<T> {
	pallet_contracts::Schedule {
		limits: pallet_contracts::Limits {
			runtime_memory: 1024 * 1024 * 1024,
			validator_runtime_memory: 2 * 1024 * 1024 * 1024,
			..Default::default()
		},
		..Default::default()
	}
}

// randomness-collective-flip is insecure. Provide dummy randomness as placeholder for the
// deprecated trait. https://github.com/paritytech/polkadot-sdk/blob/9bf1a5e23884921498b381728bfddaae93f83744/substrate/frame/contracts/mock-network/src/parachain/contracts_config.rs#L45
pub struct DummyRandomness<T: pallet_contracts::Config>(PhantomData<T>);

impl<T: pallet_contracts::Config> Randomness<T::Hash, BlockNumberFor<T>> for DummyRandomness<T> {
	fn random(_subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
		(Default::default(), Default::default())
	}
}

// 18 decimals
const ETH: u128 = 1_000_000_000_000_000_000;

parameter_types! {
	pub ChainId: u64 = u32::from(crate::genesis::PARA_ID) as u64;
	pub Schedule: pallet_contracts::Schedule<Runtime> = schedule::<Runtime>();
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	// 30 percent of storage deposit held for using a code hash.
	pub const CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
	pub const NativeToEthRatio: u32 = (ETH/UNIT) as u32;
}

impl pallet_contracts::Config for Runtime {
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type ApiVersion = ();
	// IMPORTANT: only runtime calls through the api are allowed.
	type CallFilter = Nothing;
	type CallStack = [pallet_contracts::Frame<Self>; 23];
	type ChainExtension = api::Extension<Config>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	type DefaultDepositLimit = DefaultDepositLimit;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type Environment = ();
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	// This node is geared towards development and testing of contracts.
	// We decided to increase the default allowed contract size for this
	// reason (the default is `128 * 1024`).
	//
	// Our reasoning is that the error code `CodeTooLarge` is thrown
	// if a too-large contract is uploaded. We noticed that it poses
	// less friction during development when the requirement here is
	// just more lax.
	type MaxCodeLen = ConstU32<{ 256 * 1024 }>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxDelegateDependencies = ConstU32<32>;
	type MaxStorageKeyLen = ConstU32<128>;
	type MaxTransientStorageSize = ConstU32<{ 1024 * 1024 }>;
	type Migrations = ();
	type Randomness = DummyRandomness<Self>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Schedule = Schedule;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ConstBool<true>;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type Xcm = pallet_xcm::Pallet<Self>;
}

impl pallet_revive::Config for Runtime {
	type AddressMapper = pallet_revive::AccountId32Mapper<Self>;
	type ChainId = ChainId;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type EthGasEncoder = ();
	type FindAuthor = <Runtime as pallet_authorship::Config>::FindAuthor;
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	// 1 ETH : 1_000_000 UNIT
	type NativeToEthRatio = NativeToEthRatio;
	// 512 MB. Used in an integrity test that verifies the runtime has enough memory.
	type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
	type Precompiles =
		(Fungibles<100, TrustBackedAssetsInstance>, Erc20<101, TrustBackedAssetsInstance>);
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	// 128 MB. Used in an integrity that verifies the runtime has enough memory.
	type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
	type Time = Timestamp;
	// Enables access to unsafe host fns such as xcm_send.
	type UnsafeUnstableInterface = ConstBool<true>;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = pallet_revive::weights::SubstrateWeight<Self>;
	type WeightPrice = TransactionPayment;
}

impl TryFrom<RuntimeCall> for pallet_revive::Call<Runtime> {
	type Error = ();

	fn try_from(value: RuntimeCall) -> Result<Self, Self::Error> {
		match value {
			RuntimeCall::Revive(call) => Ok(call),
			_ => Err(()),
		}
	}
}

// IMPORTANT: only runtime calls through the api are allowed.
#[test]
fn contracts_prevents_runtime_calls() {
	use std::any::TypeId;
	assert_eq!(
		TypeId::of::<<Runtime as pallet_contracts::Config>::CallFilter>(),
		TypeId::of::<Nothing>()
	);
}

#[cfg(test)]
mod tests {
	use codec::Encode;
	use frame_support::{assert_ok, traits::fungible::Mutate};
	use frame_system::pallet_prelude::OriginFor;
	use pallet_api_vnext::fungibles::precompiles::{IFungibles::*, IERC20};
	use pallet_revive::{
		precompiles::alloy::{
			primitives,
			sol_types::{SolCall, SolType, SolValue},
		},
		AddressMapper, Code, DepositLimit,
	};
	use sp_core::{bytes::to_hex, H160, U256};
	use sp_keyring::Sr25519Keyring::{Alice, Bob};
	use sp_runtime::Weight;

	use super::*;
	use crate::{Assets, Revive, RuntimeOrigin, System};

	type AccountId32Mapper = pallet_revive::AccountId32Mapper<Runtime>;
	type Asset = pallet_assets::Asset<Runtime, TrustBackedAssetsInstance>;
	type NextAssetId = pallet_assets::NextAssetId<Runtime, TrustBackedAssetsInstance>;

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::new_empty();
		ext.execute_with(|| {
			System::set_block_number(1);
			Balances::set_balance(&Alice.to_account_id(), 1_000 * UNIT);
			Balances::set_balance(&Bob.to_account_id(), 1 * UNIT);
			NextAssetId::put(1);
		});
		ext
	}

	#[test]
	fn fungibles_precompiles_work() {
		let caller = Alice.to_account_id();
		let origin = RuntimeOrigin::signed(caller.clone());
		let origin_addr = AccountId32Mapper::to_address(&caller);
		let id = 1;
		let fungibles_addr: H160 = Fungibles::<100, TrustBackedAssetsInstance>::address().into();
		let erc20_addr: H160 = Erc20::<101, TrustBackedAssetsInstance>::address(id).into();
		let total_supply: Balance = 10_000;
		new_test_ext().execute_with(|| {
			assert_ok!(Revive::map_account(origin.clone()));
			assert_ok!(Revive::map_account(RuntimeOrigin::signed(Bob.to_account_id())));

			// Create a token via fungibles precompile
			println!("IFungibles precompile: {}", to_hex(&fungibles_addr.0, false));
			let call =
				createCall { admin: origin_addr.0.into(), minBalance: primitives::U256::from(1) }
					.abi_encode();
			println!("IFungibles.create: {}", to_hex(&call, false));
			assert_ok!(Revive::call(origin.clone(), fungibles_addr, 0, Weight::zero(), 0, call));
			let asset_details = Asset::get(id).unwrap();
			assert_eq!(asset_details.owner, caller);
			assert_eq!(asset_details.admin, caller);

			// Mint via fungibles precompile
			let call = mintCall {
				id,
				account: origin_addr.0.into(),
				value: primitives::U256::from(total_supply),
			}
			.abi_encode();
			println!("IFungibles.mint: {}", to_hex(&call, false));
			assert_ok!(Revive::call(origin.clone(), fungibles_addr, 0, Weight::zero(), 0, call));

			// Transfer via erc20 precompile
			println!("IERC20 precompile: {}", to_hex(&erc20_addr.0, false));
			let call = IERC20::transferCall {
				to: AccountId32Mapper::to_address(&Bob.to_account_id()).0.into(),
				value: primitives::U256::from(total_supply / 2),
			}
			.abi_encode();
			println!("IERC20.transfer: {}", to_hex(&call, false));
			assert_ok!(Revive::call(origin.clone(), erc20_addr, 0, Weight::zero(), 0, call));
			assert_eq!(Assets::balance(id, &Bob.to_account_id()), total_supply / 2);
		})
	}

	// Currently ignored due to explicit dependency on built contract
	#[ignore]
	#[test]
	fn fungibles_precompiles_via_contract_works() {
		let contract = include_bytes!(
			"../../../../pop-api-vnext/examples/fungibles-vnext/target/ink/fungibles.polkavm"
		);
		let caller = Alice.to_account_id();
		let origin = RuntimeOrigin::signed(caller.clone());
		let origin_addr = primitives::Address::new(AccountId32Mapper::to_address(&caller).0);
		let minimum_value = U256::from(1);
		let endowment = primitives::U256::from(10_000);
		new_test_ext().execute_with(|| {
			assert_ok!(Revive::map_account(origin.clone()));

			// Instantiate contract with some value, required to create underlying asset
			let result = Revive::bare_instantiate(
				origin.clone(),
				10 * UNIT,
				Weight::MAX,
				DepositLimit::Unchecked,
				Code::Upload(contract.to_vec()),
				// Constructors are not yet using Solidity encoding
				[blake_selector("new"), minimum_value.encode()].concat(),
				None,
			)
			.result
			.unwrap();
			assert!(!result.result.did_revert());

			// Interact with contract as Erc20
			call::<()>(
				origin.clone(),
				result.addr,
				[keccak_selector("mint(address,uint256)"), (origin_addr, endowment).abi_encode()]
					.concat(),
			);

			let total_supply = call::<primitives::U256>(
				origin.clone(),
				result.addr,
				keccak_selector("totalSupply()"),
			);
			assert_eq!(total_supply, endowment);

			let balance_of = call::<primitives::U256>(
				origin.clone(),
				result.addr,
				[keccak_selector("balanceOf(address)"), (origin_addr,).abi_encode()].concat(),
			);
			assert_eq!(balance_of, endowment);
		});

		fn call<T: SolValue + From<<T::SolType as SolType>::RustType>>(
			origin: OriginFor<Runtime>,
			contract: H160,
			data: Vec<u8>,
		) -> T {
			let result =
				Revive::bare_call(origin, contract, 0, Weight::MAX, DepositLimit::Unchecked, data)
					.result
					.unwrap();
			assert!(!result.did_revert());
			T::abi_decode(&result.data, true).unwrap()
		}
	}

	fn blake_selector(name: &str) -> Vec<u8> {
		let hash = sp_io::hashing::blake2_256(name.as_bytes());
		[hash[0..4].to_vec()].concat()
	}

	fn keccak_selector(name: &str) -> Vec<u8> {
		let hash = sp_io::hashing::keccak_256(name.as_bytes());
		[hash[0..4].to_vec()].concat()
	}

	#[test]
	fn selectors_work() {
		// Constructors currently still use blake encoding
		assert_eq!(hex::encode(blake_selector("new")), "9bae9d5e");

		// Erc20 selectors
		assert_eq!(hex::encode(keccak_selector("exists()")), "267c4ae4");
		assert_eq!(hex::encode(keccak_selector("totalSupply()")), "18160ddd");
		assert_eq!(hex::encode(keccak_selector("balanceOf(address)")), "70a08231");
	}
}
