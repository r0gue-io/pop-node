use frame_support::{
	parameter_types,
	traits::{ConstBool, ConstU32, ConstU64, Nothing},
};
use frame_system::EnsureSigned;

use crate::{
	deposit, Balance, Balances, Perbill, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeHoldReason, Timestamp, TransactionPayment, UNIT,
};

const ETH: u128 = 1_000_000_000_000_000_000;

parameter_types! {
	// TODO: review implications of this
	pub const DepositPerItem: Balance = deposit(1, 0);
	// TODO: review implications of this
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(30);
	pub const NativeToEthRatio: u32 = (ETH/UNIT) as u32;
}

impl pallet_revive::Config for Runtime {
	type AddressMapper = pallet_revive::AccountId32Mapper<Self>;
	type CallFilter = Nothing;
	type ChainExtension = ();
	type ChainId = ConstU64<3395>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type NativeToEthRatio = NativeToEthRatio;
	// TODO: review implications of this
	type PVFMemory = ConstU32<{ 512 * 1024 * 1024 }>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	// TODO: review implications of this
	type RuntimeMemory = ConstU32<{ 128 * 1024 * 1024 }>;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ConstBool<false>;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = pallet_revive::weights::SubstrateWeight<Self>;
	type WeightPrice = TransactionPayment;
	type Xcm = PolkadotXcm;
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

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use frame_support::pallet_prelude::Get;
	use pallet_revive::Config;

	use super::*;

	// 18 decimals
	const ONE_ETH: u128 = 1_000_000_000_000_000_000;

	#[test]
	fn address_mapper_is_account_id32_mapper() {
		assert_eq!(
			TypeId::of::<<Runtime as Config>::AddressMapper>(),
			TypeId::of::<pallet_revive::AccountId32Mapper<Runtime>>(),
		);
	}

	#[test]
	fn call_filter_is_nothing() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_revive::Config>::CallFilter>(),
			TypeId::of::<Nothing>(),
		);
	}

	#[test]
	fn chain_extension_is_unset() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_revive::Config>::ChainExtension>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn chain_id_is_correct() {
		assert_eq!(<<Runtime as Config>::ChainId as Get<u64>>::get(), 3395);
	}

	#[test]
	fn code_hash_lockup_deposit_percent_is_correct() {
		// 30 percent
		assert_eq!(
			TypeId::of::<<Runtime as Config>::CodeHashLockupDepositPercent>(),
			TypeId::of::<CodeHashLockupDepositPercent>(),
		);
	}

	#[test]
	fn currency_is_balances() {
		assert_eq!(TypeId::of::<<Runtime as Config>::Currency>(), TypeId::of::<Balances>(),);
	}

	#[test]
	fn debug_is_unset() {
		assert_eq!(TypeId::of::<<Runtime as Config>::Debug>(), TypeId::of::<()>(),);
	}

	#[test]
	fn deposit_per_byte_is_correct() {
		assert_eq!(<<Runtime as Config>::DepositPerByte as Get<Balance>>::get(), deposit(0, 1),);
	}

	#[test]
	fn deposit_per_item_is_correct() {
		assert_eq!(<<Runtime as Config>::DepositPerItem as Get<Balance>>::get(), deposit(1, 0),);
	}

	#[test]
	fn instantiate_origin_is_ensure_signed() {
		assert_eq!(
			TypeId::of::<<Runtime as Config>::InstantiateOrigin>(),
			TypeId::of::<EnsureSigned<<Runtime as frame_system::Config>::AccountId>>(),
		);
	}

	#[test]
	fn eth_is_18_decimals() {
		assert_eq!(ETH, ONE_ETH);
	}

	#[test]
	fn native_to_eth_ratio_is_correct() {
		// 18 decimals
		let expected_ratio = (ONE_ETH / UNIT) as u32;
		assert_eq!(<<Runtime as Config>::NativeToEthRatio as Get<u32>>::get(), expected_ratio);
	}

	#[test]
	fn pvf_memory_is_correct() {
		// 512 MB
		assert_eq!(<<Runtime as Config>::PVFMemory as Get<u32>>::get(), 512 * 1024 * 1024);
	}

	#[test]
	fn runtime_memory_is_correct() {
		// 128 MB
		assert_eq!(<<Runtime as Config>::RuntimeMemory as Get<u32>>::get(), 128 * 1024 * 1024);
	}

	#[test]
	fn time_is_timestamp() {
		assert_eq!(TypeId::of::<<Runtime as Config>::Time>(), TypeId::of::<Timestamp>(),);
	}

	#[test]
	fn unsafe_unstable_interface_is_disabled() {
		assert_eq!(<<Runtime as Config>::UnsafeUnstableInterface as Get<bool>>::get(), false,);
	}

	#[test]
	fn upload_origin_is_ensure_signed() {
		assert_eq!(
			TypeId::of::<<Runtime as Config>::UploadOrigin>(),
			TypeId::of::<EnsureSigned<<Runtime as frame_system::Config>::AccountId>>(),
		);
	}

	#[test]
	fn weight_is_not_default() {
		assert_ne!(TypeId::of::<<Runtime as Config>::WeightInfo>(), TypeId::of::<()>(),);
	}

	#[test]
	fn weight_price_uses_transaction_payment() {
		assert_eq!(
			TypeId::of::<<Runtime as Config>::WeightPrice>(),
			TypeId::of::<pallet_transaction_payment::Pallet<Runtime>>(),
		);
	}

	#[test]
	fn xcm_is_pallet_xcm() {
		assert_eq!(
			TypeId::of::<<Runtime as Config>::Xcm>(),
			TypeId::of::<pallet_xcm::Pallet<Runtime>>(),
		);
	}
}
