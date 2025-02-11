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
	Runtime, RuntimeEvent, DAYS,
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
	// Accounts for `Collection` +
	// `CollectionRoleOf` +
	// `CollectionConfigOf` +
	// `CollectionAccount`
	// Refer to `ensure_collection_deposit` test for specifics.
	pub const NftsCollectionDeposit: Balance = deposit(4, 294);
	// Key = 116 bytes (4+16+32+16+32+16), Value = 21 bytes (1+4+16)
	pub const NftsCollectionApprovalDeposit: Balance = deposit(1, 137);
	// Accounts for `Item` storage item max size.
	pub const NftsItemDeposit: Balance = deposit(1, 861);
	pub const NftsMetadataDepositBase: Balance = deposit(1, 56);
	pub const NftsAttributeDepositBase: Balance = deposit(1, 175);
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
		use pallet_nfts::PalletFeature::*;
		use sp_runtime::{MultiSignature, MultiSigner};

		use super::*;

		#[test]
		fn item_approvals_limit_is_20() {
			assert_eq!(<<Runtime as pallet_nfts::Config>::ApprovalsLimit as Get<u32>>::get(), 20);
		}

		#[test]
		fn ensure_attribute_deposit_base() {
			// Max possible size of Attribute.
			let max_size = pallet_nfts::Attribute::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			// Size of Attribute value: `(BoundedVec<u8, T::ValueLimit>, AttributeDepositOf<T, I>)`.
			let value_size = <<Runtime as pallet_nfts::Config>::ValueLimit as Get<u32>>::get() +
				AccountId::max_encoded_len() as u32 +
				Balance::max_encoded_len() as u32;
			// We only account for the key length as the deposit base.
			assert_eq!(deposit(1, max_size - value_size), NftsAttributeDepositBase::get());
		}

		#[test]
		fn ensure_collection_approval_deposit() {
			let max_size = pallet_nfts::CollectionApprovals::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			assert_eq!(deposit(1, max_size), NftsCollectionApprovalDeposit::get());
		}

		#[test]
		fn ensure_account_balance_deposit() {
			let max_size = pallet_nfts::AccountBalance::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			assert_eq!(deposit(1, max_size), NftsCollectionBalanceDeposit::get());
		}

		#[test]
		fn ensure_collection_deposit() {
			let max_collection_size = pallet_nfts::Collection::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();

			// Left for the reviewer to verify discrepancy.
			// println!("STORAGE INFO MAX SIZE: {:?}", &max_collection_size);
			// let key = Blake2_128Concat::max_len::<CollectionId>();
			// let value_size = AccountId::max_encoded_len() + Balance::max_encoded_len() + 8 + 8 +
			// 8 + 8; println!("MAX CALCULATED SIZE: {:?}", key + value_size);

			let max_collection_role_size = pallet_nfts::CollectionRoleOf::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();

			let max_collection_config_size =
				pallet_nfts::CollectionConfigOf::<Runtime>::storage_info()
					.first()
					.and_then(|info| info.max_size)
					.unwrap_or_default();

			let max_collection_account_size =
				pallet_nfts::CollectionAccount::<Runtime>::storage_info()
					.first()
					.and_then(|info| info.max_size)
					.unwrap_or_default();

			let total_collection_size = max_collection_size +
				max_collection_role_size +
				max_collection_config_size +
				max_collection_account_size;

			// 4 different storage items means 4 different keys.
			assert_eq!(deposit(4, total_collection_size), NftsCollectionDeposit::get());
		}

		#[test]
		fn collection_id_is_u32() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::CollectionId>(),
				TypeId::of::<u32>(),
			);
		}

		#[test]
		fn create_origin_ensures_signed() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::CreateOrigin>(),
				TypeId::of::<AsEnsureOriginWithArg<EnsureSigned<AccountId>>>(),
			);
		}

		#[test]
		fn balances_provides_currency() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::Currency>(),
				TypeId::of::<Balances>(),
			);
		}

		#[test]
		fn ensure_deposit_per_byte_deposit() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::DepositPerByte>(),
				TypeId::of::<NftsDepositPerByte>(),
			);
			assert_eq!(
				<<Runtime as pallet_nfts::Config>::DepositPerByte as Get<Balance>>::get(),
				deposit(0, 1)
			);
		}

		#[test]
		fn all_feature_are_active() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::Features>(),
				TypeId::of::<NftsPalletFeatures>(),
			);

			let features = [Trading, Attributes, Approvals, Swaps];

			for feat in features {
				assert!(<Runtime as pallet_nfts::Config>::Features::get().is_enabled(feat));
			}
		}

		#[test]
		fn force_origin_ensures_root() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::ForceOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[cfg(feature = "runtime-benchmarks")]
		#[test]
		fn benchmark_helper_is_default() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::Helper>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn max_attributes_per_item_is_30() {
			assert_eq!(
				<<Runtime as pallet_nfts::Config>::ItemAttributesApprovalsLimit as Get<u32>>::get(),
				30
			);
		}

		#[test]
		fn ensure_item_deposit_deposit() {
			let max_size = pallet_nfts::Item::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			assert_eq!(deposit(1, max_size), NftsItemDeposit::get());
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::ItemDeposit>(),
				TypeId::of::<NftsItemDeposit>(),
			);
		}

		#[test]
		fn item_id_is_u32() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::ItemId>(),
				TypeId::of::<u32>(),
			);
		}

		#[test]
		fn attribute_key_maximum_lenght_is_64() {
			assert_eq!(<<Runtime as pallet_nfts::Config>::KeyLimit as Get<u32>>::get(), 64,);
		}

		#[test]
		fn locker_is_default() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::Locker>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn max_attributes_set_per_call_is_10() {
			assert_eq!(
				<<Runtime as pallet_nfts::Config>::MaxAttributesPerCall as Get<u32>>::get(),
				10,
			);
		}

		#[test]
		fn deadline_duration_is_360_days() {
			assert_eq!(NftsMaxDeadlineDuration::get(), 12 * 30 * DAYS);
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::MaxDeadlineDuration>(),
				TypeId::of::<NftsMaxDeadlineDuration>(),
			);
		}

		#[test]
		fn max_tips_paid_at_once_is_10() {
			assert_eq!(<<Runtime as pallet_nfts::Config>::MaxTips as Get<u32>>::get(), 10,);
		}

		#[test]
		fn ensure_metadata_deposit_base() {
			// MetadataDepositBase is used for both, items and collections.
			let item_metadata_key_size =
				Blake2_128Concat::max_len::<CollectionId>() + Blake2_128Concat::max_len::<ItemId>();
			let collection_metadata_key_size = Blake2_128Concat::max_len::<CollectionId>();
			// We take the bigger of both sizes and size_of(Balance) which is always written.
			let base_size = item_metadata_key_size.max(collection_metadata_key_size) +
				Balance::max_encoded_len();
			assert_eq!(NftsMetadataDepositBase::get(), deposit(1, base_size as u32));
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::MetadataDepositBase>(),
				TypeId::of::<NftsMetadataDepositBase>(),
			);
		}

		#[test]
		fn off_chain_public_identifies_as_multisigner() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::OffchainPublic>(),
				TypeId::of::<<Signature as Verify>::Signer>(),
			);

			assert_eq!(TypeId::of::<<Signature as Verify>::Signer>(), TypeId::of::<MultiSigner>(),);
		}

		#[test]
		fn off_chain_signature_is_multisignature() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::OffchainSignature>(),
				TypeId::of::<Signature>(),
			);

			assert_eq!(TypeId::of::<Signature>(), TypeId::of::<MultiSignature>(),);
		}

		#[test]
		fn string_limit_is_256() {
			assert_eq!(<<Runtime as pallet_nfts::Config>::StringLimit as Get<u32>>::get(), 256,);
		}

		#[test]
		fn value_limit_is_256() {
			assert_eq!(<<Runtime as pallet_nfts::Config>::ValueLimit as Get<u32>>::get(), 256,);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}
