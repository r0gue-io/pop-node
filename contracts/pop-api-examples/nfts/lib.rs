#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::nfts;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    NftsError(nfts::Error),
}

impl From<nfts::Error> for ContractError {
    fn from(value: nfts::Error) -> Self {
        ContractError::NftsError(value)
    }
}

mod pop_api_extension_demo {
    use super::ContractError;

    #[ink(storage)]
    #[derive(Default)]
    pub struct PopApiExtensionDemo;

    impl PopApiExtensionDemo {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiExtensionDemo::new");
            Default::default()
        }

        #[ink(message)]
        pub fn mint_through_runtime(
            &mut self,
            collection_id: u32,
            item_id: u32,
            receiver: AccountId,
        ) -> Result<(), ContractError> {
            ink::env::debug_println!("PopApiExtensionDemo::mint_through_runtime: collection_id: {:?} \nitem_id {:?} \nreceiver: {:?}, ", collection_id, item_id, receiver);

            // simplified API call
            let result = pop_api::nfts::mint(collection_id, item_id, receiver);
            ink::env::debug_println!(
                "PopApiExtensionDemo::mint_through_runtime result: {result:?}"
            );
            if let Err(pop_api::nfts::Error::NoConfig) = result {
                ink::env::debug_println!(
                    "PopApiExtensionDemo::mint_through_runtime expected error received"
                );
            }
            result?;

            ink::env::debug_println!("PopApiExtensionDemo::mint_through_runtime end");
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            PopApiExtensionDemo::new();
        }
    }
}
