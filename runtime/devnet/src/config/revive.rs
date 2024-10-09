use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, ConstU64, Nothing},
};
use frame_system::EnsureSigned;
use pallet_api_revive::Extension;
use pallet_revive::{
	chain_extension::{
		ChainExtension, Environment, Ext, RegisteredChainExtension, Result as ExtensionResult,
		RetVal, ReturnFlags,
	},
	wasm::Memory,
};
use sp_std::vec;

use super::api_revive::{self, Config};
use crate::{
	deposit, Balance, Balances, Perbill, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
	Timestamp,
};

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	pub const CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
}

// impl pallet_revive::chain_extension::RegisteredChainExtension<Runtime> for Extension<Config> {
// 	const ID: u16 = 0;
// }

// #[derive(Default)]
// pub struct RevertingExtension;
// impl ChainExtension<Runtime> for RevertingExtension {
// 	fn call<E, M>(&mut self, _env: Environment<E, M>) -> ExtensionResult<RetVal>
// 	where
// 		E: Ext<T = Runtime>,
// 		M: ?Sized + Memory<E::T>,
// 	{
// 		log::info!("CALLED REVERTING EXTENSION");
// 		Ok(RetVal::Diverging { flags: ReturnFlags::REVERT, data: vec![0x4B, 0x1D] })
// 	}
// }
// impl RegisteredChainExtension<Runtime> for RevertingExtension {
// 	const ID: u16 = 0;
// }

impl pallet_revive::Config for Runtime {
	type AddressMapper = pallet_revive::DefaultAddressMapper;
	type CallFilter = Nothing;
	type ChainExtension = api_revive::Extension<Config>;
	type ChainId = ConstU64<4001>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ConstBool<true>;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = pallet_revive::weights::SubstrateWeight<Self>;
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type Xcm = ();
}
