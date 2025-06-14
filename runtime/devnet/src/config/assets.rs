use frame_support::{
	pallet_prelude::Get,
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32},
	BoundedVec, PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::PalletFeatures;
use parachains_common::{AssetIdForTrustBackedAssets, CollectionId, ItemId, Signature};
use sp_runtime::traits::Verify;

use crate::{
	deposit, AccountId, Assets, Balance, Balances, BlockNumber, Nfts, Runtime, RuntimeEvent,
	RuntimeHoldReason, DAYS, EXISTENTIAL_DEPOSIT, UNIT,
};

/// We allow root to execute privileged asset operations.
pub type AssetsForceOrigin = EnsureRoot<AccountId>;

parameter_types! {
	pub const AssetDeposit: Balance = 10 * UNIT;
	pub const AssetAccountDeposit: Balance = deposit(1, 16);
	pub const ApprovalDeposit: Balance = EXISTENTIAL_DEPOSIT;
	pub const AssetsStringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

parameter_types! {
	pub NftsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
	// Key = 68 bytes (4+16+32+16), Value = 52 bytes (4+32+16)
	pub const NftsCollectionBalanceDeposit: Balance = deposit(1, 120);
	pub const NftsCollectionDeposit: Balance = 10 * UNIT;
	// Key = 116 bytes (4+16+32+16+32+16), Value = 21 bytes (1+4+16)
	pub const NftsCollectionApprovalDeposit: Balance = deposit(1, 137);
	pub const NftsItemDeposit: Balance = UNIT / 100;
	pub const NftsMetadataDepositBase: Balance = deposit(1, 129);
	pub const NftsAttributeDepositBase: Balance = deposit(1, 0);
	pub const NftsDepositPerByte: Balance = deposit(0, 1);
	pub const NftsMaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
/// The maximum length of an attribute key.
pub struct KeyLimit<const N: u32>;

impl<const N: u32> Get<u32> for KeyLimit<N> {
	fn get() -> u32 {
		N
	}
}

// Trust backed NFTs as an instance of the `pallet-nfts` module. The name "TrustBacked" reflects the
// assumption that non-fungible tokens are registered by an account and are trusted to have some
// claimed backing.
pub(crate) type TrustBackedNftsInstance = pallet_nfts::Instance1;
/// Call type for trust backed NFTs. The type represents the calls that can be made to the
/// `pallet-nfts` module with the `TrustBackedNftsInstance` configuration.
pub type TrustBackedNftsCall = pallet_nfts::Call<Runtime, TrustBackedNftsInstance>;
impl pallet_nfts::Config<TrustBackedNftsInstance> for Runtime {
	// TODO: source from primitives
	type ApprovalsLimit = ConstU32<20>;
	type AttributeDepositBase = NftsAttributeDepositBase;
	type BlockNumberProvider = frame_system::Pallet<Runtime>;
	type CollectionApprovalDeposit = NftsCollectionApprovalDeposit;
	type CollectionBalanceDeposit = NftsCollectionBalanceDeposit;
	type CollectionDeposit = NftsCollectionDeposit;
	// TODO: source from primitives
	type CollectionId = CollectionId;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Currency = Balances;
	type DepositPerByte = NftsDepositPerByte;
	type Features = NftsPalletFeatures;
	type ForceOrigin = AssetsForceOrigin;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type ItemAttributesApprovalsLimit = ConstU32<30>;
	type ItemDeposit = NftsItemDeposit;
	// TODO: source from primitives
	type ItemId = ItemId;
	// TODO: source from primitives
	type KeyLimit = KeyLimit<64>;
	type Locker = ();
	type MaxAttributesPerCall = ConstU32<10>;
	type MaxDeadlineDuration = NftsMaxDeadlineDuration;
	type MaxTips = ConstU32<10>;
	type MetadataDepositBase = NftsMetadataDepositBase;
	type OffchainPublic = <Signature as Verify>::Signer;
	type OffchainSignature = Signature;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<256>;
	type ValueLimit = ConstU32<256>;
	type WeightInfo = pallet_nfts::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub const NftFractionalizationPalletId: PalletId = PalletId(*b"fraction");
	pub NewAssetSymbol: BoundedVec<u8, AssetsStringLimit> = (*b"FRAC").to_vec().try_into().unwrap();
	pub NewAssetName: BoundedVec<u8, AssetsStringLimit> = (*b"Frac").to_vec().try_into().unwrap();
}

impl pallet_nft_fractionalization::Config for Runtime {
	type AssetBalance = <Self as pallet_assets::Config<TrustBackedAssetsInstance>>::Balance;
	type AssetId = <Self as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetId;
	type Assets = Assets;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type Currency = Balances;
	type Deposit = AssetDeposit;
	type NewAssetName = NewAssetName;
	type NewAssetSymbol = NewAssetSymbol;
	type NftCollectionId = <Self as pallet_nfts::Config<TrustBackedNftsInstance>>::CollectionId;
	type NftId = <Self as pallet_nfts::Config<TrustBackedNftsInstance>>::ItemId;
	type Nfts = Nfts;
	type PalletId = NftFractionalizationPalletId;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type StringLimit = AssetsStringLimit;
	type WeightInfo = pallet_nft_fractionalization::weights::SubstrateWeight<Self>;
}

pub(crate) type TrustBackedAssetsInstance = pallet_assets::Instance1;
pub type TrustBackedAssetsCall = pallet_assets::Call<Runtime, TrustBackedAssetsInstance>;
impl pallet_assets::Config<TrustBackedAssetsInstance> for Runtime {
	type ApprovalDeposit = ApprovalDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type AssetDeposit = AssetDeposit;
	type AssetId = AssetIdForTrustBackedAssets;
	type AssetIdParameter = codec::Compact<AssetIdForTrustBackedAssets>;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type CallbackHandle = pallet_assets::AutoIncAssetId<Runtime, TrustBackedAssetsInstance>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = AssetsForceOrigin;
	type Freezer = ();
	type Holder = ();
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type RemoveItemsLimit = ConstU32<1000>;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = AssetsStringLimit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Self>;
}

#[cfg(test)]
mod tests {
	use frame_support::traits::StorageInfoTrait;

	use super::*;

	#[test]
	fn ensure_account_balance_deposit() {
		let max_size =
			pallet_nfts::AccountBalance::<Runtime, TrustBackedNftsInstance>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
		assert_eq!(deposit(1, max_size), NftsCollectionBalanceDeposit::get());
	}

	#[test]
	fn ensure_collection_approval_deposit() {
		let max_size =
			pallet_nfts::CollectionApprovals::<Runtime, TrustBackedNftsInstance>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
		assert_eq!(deposit(1, max_size), NftsCollectionApprovalDeposit::get());
	}
}
