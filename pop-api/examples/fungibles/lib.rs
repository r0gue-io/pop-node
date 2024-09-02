#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(unexpected_cfgs)]

use pop_api::{
	assets::fungibles::{self as api, events::Create, FungiblesError},
	primitives::{v0::error::Error, AssetId},
};

pub type Result<T> = core::result::Result<T, Error>;

#[ink::contract]
mod api_example {
	use super::*;

	#[ink(storage)]
	pub struct ApiExample {
		asset_id: AssetId,
	}

	impl ApiExample {
		#[ink(constructor, payable)]
		pub fn new(asset_id: AssetId, min_balance: Balance) -> Result<Self> {
			let contract = Self { asset_id };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().account_id();
			// match api::create(asset_id, owner, min_balance) {
			//     Error::Module { }
			// }

			contract.env().emit_event(Create {
				id: asset_id,
				creator: owner.clone(),
				admin: owner,
			});
			Ok(contract)
		}

		#[ink(message)]
		pub fn asset_exists(&self) -> Result<bool> {
			api::asset_exists(self.asset_id).map_err(|e| e.into())
		}
	}

	#[cfg(test)]
	mod tests {
		use drink::{
			sandbox::{AccountIdFor, BlockBuilder, Extension, RuntimeMetadataPrefixed},
			session::{Session, NO_SALT},
			Sandbox,
		};
		use frame_support::__private::TestExternalities;
		use ink::scale::Encode;
		use pop_api::assets::fungibles::events::Create;
		use pop_runtime_devnet::Runtime as PopRuntime;
		use std::error::Error;

		pub struct PopSandbox {
			ext: TestExternalities,
		}

		const INITIAL_BALANCE: u128 = 1_000_000_000_000_000;
		// const DEFAULT_ACCOUNT: AccountId32 = ;

		impl Default for PopSandbox {
			fn default() -> Self {
				let ext = BlockBuilder::<PopRuntime>::new_ext(vec![(
					AccountIdFor::<PopRuntime>::from([1u8; 32]),
					INITIAL_BALANCE,
				)]);
				Self { ext }
			}
		}

		impl Sandbox for PopSandbox {
			type Runtime = PopRuntime;

			fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
				self.ext.execute_with(execute)
			}

			fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
				// Make a backup of the backend.
				let backend_backup = self.ext.as_backend();
				// Run the action, potentially modifying storage. Ensure, that there are no pending changes
				// that would affect the reverted backend.
				let result = action(self);
				self.ext.commit_all().expect("Failed to commit changes");

				// Restore the backend.
				self.ext.backend = backend_backup;
				result
			}

			fn register_extension<E: ::core::any::Any + Extension>(&mut self, ext: E) {
				self.ext.register_extension(ext);
			}

			fn initialize_block(
				height: frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
				parent_hash: <Self::Runtime as frame_system::Config>::Hash,
			) {
				BlockBuilder::<Self::Runtime>::initialize_block(height, parent_hash)
			}

			fn finalize_block(
				height: frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
			) -> <Self::Runtime as frame_system::Config>::Hash {
				BlockBuilder::<Self::Runtime>::finalize_block(height)
			}

			fn default_actor() -> AccountIdFor<Self::Runtime> {
				AccountIdFor::<PopRuntime>::from([1u8; 32])
			}

			fn get_metadata() -> RuntimeMetadataPrefixed {
				Self::Runtime::metadata()
			}

			fn convert_account_to_origin(account: AccountIdFor<Self::Runtime>) -> <<Self::Runtime as frame_system::Config>::RuntimeCall as frame_support::sp_runtime::traits::Dispatchable>::RuntimeOrigin{
				Some(account).into()
			}
		}

		#[drink::contract_bundle_provider]
		enum BundleProvider {}

		#[drink::test(sandbox = "PopSandbox")]
		fn new_works(mut session: Session<PopRuntime>) -> Result<(), Box<dyn Error>> {
			let contract_bundle = BundleProvider::local()?;
			let contract = session
				.deploy_bundle(contract_bundle.clone(), "new", &["10", "100"], NO_SALT, None)
				.expect("Contract deployment failed");
			let bytes: [u8; 32] = contract.into();
			let contract = pop_api::primitives::AccountId::from(bytes);

			// // Now we can inspect the emitted events.
			let record = session.record();
			let contract_events = record
				.last_event_batch()
				// We can use the `contract_events_decoded` method to decode the events into
				// `contract_transcode::Value` objects.
				.contract_events();

			assert_eq!(contract_events.len(), 1);
			let event = Create { id: 10, creator: contract.clone(), admin: contract };
			assert_eq!(contract_events.last().unwrap(), &event.encode());

			// let result = session.call("asset_exists", &["10u32"], NO_ENDOWMENT);
			Ok(())
		}
	}
}
