use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::PalletFeatures;
use pallet_nfts_sdk as pallet_nfts;
use parachains_common::{AssetIdForTrustBackedAssets, CollectionId, ItemId, Signature};
use sp_runtime::traits::Verify;

use crate::{
	config::monetary::ExistentialDeposit, deposit, weights, AccountId, Balance, Balances,
	BlockNumber, Runtime, RuntimeEvent, System, DAYS,
};

/// We allow root to execute privileged asset operations.
pub type AssetsForceOrigin = EnsureRoot<AccountId>;

parameter_types! {
	// Accounts for `Asset` max size.
	// For details, refer to `ensure_asset_deposit`.
	pub const AssetDeposit: Balance = deposit(1, 210);
	// Enough to keep the balance in state / 100.
	// For details, refer to `ensure_asset_account_deposit`.
	pub const AssetAccountDeposit: Balance = deposit(1, 16) / 100;
	pub const ApprovalDeposit: Balance = ExistentialDeposit::get();
	pub const AssetsStringLimit: u32 = 50;
	// Accounts for `Metadata` key size + some elements from `AssetMetadata`.
	// For details, refer to `ensure_metadata_deposit_base`.
	pub const MetadataDepositBase: Balance = deposit(1, 38);
	pub const MetadataDepositPerByte: Balance = deposit(0, 1);
}

pub(crate) type TrustBackedAssetsInstance = pallet_assets::Instance1;
pub(crate) type TrustBackedAssetsCall = pallet_assets::Call<Runtime, TrustBackedAssetsInstance>;
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
	type WeightInfo = weights::pallet_assets::WeightInfo<Runtime>;
}

parameter_types! {
	// All features enabled.
	pub NftsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
	// Accounts for all the required elements to store a collection.
	// For details, refer to `ensure_collection_deposit`.
	pub const NftsCollectionDeposit: Balance = deposit(4, 294);
	// Accounts for the required elements to keep one item of a collection in state.
	// For details, refer to `ensure_item_deposit_deposit`.
	pub const NftsItemDeposit: Balance = deposit(1, 861) / 100;
	// Accounts for the base cost to include metadata for a collection or item.
	// For details, refer to `ensure_metadata_deposit_base`.
	pub const NftsMetadataDepositBase: Balance = deposit(1, 56) / 100;
	// Accounts for the base cost to include attributes to a collection or item.
	// For details, refer to `ensure_attribute_deposit_base`.
	pub const NftsAttributeDepositBase: Balance = deposit(1, 89) / 100;
	pub const NftsDepositPerByte: Balance = deposit(0, 1);
	pub const NftsMaxDeadlineDuration: BlockNumber = 12 * 30 * DAYS;
}

impl pallet_nfts::Config for Runtime {
	type ApprovalsLimit = ConstU32<20>;
	type AttributeDepositBase = NftsAttributeDepositBase;
	type BlockNumberProvider = System;
	type CollectionDeposit = NftsCollectionDeposit;
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
	type ItemId = ItemId;
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
	type WeightInfo = weights::pallet_nfts::WeightInfo<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use codec::MaxEncodedLen;
	use frame_support::{traits::StorageInfoTrait, Blake2_128Concat, StorageHasher};
	use sp_runtime::traits::Get;

	use super::*;
	use crate::{AccountId, Balance};

	mod assets {
		use frame_support::traits::Incrementable;
		use pallet_assets::{AssetsCallback, NextAssetId};
		use sp_keyring::Sr25519Keyring::Alice;

		use super::*;
		use crate::System;

		fn new_test_ext() -> sp_io::TestExternalities {
			let mut ext = sp_io::TestExternalities::new_empty();
			ext.execute_with(|| System::set_block_number(1));
			ext
		}

		#[test]
		fn ensure_asset_approval_deposit() {
			assert_eq!(ExistentialDeposit::get(), ApprovalDeposit::get());
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
			assert_eq!(Balance::max_encoded_len(), 16);
			assert_eq!(
				deposit(1, Balance::max_encoded_len() as u32) / 100,
				AssetAccountDeposit::get()
			);

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
			assert_eq!(max_size as u32, 210);
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
		fn callback_handle_is_auto_inc_asset_id() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::CallbackHandle,
				>(),
				TypeId::of::<pallet_assets::AutoIncAssetId<Runtime, TrustBackedAssetsInstance>>(),
			);
		}

