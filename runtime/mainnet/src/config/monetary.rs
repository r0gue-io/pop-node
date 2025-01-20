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
