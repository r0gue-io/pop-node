use crate::{
	config::system::RuntimeBlockWeights, deposit, parameter_types, weights, AccountId, Balance,
	Balances, ConstU32, EnsureRoot, EqualPrivilegeOnly, HoldConsideration, LinearStoragePrice,
	OriginCaller, Perbill, Preimage, Runtime, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
	RuntimeOrigin, Weight,
};

parameter_types! {
	// Accounts for the base cost of creating a multisig execution.
	// For details, refer to `call_deposit_has_base_amount`.
	pub const DepositBase: Balance = deposit(1, 120);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

parameter_types! {
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
	// Accounts for the base cost of noting a preimage.
	// For details, refer to `base_deposit_matches_configuration`.
	pub const PreimageBaseDeposit: Balance = deposit(2, 68);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(60) *
		RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	#[cfg(feature = "runtime-benchmarks")]
	type MaxScheduledPerBlock = ConstU32<512>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxScheduledPerBlock = ConstU32<50>;
	type MaximumWeight = MaximumSchedulerWeight;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PalletsOrigin = OriginCaller;
	type Preimages = Preimage;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use frame_support::traits::Get;

	use super::*;

	mod multisig {
		use codec::MaxEncodedLen;
		use parachains_common::BlockNumber;

		use super::*;

		#[test]
		fn balances_is_used_for_deposits() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_multisig::Config>::Currency>(),
				TypeId::of::<Balances>(),
			);
		}

		#[test]
		fn call_deposit_has_base_amount() {
			// From pallet_multisig:
			// This is held for an additional storage item whose value size is 4 +
			// sizeof((BlockNumber, Balance, AccountId)) bytes, and whose key size is 32 +
			// sizeof(AccountId) bytes.
			let key_size = 32 + AccountId::max_encoded_len();
			let value_size = 4 +
				BlockNumber::max_encoded_len() +
				Balance::max_encoded_len() +
				AccountId::max_encoded_len();
			let max_size = key_size + value_size;
			assert_eq!(max_size, 120);
			assert_eq!(
				<<Runtime as pallet_multisig::Config>::DepositBase as Get<Balance>>::get(),
				deposit(1, 120)
			);
		}

		#[test]
		fn call_deposit_has_additional_factor() {
			assert_eq!(
				<<Runtime as pallet_multisig::Config>::DepositFactor as Get<Balance>>::get(),
				deposit(0, 32)
			);
		}

		#[test]
		fn number_of_signatories_is_limited() {
			assert_eq!(
				<<Runtime as pallet_multisig::Config>::MaxSignatories as Get<u32>>::get(),
				100
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_multisig::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}

	mod preimage {
		use frame_support::{Identity, StorageHasher};

		use super::*;

		#[test]
		fn base_deposit_matches_configuration() {
			// Accounts for the key size of:
			// `RequestStatusFor` +
			// `PreimageFor`
			let preimage_for_key_size =
				Identity::max_len::<(<Runtime as frame_system::Config>::Hash, u32)>();
			let request_status_for_key_size =
				Identity::max_len::<<Runtime as frame_system::Config>::Hash>();
			let base_key_size = preimage_for_key_size + request_status_for_key_size;
			assert_eq!(base_key_size, 68);
			assert_eq!(PreimageBaseDeposit::get(), deposit(2, base_key_size as u32));
		}

		#[test]
		fn byte_deposit_matches_configuration() {
			assert_eq!(PreimageByteDeposit::get(), deposit(0, 1));
		}

		#[test]
		fn hold_reason_uses_linear_price() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_preimage::Config>::Consideration>(),
				TypeId::of::<
					HoldConsideration<
						AccountId,
						Balances,
						PreimageHoldReason,
						LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
					>,
				>()
			);
		}

		#[test]
		fn balances_provides_currency() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_preimage::Config>::Currency>(),
				TypeId::of::<Balances>(),
			);
		}

		#[test]
		fn manage_origin_is_root() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_preimage::Config>::ManagerOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_preimage::Config>::WeightInfo>(),
				TypeId::of::<()>()
			);
		}
	}

	mod scheduler {
		use super::*;

		#[test]
		#[cfg(feature = "runtime-benchmarks")]
		fn call_queue_per_block_is_limited() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::MaxScheduledPerBlock>(),
				TypeId::of::<ConstU32<512>>(),
			);
		}

		#[test]
		#[cfg(not(feature = "runtime-benchmarks"))]
		fn call_queue_per_block_is_limited() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::MaxScheduledPerBlock>(),
				TypeId::of::<ConstU32<50>>(),
			);
		}

		#[test]
		fn weight_per_dispatchable_is_limited() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::MaximumWeight>(),
				TypeId::of::<MaximumSchedulerWeight>(),
			);

			assert_eq!(
				<<Runtime as pallet_scheduler::Config>::MaximumWeight as Get<Weight>>::get(),
				Perbill::from_percent(60) * RuntimeBlockWeights::get().max_block,
			);
		}

		#[test]
		fn privilege_cmp_is_equal_privilege_only() {
			// EqualPrvilegeOnly can be used while ScheduleOrigin is reserve to Root.
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::OriginPrivilegeCmp>(),
				TypeId::of::<EqualPrivilegeOnly>(),
			);
		}

		#[test]
		fn pallets_origin_provided_by_runtime() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::PalletsOrigin>(),
				TypeId::of::<OriginCaller>(),
			);
		}

		#[test]
		fn preimage_is_used_to_look_up_calls() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::Preimages>(),
				TypeId::of::<Preimage>(),
			);
		}

		#[test]
		fn only_root_can_schedule() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::ScheduleOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_scheduler::Config>::WeightInfo>(),
				TypeId::of::<()>()
			);
		}
	}

	mod utility {
		use super::*;

		#[test]
		fn caller_origin_provided_by_runtime() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_utility::Config>::PalletsOrigin>(),
				TypeId::of::<OriginCaller>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_utility::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}
