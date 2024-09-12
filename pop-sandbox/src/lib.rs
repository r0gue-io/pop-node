use core::any::Any;

use frame_metadata::RuntimeMetadataPrefixed;
use frame_support::{
	sp_runtime::{traits::Header, AccountId32},
	traits::Hooks,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pop_runtime_devnet::{BuildStorage, Runtime};
use sp_io::TestExternalities;
use frame_support::sp_runtime::traits::One;
use pop_runtime_devnet::Balance;

/// Alias for the account ID type.
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;

/// Default initial balance for the default account.
pub const UNIT: Balance = 10_000_000_000;
pub const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
pub const INIT_VALUE: Balance = 100 * UNIT;
pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);

/// A helper struct for initializing and finalizing blocks.
pub struct BlockBuilder<T>(std::marker::PhantomData<T>);

impl<
		T: pallet_balances::Config + pallet_timestamp::Config<Moment = u64> + pallet_contracts::Config + pallet_aura::Config,
	> BlockBuilder<T>
{
	/// Create a new externalities with the given balances.
	pub fn new_ext(balances: Vec<(T::AccountId, T::Balance)>) -> TestExternalities {
		log::debug!("new externalities: balances={:?}", balances);
		let mut storage = frame_system::GenesisConfig::<T>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<T> { balances }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = TestExternalities::new(storage);

		ext.execute_with(|| Self::initialize_block(BlockNumberFor::<T>::one(), Default::default()));
		ext
	}

	/// Initialize a new block at particular height.
	pub fn initialize_block(
		height: frame_system::pallet_prelude::BlockNumberFor<T>,
		parent_hash: <T as frame_system::Config>::Hash,
	) {
		frame_system::Pallet::<T>::reset_events();
		frame_system::Pallet::<T>::initialize(&height, &parent_hash, &Default::default());
		pallet_balances::Pallet::<T>::on_initialize(height);
		pallet_aura::Pallet::<T>::on_initialize(height);		
		// TODO: Resolve an issue with pallet-aura to simulate the time.
		// pallet_timestamp::Pallet::<T>::set_timestamp(
		// 	SystemTime::now()
		// 		.duration_since(SystemTime::UNIX_EPOCH)
		// 		.expect("Time went backwards")
		// 		.as_secs(),
		// );
		pallet_timestamp::Pallet::<T>::on_initialize(height);
		pallet_contracts::Pallet::<T>::on_initialize(height);
		frame_system::Pallet::<T>::note_finished_initialize();
	}

	/// Finalize a block at particular height.
	pub fn finalize_block(
		height: frame_system::pallet_prelude::BlockNumberFor<T>,
	) -> <T as frame_system::Config>::Hash {
		pallet_contracts::Pallet::<T>::on_finalize(height);
		pallet_timestamp::Pallet::<T>::on_finalize(height);
		pallet_balances::Pallet::<T>::on_finalize(height);
		frame_system::Pallet::<T>::finalize().hash()
	}
}

pub struct PopSandbox {
	ext: TestExternalities,
}

impl Default for PopSandbox {
	fn default() -> Self {
		let balances : Vec<(AccountId32, u128)> = vec![(ALICE, INIT_AMOUNT)];
		let ext = BlockBuilder::<Runtime>::new_ext(balances);
		Self { ext }
	}
}

impl drink::Sandbox for PopSandbox {
	type Runtime = Runtime;

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

	fn register_extension<E: Any + drink::ink_sandbox::Extension>(&mut self, ext: E) {
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
	ALICE	
	}

	fn get_metadata() -> RuntimeMetadataPrefixed {
		Self::Runtime::metadata()
	}

	fn convert_account_to_origin(
                account: AccountIdFor<Self::Runtime>,
	) -> <<Self::Runtime as frame_system::Config>::RuntimeCall as frame_support::sp_runtime::traits::Dispatchable>::RuntimeOrigin{
		Some(account).into()
	}
}
