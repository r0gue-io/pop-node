#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod sponsored;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use crate::weights::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_revive::Config {
		/// Overarching runtime event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Register of sponsorships.
	/// - K1: Account acting as sponsor.
	/// - K2: Sponsored account.
	/// - V: If specified, the sponsored amount.
	#[pallet::storage]
	pub type Sponsorships<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		T::AccountId,
		Weight,
		OptionQuery,
	>;

	/// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An account has been sponsored.
		NewSponsorship {
			/// The sponsor account.
			sponsor: T::AccountId,
			/// The sponsored account.
			beneficiary: T::AccountId,
		},
		/// An account is no longer sponsored.
		SponsorshipRemoved {
			/// The account no longer being the sponsor.
			was_sponsor: T::AccountId,
			/// The account no longer being the sponsored.
			was_beneficiary: T::AccountId,
		},
		/// A call has been sponsored.
		CallSponsored {
			/// The account paying for the fee costs.
			by: T::AccountId,
		},
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The account is already being sponsored.
		AlreadySponsored,
		/// This action cannot be sponsored.
		CantSponsor,
		/// The cost is higher than the max sponsored.
		SponsorshipOutOfLimit,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Include an account to the list of sponsored accounts managed by origin.
		///
		/// Parameters
		/// - `beneficiary`: Account to be sponsored by the caller.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::sponsor_account())]
		pub fn sponsor_account(origin: OriginFor<T>, beneficiary: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let amount = Weight::from_parts(1, 1);

			if <Sponsorships<T>>::contains_key(&who, &beneficiary) {
				Err(<Error<T>>::AlreadySponsored.into())
			} else {
				// Update sponsorships.
				<Sponsorships<T>>::set(&who, &beneficiary, Some(amount));

				Self::deposit_event(Event::NewSponsorship { sponsor: who, beneficiary });
				Ok(().into())
			}
		}

		/// Remove an account from the list of sponsored accounts managed by origin.
		///
		/// Parameters
		/// - `account`: Account to be removed from a sponsorship.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_sponsorship())]
		pub fn remove_sponsorship_for(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if <Sponsorships<T>>::take(&who, &account).is_some() {
				Self::deposit_event(Event::SponsorshipRemoved {
					was_sponsor: who,
					was_beneficiary: account,
				});
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Whether the sponsorship relation between two given accounts exist, and if so
		/// what's the sponsored value.
		/// Parameters
		/// - `account`: Potential beneficiary of the sponsorship.
		/// - `sponsor`: The account that could be acting as a sponsor.
		pub fn is_sponsored_by(account: &T::AccountId, sponsor: &T::AccountId) -> Option<Weight> {
			if <Sponsorships<T>>::contains_key(sponsor, account) {
				<Sponsorships<T>>::get(sponsor, account)
			} else {
				None
			}
		}
	}
}
