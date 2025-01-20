use pop_runtime_common::UNIT;

use crate::{
	parameter_types, AccountId, Balance, Balances, ConstU32, ConstU8, ConstantMultiplier,
	ResolveTo, Runtime, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason,
	SlowAdjustingFeeUpdate, Ss58Codec, System, VariantCountOf, EXISTENTIAL_DEPOSIT,
};

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	// src: https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/constants/src/polkadot.rs#L70
	(items as Balance * 20 * UNIT + (bytes as Balance) * 100 * fee::MILLI_CENTS) / 100
}

/// Constants related to Polkadot fee payment.
/// Source: https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/constants/src/polkadot.rs#L65C47-L65C58
pub mod fee {
	use frame_support::{
		pallet_prelude::Weight,
		weights::{
			constants::ExtrinsicBaseWeight, FeePolynomial, WeightToFeeCoefficient,
			WeightToFeeCoefficients, WeightToFeePolynomial,
		},
	};
	use pop_runtime_common::{Balance, MILLI_UNIT};
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	pub const CENTS: Balance = MILLI_UNIT * 10; // 100_000_000
	pub const MILLI_CENTS: Balance = CENTS / 1_000; // 100_000

	/// Cost of every transaction byte at Polkadot system parachains.
	///
	/// It is the Relay Chain (Polkadot) `TransactionByteFee` / 20.
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

parameter_types! {
	// increase ED 100 times to match system chains: 1_000_000_000
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT * 100;
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DoneSlashHandler = ();
	type DustRemoval = ();
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
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = fee::TRANSACTION_BYTE_FEE;
	pub SudoAddress: AccountId = AccountId::from_ss58check("15NMV2JX1NeMwarQiiZvuJ8ixUcvayFDcu1F9Wz1HNpSc8gP").expect("sudo address is valid SS58");
}

impl pallet_transaction_payment::Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type OnChargeTransaction =
		pallet_transaction_payment::FungibleAdapter<Balances, ResolveTo<SudoAddress, Balances>>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
	type WeightToFee = fee::WeightToFee;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use frame_support::{
		dispatch::GetDispatchInfo,
		traits::{fungible::Mutate, Get},
	};
	use pallet_transaction_payment::OnChargeTransaction as OnChargeTransactionT;
	use pop_runtime_common::{MICRO_UNIT, MILLI_UNIT};
	use sp_runtime::{traits::Dispatchable, BuildStorage};

	use super::*;
	use crate::{RuntimeCall, RuntimeOrigin, UNIT};

	type OnChargeTransaction = <Runtime as pallet_transaction_payment::Config>::OnChargeTransaction;

