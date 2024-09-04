//! The fungibles pallet offers a streamlined interface for interacting with fungible assets. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

pub use pallet::*;

#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		#[codec(index = 0)]
		Dummy(AccountIdOf<T>),
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::default())]
		pub fn transfer(_origin: OriginFor<T>) -> DispatchResult {
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Reads fungible asset state based on the provided value.
		///
		/// This function matches the value to determine the type of state query and returns the
		/// encoded result.
		///
		/// # Parameter
		/// - `value` - An instance of `Read<T>`, which specifies the type of state query and
		///   the associated parameters.
		pub fn read_state(value: Read<T>) -> Vec<u8> {
			use Read::*;
			vec![]
		}
	}
}
