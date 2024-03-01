use crate::{
    deposit, RuntimeOrigin, xcm_config::LocationToAccountId, AccountId, Assets, Balance, Balances, BlockNumber, Nfts, Runtime, RuntimeEvent, RuntimeHoldReason, DAYS, EXISTENTIAL_DEPOSIT, UNIT
};
use cumulus_primitives_core::AssetInstance;
use frame_support::{
    parameter_types,
    traits::{EnsureOriginWithArg, Everything, EnsureOrigin, AsEnsureOriginWithArg, ConstU32},
    BoundedVec, PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::PalletFeatures;
use parachains_common::{AssetIdForTrustBackedAssets, CollectionId, ItemId, Signature};
use sp_runtime::traits::Verify;
use crate::nonfungibles_pop::MultiLocationCollectionId;
use xcm_executor::traits::ConvertLocation;

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
    pub const NftsItemDeposit: Balance = UNIT / 100;
    pub const NftsMetadataDepositBase: Balance = deposit(1, 129);
    pub const NftsAttributeDepositBase: Balance = deposit(1, 0);
    pub const NftsDepositPerByte: Balance = deposit(0, 1);
    pub const NftsMaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
}

pub type TrustBackedNfts = pallet_nfts::Instance1;
impl pallet_nfts::Config<TrustBackedNfts> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = CollectionId;
    type ItemId = ItemId;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type ForceOrigin = AssetsForceOrigin;
    type Locker = ();
    type CollectionDeposit = NftsCollectionDeposit;
    type ItemDeposit = NftsItemDeposit;
    type MetadataDepositBase = NftsMetadataDepositBase;
    type AttributeDepositBase = NftsAttributeDepositBase;
    type DepositPerByte = NftsDepositPerByte;
    type StringLimit = ConstU32<256>;
    type KeyLimit = ConstU32<64>;
    type ValueLimit = ConstU32<256>;
    type ApprovalsLimit = ConstU32<20>;
    type ItemAttributesApprovalsLimit = ConstU32<30>;
    type MaxTips = ConstU32<10>;
    type MaxDeadlineDuration = NftsMaxDeadlineDuration;
    type MaxAttributesPerCall = ConstU32<10>;
    type Features = NftsPalletFeatures;
    type OffchainSignature = Signature;
    type OffchainPublic = <Signature as Verify>::Signer;
    type WeightInfo = pallet_nfts::weights::SubstrateWeight<Self>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
}

parameter_types! {
    pub const NftFractionalizationPalletId: PalletId = PalletId(*b"fraction");
    pub NewAssetSymbol: BoundedVec<u8, AssetsStringLimit> = (*b"FRAC").to_vec().try_into().unwrap();
    pub NewAssetName: BoundedVec<u8, AssetsStringLimit> = (*b"Frac").to_vec().try_into().unwrap();
}

impl pallet_nft_fractionalization::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Deposit = AssetDeposit;
    type Currency = Balances;
    type NewAssetSymbol = NewAssetSymbol;
    type NewAssetName = NewAssetName;
    type StringLimit = AssetsStringLimit;
    type NftCollectionId = <Self as pallet_nfts::Config<TrustBackedNfts>>::CollectionId;
    type NftId = <Self as pallet_nfts::Config<TrustBackedNfts>>::ItemId;
    type AssetBalance = <Self as pallet_assets::Config<TrustBackedAssets>>::Balance;
    type AssetId = <Self as pallet_assets::Config<TrustBackedAssets>>::AssetId;
    type Assets = Assets;
    type Nfts = Nfts;
    type PalletId = NftFractionalizationPalletId;
    type WeightInfo = pallet_nft_fractionalization::weights::SubstrateWeight<Self>;
    type RuntimeHoldReason = RuntimeHoldReason;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub struct ForeignCreatorsNfts;

impl EnsureOriginWithArg<RuntimeOrigin, MultiLocationCollectionId> for ForeignCreatorsNfts {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		a: &MultiLocationCollectionId,
	) -> sp_std::result::Result<Self::Success, RuntimeOrigin> {
		let origin_location = pallet_xcm::EnsureXcm::<Everything>::try_origin(o.clone())?;
		if !a.inner().starts_with(&origin_location.clone().try_into().unwrap()) {
			return Err(o)
		}
		LocationToAccountId::convert_location(&origin_location).ok_or(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(a: &MultiLocationCollectionId) -> Result<RuntimeOrigin, ()> {
		Ok(pallet_xcm::Origin::Xcm(a.clone().into()).into())
	}
}

pub type ForeignNfts = pallet_nfts::Instance2;
impl pallet_nfts::Config<ForeignNfts> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = MultiLocationCollectionId;
    type ItemId = AssetInstance;
    type Currency = Balances;
    type CreateOrigin = ForeignCreatorsNfts;
    type ForceOrigin = AssetsForceOrigin;
    type Locker = ();
    type CollectionDeposit = NftsCollectionDeposit;
    type ItemDeposit = NftsItemDeposit;
    type MetadataDepositBase = NftsMetadataDepositBase;
    type AttributeDepositBase = NftsAttributeDepositBase;
    type DepositPerByte = NftsDepositPerByte;
    type StringLimit = ConstU32<256>;
    type KeyLimit = ConstU32<64>;
    type ValueLimit = ConstU32<256>;
    type ApprovalsLimit = ConstU32<20>;
    type ItemAttributesApprovalsLimit = ConstU32<30>;
    type MaxTips = ConstU32<10>;
    type MaxDeadlineDuration = NftsMaxDeadlineDuration;
    type MaxAttributesPerCall = ConstU32<10>;
    type Features = NftsPalletFeatures;
    type OffchainSignature = Signature;
    type OffchainPublic = <Signature as Verify>::Signer;
    type WeightInfo = pallet_nfts::weights::SubstrateWeight<Self>;
    #[cfg(feature = "runtime-benchmarks")]
    type Helper = ();
}

pub type TrustBackedAssets = pallet_assets::Instance1;
impl pallet_assets::Config<TrustBackedAssets> for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = AssetIdForTrustBackedAssets;
    type AssetIdParameter = codec::Compact<AssetIdForTrustBackedAssets>;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type ForceOrigin = AssetsForceOrigin;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Self>;
    type CallbackHandle = ();
    type AssetAccountDeposit = AssetAccountDeposit;
    type RemoveItemsLimit = ConstU32<1000>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}
