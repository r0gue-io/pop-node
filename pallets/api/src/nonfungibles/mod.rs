//! The non-fungibles pallet offers a streamlined interface for interacting with non-fungible
//! assets. The goal is to provide a simplified, consistent API that adheres to standards in the
//! smart contract space.

use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use pallet_nfts::WeightInfo as NftsWeightInfoTrait;
pub use pallet_nfts::{
	AttributeNamespace, CancelAttributesApprovalWitness, CollectionConfig, CollectionDetails,
	CollectionSetting, CollectionSettings, DestroyWitness, ItemDeposit, ItemDetails, ItemMetadata,
	ItemSetting, MintSettings, MintType, MintWitness,
};
use sp_runtime::traits::StaticLookup;
use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
pub mod weights;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type NftsOf<T> = pallet_nfts::Pallet<T, NftsInstanceOf<T>>;
type NftsErrorOf<T> = pallet_nfts::Error<T, NftsInstanceOf<T>>;
type NftsWeightInfoOf<T> = <T as pallet_nfts::Config<NftsInstanceOf<T>>>::WeightInfo;
type NftsInstanceOf<T> = <T as Config>::NftsInstance;
type BalanceOf<T> = <<T as pallet_nfts::Config<NftsInstanceOf<T>>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
type ItemIdOf<T> = <NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
type ItemPriceOf<T> = BalanceOf<T>;
type CollectionDetailsFor<T> = CollectionDetails<AccountIdOf<T>, BalanceOf<T>>;
type AttributeNamespaceOf<T> = AttributeNamespace<AccountIdOf<T>>;
type CollectionConfigFor<T> =
	CollectionConfig<ItemPriceOf<T>, BlockNumberFor<T>, CollectionIdOf<T>>;
