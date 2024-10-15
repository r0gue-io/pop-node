#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// // FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// // contains a mock runtime specific for testing this pallet's functionality.
// #[cfg(test)]
// mod mock;

// // This module contains the unit tests for this pallet.
// // Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
// #[cfg(test)]
// mod tests;

// // Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// // estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// // for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
pub mod check_sponsored;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use check_sponsored::BalanceOf;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_transaction_payment::Config
		+ pallet_contracts::Config
		+ pallet_balances::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		// Weight information for dispatchables in this pallet.
		// type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Contract usage.
	#[pallet::storage]
	pub(super) type ContractUsage<T> = StorageMap<_, Twox64Concat, AccountIdOf<T>, BalanceOf<T>>;

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		IncentivesClaimed {
			/// The beneficiary of the incentives.
			beneficiary: AccountIdOf<T>,
			// The rewared amount.
			// value: BalanceOf<T>,
		},
		/// A contract has been called.
		ContractCalled {
			/// The contract which has been called.
			contract: AccountIdOf<T>,
		},
	}

	/// Errors inform users that something went wrong.
	pub enum Error {
		/// Can't claim rewards.
		NotOwner,
	}

	/// The dispatchable functions available.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// A contract has been called.
		///
		/// Parameters:
		/// - 'contract': The address of the contracte.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn contract_call(origin: OriginFor<T>, contract: AccountIdOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			//let who = ensure_signed(origin)?;
			Self::deposit_event(Event::IncentivesClaimed { beneficiary: contract });
			Ok(())
		}
	}
}
