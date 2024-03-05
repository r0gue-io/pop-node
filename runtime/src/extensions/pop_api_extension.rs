use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    pallet_prelude::*,
};
use log;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};

use crate::extensions::ext_impl::{dispatch::dispatch, read_state::read_state};

const LOG_TARGET: &str = "popapi::extension";

#[derive(Default)]
pub struct PopApiExtension;

fn convert_err(err_msg: &'static str) -> impl FnOnce(DispatchError) -> DispatchError {
    move |err| {
        log::trace!(
            target: LOG_TARGET,
            "Pop API failed:{:?}",
            err
        );
        DispatchError::Other(err_msg)
    }
}

pub mod v0 {
    #[derive(Debug)]
    pub enum FuncId {
        Dispatch,
        ReadState,
    }
}

impl TryFrom<u16> for v0::FuncId {
    type Error = DispatchError;

    fn try_from(func_id: u16) -> Result<Self, Self::Error> {
        let id = match func_id {
            0x0 => Self::Dispatch,
            0x1 => Self::ReadState,
            _ => {
                log::error!("Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"));
            }
        };

        Ok(id)
    }
}

impl<T> ChainExtension<T> for PopApiExtension
where
    T: pallet_contracts::Config + cumulus_pallet_parachain_system::Config,
    <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
    <T as SysConfig>::RuntimeCall: Parameter
        + Dispatchable<RuntimeOrigin = <T as SysConfig>::RuntimeOrigin, PostInfo = PostDispatchInfo>
        + GetDispatchInfo
        + From<frame_system::Call<T>>,
{
    fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
        <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        log::debug!(target:LOG_TARGET, " extension called ");
        match v0::FuncId::try_from(env.func_id())? {
            v0::FuncId::Dispatch => {
                match dispatch::<T, E>(env) {
                    Ok(()) => Ok(RetVal::Converging(0)),
                    Err(DispatchError::Module(error)) => {
                        // encode status code = pallet index in runtime + error index, allowing for 999 errors
                        Ok(RetVal::Converging(
                            (error.index as u32 * 1_000) + u32::from_le_bytes(error.error),
                        ))
                    }
                    Err(e) => Err(e),
                }
            }
            v0::FuncId::ReadState => {
                read_state::<T, E>(env)?;
                Ok(RetVal::Converging(0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    pub use super::*;
    pub use crate::*;
    use enumflags2::BitFlags;
    pub use pallet_contracts::Code;
    use pallet_nfts::{CollectionConfig, CollectionSetting, CollectionSettings, MintSettings};
    use parachains_common::CollectionId;
    pub use sp_runtime::{traits::Hash, AccountId32};

    pub const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;

    pub const ALICE: AccountId32 = AccountId32::new([1_u8; 32]);
    pub const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
    pub const INITIAL_AMOUNT: u128 = 100_000 * UNIT;
    pub const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .expect("Frame system builds valid default genesis config");

        pallet_balances::GenesisConfig::<Runtime> {
            balances: vec![(ALICE, INITIAL_AMOUNT), (BOB, INITIAL_AMOUNT)],
        }
        .assimilate_storage(&mut t)
        .expect("Pallet balances storage can be assimilated");

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    pub fn load_wasm_module<T>(
        path: &str,
    ) -> std::io::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
    where
        T: frame_system::Config,
    {
        let wasm_binary = std::fs::read(path)?;
        let code_hash = T::Hashing::hash(&wasm_binary);
        Ok((wasm_binary, code_hash))
    }

    pub fn function_selector(name: &str) -> Vec<u8> {
        let hash = sp_io::hashing::blake2_256(name.as_bytes());
        [hash[0..4].to_vec()].concat()
    }

    // NFT helper functions
    fn collection_config_from_disabled_settings(
        settings: BitFlags<CollectionSetting>,
    ) -> CollectionConfig<Balance, BlockNumber, CollectionId> {
        CollectionConfig {
            settings: CollectionSettings::from_disabled(settings),
            max_supply: None,
            mint_settings: MintSettings::default(),
        }
    }

    fn default_collection_config() -> CollectionConfig<Balance, BlockNumber, CollectionId> {
        collection_config_from_disabled_settings(CollectionSetting::DepositRequired.into())
    }

    #[test]
    fn dispatch_balance_transfer_from_contract_works() {
        new_test_ext().execute_with(|| {
            let _ = env_logger::try_init();

            let (wasm_binary, _) = load_wasm_module::<Runtime>("../contracts/pop-api-examples/balance-transfer/target/ink/pop_api_extension_demo.wasm").unwrap();

            let init_value = 100;

            let result = Contracts::bare_instantiate(
                ALICE,
                init_value,
                GAS_LIMIT,
                None,
                Code::Upload(wasm_binary),
                function_selector("new"),
                vec![],
                DEBUG_OUTPUT,
                pallet_contracts::CollectEvents::Skip,
            )
            .result
            .unwrap();

            assert!(
                !result.result.did_revert(),
                "deploying contract reverted {:?}",
                result
            );

            let addr = result.account_id;

            let function = function_selector("transfer_through_runtime");
            let value_to_send: u128 = 1_000_000_000_000_000;
            let params = [function, BOB.encode(), value_to_send.encode()].concat();

            let bob_balance_before = Balances::free_balance(&BOB);
            assert_eq!(bob_balance_before, INITIAL_AMOUNT);

            let result = Contracts::bare_call(
                ALICE,
                addr.clone(),
                0,
                Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
                None,
                params,
                DEBUG_OUTPUT,
                pallet_contracts::CollectEvents::Skip,
                pallet_contracts::Determinism::Enforced,
            );

            if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
                log::debug!(
                    "Contract debug buffer - {:?}",
                    String::from_utf8(result.debug_message.clone())
                );
                log::debug!("result: {:?}", result);
            }

            // check for revert
            assert!(!result.result.unwrap().did_revert(), "Contract reverted!");

            let bob_balance_after = Balances::free_balance(&BOB);
            assert_eq!(bob_balance_before + value_to_send, bob_balance_after);
        });
    }

    #[test]
    fn dispatch_nfts_mint_from_contract_works() {
        new_test_ext().execute_with(|| {
            let _ = env_logger::try_init();

            let (wasm_binary, _) = load_wasm_module::<Runtime>(
                "../contracts/pop-api-examples/nfts/target/ink/pop_api_nft_example.wasm",
            )
            .unwrap();

            let init_value = 100;

            let result = Contracts::bare_instantiate(
                ALICE,
                init_value,
                GAS_LIMIT,
                None,
                Code::Upload(wasm_binary),
                function_selector("new"),
                vec![],
                DEBUG_OUTPUT,
                pallet_contracts::CollectEvents::Skip,
            )
            .result
            .unwrap();

            assert!(
                !result.result.did_revert(),
                "deploying contract reverted {:?}",
                result
            );

            let addr = result.account_id;

            let collection_id: u32 = 0;
            let item_id: u32 = 1;

            // create nft collection
            assert_eq!(
                Nfts::force_create(
                    RuntimeOrigin::root(),
                    ALICE.into(),
                    default_collection_config()
                ),
                Ok(())
            );

            assert_eq!(Nfts::collection_owner(collection_id), Some(ALICE.into()));
            // assert that the item does not exist yet
            assert_eq!(Nfts::owner(collection_id, item_id), None);

            let function = function_selector("mint_through_runtime");

            let params = [
                function,
                collection_id.encode(),
                item_id.encode(),
                BOB.encode(),
            ]
            .concat();

            let result = Contracts::bare_call(
                ALICE,
                addr.clone(),
                0,
                Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
                None,
                params,
                DEBUG_OUTPUT,
                pallet_contracts::CollectEvents::Skip,
                pallet_contracts::Determinism::Enforced,
            );

            if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
                log::debug!(
                    "Contract debug buffer - {:?}",
                    String::from_utf8(result.debug_message.clone())
                );
                log::debug!("result: {:?}", result);
            }

            // check for revert
            assert!(!result.result.unwrap().did_revert(), "Contract reverted!");

            assert_eq!(Nfts::owner(collection_id, item_id), Some(BOB.into()));
        });
    }

    #[test]
    fn nfts_mint_surfaces_error() {
        new_test_ext().execute_with(|| {
            let _ = env_logger::try_init();

            let (wasm_binary, _) = load_wasm_module::<Runtime>(
                "../contracts/pop-api-examples/nfts/target/ink/pop_api_nft_example.wasm",
            )
            .unwrap();

            let init_value = 100;

            let result = Contracts::bare_instantiate(
                ALICE,
                init_value,
                GAS_LIMIT,
                None,
                Code::Upload(wasm_binary),
                function_selector("new"),
                vec![],
                DEBUG_OUTPUT,
                pallet_contracts::CollectEvents::Skip,
            )
            .result
            .unwrap();

            assert!(
                !result.result.did_revert(),
                "deploying contract reverted {:?}",
                result
            );

            let addr = result.account_id;

            let collection_id: u32 = 0;
            let item_id: u32 = 1;

            let function = function_selector("mint_through_runtime");

            let params = [
                function,
                collection_id.encode(),
                item_id.encode(),
                BOB.encode(),
            ]
            .concat();

            let result = Contracts::bare_call(
                ALICE,
                addr.clone(),
                0,
                Weight::from_parts(100_000_000_000, 3 * 1024 * 1024),
                None,
                params,
                DEBUG_OUTPUT,
                pallet_contracts::CollectEvents::Skip,
                pallet_contracts::Determinism::Enforced,
            );

            if DEBUG_OUTPUT == pallet_contracts::DebugInfo::UnsafeDebug {
                log::debug!(
                    "Contract debug buffer - {:?}",
                    String::from_utf8(result.debug_message.clone())
                );
                log::debug!("result: {:?}", result);
            }

            // check for revert with expected error
            let result = result.result.unwrap();
            assert!(result.did_revert());
        });
    }
}
