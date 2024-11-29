use frame_support::{
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
	pub const NftsCollectionDeposit: Balance = 10 * UNIT;
	pub const NftsCollectionApprovalDeposit: Balance = deposit(1, 0);
	pub const NftsItemDeposit: Balance = UNIT / 100;
	pub const NftsMetadataDepositBase: Balance = deposit(1, 129);
	pub const NftsAttributeDepositBase: Balance = deposit(1, 0);
	pub const NftsDepositPerByte: Balance = deposit(0, 1);
	pub const NftsMaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
}

impl pallet_nfts::Config for Runtime {
	// TODO: source from primitives
	type ApprovalsLimit = ConstU32<20>;
	type AttributeDepositBase = NftsAttributeDepositBase;
	type CollectionApprovalDeposit = NftsCollectionApprovalDeposit;
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
	type KeyLimit = ConstU32<64>;
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
	type NftCollectionId = <Self as pallet_nfts::Config>::CollectionId;
	type NftId = <Self as pallet_nfts::Config>::ItemId;
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
	type CallbackHandle = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = AssetsForceOrigin;
	type Freezer = ();
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type RemoveItemsLimit = ConstU32<1000>;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = AssetsStringLimit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Self>;
}
