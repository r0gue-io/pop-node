use frame_support::traits::{
	fungible,
	tokens::{PayFromAccount, UnityAssetBalanceConversion},
	Imbalance, NeverEnsureOrigin, OnUnbalanced,
};
use pop_runtime_common::UNIT;
use sp_core::crypto::Ss58Codec;
use sp_runtime::traits::{AccountIdConversion, IdentityLookup};

use crate::{
	parameter_types, AccountId, Balance, Balances, BlockNumber, ConstU32, ConstU8,
	ConstantMultiplier, EnsureRoot, PalletId, ResolveTo, Runtime, RuntimeEvent,
	RuntimeFreezeReason, RuntimeHoldReason, SlowAdjustingFeeUpdate, System, VariantCountOf, DAYS,
	EXISTENTIAL_DEPOSIT,
};

/// Deposit rate for stored data. 1/100th of the Relay Chain's deposit rate. `items` is the
/// number of keys in storage and `bytes` is the size of the value.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	// src: https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/constants/src/polkadot.rs#L70
	(items as Balance * 20 * UNIT + (bytes as Balance) * 100 * fee::MILLI_CENTS) / 100
}

/// Constants related to Polkadot fee payment.
// Source: https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/constants/src/polkadot.rs#L65C47-L65C58
pub mod fee {
	use frame_support::{
		pallet_prelude::Weight,
		weights::{
			constants::ExtrinsicBaseWeight, FeePolynomial, WeightToFeeCoefficient,
			WeightToFeeCoefficients, WeightToFeePolynomial,
		},
	};
	use pop_runtime_common::{Balance, UNIT};
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	pub const CENTS: Balance = UNIT / 100; // 100_000_000
	pub const MILLI_CENTS: Balance = CENTS / 1_000; // 100_000

	/// Cost of every transaction byte.
	// It is the Relay Chain (Polkadot) `TransactionByteFee` / 20.
	pub const TRANSACTION_BYTE_FEE: Balance = MILLI_CENTS / 2;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl frame_support::weights::WeightToFee for WeightToFee {
		type Balance = Balance;

		fn weight_to_fee(weight: &Weight) -> Self::Balance {
			let time_poly: FeePolynomial<Balance> = RefTimeToFee::polynomial().into();
			let proof_poly: FeePolynomial<Balance> = ProofSizeToFee::polynomial().into();

			// Take the maximum instead of the sum to charge by the more scarce resource.
			time_poly.eval(weight.ref_time()).max(proof_poly.eval(weight.proof_size()))
		}
	}

	/// Maps the reference time component of `Weight` to a fee.
	pub struct RefTimeToFee;
	impl WeightToFeePolynomial for RefTimeToFee {
		type Balance = Balance;

		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// In Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			// The standard system parachain configuration is 1/20 of that, as in 1/200 CENT.
			let p = CENTS;
			let q = 200 * Balance::from(ExtrinsicBaseWeight::get().ref_time());

			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}

	/// Maps the proof size component of `Weight` to a fee.
	pub struct ProofSizeToFee;
	impl WeightToFeePolynomial for ProofSizeToFee {
		type Balance = Balance;

		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// Map 20kb proof to 1 CENT.
			let p = CENTS;
			let q = 20_000;

			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

const TREASURY_PALLET_ID: PalletId = PalletId(*b"treasury");

parameter_types! {
	// Increase ED 100 times to match system chains: 1_000_000_000.
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT * 100;
	pub TreasuryAccount: AccountId = TREASURY_PALLET_ID.into_account_truncating();
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DoneSlashHandler = ();
	type DustRemoval = ResolveTo<TreasuryAccount, Balances>;
	type ExistentialDeposit = ExistentialDeposit;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type MaxLocks = ConstU32<0>;
	type MaxReserves = ConstU32<0>;
	type ReserveIdentifier = ();
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = fee::TRANSACTION_BYTE_FEE;
	pub MaintenanceAccount: AccountId = AccountId::from_ss58check("1Y3M8pnn3rJcxQn46SbocHcUHYfs4j8W2zHX7XNK99LGSVe").expect("maintenance address is valid SS58");
}

/// DealWithFees is used to handle fees and tips in the OnChargeTransaction trait,
/// by implementing OnUnbalanced.
pub struct DealWithFees;
impl OnUnbalanced<fungible::Credit<AccountId, Balances>> for DealWithFees {
	fn on_unbalanceds(
		mut fees_then_tips: impl Iterator<Item = fungible::Credit<AccountId, Balances>>,
	) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			let split = fees.ration(50, 50);
			ResolveTo::<TreasuryAccount, Balances>::on_unbalanced(split.0);
			ResolveTo::<MaintenanceAccount, Balances>::on_unbalanced(split.1);
		}
	}
}

