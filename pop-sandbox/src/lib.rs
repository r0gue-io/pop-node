use frame_support::{
	sp_runtime::{
		traits::{Header, One},
		AccountId32,
	},
	traits::Hooks,
};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pop_runtime_devnet::Balance;
use pop_runtime_devnet::{BuildStorage, Runtime};

/// Alias for the account ID type.
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;

/// Default initial balance for the default account.
pub const UNIT: Balance = 10_000_000_000;
pub const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
pub const INIT_VALUE: Balance = 100 * UNIT;
pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
pub const CHARLIE: AccountId32 = AccountId32::new([3_u8; 32]);

/// A helper struct for initializing and finalizing blocks.
pub struct BlockBuilder<T>(std::marker::PhantomData<T>);

impl<
		T: pallet_balances::Config + pallet_timestamp::Config<Moment = u64> + pallet_contracts::Config,
	> BlockBuilder<T>
{
	/// Create a new externalities with the given balances.
	pub fn new_ext(balances: Vec<(T::AccountId, T::Balance)>) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::<T>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<T> { balances }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(storage);

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

pub struct Sandbox {
	ext: sp_io::TestExternalities,
}

impl Default for Sandbox {
	fn default() -> Self {
		let balances: Vec<(AccountId32, u128)> =
			vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT), (CHARLIE, INIT_AMOUNT)];
		let ext = BlockBuilder::<Runtime>::new_ext(balances);
		Self { ext }
	}
}

drink::impl_sandbox!(Sandbox, Runtime, BlockBuilder, ALICE);
