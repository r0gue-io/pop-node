#![cfg_attr(not(feature = "std"), no_std)]
pub use pallet::*;

pub mod sponsored;
pub mod types;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::CheckedSub;

	use crate::{types::*, weights::WeightInfo};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_revive::Config + pallet_transaction_payment::Config
	{
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The deposit to be paid to become a sponsor.
		type SponsorshipDeposit: Get<BalanceOf<Self>>;

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
	/// - V: The sponsored amount.
	#[pallet::storage]
	pub type Sponsorships<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		AccountIdOf<T>,
		BalanceOf<T>,
		OptionQuery,
	>;

	// TODO: simplify events.
	/// Pallets use events to inform users when important changes are made.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An account has been sponsored.
		NewSponsorship {
			/// The sponsor account.
			sponsor: AccountIdOf<T>,
			/// The sponsored account.
			beneficiary: AccountIdOf<T>,
			/// The sponsored amount
			amount: BalanceOf<T>,
		},
		/// An account is no longer sponsored.
		SponsorshipRemoved {
			/// The account no longer being the sponsor.
			was_sponsor: AccountIdOf<T>,
			/// The account no longer being the sponsored.
			was_beneficiary: AccountIdOf<T>,
		},
		/// An existing sponsorship has been modified.
		SponsorshipUpdated {
			/// The sponsor account.
			sponsor: AccountIdOf<T>,
			/// The sponsored account.
			beneficiary: AccountIdOf<T>,
			/// The previous sponsored amount.
			old_amount: BalanceOf<T>,
			/// The updated sponsored amount.
			new_amount: BalanceOf<T>,
		},
		/// A call has been sponsored.
		CallSponsored {
			/// The account paying for the fee costs.
			by: AccountIdOf<T>,
		},
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The account is already being sponsored.
		AlreadySponsored,
		/// This action cannot be sponsored.
		CantSponsor,
		/// The sponsorship doesn't exist.
		UnknownSponsorship,
		/// The cost is higher than the max sponsored.
		SponsorshipOutOfLimits,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new sponsorship for `beneficiary` with a value of `amount`.
		///
		/// Parameters
		/// - `beneficiary`: Account to be sponsored by the caller.
		/// - `amount`: How much `beneficiary` is sponsored for.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::sponsor_account())]
		pub fn sponsor_account(
			origin: OriginFor<T>,
			beneficiary: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(
				!Sponsorships::<T>::contains_key(&who, &beneficiary),
				Error::<T>::AlreadySponsored,
			);
			// TODO: Reserve SponsorshipDeposit
			// Register new sponsorship.
			<Sponsorships<T>>::set(&who, &beneficiary, Some(amount));
			frame_system::Pallet::<T>::inc_providers(&beneficiary);
			Self::deposit_event(Event::NewSponsorship { sponsor: who, beneficiary, amount });
			Ok(())
		}

		/// Remove an account from the list of sponsored accounts managed by origin.
		///
		/// Parameters
		/// - `beneficiary`: Account to be removed from a sponsorship.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_sponsorship())]
		pub fn remove_sponsorship_for(
			origin: OriginFor<T>,
			beneficiary: AccountIdOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if <Sponsorships<T>>::take(&who, &beneficiary).is_some() {
				let _ = frame_system::Pallet::<T>::dec_providers(&beneficiary);
				Self::deposit_event(Event::SponsorshipRemoved {
					was_sponsor: who,
					was_beneficiary: beneficiary,
				});
			}
			Ok(())
		}

		/// Set the value of an existing sponsorship to a given amount.
		///
		/// Parameters
		/// - `beneficiary`: Account of the beneficiary.
		/// - `new_amount`: The new amount for the sponsorship.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_sponsorship())]
		pub fn set_sponsorship_amount(
			origin: OriginFor<T>,
			beneficiary: AccountIdOf<T>,
			new_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Sponsorships::<T>::try_mutate(&who, &beneficiary, |maybe_amount| -> DispatchResult {
				let amount = maybe_amount.as_mut().ok_or(Error::<T>::UnknownSponsorship)?;
				let old_amount = *amount;
				*amount = new_amount;
				Self::deposit_event(Event::SponsorshipUpdated {
					sponsor: who.clone(),
					beneficiary: beneficiary.clone(),
					old_amount,
					new_amount,
				});
				Ok(())
			})
		}
	}

	impl<T: Config> Pallet<T> {
		/// Whether the sponsorship relation between two given accounts exist, and if so
		/// what's the sponsored value.
		/// Parameters
		/// - `account`: Potential beneficiary of the sponsorship.
		/// - `sponsor`: The account that could be acting as a sponsor.
		pub fn is_sponsored_by(
			account: &AccountIdOf<T>,
			sponsor: &AccountIdOf<T>,
		) -> Option<BalanceOf<T>> {
			<Sponsorships<T>>::get(sponsor, account)
		}

		/// Whether some amount can be withdrawn from the sponsored balance.
		///
		/// Parameters
		/// - `sponsor`: The account that could be acting as a sponsor.
		/// - `beneficiary`: Potential beneficiary of the sponsorship.
		/// - `amount`: The amount representing the fee cost.
		pub fn can_decrease(
			sponsor: &AccountIdOf<T>,
			beneficiary: &AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> bool {
			<Sponsorships<T>>::get(sponsor, beneficiary)
				.map_or(false, |sponsored| sponsored >= amount)
		}

		/// Withdraws an amount from the sponsored value.
		///
		/// Parameters
		/// - `sponsor`: The account that is acting as a sponsor.
		/// - `beneficiary`: Beneficiary of the sponsorship.
		/// - `amount`: The amount representing the fee cost.
		pub fn withdraw_from_sponsorship(
			sponsor: &AccountIdOf<T>,
			beneficiary: &AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Check if the withdrawal can be made
			ensure!(
				!Self::can_decrease(beneficiary, sponsor, amount),
				Error::<T>::SponsorshipOutOfLimits
			);
			Sponsorships::<T>::mutate(beneficiary, sponsor, |maybe_sponsorship| {
				let sponsored = maybe_sponsorship.ok_or(Error::<T>::UnknownSponsorship)?;
				// Already checked in can_decrease
				ensure!(sponsored >= amount, Error::<T>::SponsorshipOutOfLimits);
				let new_value = sponsored
					.checked_sub(&amount)
					.ok_or_else(|| <DispatchError>::from(Error::<T>::SponsorshipOutOfLimits))?;
				*maybe_sponsorship = Some(new_value);

				Ok(new_value)
			})
		}
	}
}