		#[test]
		fn callback_increments_asset_id_on_asset_creation() {
			new_test_ext().execute_with(|| {
				NextAssetId::<Runtime, TrustBackedAssetsInstance>::put(1);
				let next_asset_id: u32 = NextAssetId::<Runtime, TrustBackedAssetsInstance>::get().unwrap();
				assert_eq!(next_asset_id, 1);
				assert!(<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::CallbackHandle::created(&next_asset_id, &Alice.to_account_id()).is_ok());
				assert_eq!(NextAssetId::<Runtime, TrustBackedAssetsInstance>::get().unwrap(), next_asset_id.increment().unwrap());
			})
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
		fn holder_is_default() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::Holder>(
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
			// Size doesn't include metadata name and symbol, these aren't part of the base cost,
			// rather the cost of those parameters will be calculated based on their length times
			// `NftsDepositPerByte`.
			// Everything else but these two fields is part of this deposit base.
			// src: https://github.com/paritytech/polkadot-sdk/blob/7a7e016a1da297adc13f855979232e0059df258a/substrate/frame/assets/src/types.rs#L188
			let max_size = Blake2_128Concat::max_len::<
				<Runtime as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetId,
			>() + Balance::max_encoded_len() +
				u8::max_encoded_len() +
				bool::max_encoded_len();
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
		use pallet_nfts::{AttributeNamespace, PalletFeature::*};
		use sp_runtime::{MultiSignature, MultiSigner};

		use super::*;

		#[test]
		fn item_approvals_limit_is_20() {
			assert_eq!(<<Runtime as pallet_nfts::Config>::ApprovalsLimit as Get<u32>>::get(), 20);
		}

		#[test]
		fn ensure_attribute_deposit_base() {
			// We only account for key length without the `BoundedVec<u8, T::KeyLimit>` element.
			// as per: https://github.com/paritytech/polkadot-sdk/blob/1866c3b4673b66a62b1eb9c8c82f2cd827cbd388/substrate/frame/nfts/src/lib.rs#L1414
			let key_size = Blake2_128Concat::max_len::<CollectionId>() +
				Blake2_128Concat::max_len::<ItemId>() +
				Blake2_128Concat::max_len::<AttributeNamespace<AccountId>>();
			assert_eq!(key_size, 89);
			assert_eq!(deposit(1, key_size as u32) / 100, NftsAttributeDepositBase::get());
		}

		#[test]
		fn ensure_system_is_block_number_provider() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::BlockNumberProvider>(),
				TypeId::of::<System>(),
			);
		}

		#[test]
		fn ensure_collection_deposit() {
			// We account for the different elements stored when creating a new collection:
			// src: https://github.com/paritytech/polkadot-sdk/blob/7aac8861752428e623b48741193d9a9d82e29cbf/substrate/frame/nfts/src/features/create_delete_collection.rs#L36

			let max_collection_size = pallet_nfts::Collection::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();

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
			assert_eq!(total_collection_size, 294);
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
		fn all_features_are_active() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_nfts::Config>::Features>(),
				TypeId::of::<NftsPalletFeatures>(),
			);

			assert!([Trading, Attributes, Approvals, Swaps]
				.iter()
				.all(|feat| <Runtime as pallet_nfts::Config>::Features::get().is_enabled(*feat)));
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
		fn ensure_item_deposit() {
			// Accounts for `Item` storage item max size.
			let max_size = pallet_nfts::Item::<Runtime>::storage_info()
				.first()
				.and_then(|info| info.max_size)
				.unwrap_or_default();
			assert_eq!(max_size, 861);
			assert_eq!(deposit(1, max_size) / 100, NftsItemDeposit::get());
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
		fn attribute_key_maximum_length_is_64() {
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
			// We use max of both key sizes and add size_of(Balance) which is always stored.
			let base_size = item_metadata_key_size.max(collection_metadata_key_size) +
				Balance::max_encoded_len();
			assert_eq!(base_size, 56);
			assert_eq!(NftsMetadataDepositBase::get(), deposit(1, base_size as u32) / 100);
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
