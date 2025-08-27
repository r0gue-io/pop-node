#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::hash::{Blake2x128, CryptoHash},
    prelude::vec::Vec,
    scale::Encode,
    storage::Mapping,
};
use pop_api::{
    messaging::{
        ismp::{self, Get, StorageValue},
        Callback, MessageId,
        xcm::Weight
    },
    nonfungibles::{
        self, CollectionConfig, CollectionId, CollectionSetting, CollectionSettings, ItemId,
        ItemSetting, ItemSettings, MintSettings, MintType, MintWitness,
    },
    StatusCode,
};

pub type Result<T> = core::result::Result<T, Error>;
pub type ParaId = u32;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[allow(clippy::cast_possible_truncation)]
pub enum Error {
    StatusCode(u32),
    NotReady,
    Unknown,
    DecodingFailed,
    Rejected,
    Failed,
    TransferFailed,
}

impl From<StatusCode> for Error {
    fn from(value: StatusCode) -> Self {
        Error::StatusCode(value.0)
    }
}

#[ink::contract]
mod dao {

    use crate::{Error::*, *};

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    enum RegistrationStatus {
        Pending,
        Used,
    }

    #[ink::storage_item]
    pub struct NftVerifier {
        parachain: ParaId,
        collection: CollectionId,
        requests: Mapping<MessageId, (AccountId, ItemId)>,
        next_request: MessageId,
    }

    impl NftVerifier {
        fn new(parachain: ParaId, collection: CollectionId) -> NftVerifier {
            Self {
                parachain,
                collection,
                requests: Mapping::default(),
                next_request: 0,
            }
        }

        fn verify(&mut self, height: u32, account: AccountId, item: ItemId) -> Result<()> {
            self.next_request = self.next_request.saturating_add(1);
            let key: Vec<u8> = generate_key(account, self.collection, item);
            ismp::get(
                self.next_request,
                Get::new(self.parachain, height, 0, Vec::default(), Vec::from([key])),
                0,
                Some(Callback::to(
                    0x57ad942b,
                    Weight::from_parts(2_000_000_000, 500_000),
                )),
            )?;
            self.requests.insert(self.next_request, &(account, item));
            Ok(())
        }
    }

    #[ink(storage)]
    pub struct Dao {
        verifier: NftVerifier,
        collection_id: CollectionId,
        next_item_id: ItemId,
        registered_items: Mapping<ItemId, RegistrationStatus>,
    }

    impl Dao {
        #[ink(constructor, payable)]
        pub fn new() -> Result<Self> {
            let verifier = NftVerifier::new(1000, 0);
            // Create membership token using the non fungibles api.
            let collection_id = create_collection(Self::env().account_id())?;
            let dao = Self {
                verifier,
                collection_id,
                next_item_id: 0,
                registered_items: Mapping::default(),
            };
            Ok(dao)
        }

        #[ink(message)]
        pub fn register(&mut self, height: u32, item: ItemId) -> Result<()> {
            let account = self.env().caller();
            self.verifier.verify(height, account, item)?;
            self.registered_items
                .insert(item, &RegistrationStatus::Pending);
            self.env()
                .emit_event(RegistrationRequested { account, item });
            Ok(())
        }

        #[ink(message, selector = 0x57ad942b)]
        pub fn complete_registration(
            &mut self,
            id: MessageId,
            values: Vec<StorageValue>,
        ) -> Result<()> {
            let (account, verified_item) = self.verifier.requests.get(id).ok_or(Unknown)?;
            let membership = if values[0].value.is_some() {
                self.next_item_id = self.next_item_id.saturating_add(1);
                let item = self.next_item_id;
                nonfungibles::mint(
                    account,
                    self.collection_id,
                    item,
                    Some(MintWitness {
                        owned_item: None,
                        mint_price: None,
                    }),
                )?;
                self.registered_items
                    .insert(verified_item, &RegistrationStatus::Used);
                Some(item)
            } else {
                None
            };
            self.env().emit_event(RegistrationCompleted {
                account,
                verified_item,
                membership,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn collection_id(&mut self) -> CollectionId {
            self.collection_id
        }
    }

    // Create a collection using the non fungibles api.
    fn create_collection(owner: AccountId) -> Result<CollectionId> {
        let config = CollectionConfig {
            settings: CollectionSettings::from_disabled(
                CollectionSetting::TransferableItems.into(),
            ),
            max_supply: None,
            mint_settings: MintSettings {
                mint_type: MintType::Issuer,
                price: None,
                start_block: None,
                end_block: None,
                default_item_settings: ItemSettings::from_disabled(
                    ItemSetting::Transferable.into(),
                ),
            },
        };
        let collection_id = nonfungibles::next_collection_id().unwrap_or_default();
        nonfungibles::create(owner, config)?;
        Ok(collection_id)
    }

    // This function returns the complete storage key for the NFTs pallet's `Account` storage
    // map.
    pub fn generate_key(account: AccountId, collection_id: u32, item_id: u32) -> Vec<u8> {
        // The storage map prefix.
        let storage_map_prefix: [u8; 32] = [
            232, 212, 147, 137, 194, 226, 62, 21, 47, 221, 99, 100, 218, 173, 210, 204, 185, 157,
            136, 14, 198, 129, 121, 156, 12, 243, 14, 136, 134, 55, 29, 169,
        ];
        // Hash and concatenate each component using blake2_128_concat logic.
        let hashed_account = blake2_128_concat(&account.encode());
        let hashed_collection = blake2_128_concat(&collection_id.to_le_bytes());
        let hashed_item = blake2_128_concat(&item_id.to_le_bytes());
        // Concatenate the storage map prefix with the hashed key components
        let mut complete_key = Vec::new();
        complete_key.extend_from_slice(&storage_map_prefix);
        complete_key.extend_from_slice(&hashed_account);
        complete_key.extend_from_slice(&hashed_collection);
        complete_key.extend_from_slice(&hashed_item);
        complete_key
    }

    // A helper function to perform the `blake2_128_concat` logic.
    // This will hash the input and concatenate the result with the original input.
    fn blake2_128_concat(input: &[u8]) -> Vec<u8> {
        let mut output = [0u8; 16]; // blake2_128 produces a 128-bit (16 bytes) hash
        Blake2x128::hash(input, &mut output);
        // Concatenate the hash with the original input
        let mut result = Vec::new();
        result.extend_from_slice(&output);
        result.extend_from_slice(input);
        result
    }

    #[ink::event]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct RegistrationRequested {
        pub account: AccountId,
        pub item: ItemId,
    }

    #[ink::event]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct RegistrationCompleted {
        pub account: AccountId,
        pub verified_item: ItemId,
        pub membership: Option<ItemId>,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let dao = Dao::default();
            assert_eq!(dao.get(), false);
        }
    }
}
