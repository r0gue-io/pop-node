#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Dispatchable function benchmarks.
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Dispatchable function weights.
pub mod weights;
pub use pallet::*;
use sp_runtime::DispatchResult;
pub use weights::WeightInfo;

extern crate alloc;
use alloc::{boxed::Box, vec};

/// The motion pallet.
#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::GetDispatchInfo, pallet_prelude::*, traits::UnfilteredDispatchable,
	};
	use frame_system::pallet_prelude::*;

	use super::{DispatchResult, *};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The runtime call type.
		type RuntimeCall: Parameter
			+ UnfilteredDispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo;
		/// Origin that can act as `Root` origin if a collective has achieved a simple majority
		/// consensus.
		type SimpleMajorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Origin that can act as `Root` origin if a collective has achieved a super majority
		/// consensus.
		type SuperMajorityOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Origin that can act as `Root` origin if a collective has achieved a unanimous consensus.
		type UnanimousOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A [SimpleMajorityOrigin] motion was executed. [motion_result] contains the call result
		DispatchSimpleMajority {
			/// Result of dispatching the proposed call.
			motion_result: DispatchResult,
		},
		/// A [SuperMajorityOrigin] motion was executed. [motion_result] contains the call result
		DispatchSuperMajority {
			/// Result of dispatching the proposed call.
			motion_result: DispatchResult,
		},
		/// A [UnanimousOrigin] motion was executed. [motion_result] contains the call result
		DispatchUnanimous {
			/// Result of dispatching the proposed call.
			motion_result: DispatchResult,
		},
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	/// The dispatchable functions available.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Ensures the simple majority is met and dispatches a call with `Root` origin.
		///
		/// # <weight>
		/// - O(1).
		/// - Limited storage reads.
		/// - One DB write (event).
		/// - Weight of derivative `call` execution.
		/// # </weight>
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
		/// - Weight of derivative `call` execution.
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
		/// - Weight of derivative `call` execution.
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
