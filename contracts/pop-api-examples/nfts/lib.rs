#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::PopApiError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
    SomeError,
}

impl From<PopApiError> for ContractError {
    fn from(_value: PopApiError) -> Self {
        ContractError::SomeError
    }
}

#[ink::contract(env = pop_api::PopEnv)]
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
            pop_api::nfts::mint(collection_id, item_id, receiver)?;

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
