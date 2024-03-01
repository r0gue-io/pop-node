use pallet_nfts::{AttributeNamespace, CollectionConfig, CollectionSettings, ItemTip, MintSettings, PreSignedAttributes, PreSignedMint, PriceWithDirection};
use sp_runtime::{BoundedVec, MultiAddress};
use ink::prelude::vec::Vec;

#[derive(scale::Encode)]
pub enum NftCalls<AccountId, Balance, BlockNumber, CollectionId, ItemId, KeyLimit, StringLimit, MaxTips, OffchainSignature> {
    #[codec(index = 0)]
    Create {
        admin: MultiAddress<AccountId, ()>,
        config: CollectionConfig<Balance, BlockNumber, CollectionId>
    },
    #[codec(index = 2)]
    Destroy {
        collection: CollectionId
    },
    #[codec(index = 3)]
    Mint {
        collection: CollectionId,
        item: ItemId,
        mint_to: MultiAddress<AccountId, ()>
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
        dest: MultiAddress<AccountId, ()>
    },
    #[codec(index = 7)]
    Redeposit {
        collection: CollectionId,
        items: Vec<ItemId>
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
    #[codec(index = 10)]
    LockCollection {
        collection: CollectionId,
        lock_settings: CollectionSettings,
    },
    #[codec(index = 11)]
    TransferOwnership {
        collection: CollectionId,
        new_owner: MultiAddress<AccountId, ()>
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
    #[codec(index = 19)]
    SetAttribute {
        collection: CollectionId,
        maybe_item: Option<ItemId>,
        namespace: AttributeNamespace<AccountId>,
        key: BoundedVec<u8, KeyLimit>,
        value: BoundedVec<u8, KeyLimit>,
    },
    #[codec(index = 21)]
    ClearAttribute {
        collection: CollectionId,
        maybe_item: Option<ItemId>,
        namespace: AttributeNamespace<AccountId>,
        key: BoundedVec<u8, KeyLimit>,
    },
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
    ClearCollectionMetadata {
        collection: CollectionId,
    },
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
    #[codec(index = 30)]
    UpdateMintSettings {
        collection: CollectionId,
        mint_settings: MintSettings<Balance, BlockNumber, CollectionId>,
    },
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
    #[codec(index = 33)]
    PayTips {
        tips: BoundedVec<ItemTip<CollectionId, ItemId, AccountId, Balance>, MaxTips>
    },
    #[codec(index = 34)]
    CreateSwap {
        offered_collection: CollectionId,
        offered_item: ItemId,
        desired_collection: CollectionId,
        maybe_desired_item: Option<ItemId>,
        maybe_price: Option<PriceWithDirection<Balance>>,
        duration: BlockNumber,
    },
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
    #[codec(index = 37)]
    MintPreSigned {
        mint_data: PreSignedMint<CollectionId, ItemId, AccountId, BlockNumber, Balance>,
        signature: OffchainSignature,
        signer: AccountId
    },
    #[codec(index = 38)]
    SetAttributesPreSigned {
        data: PreSignedAttributes<CollectionId, ItemId, AccountId, BlockNumber>,
        signature: OffchainSignature,
        signer: AccountId,
    }
}