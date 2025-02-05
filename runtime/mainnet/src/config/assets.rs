use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::PalletFeatures;
use parachains_common::{AssetIdForTrustBackedAssets, CollectionId, ItemId, Signature};
use sp_runtime::traits::Verify;

use crate::{
	config::monetary::ExistentialDeposit, deposit, AccountId, Balance, Balances, BlockNumber,
	Runtime, RuntimeEvent, DAYS, UNIT,
};

/// We allow root to execute privileged asset operations.
pub type AssetsForceOrigin = EnsureRoot<AccountId>;

parameter_types! {
	pub const AssetDeposit: Balance = deposit(1, 210);
	// Enough to keep the balance in state.
	pub const AssetAccountDeposit: Balance = deposit(1, 16);
	pub const ApprovalDeposit: Balance = ExistentialDeposit::get();
	pub const AssetsStringLimit: u32 = 50;
	// Key = AssetId 4 bytes + Hash length 16 bytes; Value = 26 bytes (16+4+4+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = deposit(1, 46);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
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

impl pallet_nfts::Config for Runtime {
	// TODO: source from primitives
	type ApprovalsLimit = ConstU32<20>;
	type AttributeDepositBase = NftsAttributeDepositBase;
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

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use codec::MaxEncodedLen;
	use frame_support::{traits::StorageInfoTrait, Blake2_128Concat, StorageHasher};
	use pop_runtime_common::MILLI_UNIT;
	use sp_runtime::traits::Get;

	use super::*;
	use crate::{AccountId, Balance};

	mod assets {
		use super::*;

		#[test]
		fn ensure_asset_approval_deposit() {
			assert_eq!(MILLI_UNIT * 100, ApprovalDeposit::get());
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::ApprovalDeposit,
				>(),
				TypeId::of::<ApprovalDeposit>(),
			);
		}

		#[test]
		fn ensure_asset_account_deposit() {
			// Provide a deposit enough to keep the balance in state.
			assert_eq!(deposit(1, Balance::max_encoded_len() as u32), AssetAccountDeposit::get());
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetAccountDeposit>(),
				TypeId::of::<AssetAccountDeposit>(),
			);
		}

		#[test]
		fn ensure_asset_deposit() {
			let max_size = Blake2_128Concat::max_len::<
				<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetId,
			>() +
				pallet_assets::AssetDetails::<Balance, AccountId, Balance>::max_encoded_len();
			assert_eq!(deposit(1, max_size as u32), AssetDeposit::get());
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetDeposit,
				>(),
				TypeId::of::<AssetDeposit>(),
			);
		}

		#[test]
		fn defines_specific_asset_id() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetId>(
				),
				TypeId::of::<AssetIdForTrustBackedAssets>(),
			);
		}

		#[test]
		fn defines_specific_asset_id_parameter() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetIdParameter,
				>(),
				TypeId::of::<codec::Compact<AssetIdForTrustBackedAssets>>(),
			);
		}

		#[test]
		fn units_to_record_balance() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::Balance>(
				),
				TypeId::of::<Balance>(),
			);
		}

		#[test]
		#[cfg(feature = "runtime-benchmarks")]
		fn benchmark_helper_is_default() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::BenchmarkHelper,
				>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn callback_handle_is_default() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::CallbackHandle,
				>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn create_origin_ensures_signed() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::CreateOrigin,
				>(),
				TypeId::of::<AsEnsureOriginWithArg<EnsureSigned<AccountId>>>(),
			);
		}

		#[test]
		fn balances_provide_currency() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::Currency>(
				),
				TypeId::of::<Balances>(),
			);
		}

		#[test]
		fn no_extra_data_is_stored() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::Extra>(
				),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn force_origin_ensures_root() {
			assert_eq!(TypeId::of::<AssetsForceOrigin>(), TypeId::of::<EnsureRoot<AccountId>>(),);
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::ForceOrigin,
				>(),
				TypeId::of::<AssetsForceOrigin>(),
			);
		}

		#[test]
		fn freezer_is_not_configured() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::Freezer>(
				),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn ensure_metadata_deposit_base() {
			let max_size = Blake2_128Concat::max_len::<
				<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetId,
			>() + pallet_assets::AssetMetadata::<Balance, u32>::max_encoded_len();
			assert_eq!(deposit(1, max_size as u32), MetadataDepositBase::get());

			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::MetadataDepositBase>(),
				TypeId::of::<MetadataDepositBase>(),
			);
		}

		#[test]
		fn ensure_metadata_deposit_per_byte() {
			assert_eq!(deposit(0, 1), MetadataDepositPerByte::get());
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::MetadataDepositPerByte>(),
				TypeId::of::<MetadataDepositPerByte>(),
			);
		}

		#[test]
		fn only_destroys_so_many_items_at_once() {
			assert_eq!(
				<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::RemoveItemsLimit as Get<u32>>::get(),
				1000,
			);
		}

		#[test]
		fn ensure_string_limit() {
			assert_eq!(AssetsStringLimit::get(), 50,);
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::StringLimit,
				>(),
				TypeId::of::<AssetsStringLimit>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::WeightInfo,
				>(),
				TypeId::of::<()>(),
			);
		}
	}

	mod nfts {
		use super::*;

		#[test]
		fn ensure_account_balance_deposit() {
			let max_size = pallet_nfts::AccountBalance::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			assert_eq!(deposit(1, max_size), NftsCollectionBalanceDeposit::get());
		}

		#[test]
		fn ensure_collection_approval_deposit() {
			let max_size = pallet_nfts::CollectionApprovals::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			assert_eq!(deposit(1, max_size), NftsCollectionApprovalDeposit::get());
		}
	}
}
