//! The non-fungibles pallet offers a streamlined interface for interacting with non-fungible
//! tokens. The goal is to provide a simplified, consistent API that adheres to standards in the
//! smart contract space.

extern crate alloc;

use frame_support::{
	dispatch::WithPostDispatchInfo,
	traits::{nonfungibles_v2::Inspect, Currency},
	BoundedVec,
};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet::*;
use pallet_nfts::WeightInfo as NftsWeightInfoTrait;
pub use pallet_nfts::{
	AttributeNamespace, CancelAttributesApprovalWitness, CollectionConfig, CollectionDetails,
	CollectionSetting, CollectionSettings, DestroyWitness, ItemConfig, ItemDeposit, ItemDetails,
	ItemMetadata, ItemSetting, ItemSettings, MintSettings, MintType, MintWitness,
};
use sp_runtime::traits::StaticLookup;
pub(crate) use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
/// Weights for non-fungibles dispatchables.
pub mod weights;

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AttributeNamespaceOf<T> = AttributeNamespace<AccountIdOf<T>>;
type BalanceOf<T> = <<T as pallet_nfts::Config<NftsInstanceOf<T>>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
type CollectionConfigFor<T> =
	CollectionConfig<ItemPriceOf<T>, BlockNumberFor<T>, CollectionIdOf<T>>;
type CollectionDetailsOf<T> = CollectionDetails<AccountIdOf<T>, BalanceOf<T>>;
type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
type ItemIdOf<T> = <NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
type ItemPriceOf<T> = BalanceOf<T>;
type NftsErrorOf<T> = pallet_nfts::Error<T, NftsInstanceOf<T>>;
type NftsInstanceOf<T> = <T as Config>::NftsInstance;
type NftsOf<T> = pallet_nfts::Pallet<T, NftsInstanceOf<T>>;
type NftsWeightInfoOf<T> = <T as pallet_nfts::Config<NftsInstanceOf<T>>>::WeightInfo;
type WeightOf<T> = <T as Config>::WeightInfo;
// Public due to pop-api integration tests crate.
pub type AccountBalanceOf<T> = pallet_nfts::AccountBalance<T, NftsInstanceOf<T>>;
pub type AttributeOf<T> = pallet_nfts::Attribute<T, NftsInstanceOf<T>>;
pub type AttributeKey<T> = BoundedVec<u8, <T as pallet_nfts::Config<NftsInstanceOf<T>>>::KeyLimit>;
pub type AttributeValue<T> =
	BoundedVec<u8, <T as pallet_nfts::Config<NftsInstanceOf<T>>>::ValueLimit>;
pub type CollectionOf<T> = pallet_nfts::Collection<T, NftsInstanceOf<T>>;
pub type CollectionConfigOf<T> = pallet_nfts::CollectionConfigOf<T, NftsInstanceOf<T>>;
pub type NextCollectionIdOf<T> = pallet_nfts::NextCollectionId<T, NftsInstanceOf<T>>;
pub type MetadataData<T> =
	BoundedVec<u8, <T as pallet_nfts::Config<NftsInstanceOf<T>>>::StringLimit>;

#[frame_support::pallet]
pub mod pallet {
	use alloc::vec::Vec;

	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		traits::Incrementable,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::BoundedVec;