// Type aliases for pallet-nfts storage items.
pub(super) type AccountBalanceOf<T> = pallet_nfts::AccountBalance<T, NftsInstanceOf<T>>;
pub(super) type AttributeOf<T> = pallet_nfts::Attribute<T, NftsInstanceOf<T>>;
pub(super) type NextCollectionIdOf<T> = pallet_nfts::NextCollectionId<T, NftsInstanceOf<T>>;
pub(super) type CollectionOf<T> = pallet_nfts::Collection<T, NftsInstanceOf<T>>;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
		pallet_prelude::*,
		traits::Incrementable,
	};
	use frame_system::pallet_prelude::*;
	use pallet_nfts::{CancelAttributesApprovalWitness, DestroyWitness, MintWitness};
	use sp_runtime::BoundedVec;
	use sp_std::vec::Vec;

	use super::*;

	/// State reads for the non-fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Total item supply of a specified collection.
		#[codec(index = 0)]
		TotalSupply(CollectionIdOf<T>),
		/// Account balance for a specified collection.
		#[codec(index = 1)]
		BalanceOf {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The owner of the collection .
			owner: AccountIdOf<T>,
		},
		/// Allowance for an operator approved by an owner, for a specified collection or item.
		#[codec(index = 2)]
		Allowance {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The collection item.
			item: Option<ItemIdOf<T>>,
			/// The owner of the collection item.
			owner: AccountIdOf<T>,
			/// The delegated operator of collection item.
			operator: AccountIdOf<T>,
		},
		/// Owner of a specified collection item.
		#[codec(index = 5)]
		OwnerOf {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The collection item.
			item: ItemIdOf<T>,
		},
		/// Attribute value of a specified collection item. (Error: bounded collection is not
		/// partial)
		#[codec(index = 6)]
		GetAttribute {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The collection item.
			item: ItemIdOf<T>,
			/// The namespace of the attribute.
			namespace: AttributeNamespaceOf<T>,
			/// The key of the attribute.
			key: BoundedVec<u8, T::KeyLimit>,
		},
		/// Details of a specified collection.
		#[codec(index = 9)]
		Collection(CollectionIdOf<T>),
		/// Next collection ID.
		#[codec(index = 10)]
		NextCollectionId,
		/// Metadata of a specified collection item.
		#[codec(index = 11)]
		ItemMetadata {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The collection item.
			item: ItemIdOf<T>,
		},
	}

	/// Results of state reads for the non-fungibles API.
	#[derive(Debug)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	pub enum ReadResult<T: Config> {
		/// Total item supply of a collection.
		TotalSupply(u128),
		/// Account balance for a specified collection.
		BalanceOf(u32),
		/// Allowance for an operator approved by an owner, for a specified collection or item.
		Allowance(bool),
		/// Owner of a specified collection owner.
		OwnerOf(Option<AccountIdOf<T>>),
		/// Attribute value of a specified collection item.
		GetAttribute(Option<Vec<u8>>),
		/// Details of a specified collection.
		Collection(Option<CollectionDetailsFor<T>>),
		/// Next collection ID.
		NextCollectionId(Option<CollectionIdOf<T>>),
		/// Metadata of a specified collection item.
		ItemMetadata(Option<Vec<u8>>),
	}

	impl<T: Config> ReadResult<T> {
		/// Encodes the result.
		pub fn encode(&self) -> Vec<u8> {
			use ReadResult::*;
			match self {
				OwnerOf(result) => result.encode(),
				TotalSupply(result) => result.encode(),
				BalanceOf(result) => result.encode(),
				Collection(result) => result.encode(),
				Allowance(result) => result.encode(),
				GetAttribute(result) => result.encode(),
				NextCollectionId(result) => result.encode(),
				ItemMetadata(result) => result.encode(),
			}
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config<Self::NftsInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet-nfts.
		type NftsInstance;
		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when allowance by `owner` to `operator` changes.
		Approval {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// The item which is (dis)approved. `None` for all owner's items.
			item: Option<ItemIdOf<T>>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			operator: AccountIdOf<T>,
			/// Whether allowance is set or removed.
			approved: bool,
		},
		/// Event emitted when a token transfer occurs.
		// Differing style: event name abides by the PSP22 standard.
		Transfer {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// The collection item ID.
			item: ItemIdOf<T>,
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The price of the collection item.
			price: Option<BalanceOf<T>>,
		},
		/// Event emitted when a collection is created.
		Created {
			/// The collection identifier.
			id: CollectionIdOf<T>,
			/// The creator of the collection.
			creator: AccountIdOf<T>,
			/// The administrator of the collection.
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers the collection item from the caller's account to account `to`.
		///
		/// # Parameters
		/// - `collection` - The collection of the item to transfer.
		/// - `item` - The item to transfer.
		/// - `to` - The recipient account.
		#[pallet::call_index(3)]
		#[pallet::weight(NftsWeightInfoOf::<T>::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			to: AccountIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			NftsOf::<T>::transfer(origin, collection, item, T::Lookup::unlookup(to.clone()))?;
			Self::deposit_event(Event::Transfer {
				collection,
				item,
				from: Some(from),
				to: Some(to),
				price: None,
			});
			Ok(())
		}

		/// Approves `operator` to spend the collection item on behalf of the caller.
		///
		/// # Parameters
		/// - `collection` - The collection of the item to approve for a delegated transfer.
		/// - `item` - The item to approve for a delegated transfer.
		/// - `operator` - The account that is allowed to spend the collection item.
		/// - `approved` - The approval status of the collection item.
		#[pallet::call_index(4)]
		#[pallet::weight(
            NftsWeightInfoOf::<T>::approve_transfer(item.is_some() as u32) +
    		NftsWeightInfoOf::<T>::cancel_approval(item.is_some() as u32)
        )]
		pub fn approve(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: Option<ItemIdOf<T>>,
			operator: AccountIdOf<T>,
			approved: bool,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())?;
			let weight = if approved {
				NftsOf::<T>::approve_transfer(
					origin,
					collection,
					item,
					T::Lookup::unlookup(operator.clone()),
					None,
				)
				.map_err(|e| {
					e.with_weight(NftsWeightInfoOf::<T>::approve_transfer(item.is_some() as u32))
				})?;
				NftsWeightInfoOf::<T>::approve_transfer(item.is_some() as u32)
			} else {
				NftsOf::<T>::cancel_approval(
					origin,
					collection,
					item,
					T::Lookup::unlookup(operator.clone()),
				)
				.map_err(|e| {
					e.with_weight(NftsWeightInfoOf::<T>::cancel_approval(item.is_some() as u32))
				})?;
				NftsWeightInfoOf::<T>::cancel_approval(item.is_some() as u32)
			};
			Self::deposit_event(Event::Approval { collection, item, operator, owner, approved });
			Ok(Some(weight).into())
		}

		/// Issue a new collection of non-fungible items from a public origin.
		///
		/// # Parameters
		/// - `admin` - The admin of this collection. The admin is the initial address of each
		/// member of the collection's admin team.
		/// - `config` - The configuration of the collection.
		#[pallet::call_index(7)]
		#[pallet::weight(NftsWeightInfoOf::<T>::create())]
		pub fn create(
			origin: OriginFor<T>,
			admin: AccountIdOf<T>,
			config: CollectionConfigFor<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin.clone())?;
			// TODO: re-evaluate next collection id in nfts pallet. The `Incrementable` trait causes
			//  issues for setting it to xcm's `Location`. This can easily be done differently.
			let id = NextCollectionIdOf::<T>::get()
				.or(T::CollectionId::initial_value())
				.ok_or(NftsErrorOf::<T>::UnknownCollection)?;
			NftsOf::<T>::create(origin, T::Lookup::unlookup(admin.clone()), config)?;
			Self::deposit_event(Event::Created { id, creator, admin });
			Ok(())
		}

		/// Destroy a collection of fungible items.
		///
		/// # Parameters
		/// - `collection` - The collection to destroy.
		/// - `witness` - Information on the items minted in the collection. This must be
		/// correct.
		#[pallet::call_index(8)]
		#[pallet::weight(NftsWeightInfoOf::<T>::destroy(
    		witness.item_metadatas,
    		witness.item_configs,
    		witness.attributes,
		))]
		pub fn destroy(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			witness: DestroyWitness,
		) -> DispatchResultWithPostInfo {
			NftsOf::<T>::destroy(origin, collection, witness)
		}

		/// Set an attribute for a collection or item.
		///
		/// # Parameters
		/// - `collection` - The collection whose item's metadata to set.
		/// - `maybe_item` - The item whose metadata to set.
		/// - `namespace` - Attribute's namespace.
		/// - `key` - The key of the attribute.
		/// - `value` - The value to which to set the attribute.
		#[pallet::call_index(12)]
		#[pallet::weight(NftsWeightInfoOf::<T>::set_attribute())]
		pub fn set_attribute(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: Option<ItemIdOf<T>>,
			namespace: AttributeNamespaceOf<T>,
			key: BoundedVec<u8, T::KeyLimit>,
			value: BoundedVec<u8, T::ValueLimit>,
		) -> DispatchResult {
			NftsOf::<T>::set_attribute(origin, collection, item, namespace, key, value)
		}

		/// Clear an attribute for the collection or item.
		///
		/// # Parameters
		/// - `collection` - The collection whose item's metadata to clear.
		/// - `maybe_item` - The item whose metadata to clear.
		/// - `namespace` - Attribute's namespace.
		/// - `key` - The key of the attribute.
		#[pallet::call_index(13)]
		#[pallet::weight(NftsWeightInfoOf::<T>::clear_attribute())]
		pub fn clear_attribute(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: Option<ItemIdOf<T>>,
			namespace: AttributeNamespaceOf<T>,
			key: BoundedVec<u8, T::KeyLimit>,
		) -> DispatchResult {
			NftsOf::<T>::clear_attribute(origin, collection, item, namespace, key)
		}

		/// Set the metadata for an item.
		///
		/// # Parameters
		/// - `collection` - The collection whose item's metadata to set.
		/// - `item` - The item whose metadata to set.
		/// - `data` - The general information of this item. Limited in length by `StringLimit`.
		#[pallet::call_index(14)]
		#[pallet::weight(NftsWeightInfoOf::<T>::set_metadata())]
		pub fn set_metadata(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			NftsOf::<T>::set_metadata(origin, collection, item, data)
		}

		#[pallet::call_index(15)]
		#[pallet::weight(NftsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::clear_metadata(origin, collection, item)
		}

		/// Approve item's attributes to be changed by a delegated third-party account.
		///
		/// # Parameters
		/// - `collection` - The collection of the item.
		/// - `item` - The item that holds attributes.
		/// - `delegate` - The account to delegate permission to change attributes of the item.
		#[pallet::call_index(16)]
		#[pallet::weight(NftsWeightInfoOf::<T>::approve_item_attributes())]
		pub fn approve_item_attributes(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			delegate: AccountIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::approve_item_attributes(
				origin,
				collection,
				item,
				T::Lookup::unlookup(delegate.clone()),
			)
		}

		/// Cancel the previously provided approval to change item's attributes.
		/// All the previously set attributes by the `delegate` will be removed.
		///
		/// # Parameters
		/// - `collection` - The collection that the item is contained within.
		/// - `item` - The item that holds attributes.
		/// - `delegate` - The previously approved account to remove.
		/// - `witness` - A witness data to cancel attributes approval operation.
		#[pallet::call_index(17)]
		#[pallet::weight(NftsWeightInfoOf::<T>::cancel_item_attributes_approval(witness.account_attributes))]
		pub fn cancel_item_attributes_approval(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			delegate: AccountIdOf<T>,
			witness: CancelAttributesApprovalWitness,
		) -> DispatchResult {
			NftsOf::<T>::cancel_item_attributes_approval(
				origin,
				collection,
				item,
				T::Lookup::unlookup(delegate.clone()),
				witness,
			)
		}

		/// Set the maximum number of items a collection could have.
		///
		/// # Parameters
		/// - `collection` - The identifier of the collection to change.
		/// - `max_supply` - The maximum number of items a collection could have.
		#[pallet::call_index(18)]
		#[pallet::weight(NftsWeightInfoOf::<T>::set_collection_max_supply())]
		pub fn set_max_supply(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			max_supply: u32,
		) -> DispatchResult {
			NftsOf::<T>::set_collection_max_supply(origin, collection, max_supply)
		}

		/// Mint an item of a particular collection.
		///
		/// # Parameters
		/// - `to` - Account into which the item will be minted.
		/// - `collection` - The collection of the item to mint.
		/// - `item` - An identifier of the new item.
		/// - `witness_data` - When the mint type is `HolderOf(collection_id)`, then the owned
		///   item_id from that collection needs to be provided within the witness data object. If
		///   the mint price is set, then it should be additionally confirmed in the `witness_data`.
		#[pallet::call_index(19)]
		#[pallet::weight(NftsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			to: AccountIdOf<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			witness: MintWitness<ItemIdOf<T>, ItemPriceOf<T>>,
		) -> DispatchResult {
			let account = ensure_signed(origin.clone())?;
			let mint_price = witness.mint_price;
			NftsOf::<T>::mint(
				origin,
				collection,
				item,
				T::Lookup::unlookup(to.clone()),
				Some(witness),
			)?;
			Self::deposit_event(Event::Transfer {
				collection,
				item,
				from: None,
				to: Some(account),
				price: mint_price,
			});
			Ok(())
		}

		/// Destroy a single collection item.
		///
		/// # Parameters
		/// - `collection` - The collection of the item to burn.
		/// - `item` - The item to burn.
		#[pallet::call_index(20)]
		#[pallet::weight(NftsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			let account = ensure_signed(origin.clone())?;
			NftsOf::<T>::burn(origin, collection, item)?;
			Self::deposit_event(Event::Transfer {
				collection,
				item,
				from: Some(account),
				to: None,
				price: None,
			});
			Ok(())
		}
	}

	impl<T: Config> crate::Read for Pallet<T> {
		/// The type of read requested.
		type Read = Read<T>;
		/// The type or result returned.
		type Result = ReadResult<T>;

		/// Determines the weight of the requested read, used to charge the appropriate weight
		/// before the read is performed.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn weight(request: &Self::Read) -> Weight {
			use Read::*;
			match request {
				TotalSupply(_) => <T as Config>::WeightInfo::total_supply(),
				BalanceOf { .. } => <T as Config>::WeightInfo::balance_of(),
				Allowance { .. } => <T as Config>::WeightInfo::allowance(),
				OwnerOf { .. } => <T as Config>::WeightInfo::owner_of(),
				GetAttribute { .. } => <T as Config>::WeightInfo::get_attribute(),
				Collection(_) => <T as Config>::WeightInfo::collection(),
				ItemMetadata { .. } => <T as Config>::WeightInfo::item_metadata(),
				NextCollectionId => <T as Config>::WeightInfo::next_collection_id(),
			}
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(value: Self::Read) -> Self::Result {
			use Read::*;
			match value {
				TotalSupply(collection) => ReadResult::TotalSupply(
					NftsOf::<T>::collection_items(collection).unwrap_or_default() as u128,
				),
				BalanceOf { collection, owner } =>
					ReadResult::BalanceOf(AccountBalanceOf::<T>::get(owner, collection)),
				Allowance { collection, owner, operator, item } => ReadResult::Allowance(
					NftsOf::<T>::check_approval(&collection, &item, &owner, &operator).is_ok(),
				),
				OwnerOf { collection, item } =>
					ReadResult::OwnerOf(NftsOf::<T>::owner(collection, item)),
				GetAttribute { collection, item, namespace, key } => ReadResult::GetAttribute(
					AttributeOf::<T>::get((collection, Some(item), namespace, key))
						.map(|attribute| attribute.0.into()),
				),
				Collection(collection) =>
					ReadResult::Collection(CollectionOf::<T>::get(collection)),
				ItemMetadata { collection, item } => ReadResult::ItemMetadata(
					NftsOf::<T>::item_metadata(collection, item).map(|metadata| metadata.into()),
				),
				NextCollectionId => ReadResult::NextCollectionId(
					NextCollectionIdOf::<T>::get().or(T::CollectionId::initial_value()),
				),
			}
		}
	}
}