	fn new_test_ext() -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
		let mut ext = sp_io::TestExternalities::new(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
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
	fn deposit_works() {
		const UNITS: Balance = 10_000_000_000;
		const DOLLARS: Balance = UNITS; // 10_000_000_000
		const CENTS: Balance = DOLLARS / 100; // 100_000_000
		const MILLI_CENTS: Balance = CENTS / 1_000; // 100_000

		// https://github.com/polkadot-fellows/runtimes/blob/e220854a081f30183999848ce6c11ca62647bcfa/relay/polkadot/constants/src/lib.rs#L36
		fn relay_deposit(items: u32, bytes: u32) -> Balance {
			items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLI_CENTS
		}

		// https://github.com/polkadot-fellows/runtimes/blob/e220854a081f30183999848ce6c11ca62647bcfa/system-parachains/constants/src/polkadot.rs#L70
		fn system_para_deposit(items: u32, bytes: u32) -> Balance {
			relay_deposit(items, bytes) / 100
		}

		assert_eq!(deposit(2, 64), system_para_deposit(2, 64))
	}

	#[test]
	fn balances_stores_account_balances_in_system() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::AccountStore>(),
			TypeId::of::<System>(),
		);
	}

	#[test]
	fn balances_uses_u128_balance() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::Balance>(),
			TypeId::of::<u128>(),
		);
	}

	#[test]
	fn balances_dust_removal_handler_burns_tokens() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::DustRemoval>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn balances_requires_existential_deposit() {
		assert_eq!(ExistentialDeposit::get(), EXISTENTIAL_DEPOSIT * 100);
		assert_eq!(ExistentialDeposit::get(), 1_000_000_000);
	}

	#[test]
	fn balances_freeze_identifier_uses_runtime_freeze_reason() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::FreezeIdentifier>(),
			TypeId::of::<RuntimeFreezeReason>(),
		);
	}

	#[test]
	fn balances_max_freezes_uses_runtime_freeze_reason() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::MaxFreezes>(),
			TypeId::of::<VariantCountOf<RuntimeFreezeReason>>(),
		);
	}

	#[test]
	fn balances_max_locks_disabled() {
		// Use of locks is deprecated in favour of freezes. See `https://github.com/paritytech/substrate/pull/12951/`
		assert_eq!(<<Runtime as pallet_balances::Config>::MaxLocks as Get<u32>>::get(), 0);
	}

	#[test]
	fn balances_max_reserves_disabled() {
		// Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`
		assert_eq!(<<Runtime as pallet_balances::Config>::MaxReserves as Get<u32>>::get(), 0);
	}

	#[test]
	fn balances_reserve_identifier_disabled() {
		// Use of reserves is deprecated in favour of holds. See `https://github.com/paritytech/substrate/pull/12951/`
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::ReserveIdentifier>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn balances_uses_runtime_freeze_reason() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::RuntimeFreezeReason>(),
			TypeId::of::<RuntimeFreezeReason>(),
		);
	}

	#[test]
	fn balances_uses_runtime_hold_reason() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_balances::Config>::RuntimeHoldReason>(),
			TypeId::of::<RuntimeHoldReason>(),
		);
	}

	#[test]
	fn balances_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_balances::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn transaction_payment_uses_slow_adjusting_fee_multiplier() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_transaction_payment::Config>::FeeMultiplierUpdate>(),
			TypeId::of::<SlowAdjustingFeeUpdate<Runtime>>(),
		);
	}

	#[test]
	fn transaction_payment_uses_constant_length_to_fee_multiplier() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_transaction_payment::Config>::LengthToFee>(),
			TypeId::of::<ConstantMultiplier<Balance, TransactionByteFee>>(),
		);
	}

	#[test]
	fn transaction_payment_charges_fees_via_balances_and_funds_sudo() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_transaction_payment::Config>::OnChargeTransaction>(),
			TypeId::of::<
				pallet_transaction_payment::FungibleAdapter<
					Balances,
					ResolveTo<SudoAddress, Balances>,
				>,
			>(),
		);

		new_test_ext().execute_with(|| {
			let who = AccountId::from([1u8; 32]);
			let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
			let fee = UNIT / 10;
			let tip = UNIT / 2;
			let fee_plus_tip = fee + tip;
			let sudo_balance = Balances::free_balance(&SudoAddress::get());
			let dispatch_info = call.get_dispatch_info();
			let existential_deposit =
				<<Runtime as pallet_balances::Config>::ExistentialDeposit>::get();

			// NOTE: OnChargeTransaction functions expect tip to be included within fee
			Balances::set_balance(&who, fee + tip + existential_deposit);
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

			assert_eq!(Balances::free_balance(&SudoAddress::get()), sudo_balance + fee + tip);
			assert_eq!(Balances::free_balance(&who), existential_deposit);
		})
	}

	#[test]
	fn transaction_payment_uses_5x_operational_fee_multiplier() {
		assert_eq!(
			<<Runtime as pallet_transaction_payment::Config>::OperationalFeeMultiplier as Get<
				u8,
			>>::get(),
			5
		);
	}

	#[test]
	fn transaction_payment_uses_weight_to_fee_conversion() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_transaction_payment::Config>::WeightToFee>(),
			TypeId::of::<fee::WeightToFee>(),
		);
	}
}