	use super::*;

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
		/// Event emitted when a token transfer occurs.
		// Differing style: event name abides by the PSP34 standard.
		Transfer {
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The item transferred (or minted/burned).
			item: ItemIdOf<T>,
		},
		/// Event emitted when a token approve occurs.
		// Differing style: event name abides by the PSP34 standard.
		Approval {
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			operator: AccountIdOf<T>,
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The item which is (dis)approved. `None` for all owner's items.
			item: Option<ItemIdOf<T>>,
			/// Whether allowance is set or removed.
			approved: bool,
		},
		/// Event emitted when an attribute is set for a token.
		// Differing style: event name abides by the PSP34 standard.
		AttributeSet {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The item whose attribute is set.
			item: Option<ItemIdOf<T>>,
			/// The key for the attribute.
			key: Vec<u8>,
			/// The data for the attribute.
			data: Vec<u8>,
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

	/// The non-fungibles dispatchables. For more information about a dispatchable refer to
	/// `pallet-nfts`.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Approves operator to transfer item(s) from the owner's account.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `operator` - The account that is allowed to transfer the item.
		/// - `item` - Optional item. `None` means all items owned in the specified collection.
		/// - `approved` - Whether the operator is given or removed the right to transfer the
		///   item(s).
		/// - `deadline`: The optional deadline (in block numbers) specifying the time limit for the
		///   approval, only required if `approved` is true.
		#[pallet::call_index(3)]
		// TODO: Resolve weight with a proper approach (#463).
		#[pallet::weight(
			NftsWeightInfoOf::<T>::approve_transfer()
			.max(NftsWeightInfoOf::<T>::approve_collection_transfer())
			.max(NftsWeightInfoOf::<T>::cancel_approval())
			.max(NftsWeightInfoOf::<T>::cancel_collection_approval()))
		]
		pub fn approve(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			operator: AccountIdOf<T>,
			item: Option<ItemIdOf<T>>,
			approved: bool,
			deadline: Option<BlockNumberFor<T>>,
		) -> DispatchResultWithPostInfo {
			let operator_lookup = T::Lookup::unlookup(operator.clone());
			let result = if approved {
				Self::do_approve(origin.clone(), collection, item, operator_lookup, deadline)?
			} else {
				Self::do_cancel_approval(origin.clone(), collection, item, operator_lookup)?
			};
			let owner = ensure_signed(origin)?;
			Self::deposit_event(Event::Approval { collection, item, operator, owner, approved });
			Ok(result)
		}

		/// Transfers an owned or approved item to the specified recipient.
		///
		/// Origin must be either the item's owner or an account approved by the owner to
		/// transfer the item.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `to` - The recipient account.
		/// - `item` - The item.
		#[pallet::call_index(4)]
		#[pallet::weight(NftsWeightInfoOf::<T>::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			to: AccountIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			let owner =
				NftsOf::<T>::owner(collection, item).ok_or(NftsErrorOf::<T>::UnknownItem)?;
			NftsOf::<T>::transfer(origin, collection, item, T::Lookup::unlookup(to.clone()))?;
			Self::deposit_event(Event::Transfer {
				collection,
				item,
				from: Some(owner),
				to: Some(to),
			});
			Ok(())
		}

		/// Creates an NFT collection.
		///
		/// # Parameters
		/// - `admin` - The admin account of the collection.
		/// - `config` - Settings and config to be set for the new collection.
		#[pallet::call_index(9)]
		#[pallet::weight(NftsWeightInfoOf::<T>::create())]
		pub fn create(
			origin: OriginFor<T>,
			admin: AccountIdOf<T>,
			config: CollectionConfigFor<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin.clone())?;
			let id = NextCollectionIdOf::<T>::get()
				.or(T::CollectionId::initial_value())
				.ok_or(NftsErrorOf::<T>::UnknownCollection)?;
			NftsOf::<T>::create(origin, T::Lookup::unlookup(admin.clone()), config)?;
			Self::deposit_event(Event::Created { id, creator, admin });
			Ok(())
		}

		/// Destroy an NFT collection.
		///
		/// # Parameters
		/// - `collection` - The collection to be destroyed.
		/// - `witness` - Information on the items minted in the `collection`. This must be
		/// correct.
		#[pallet::call_index(10)]
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
		/// Origin must be Signed and must conform to the namespace ruleset:
		/// - `CollectionOwner` namespace could be modified by the `collection` Admin only;
		/// - `ItemOwner` namespace could be modified by the `item` owner only. `item` should be set
		///   in that case;
		/// - `Account(AccountId)` namespace could be modified only when the `origin` was given a
		///   permission to do so;
		///
		/// The funds of `origin` are reserved according to the formula:
		/// `AttributeDepositBase + DepositPerByte * (key.len + value.len)` taking into
		/// account any already reserved funds.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `item` - The optional item whose attribute to set. If `None`, the `collection`'s
		///   attribute is set.
		/// - `namespace` - The attribute's namespace.
		/// - `key` - The key of the attribute.
		/// - `value` - The value to which to set the attribute.
		#[pallet::call_index(11)]
		#[pallet::weight(NftsWeightInfoOf::<T>::set_attribute())]
		pub fn set_attribute(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: Option<ItemIdOf<T>>,
			namespace: AttributeNamespaceOf<T>,
			key: BoundedVec<u8, T::KeyLimit>,
			value: BoundedVec<u8, T::ValueLimit>,
		) -> DispatchResult {
			let key_vec = key.to_vec();
			let value_vec = value.to_vec();
			NftsOf::<T>::set_attribute(origin, collection, item, namespace, key, value)?;
			Self::deposit_event(Event::AttributeSet {
				collection,
				item,
				key: key_vec,
				data: value_vec,
			});
			Ok(())
		}

		/// Clear an attribute for the collection or item.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `item` - The optional item whose metadata to clear. If `None`, metadata of the
		///   `collection` will be cleared.
		/// - `namespace` - The attribute's namespace.
		/// - `key` - The key of the attribute.
		#[pallet::call_index(12)]
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
		/// Caller must be the admin of the collection.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `item` - The item. If `None`, set metadata for the collection.
		/// - `data` - The metadata. Limited in length by `StringLimit`.
		#[pallet::call_index(13)]
		#[pallet::weight(NftsWeightInfoOf::<T>::set_metadata())]
		pub fn set_metadata(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			data: BoundedVec<u8, T::StringLimit>,
		) -> DispatchResult {
			NftsOf::<T>::set_metadata(origin, collection, item, data)
		}

		/// Clear the metadata for an item or collection.
		///
		/// Caller must be the admin of the collection.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `item` - The item.
		#[pallet::call_index(14)]
		#[pallet::weight(NftsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::clear_metadata(origin, collection, item)
		}

		/// Set the maximum number of items a collection could have.
		///
		/// Caller must be the owner of the collection.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `max_supply` - The collection's max supply.
		#[pallet::call_index(15)]
		#[pallet::weight(NftsWeightInfoOf::<T>::set_collection_max_supply())]
		pub fn set_max_supply(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			max_supply: u32,
		) -> DispatchResult {
			NftsOf::<T>::set_collection_max_supply(origin, collection, max_supply)
		}

		/// Approve item's attributes to be changed by a delegated third-party account.
		///
		/// Caller must be the owner of the item.
		///
		/// # Parameters
		/// - `collection` - The collection.
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
				T::Lookup::unlookup(delegate),
			)
		}

		/// Cancel the previously provided approval to change item's attributes.
		/// All the previously set attributes by the `delegate` will be removed.
		///
		/// # Parameters
		/// - `collection` - The collection.
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
				T::Lookup::unlookup(delegate),
				witness,
			)
		}

		/// Cancel all the approvals of a specific item.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `item` - The item of the collection of whose approvals will be cleared.
		#[pallet::call_index(18)]
		#[pallet::weight(NftsWeightInfoOf::<T>::clear_all_transfer_approvals())]
		pub fn clear_all_transfer_approvals(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::clear_all_transfer_approvals(origin, collection, item)
		}

		/// Cancel approvals to transfer all owner's collection items.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `limit` - The amount of collection approvals that will be cleared.
		#[pallet::call_index(19)]
		#[pallet::weight(NftsWeightInfoOf::<T>::clear_collection_approvals(*limit))]
		pub fn clear_collection_approvals(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			NftsOf::<T>::clear_collection_approvals(origin, collection, limit)
		}

		/// Mints an item to the specified recipient account.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `to` - The recipient account.
		/// - `item` - The identifier for the new item.
		/// - `witness` - When the mint type is `HolderOf(collection_id)`, then the owned item_id
		///   from that collection needs to be provided within the witness data object. If the mint
		///   price is set, then it should be additionally confirmed in the `witness`.
		///
		/// Note: The deposit will be taken from the `origin` and not the `owner` of the `item`.
		#[pallet::call_index(20)]
		#[pallet::weight(NftsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			to: AccountIdOf<T>,
			item: ItemIdOf<T>,
			witness: Option<MintWitness<ItemIdOf<T>, ItemPriceOf<T>>>,
		) -> DispatchResult {
			NftsOf::<T>::mint(origin, collection, item, T::Lookup::unlookup(to.clone()), witness)?;
			Self::deposit_event(Event::Transfer { collection, item, from: None, to: Some(to) });
			Ok(())
		}

		/// Destroys the specified item. Clearing the corresponding approvals.
		///
		/// # Parameters
		/// - `collection` - The collection.
		/// - `item` - The item to burn.
		#[pallet::call_index(21)]
		#[pallet::weight(NftsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			let owner =
				NftsOf::<T>::owner(collection, item).ok_or(NftsErrorOf::<T>::UnknownItem)?;
			NftsOf::<T>::burn(origin, collection, item)?;
			Self::deposit_event(Event::Transfer { collection, item, from: Some(owner), to: None });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Approves the transfer of a specific item or all collection items owned by the `owner` to
		// an `operator`.
		fn do_approve(
			owner: OriginFor<T>,
			collection: CollectionIdOf<T>,
			maybe_item: Option<ItemIdOf<T>>,
			operator: AccountIdLookupOf<T>,
			deadline: Option<BlockNumberFor<T>>,
		) -> DispatchResultWithPostInfo {
			Ok(Some(match maybe_item {
				Some(item) => {
					NftsOf::<T>::approve_transfer(owner, collection, item, operator, deadline)
						.map_err(|e| e.with_weight(NftsWeightInfoOf::<T>::approve_transfer()))?;
					NftsWeightInfoOf::<T>::approve_transfer()
				},
				None => {
					NftsOf::<T>::approve_collection_transfer(owner, collection, operator, deadline)
						.map_err(|e| {
							e.with_weight(NftsWeightInfoOf::<T>::approve_collection_transfer())
						})?;
					NftsWeightInfoOf::<T>::approve_collection_transfer()
				},
			})
			.into())
		}

		// Cancel an approval to transfer a specific item or all items within a collection owned by
		// the `owner`.
		fn do_cancel_approval(
			owner: OriginFor<T>,
			collection: CollectionIdOf<T>,
			maybe_item: Option<ItemIdOf<T>>,
			operator: AccountIdLookupOf<T>,
		) -> DispatchResultWithPostInfo {
			Ok(Some(match maybe_item {
				Some(item) => {
					NftsOf::<T>::cancel_approval(owner, collection, item, operator)
						.map_err(|e| e.with_weight(NftsWeightInfoOf::<T>::cancel_approval()))?;
					NftsWeightInfoOf::<T>::cancel_approval()
				},
				None => {
					NftsOf::<T>::cancel_collection_approval(owner, collection, operator).map_err(
						|e| e.with_weight(NftsWeightInfoOf::<T>::cancel_collection_approval()),
					)?;
					NftsWeightInfoOf::<T>::cancel_collection_approval()
				},
			})
			.into())
		}
	}

	/// State reads for the non-fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Returns the amount of items the owner has within a `collection`.
		#[codec(index = 0)]
		BalanceOf {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The account whose balance is being queried.
			owner: AccountIdOf<T>,
		},
		/// Returns the owner of an item within a specified collection, if any.
		#[codec(index = 1)]
		OwnerOf {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The item.
			item: ItemIdOf<T>,
		},
		/// Returns whether the `operator` is approved by the `owner` to withdraw `item`. If `item`
		/// is `None`, it returns whether the `operator` is approved to withdraw all `owner`'s
		/// items for the given `collection`.
		#[codec(index = 2)]
		Allowance {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The account that is allowed to transfer the collection item(s).
			operator: AccountIdOf<T>,
			/// The item. If `None`, it is regarding all owner's collection items.
			item: Option<ItemIdOf<T>>,
		},
		/// Returns the total supply of a collection.
		#[codec(index = 5)]
		TotalSupply(CollectionIdOf<T>),
		/// Returns the attribute value of `item` for a given `key`, if any.
		#[codec(index = 6)]
		GetAttribute {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The item. If `None` the attributes for the collection are queried.
			item: Option<ItemIdOf<T>>,
			/// The attribute's namespace.
			namespace: AttributeNamespaceOf<T>,
			/// The key of the attribute.
			key: BoundedVec<u8, T::KeyLimit>,
		},
		/// Returns the next collection identifier, if any.
		#[codec(index = 7)]
		NextCollectionId,
		/// Returns the metadata of a specified collection item, if any.
		#[codec(index = 8)]
		ItemMetadata {
			/// The collection.
			collection: CollectionIdOf<T>,
			/// The item.
			item: ItemIdOf<T>,
		},
	}

	/// Results of state reads for the non-fungibles API.
	#[derive(Debug)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	pub enum ReadResult<T: Config> {
		/// Returns the amount of items the owner has within a collection.
		BalanceOf(u32),
		/// Returns the owner of an item within a specified collection, if any.
		OwnerOf(Option<AccountIdOf<T>>),
		/// Returns whether the operator is approved by the owner to withdraw item. If item is not
		/// provided, it returns whether the operator is approved to withdraw all owner's items for
		/// the given collection.
		Allowance(bool),
		/// Returns the total supply of a collection.
		TotalSupply(u128),
		/// Returns the attribute value of item for a given key, if any.
		GetAttribute(Option<Vec<u8>>),
		/// Returns the next collection identifier, if any.
		NextCollectionId(Option<CollectionIdOf<T>>),
		/// Returns the metadata of a specified collection item, if any.
		ItemMetadata(Option<Vec<u8>>),
	}

	impl<T: Config> ReadResult<T> {
		/// Encodes the result.
		pub fn encode(&self) -> Vec<u8> {
			use ReadResult::*;
			match self {
				BalanceOf(result) => result.encode(),
				OwnerOf(result) => result.encode(),
				Allowance(result) => result.encode(),
				TotalSupply(result) => result.encode(),
				GetAttribute(result) => result.encode(),
				NextCollectionId(result) => result.encode(),
				ItemMetadata(result) => result.encode(),
			}
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
				BalanceOf { .. } => WeightOf::<T>::balance_of(),
				OwnerOf { .. } => WeightOf::<T>::owner_of(),
				Allowance { .. } => WeightOf::<T>::allowance(),
				TotalSupply(_) => WeightOf::<T>::total_supply(),
				GetAttribute { .. } => WeightOf::<T>::get_attribute(),
				NextCollectionId => WeightOf::<T>::next_collection_id(),
				ItemMetadata { .. } => WeightOf::<T>::item_metadata(),
			}
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(request: Self::Read) -> Self::Result {
			use Read::*;
			match request {
				BalanceOf { collection, owner } => ReadResult::BalanceOf(
					AccountBalanceOf::<T>::get(collection, owner)
						.map(|(balance, _)| balance)
						.unwrap_or_default(),
				),
				OwnerOf { collection, item } =>
					ReadResult::OwnerOf(NftsOf::<T>::owner(collection, item)),
				Allowance { collection, owner, operator, item } => ReadResult::Allowance(
					NftsOf::<T>::check_approval_permission(&collection, &item, &owner, &operator)
						.is_ok(),
				),
				TotalSupply(collection) => ReadResult::TotalSupply(
					NftsOf::<T>::collection_items(collection).unwrap_or_default() as u128,
				),
				GetAttribute { collection, item, namespace, key } => ReadResult::GetAttribute(
					AttributeOf::<T>::get((collection, item, namespace, key))
						.map(|attribute| attribute.0.into()),
				),
				NextCollectionId => ReadResult::NextCollectionId(
					NextCollectionIdOf::<T>::get().or(T::CollectionId::initial_value()),
				),
				ItemMetadata { collection, item } => ReadResult::ItemMetadata(
					NftsOf::<T>::item_metadata(collection, item).map(|metadata| metadata.into()),
				),
			}
		}
	}
}