/// The type responsible for payment in pallet_transaction_payment.
pub type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;

impl pallet_transaction_payment::Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type OnChargeTransaction = OnChargeTransaction;
	type OperationalFeeMultiplier = ConstU8<5>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
	type WeightToFee = fee::WeightToFee;
}

pub(crate) type TreasuryPaymaster<T> = PayFromAccount<T, TreasuryAccount>;
parameter_types! {
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;
	pub const MaxApprovals: u32 = 100;
	pub const PayoutPeriod: BlockNumber = 30 * DAYS;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct BenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_treasury::ArgumentsFactory<(), AccountId> for BenchmarkHelper {
	fn create_asset_kind(_seed: u32) -> () {
		()
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId {
		let account_id = AccountId::from(seed);
		Balances::force_set_balance(
			crate::RuntimeOrigin::root(),
			account_id.clone().into(),
			ExistentialDeposit::get(),
		)
		.unwrap();
		account_id
	}
}
impl pallet_treasury::Config for Runtime {
	type AssetKind = ();
	type BalanceConverter = UnityAssetBalanceConversion;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelper;
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type BlockNumberProvider = System;
	type Burn = ();
	type BurnDestination = ();
	type Currency = Balances;
	type MaxApprovals = MaxApprovals;
	type PalletId = TreasuryPalletId;
	type Paymaster = TreasuryPaymaster<Self::Currency>;
	type PayoutPeriod = PayoutPeriod;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendFunds = ();
	/// Never allow origins except via the proposals process.
	type SpendOrigin = NeverEnsureOrigin<Balance>;
	type SpendPeriod = SpendPeriod;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use codec::Encode;
	use frame_support::{
		assert_ok,
		dispatch::GetDispatchInfo,
		traits::{fungible::Mutate, Get},
		weights::{constants::ExtrinsicBaseWeight, Weight, WeightToFee},
	};
	use pallet_transaction_payment::OnChargeTransaction as OnChargeTransactionT;
	use pop_runtime_common::{MICRO_UNIT, MILLI_UNIT};
	use sp_keyring::sr25519::Keyring;
	use sp_runtime::{traits::Dispatchable, BuildStorage};

	use super::*;
	use crate::{AccountId, RuntimeCall, RuntimeOrigin, UNIT};

	type OnChargeTransaction = <Runtime as pallet_transaction_payment::Config>::OnChargeTransaction;

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let initial_balance = 100_000_000 * UNIT;
		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				(TreasuryAccount::get(), initial_balance),
				(MaintenanceAccount::get(), initial_balance),
				(Keyring::Alice.to_account_id(), initial_balance),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	mod deposit_and_fees {
		use super::*;

		#[test]
		fn deposit_works() {
			const CENTS: Balance = UNIT / 100; // 100_000_000
			const MILLI_CENTS: Balance = CENTS / 1_000; // 100_000

			// https://github.com/polkadot-fellows/runtimes/blob/e220854a081f30183999848ce6c11ca62647bcfa/relay/polkadot/constants/src/lib.rs#L36
			fn relay_deposit(items: u32, bytes: u32) -> Balance {
				items as Balance * 20 * UNIT + (bytes as Balance) * 100 * MILLI_CENTS
			}

			// https://github.com/polkadot-fellows/runtimes/blob/e220854a081f30183999848ce6c11ca62647bcfa/system-parachains/constants/src/polkadot.rs#L70
			fn system_para_deposit(items: u32, bytes: u32) -> Balance {
				relay_deposit(items, bytes) / 100
			}

			assert_eq!(deposit(2, 64), system_para_deposit(2, 64))
		}

		#[test]
		fn units_are_correct() {
			// UNIT should have 10 decimals
			assert_eq!(UNIT, 10_000_000_000);
			assert_eq!(MILLI_UNIT, 10_000_000);
			assert_eq!(MICRO_UNIT, 10_000);

			// fee specific units
			assert_eq!(fee::CENTS, 100_000_000);
			assert_eq!(fee::MILLI_CENTS, 100_000);
		}

		#[test]
		fn transaction_byte_fee_is_correct() {
			assert_eq!(fee::TRANSACTION_BYTE_FEE, 50_000);
		}

		#[test]
		fn balances_stores_account_balances_in_system() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::AccountStore>(),
				TypeId::of::<System>(),
			);
		}
	}

	mod balances {
		use super::*;

		#[test]
		fn uses_u128_balance() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::Balance>(),
				TypeId::of::<u128>(),
			);
		}

		#[test]
		fn done_slash_handler_does_not_have_callbacks() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::DoneSlashHandler>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn dust_removal_handler_resolves_to_treasury_account() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::DustRemoval>(),
				TypeId::of::<ResolveTo<TreasuryAccount, Balances>>(),
			);

			new_test_ext().execute_with(|| {
				let existential_deposit =
					<<Runtime as pallet_balances::Config>::ExistentialDeposit>::get();
				let who = AccountId::from([1u8; 32]);
				let beneficiary = AccountId::from([2u8; 32]);
				let call = RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
					dest: sp_runtime::MultiAddress::Id(beneficiary),
					value: UNIT + (existential_deposit / 2),
				});
				Balances::set_balance(&who, existential_deposit + UNIT);
				Balances::set_balance(&TreasuryAccount::get(), existential_deposit);

				// `who`'s balance goes under ED.
				assert_ok!(call.dispatch(RuntimeOrigin::signed(who.clone())));
				// `who` has been dusted.
				assert_eq!(Balances::free_balance(&who), 0);
				System::assert_has_event(RuntimeEvent::Balances(
					pallet_balances::Event::DustLost {
						account: who,
						amount: existential_deposit / 2,
					},
				));
				// Treasury balance equals its initial balance + the dusted amount from `who`.
				assert_eq!(
					Balances::free_balance(&TreasuryAccount::get()),
					existential_deposit + existential_deposit / 2
				);
			})
		}

		#[test]
		fn requires_existential_deposit() {
			// Verify type definition.
			assert_eq!(ExistentialDeposit::get(), EXISTENTIAL_DEPOSIT * 100);
			assert_eq!(ExistentialDeposit::get(), 1_000_000_000);
			// Verify pallet configuration.
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::ExistentialDeposit>(),
				TypeId::of::<ExistentialDeposit>()
			);
		}

		#[test]
		fn freeze_identifier_uses_runtime_freeze_reason() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::FreezeIdentifier>(),
				TypeId::of::<RuntimeFreezeReason>(),
			);
		}

		#[test]
		fn max_freezes_uses_runtime_freeze_reason() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::MaxFreezes>(),
				TypeId::of::<VariantCountOf<RuntimeFreezeReason>>(),
			);
		}

		#[test]
		fn max_locks_disabled() {
			// Use of locks is deprecated in favour of freezes. See `https://github.com/paritytech/substrate/pull/12951/`
			assert_eq!(<<Runtime as pallet_balances::Config>::MaxLocks as Get<u32>>::get(), 0);
		}

		#[test]
		fn max_reserves_disabled() {
			// Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`
			assert_eq!(<<Runtime as pallet_balances::Config>::MaxReserves as Get<u32>>::get(), 0);
		}

		#[test]
		fn reserve_identifier_disabled() {
			// Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::ReserveIdentifier>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn uses_runtime_freeze_reason() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::RuntimeFreezeReason>(),
				TypeId::of::<RuntimeFreezeReason>(),
			);
		}

		#[test]
		fn uses_runtime_hold_reason() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_balances::Config>::RuntimeHoldReason>(),
				TypeId::of::<RuntimeHoldReason>(),
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_balances::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}

	mod transaction_payment {
		use super::*;

		#[test]
		fn test_fees_and_tip_split() {
			new_test_ext().execute_with(|| {
				let fee_amount = 10;
				let fee = <Balances as fungible::Balanced<AccountId>>::issue(fee_amount);
				let tip_amount = 20;
				let tip = <Balances as fungible::Balanced<AccountId>>::issue(tip_amount);
				let treasury_balance = Balances::free_balance(&TreasuryAccount::get());
				let maintenance_balance = Balances::free_balance(&MaintenanceAccount::get());
				DealWithFees::on_unbalanceds(vec![fee, tip].into_iter());

				// Each to get 50%, total is 30 so 15 each.
				assert_eq!(
					Balances::free_balance(&TreasuryAccount::get()),
					treasury_balance + ((fee_amount + tip_amount) / 2)
				);
				assert_eq!(
					Balances::free_balance(&MaintenanceAccount::get()),
					maintenance_balance + ((fee_amount + tip_amount) / 2)
				);
			});
		}

		#[test]
		fn uses_slow_adjusting_fee_multiplier() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_transaction_payment::Config>::FeeMultiplierUpdate>(
				),
				TypeId::of::<SlowAdjustingFeeUpdate<Runtime>>(),
			);
		}

		#[test]
		fn uses_constant_length_to_fee_multiplier() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_transaction_payment::Config>::LengthToFee>(),
				TypeId::of::<ConstantMultiplier<Balance, TransactionByteFee>>(),
			);
		}

		#[test]
		fn charges_fees_via_balances_and_funds_treasury_and_maintenance_equally() {
			new_test_ext().execute_with(|| {
				let who: AccountId = Keyring::Alice.to_account_id();
				let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
				let fee = UNIT / 10;
				let tip = UNIT / 2;
				let fee_plus_tip = fee + tip;
				let treasury_balance = Balances::free_balance(&TreasuryAccount::get());
				let maintenance_balance = Balances::free_balance(&MaintenanceAccount::get());
				let who_balance = Balances::free_balance(&who);
				let dispatch_info = call.get_dispatch_info();

				// NOTE: OnChargeTransaction functions expect tip to be included within fee
				let liquidity_info =
					<OnChargeTransaction as OnChargeTransactionT<Runtime>>::withdraw_fee(
						&who,
						&call,
						&dispatch_info,
						fee_plus_tip,
						0,
					)
					.unwrap();
				<OnChargeTransaction as OnChargeTransactionT<Runtime>>::correct_and_deposit_fee(
					&who,
					&dispatch_info,
					&call.dispatch(RuntimeOrigin::signed(who.clone())).unwrap(),
					fee_plus_tip,
					0,
					liquidity_info,
				)
				.unwrap();

				let treasury_expected_balance = treasury_balance + (fee_plus_tip / 2);
				let maintenance_expected_balance = maintenance_balance + (fee_plus_tip / 2);
				let who_expected_balance = who_balance - fee_plus_tip;

				assert_ne!(treasury_balance, 0);
				assert_ne!(maintenance_expected_balance, 0);

				assert_eq!(
					Balances::free_balance(&TreasuryAccount::get()),
					treasury_expected_balance
				);
				assert_eq!(
					Balances::free_balance(&MaintenanceAccount::get()),
					maintenance_expected_balance
				);
				assert_eq!(Balances::free_balance(&who), who_expected_balance);
			})
		}

		#[test]
		fn uses_5x_operational_fee_multiplier() {
			assert_eq!(
				<<Runtime as pallet_transaction_payment::Config>::OperationalFeeMultiplier as Get<
					u8,
				>>::get(),
				5
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_transaction_payment::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn uses_weight_to_fee_conversion() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_transaction_payment::Config>::WeightToFee>(),
				TypeId::of::<fee::WeightToFee>(),
			);
		}

		#[test]
		fn weight_to_fee_as_expected() {
			let arb_weight = Weight::from_parts(126_142_001, 123_456);
			let no_proof_size_weight = Weight::from_parts(126_142_001, 0);
			let weights: [Weight; 4] = [
				Weight::zero(),
				<ExtrinsicBaseWeight as Get<Weight>>::get(),
				arb_weight,
				no_proof_size_weight,
			];
			let fees: [u128; 4] = [0, 500_000, 617_280_000, 583_143];
			for (w, f) in weights.iter().zip(fees.iter()) {
				assert_eq!(
					<Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(w),
					*f
				);
			}
		}
	}

	mod treasury {
		use super::*;

		#[test]
		fn asset_kind_is_nothing() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::AssetKind>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn balance_converter_is_set() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BalanceConverter>(),
				TypeId::of::<UnityAssetBalanceConversion>(),
			);
		}

		#[cfg(feature = "runtime-benchmarks")]
		#[test]
		fn benchmark_helper_is_correct_type() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BenchmarkHelper>(),
				TypeId::of::<BenchmarkHelper>(),
			);
		}

		#[test]
		fn beneficiary_is_account_id() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::Beneficiary>(),
				TypeId::of::<AccountId>(),
			);
		}

		#[test]
		fn beneficiary_lookup_is_identity_lookup() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BeneficiaryLookup>(),
				TypeId::of::<IdentityLookup<AccountId>>(),
			);
		}

		#[test]
		fn block_number_provider_is_set() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BlockNumberProvider>(),
				TypeId::of::<System>(),
			);
		}

		#[test]
		fn burn_is_nothing() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::Burn>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn max_approvals_is_set() {
			assert_eq!(<Runtime as pallet_treasury::Config>::MaxApprovals::get(), 100);
		}

		#[test]
		fn pallet_id_is_set() {
			assert_eq!(
				<Runtime as pallet_treasury::Config>::PalletId::get().encode(),
				PalletId(*b"treasury").encode()
			);
		}

		#[test]
		fn treasury_account_is_pallet_id_truncated() {
			assert_eq!(TreasuryAccount::get(), TREASURY_PALLET_ID.into_account_truncating());
		}

		#[test]
		fn paymaster_is_correct_type() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::Paymaster>(),
				TypeId::of::<TreasuryPaymaster<<Runtime as pallet_treasury::Config>::Currency>>(),
			);
		}

		#[test]
		fn payout_period_is_set() {
			assert_eq!(<Runtime as pallet_treasury::Config>::PayoutPeriod::get(), 30 * DAYS);
		}

		#[test]
		fn reject_origin_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::RejectOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}
		#[test]
		fn spend_funds_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::SpendFunds>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn spend_origin_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::SpendOrigin>(),
				TypeId::of::<NeverEnsureOrigin<Balance>>(),
			);
		}

		#[test]
		fn spend_period_is_six_days() {
			assert_eq!(<Runtime as pallet_treasury::Config>::SpendPeriod::get(), 6 * DAYS);
		}

		#[test]
		fn type_of_on_charge_transaction_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_transaction_payment::Config>::OnChargeTransaction>(
				),
				TypeId::of::<OnChargeTransaction>(),
			);
		}

		#[test]
		fn weight_info_is_not_default() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}
