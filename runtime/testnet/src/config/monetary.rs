use frame_support::traits::{
	fungible,
	tokens::{imbalance::ResolveTo, PayFromAccount, UnityAssetBalanceConversion},
	Imbalance, NeverEnsureOrigin, OnUnbalanced,
};
use pop_runtime_common::{EXISTENTIAL_DEPOSIT, MICRO_UNIT};
use sp_core::crypto::Ss58Codec;
use sp_runtime::traits::{AccountIdConversion, IdentityLookup};

use crate::{
	parameter_types, AccountId, Balance, Balances, BlockNumber, ConstU32, ConstU8,
	ConstantMultiplier, EnsureRoot, PalletId, Runtime, RuntimeEvent, RuntimeFreezeReason,
	RuntimeHoldReason, SlowAdjustingFeeUpdate, System, VariantCountOf, WeightToFee, DAYS,
};

const TREASURY_PALLET_ID: PalletId = PalletId(*b"treasury");

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub TreasuryAccount: AccountId = TREASURY_PALLET_ID.into_account_truncating();
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
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
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MICRO_UNIT;
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
	type WeightInfo = ();
	type WeightToFee = WeightToFee;
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
			EXISTENTIAL_DEPOSIT,
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
