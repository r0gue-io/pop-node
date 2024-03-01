#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;

/// A part of the runtime dispatchable API.
///
/// For now, `ink!` doesn't provide any support for exposing the real `RuntimeCall` enum,
/// which fully describes the composed API of all the pallets present in runtime. Hence,
/// in order to use `call-runtime` functionality, we have to provide at least a partial
/// object, which correctly encodes the target extrinsic.
///
/// You can investigate the full `RuntimeCall` definition by either expanding
/// `construct_runtime!` macro application or by using secondary tools for reading chain
/// metadata, like `subxt`.
#[derive(scale::Encode)]
enum RuntimeCall {
    /// This index can be found by investigating runtime configuration. You can check the
    /// pallet order inside `construct_runtime!` block and read the position of your
    /// pallet (0-based).
    ///
    ///
    /// [See here for more.](https://substrate.stackexchange.com/questions/778/how-to-get-pallet-index-u8-of-a-pallet-in-runtime)
    #[codec(index = 10)]
    Balances(BalancesCall),
}

#[derive(scale::Encode)]
enum BalancesCall {
    /// This index can be found by investigating the pallet dispatchable API. In your
    /// pallet code, look for `#[pallet::call]` section and check
    /// `#[pallet::call_index(x)]` attribute of the call. If these attributes are
    /// missing, use source-code order (0-based).
    #[codec(index = 3)]
    TransferKeepAlive {
        dest: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        value: u128,
    },
    #[codec(index = 8)]
    ForceSetBalance {
        who: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        new_free: u128,
    },
}

// SAFE_KEYS should live in pop-api repo, both this runtime and the contract
// can depend on pop-api to be able to interface with each other.
// Right now is just impl in both sides, contract and extension until this
// is merged into pop-api
enum SafeKeys {
    RelayBlockNumber,
}

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
    #[ink(extension = 0xfecb)]
    fn dispatch(call: RuntimeCall) -> Result<Vec<u8>>;

    #[ink(extension = 0xfeca)]
    fn read_state(key: SafeKeys) -> Result<Vec<u8>>;

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

    // impl From<EnvError> for RuntimeError {
    //     fn from(e: EnvError) -> Self {
    //         use ink::env::ReturnErrorCode;
    //         match e {
    //             EnvError::ReturnError(ReturnErrorCode::CallRuntimeFailed) => {
    //                 RuntimeError::CallRuntimeFailed
    //             }
    //             _ => panic!("Unexpected error from `pallet-contracts`."),
    //         }
    //     }
    // }

    impl PopApiExtensionDemo {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiExtensionDemo::new");
            Default::default()
        }

        #[ink(message)]
        pub fn read_runtime_state(
            &self,
            key: SAFE_KEYS
        ) {
            let state = self.env().extension().read_state(key);
            ink::env::debug_println!("{:?}", state);
        }

        #[ink(message)]
        pub fn transfer_through_runtime(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) {
            ink::env::debug_println!("PopApiExtensionDemo::transfer_through_runtime: \nreceiver: {:?}, \nvalue: {:?}", receiver, value);

            let call = RuntimeCall::Balances(BalancesCall::TransferKeepAlive {
                    dest: receiver.into(),
                    value: value,
                });
            self.env().extension().dispatch(call);

            ink::env::debug_println!("PopApiExtensionDemo::transfer_through_runtime end");
            
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