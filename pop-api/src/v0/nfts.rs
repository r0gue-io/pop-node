use super::RuntimeCall;
use crate::{PopApiError::UnknownStatusCode, *};
use ink::prelude::vec::Vec;
use sp_runtime::{BoundedVec, MultiAddress};

type Result<T> = core::result::Result<T, Error>;

pub fn mint(
    collection: CollectionId,
    item: ItemId,
    mint_to: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
    Ok(dispatch(RuntimeCall::Nfts(NftCalls::Mint {
        collection,
        item,
        mint_to: mint_to.into(),
        witness_data: None,
    }))?)
}

#[derive(scale::Encode)]
#[allow(dead_code)]
pub(crate) enum NftCalls {
    // #[codec(index = 0)]
    // Create {
    //     admin: MultiAddress<AccountId, ()>,
    //     config: CollectionConfig<Balance, BlockNumber, CollectionId>
    // },
    #[codec(index = 2)]
    Destroy { collection: CollectionId },
    #[codec(index = 3)]
    Mint {
        collection: CollectionId,
        item: ItemId,
        mint_to: MultiAddress<AccountId, ()>,
        witness_data: Option<()>,
    },
    #[codec(index = 5)]
    Burn {
        collection: CollectionId,
        item: ItemId,
    },
    #[codec(index = 6)]
    Transfer {
        collection: CollectionId,
        item: ItemId,
        dest: MultiAddress<AccountId, ()>,
    },
    #[codec(index = 7)]
    Redeposit {
        collection: CollectionId,
        items: Vec<ItemId>,
    },
    #[codec(index = 8)]
    LockItemTransfer {
        collection: CollectionId,
        item: ItemId,
    },
    #[codec(index = 9)]
    UnlockItemTransfer {
        collection: CollectionId,
        item: ItemId,
    },
    // #[codec(index = 10)]
    // LockCollection {
    //     collection: CollectionId,
    //     lock_settings: CollectionSettings,
    // },
    #[codec(index = 11)]
    TransferOwnership {
        collection: CollectionId,
        new_owner: MultiAddress<AccountId, ()>,
    },
    #[codec(index = 12)]
    SetTeam {
        collection: CollectionId,
        issuer: Option<MultiAddress<AccountId, ()>>,
        admin: Option<MultiAddress<AccountId, ()>>,
        freezer: Option<MultiAddress<AccountId, ()>>,
    },
    #[codec(index = 15)]
    ApproveTransfer {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
        maybe_deadline: Option<BlockNumber>,
    },
    #[codec(index = 16)]
    CancelApproval {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
    },
    #[codec(index = 17)]
    ClearAllTransferApprovals {
        collection: CollectionId,
        item: ItemId,
    },
    #[codec(index = 18)]
    LockItemProperties {
        collection: CollectionId,
        item: ItemId,
        lock_metadata: bool,
        lock_attributes: bool,
    },
    // #[codec(index = 19)]
    // SetAttribute {
    //     collection: CollectionId,
    //     maybe_item: Option<ItemId>,
    //     namespace: AttributeNamespace<AccountId>,
    //     key: BoundedVec<u8, KeyLimit>,
    //     value: BoundedVec<u8, KeyLimit>,
    // },
    // #[codec(index = 21)]
    // ClearAttribute {
    //     collection: CollectionId,
    //     maybe_item: Option<ItemId>,
    //     namespace: AttributeNamespace<AccountId>,
    //     key: BoundedVec<u8, KeyLimit>,
    // },
    #[codec(index = 22)]
    ApproveItemAttribute {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
    },
    #[codec(index = 23)]
    CancelItemAttributesApproval {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
    },
    #[codec(index = 24)]
    SetMetadata {
        collection: CollectionId,
        item: ItemId,
        data: BoundedVec<u8, StringLimit>,
    },
    #[codec(index = 25)]
    ClearMetadata {
        collection: CollectionId,
        item: ItemId,
    },
    #[codec(index = 26)]
    SetCollectionMetadata {
        collection: CollectionId,
        data: BoundedVec<u8, StringLimit>,
    },
    #[codec(index = 27)]
    ClearCollectionMetadata { collection: CollectionId },
    #[codec(index = 28)]
    SetAcceptOwnership {
        collection: CollectionId,
        maybe_collection: Option<CollectionId>,
    },
    #[codec(index = 29)]
    SetCollectionMaxSupply {
        collection: CollectionId,
        max_supply: u32,
    },
    // #[codec(index = 30)]
    // UpdateMintSettings {
    //     collection: CollectionId,
    //     mint_settings: MintSettings<Balance, BlockNumber, CollectionId>,
    // },
    #[codec(index = 31)]
    Price {
        collection: CollectionId,
        item: ItemId,
        price: Option<Balance>,
    },
    #[codec(index = 32)]
    BuyItem {
        collection: CollectionId,
        item: ItemId,
        bid_price: Balance,
    },
    // #[codec(index = 33)]
    // PayTips {
    //     tips: BoundedVec<ItemTip<CollectionId, ItemId, AccountId, Balance>, MaxTips>
    // },
    // #[codec(index = 34)]
    // CreateSwap {
    //     offered_collection: CollectionId,
    //     offered_item: ItemId,
    //     desired_collection: CollectionId,
    //     maybe_desired_item: Option<ItemId>,
    //     maybe_price: Option<PriceWithDirection<Balance>>,
    //     duration: BlockNumber,
    // },
    #[codec(index = 35)]
    CancelSwap {
        offered_collection: CollectionId,
        offered_item: ItemId,
    },
    #[codec(index = 36)]
    ClaimSwap {
        send_collection: CollectionId,
        send_item: ItemId,
        receive_collection: CollectionId,
        receive_item: ItemId,
    },
    // #[codec(index = 37)]
    // MintPreSigned {
    //     mint_data: PreSignedMint<CollectionId, ItemId, AccountId, BlockNumber, Balance>,
    //     signature: OffchainSignature,
    //     signer: AccountId
    // },
    // #[codec(index = 38)]
    // SetAttributesPreSigned {
    //     data: PreSignedAttributes<CollectionId, ItemId, AccountId, BlockNumber>,
    //     signature: OffchainSignature,
    //     signer: AccountId,
    // }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// The signing account has no permission to do the operation.
    NoPermission,
    /// The given item ID is unknown.
    UnknownCollection,
    /// The item ID has already been used for an item.
    AlreadyExists,
    /// The approval had a deadline that expired, so the approval isn't valid anymore.
    ApprovalExpired,
    /// The owner turned out to be different to what was expected.
    WrongOwner,
    /// The witness data given does not match the current state of the chain.
    BadWitness,
    /// Collection ID is already taken.
    CollectionIdInUse,
    /// Items within that collection are non-transferable.
    ItemsNonTransferable,
    /// The provided account is not a delegate.
    NotDelegate,
    /// The delegate turned out to be different to what was expected.
    WrongDelegate,
    /// No approval exists that would allow the transfer.
    Unapproved,
    /// The named owner has not signed ownership acceptance of the collection.
    Unaccepted,
    /// The item is locked (non-transferable).
    ItemLocked,
    /// Item's attributes are locked.
    LockedItemAttributes,
    /// Collection's attributes are locked.
    LockedCollectionAttributes,
    /// Item's metadata is locked.
    LockedItemMetadata,
    /// Collection's metadata is locked.
    LockedCollectionMetadata,
    /// All items have been minted.
    MaxSupplyReached,
    /// The max supply is locked and can't be changed.
    MaxSupplyLocked,
    /// The provided max supply is less than the number of items a collection already has.
    MaxSupplyTooSmall,
    /// The given item ID is unknown.
    UnknownItem,
    /// Swap doesn't exist.
    UnknownSwap,
    /// The given item has no metadata set.
    MetadataNotFound,
    /// The provided attribute can't be found.
    AttributeNotFound,
    /// Item is not for sale.
    NotForSale,
    /// The provided bid is too low.
    BidTooLow,
    /// The item has reached its approval limit.
    ReachedApprovalLimit,
    /// The deadline has already expired.
    DeadlineExpired,
    /// The duration provided should be less than or equal to `MaxDeadlineDuration`.
    WrongDuration,
    /// The method is disabled by system settings.
    MethodDisabled,
    /// The provided setting can't be set.
    WrongSetting,
    /// Item's config already exists and should be equal to the provided one.
    InconsistentItemConfig,
    /// Config for a collection or an item can't be found.
    NoConfig,
    /// Some roles were not cleared.
    RolesNotCleared,
    /// Mint has not started yet.
    MintNotStarted,
    /// Mint has already ended.
    MintEnded,
    /// The provided Item was already used for claiming.
    AlreadyClaimed,
    /// The provided data is incorrect.
    IncorrectData,
    /// The extrinsic was sent by the wrong origin.
    WrongOrigin,
    /// The provided signature is incorrect.
    WrongSignature,
    /// The provided metadata might be too long.
    IncorrectMetadata,
    /// Can't set more attributes per one call.
    MaxAttributesLimitReached,
    /// The provided namespace isn't supported in this call.
    WrongNamespace,
    /// Can't delete non-empty collections.
    CollectionNotEmpty,
    /// The witness data should be provided.
    WitnessRequired,
}

