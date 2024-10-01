use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use enumflags2::{bitflags, BitFlags};
use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::{build::Fields, *};
use sp_runtime::BoundedBTreeMap;

use super::Config;

pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub(super) type NftsOf<T> = pallet_nfts::Pallet<T>;

/// Weight information for extrinsics in this pallet.
pub(super) type NftsWeightInfoOf<T> = <T as pallet_nfts::Config>::WeightInfo;

/// A type alias for the collection ID.
pub(super) type CollectionIdOf<T> =
	<pallet_nfts::Pallet<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;

/// A type alias for the collection item ID.
pub(super) type ItemIdOf<T> =
	<pallet_nfts::Pallet<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;

pub(super) type CollectionConfigFor<T> =
	CollectionConfig<DepositBalanceOf<T>, BlockNumberFor<T>, CollectionIdOf<T>>;

// TODO: Even though this serves the `allowance` method, it creates the maintenance cost.

/// A type that holds the deposit for a single item.
pub(super) type ItemDepositOf<T> =
	ItemDeposit<DepositBalanceOf<T>, <T as frame_system::Config>::AccountId>;

/// A type alias for handling balance deposits.
pub(super) type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

/// A type alias for keeping track of approvals used by a single item.
pub(super) type ApprovalsOf<T> = BoundedBTreeMap<
	AccountIdOf<T>,
	Option<BlockNumberFor<T>>,
	<T as pallet_nfts::Config>::ApprovalsLimit,
>;

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub(super) struct ItemDetails<T: Config> {
	/// The owner of this item.
	pub(super) owner: AccountIdOf<T>,
	/// The approved transferrer of this item, if one is set.
	pub(super) approvals: ApprovalsOf<T>,
	/// The amount held in the pallet's default account for this item. Free-hold items will have
	/// this as zero.
	pub(super) deposit: ItemDepositOf<T>,
}

/// Information about the reserved item deposit.
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct ItemDeposit<DepositBalance, AccountId> {
	/// A depositor account.
	pub(super) account: AccountId,
	/// An amount that gets reserved.
	pub(super) amount: DepositBalance,
}

/// Support for up to 64 user-enabled features on a collection.
#[bitflags]
#[repr(u64)]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum CollectionSetting {
	/// Items in this collection are transferable.
	TransferableItems,
	/// The metadata of this collection can be modified.
	UnlockedMetadata,
	/// Attributes of this collection can be modified.
	UnlockedAttributes,
	/// The supply of this collection can be modified.
	UnlockedMaxSupply,
	/// When this isn't set then the deposit is required to hold the items of this collection.
	DepositRequired,
}

macro_rules! impl_codec_bitflags {
	($wrapper:ty, $size:ty, $bitflag_enum:ty) => {
		impl MaxEncodedLen for $wrapper {
			fn max_encoded_len() -> usize {
				<$size>::max_encoded_len()
			}
		}
		impl Encode for $wrapper {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.0.bits().using_encoded(f)
			}
		}
		impl EncodeLike for $wrapper {}
		impl Decode for $wrapper {
			fn decode<I: codec::Input>(
				input: &mut I,
			) -> ::core::result::Result<Self, codec::Error> {
				let field = <$size>::decode(input)?;
				Ok(Self(BitFlags::from_bits(field as $size).map_err(|_| "invalid value")?))
			}
		}

		impl TypeInfo for $wrapper {
			type Identity = Self;

			fn type_info() -> Type {
				Type::builder()
					.path(Path::new("BitFlags", module_path!()))
					.type_params(vec![TypeParameter::new("T", Some(meta_type::<$bitflag_enum>()))])
					.composite(
						Fields::unnamed()
							.field(|f| f.ty::<$size>().type_name(stringify!($bitflag_enum))),
					)
			}
		}
	};
}

/// Wrapper type for `BitFlags<CollectionSetting>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CollectionSettings(pub BitFlags<CollectionSetting>);

impl CollectionSettings {
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}
	pub fn get_disabled(&self) -> BitFlags<CollectionSetting> {
		self.0
	}
	pub fn is_disabled(&self, setting: CollectionSetting) -> bool {
		self.0.contains(setting)
	}
	pub fn from_disabled(settings: BitFlags<CollectionSetting>) -> Self {
		Self(settings)
	}
}

impl_codec_bitflags!(CollectionSettings, u64, CollectionSetting);

/// Collection's configuration.
#[derive(Clone, Copy, Decode, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct CollectionConfig<Price, BlockNumber, CollectionId> {
	/// Collection's settings.
	pub settings: CollectionSettings,
	/// Collection's max supply.
	pub max_supply: Option<u32>,
	/// Default settings each item will get during the mint.
	pub mint_settings: MintSettings<Price, BlockNumber, CollectionId>,
}

impl<Price, BlockNumber, CollectionId> CollectionConfig<Price, BlockNumber, CollectionId> {
	pub fn is_setting_enabled(&self, setting: CollectionSetting) -> bool {
		!self.settings.is_disabled(setting)
	}
	pub fn has_disabled_setting(&self, setting: CollectionSetting) -> bool {
		self.settings.is_disabled(setting)
	}
	pub fn enable_setting(&mut self, setting: CollectionSetting) {
		self.settings.0.remove(setting);
	}
	pub fn disable_setting(&mut self, setting: CollectionSetting) {
		self.settings.0.insert(setting);
	}
}

/// Mint type. Can the NFT be create by anyone, or only the creator of the collection,
/// or only by wallets that already hold an NFT from a certain collection?
/// The ownership of a privately minted NFT is still publicly visible.
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub enum MintType<CollectionId> {
	/// Only an `Issuer` could mint items.
	Issuer,
	/// Anyone could mint items.
	Public,
	/// Only holders of items in specified collection could mint new items.
	HolderOf(CollectionId),
}

/// Holds the information about minting.
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct MintSettings<Price, BlockNumber, CollectionId> {
	/// Whether anyone can mint or if minters are restricted to some subset.
	pub mint_type: MintType<CollectionId>,
	/// An optional price per mint.
	pub price: Option<Price>,
	/// When the mint starts.
	pub start_block: Option<BlockNumber>,
	/// When the mint ends.
	pub end_block: Option<BlockNumber>,
	/// Default settings each item will get during the mint.
	pub default_item_settings: ItemSettings,
}

/// Support for up to 64 user-enabled features on an item.
#[bitflags]
#[repr(u64)]
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ItemSetting {
	/// This item is transferable.
	Transferable,
	/// The metadata of this item can be modified.
	UnlockedMetadata,
	/// Attributes of this item can be modified.
	UnlockedAttributes,
}

/// Wrapper type for `BitFlags<ItemSetting>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemSettings(pub BitFlags<ItemSetting>);

impl ItemSettings {
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}
	pub fn get_disabled(&self) -> BitFlags<ItemSetting> {
		self.0
	}
	pub fn is_disabled(&self, setting: ItemSetting) -> bool {
		self.0.contains(setting)
	}
	pub fn from_disabled(settings: BitFlags<ItemSetting>) -> Self {
		Self(settings)
	}
}

impl_codec_bitflags!(ItemSettings, u64, ItemSetting);
