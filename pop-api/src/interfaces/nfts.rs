use pallet_nfts::{AttributeNamespace, CollectionConfig, CollectionSettings, ItemTip, MintSettings, PreSignedAttributes, PreSignedMint, PriceWithDirection};
use sp_runtime::{BoundedVec, MultiAddress};

pub struct Nft {
    pub pallet_index: u8,
}

impl Nft {
    pub fn new(pallet_index: u8) -> Self {
        Self { pallet_index }
    }
}

pub enum NftCalls<AccountId, Balance, BlockNumber, CollectionId, ItemId, KeyLimit, StringLimit, MaxTips, OffchainSignature> {
    Create {
        admin: MultiAddress<AccountId, ()>,
        config: CollectionConfig<Balance, BlockNumber, CollectionId>
    },
    Destroy {
        collection: CollectionId
    },
    Mint {
        collection: CollectionId,
        item: ItemId,
        mint_to: MultiAddress<AccountId, ()>
    },
    Burn {
        collection: CollectionId,
        item: ItemId,
    },
    Transfer {
        collection: CollectionId,
        item: ItemId,
        dest: MultiAddress<AccountId, ()>
    },
    Redeposit {
        collection: CollectionId,
        items: Vec<ItemId>
    },
    LockItemTransfer {
        collection: CollectionId,
        item: ItemId,
    },
    UnlockItemTransfer {
        collection: CollectionId,
        item: ItemId,
    },
    LockCollection {
        collection: CollectionId,
        lock_settings: CollectionSettings,
    },
    TransferOwnership {
        collection: CollectionId,
        new_owner: MultiAddress<AccountId, ()>
    },
    SetTeam {
        collection: CollectionId,
        issuer: Option<MultiAddress<AccountId, ()>>,
        admin: Option<MultiAddress<AccountId, ()>>,
        freezer: Option<MultiAddress<AccountId, ()>>,
    },
    ApproveTransfer {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
        maybe_deadline: Option<BlockNumber>,
    },
    CancelApproval {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
    },
    ClearAllTransferApprovals {
        collection: CollectionId,
        item: ItemId,
    },
    LockItemProperties {
        collection: CollectionId,
        item: ItemId,
        lock_metadata: bool,
        lock_attributes: bool,
    },
    SetAttribute {
        collection: CollectionId,
        maybe_item: Option<ItemId>,
        namespace: AttributeNamespace<AccountId>,
        key: BoundedVec<u8, KeyLimit>,
        value: BoundedVec<u8, KeyLimit>,
    },
    ClearAttribute {
        collection: CollectionId,
        maybe_item: Option<ItemId>,
        namespace: AttributeNamespace<AccountId>,
        key: BoundedVec<u8, KeyLimit>,
    },
    ApproveItemAttribute {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
    },
    CancelItemAttributesApproval {
        collection: CollectionId,
        item: ItemId,
        delegate: MultiAddress<AccountId, ()>,
    },
    SetMetadata {
        collection: CollectionId,
        item: ItemId,
        data: BoundedVec<u8, StringLimit>,
    },
    ClearMetadata {
        collection: CollectionId,
        item: ItemId,
    },
    SetCollectionMetadata {
        collection: CollectionId,
        data: BoundedVec<u8, StringLimit>,
    },
    ClearCollectionMetadata {
        collection: CollectionId,
    },
    SetAcceptOwnership {
        collection: CollectionId,
        maybe_collection: Option<CollectionId>,
    },
    SetCollectionMaxSupply {
        collection: CollectionId,
        max_supply: u32,
    },
    UpdateMintSettings {
        collection: CollectionId,
        mint_settings: MintSettings<Balance, BlockNumber, CollectionId>,
    },
    Price {
        collection: CollectionId,
        item: ItemId,
        price: Option<Balance>,
    },
    BuyItem {
        collection: CollectionId,
        item: ItemId,
        bid_price: Balance,
    },
    PayTips {
        tips: BoundedVec<ItemTip<CollectionId, ItemId, AccountId, Balance>, MaxTips>
    },
    CreateSwap {
        offered_collection: CollectionId,
        offered_item: ItemId,
        desired_collection: CollectionId,
        maybe_desired_item: Option<ItemId>,
        maybe_price: Option<PriceWithDirection<Balance>>,
        duration: BlockNumber,
    },
    CancelSwap {
        offered_collection: CollectionId,
        offered_item: ItemId,
    },
    ClaimSwap {
        send_collection: CollectionId,
        send_item: ItemId,
        receive_collection: CollectionId,
        receive_item: ItemId,
    },
    MintPreSigned {
        mint_data: PreSignedMint<CollectionId, ItemId, AccountId, BlockNumber, Balance>,
        signature: OffchainSignature,
        signer: AccountId
    },
    SetAttributesPreSigned {
        data: PreSignedAttributes<CollectionId, ItemId, AccountId, BlockNumber>,
        signature: OffchainSignature,
        signer: AccountId,
    }
}