impl TryFrom<u32> for Error {
    type Error = PopApiError;

    fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
        use Error::*;
        match status_code {
            0 => Ok(NoPermission),
            1 => Ok(UnknownCollection),
            2 => Ok(AlreadyExists),
            3 => Ok(ApprovalExpired),
            4 => Ok(WrongOwner),
            5 => Ok(BadWitness),
            6 => Ok(CollectionIdInUse),
            7 => Ok(ItemsNonTransferable),
            8 => Ok(NotDelegate),
            9 => Ok(WrongDelegate),
            10 => Ok(Unapproved),
            11 => Ok(Unaccepted),
            12 => Ok(ItemLocked),
            13 => Ok(LockedItemAttributes),
            14 => Ok(LockedCollectionAttributes),
            15 => Ok(LockedItemMetadata),
            16 => Ok(LockedCollectionMetadata),
            17 => Ok(MaxSupplyReached),
            18 => Ok(MaxSupplyLocked),
            19 => Ok(MaxSupplyTooSmall),
            20 => Ok(UnknownItem),
            21 => Ok(UnknownSwap),
            22 => Ok(MetadataNotFound),
            23 => Ok(AttributeNotFound),
            24 => Ok(NotForSale),
            25 => Ok(BidTooLow),
            26 => Ok(ReachedApprovalLimit),
            27 => Ok(DeadlineExpired),
            28 => Ok(WrongDuration),
            29 => Ok(MethodDisabled),
            30 => Ok(WrongSetting),
            31 => Ok(InconsistentItemConfig),
            32 => Ok(NoConfig),
            33 => Ok(RolesNotCleared),
            34 => Ok(MintNotStarted),
            35 => Ok(MintEnded),
            36 => Ok(AlreadyClaimed),
            37 => Ok(IncorrectData),
            38 => Ok(WrongOrigin),
            39 => Ok(WrongSignature),
            40 => Ok(IncorrectMetadata),
            41 => Ok(MaxAttributesLimitReached),
            42 => Ok(WrongNamespace),
            43 => Ok(CollectionNotEmpty),
            44 => Ok(WitnessRequired),
            _ => Err(UnknownStatusCode(status_code)),
        }
    }
}

impl From<PopApiError> for Error {
    fn from(error: PopApiError) -> Self {
        match error {
            PopApiError::Nfts(e) => e,
            _ => panic!("expected nfts error"),
        }
    }
}
