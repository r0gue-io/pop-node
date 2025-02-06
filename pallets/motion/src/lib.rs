#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use pallet::*;
use sp_runtime::DispatchResult;
use sp_std::prelude::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::GetDispatchInfo, pallet_prelude::*, traits::UnfilteredDispatchable,
	};
	use frame_system::pallet_prelude::*;

	use super::{DispatchResult, *};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type RuntimeCall: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo;
		#[cfg(feature = "simple-majority")]
		type SimpleMajorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type SuperMajorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type UnanimousOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A SimpleMajority motion was executed. motion_result contains the call result
		#[cfg(feature = "simple-majority")]
		DispatchSimpleMajority { motion_result: DispatchResult },
		/// A SuperMajority motion was executed. motion_result contains the call result
		DispatchSuperMajority { motion_result: DispatchResult },
		/// A Unanimous motion was executed. motion_result contains the call result
		DispatchUnanimous { motion_result: DispatchResult },
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Ensures the simple majority is met and dispatches a call with `Root` origin.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution + 10,000.
		/// # </weight>
		#[cfg(feature = "simple-majority")]
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(T::WeightInfo::simple_majority().saturating_add(dispatch_info.call_weight), dispatch_info.class)
		})]
		#[pallet::call_index(1)]
		pub fn simple_majority(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			T::SimpleMajorityOrigin::ensure_origin(origin)?;

			let motion_result = Self::do_dispatch(*call);
			Self::deposit_event(Event::DispatchSimpleMajority { motion_result });

			Ok(Pays::No.into())
		}

		/// Ensures the super majority is met and dispatches a call with `Root` origin.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution + 10,000.
		/// # </weight>
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(T::WeightInfo::super_majority().saturating_add(dispatch_info.call_weight), dispatch_info.class)
		})]
		#[pallet::call_index(2)]
		pub fn super_majority(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			T::SuperMajorityOrigin::ensure_origin(origin)?;

			let motion_result = Self::do_dispatch(*call);
			Self::deposit_event(Event::DispatchSuperMajority { motion_result });

			Ok(Pays::No.into())
		}

		/// Ensures unanimous voting is met and dispatches a call with `Root` origin.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution + 10,000.
		/// # </weight>
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(T::WeightInfo::unanimous().saturating_add(dispatch_info.call_weight), dispatch_info.class)
		})]
		#[pallet::call_index(3)]
		pub fn unanimous(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			T::UnanimousOrigin::ensure_origin(origin)?;

			let motion_result = Self::do_dispatch(*call);
			Self::deposit_event(Event::DispatchUnanimous { motion_result });

			Ok(Pays::No.into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Helper to actually dispatch RuntimeCall.
		///
		/// Should only be called after the origin is ensured.
		///
		/// Returns the `DispatchResult` from the dispatched call.
		fn do_dispatch(call: <T as Config>::RuntimeCall) -> DispatchResult {
			let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
			res.map(|_| ()).map_err(|e| e.error)
		}
	}
}
