use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, ConstU64, Nothing},
};
use frame_system::EnsureSigned;

use super::api::{self, Config};
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

impl pallet_revive::Config for Runtime {
	type AddressMapper = pallet_revive::AccountId32Mapper<Runtime>;
	type CallFilter = Nothing;
	type ChainExtension = ();
	// todo!("Call with peter, currently we are only implementing the extension for
 // pallet-contracts");
	type ChainId = ConstU64<4001>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type NativeToEthRatio = ConstU32<1_000_000>;
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
	type Xcm = pallet_xcm::Pallet<Self>;
}

// Mock implementation running for messaging.
// remove extrinsic tests / benchmarking.
// xcm spike
// ismp spike
