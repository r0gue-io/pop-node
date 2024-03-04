#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;
use pop_api::storage_keys::RuntimeStateKeys::ParachainSystemKeys;


#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopApiError {
    TotalSupplyFailed,
}

pub type Result<T> = core::result::Result<T, PopApiError>;

use scale;
impl From<scale::Error> for PopApiError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

/// This is an example of how an ink! contract may call the Substrate
/// runtime function `RandomnessCollectiveFlip::random_seed`. See the
/// file `runtime/chain-extension-example.rs` for that implementation.
///
/// Here we define the operations to interact with the Substrate runtime.
#[ink::chain_extension]
pub trait PopApi {
    type ErrorCode = PopApiError;

    /// Note: this gives the operation a corresponding `func_id` (1101 in this case),
    /// and the chain-side chain extension will get the `func_id` to do further
    /// operations.

    #[ink(extension = 0xfeca)]
    fn read_relay_block_number(key: LastRelayChainBlockNumber) -> Result<BlockNumber>;

}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::TotalSupplyFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = PopApi;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod pop_api_extension_demo {
    use crate::{
        BalancesCall,
        RuntimeCall,
    };

    use super::PopApiError;

    use ink::env::Error as EnvError;

    #[ink(event)]
    pub struct RelayBlockNumberRead {
        value: BlockNumber
    }

    /// A trivial contract with a single message, that uses `call-runtime` API for
    /// performing native token transfer.
    #[ink(storage)]
    #[derive(Default)]
    pub struct PopApiExtensionDemo;

    impl From<EnvError> for PopApiError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::CallRuntimeFailed => PopApiError::TotalSupplyFailed,
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
    }

    impl PopApiExtensionDemo {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiExtensionDemo::new");
            Default::default()
        }

        #[ink(message)]
        pub fn read_relay_block_number(
            &self
        ) {
            let state = self.env().extension().read_state(LastRelayChainBlockNumber);
            ink::env::debug_println!("{:?}", state);
            ink::env().emit_event(
                RelayBlockNumberRead {value: state}
            );
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
        };

        use ink::{
            env::{
                test::default_accounts,
                DefaultEnvironment,
            },
            primitives::AccountId,
        };

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        /// The base number of indivisible units for balances on the
        /// `substrate-contracts-node`.
        const UNIT: Balance = 1_000_000_000_000;

        /// The contract will be given 1000 tokens during instantiation.
        const CONTRACT_BALANCE: Balance = 1_000 * UNIT;

        /// The receiver will get enough funds to have the required existential deposit.
        ///
        /// If your chain has this threshold higher, increase the transfer value.
        const TRANSFER_VALUE: Balance = 1 / 10 * UNIT;

        /// An amount that is below the existential deposit, so that a transfer to an
        /// empty account fails.
        ///
        /// Must not be zero, because such an operation would be a successful no-op.
        const INSUFFICIENT_TRANSFER_VALUE: Balance = 1;

        /// Positive case scenario:
        ///  - the call is valid
        ///  - the call execution succeeds
        #[ink_e2e::test]
        async fn transfer_with_call_runtime_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = RuntimeCallerRef::new();
            let contract = client
                .instantiate("call-runtime", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<RuntimeCaller>();

            let accounts = default_accounts::<DefaultEnvironment>();

            let receiver: AccountId = accounts.bob;

            let sender_balance_before = client
                .free_balance(accounts.alice)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_before = client
                .free_balance(receiver)
                .await
                .expect("Failed to get account balance");

            // when
            let transfer_message =
                call_builder.transfer_through_runtime(receiver, TRANSFER_VALUE);

            let call_res = client
                .call(&ink_e2e::alice(), &transfer_message)
                .submit()
                .await
                .expect("call failed");

            assert!(call_res.return_value().is_ok());

            // then
            let sender_balance_after = client
                .free_balance(accounts.alice)
                .await
                .expect("Failed to get account balance");
            let receiver_balance_after = client
                .free_balance(receiver)
                .await
                .expect("Failed to get account balance");

            assert_eq!(
                contract_balance_before,
                contract_balance_after + TRANSFER_VALUE
            );
            assert_eq!(
                receiver_balance_before,
                receiver_balance_after - TRANSFER_VALUE
            );

            Ok(())
        }

    } 
}