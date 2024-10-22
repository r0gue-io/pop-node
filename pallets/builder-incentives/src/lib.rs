#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub mod contract_fee_handler;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero},
		Permill,
	};

	use super::*;
	use crate::types::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_transaction_payment::Config + pallet_revive::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// Duration of an era in terms of the number of blocks.
		#[pallet::constant]
		type EraDuration: Get<BlockNumberFor<Self>>;
		/// The pallet's id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		// Weight information for dispatchables in this pallet.
		// type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Registered contracts to their corresponding beneficiaries.
	#[pallet::storage]
	pub type RegisteredContracts<T> = StorageMap<_, Twox64Concat, AccountIdOf<T>, AccountIdOf<T>>;

	/// Tracks the usage of each contract by era.
	#[pallet::storage]
	pub(super) type ContractUsagePerEra<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		EraNumber,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Information about each era.
	#[pallet::storage]
	pub type EraInformation<T> = StorageMap<_, Twox64Concat, EraNumber, EraInfo<T>, ValueQuery>;

	/// Current era number.
	#[pallet::storage]
	pub(super) type CurrentEra<T> = StorageValue<_, EraNumber, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A smart contract has been called.
		ContractCalled {
			/// The smart contract's account that was called.
			contract: AccountIdOf<T>,
		},
		/// A smart contract has been registered to receive rewards.
		ContractRegistered {
			/// The beneficiary of the incentives.
			beneficiary: T::AccountId,
			/// The smart contract's account being registered.
			contract: T::AccountId,
		},
		/// A new era has started.
		NewEra {
			/// The number of the newly started era.
			era: EraNumber,
		},
		/// New incentives have been deposited into the reward pool.
		IncentivesDeposited {
			/// The account that deposited the incentives.
			source: AccountIdOf<T>,
			/// The amount of incentives deposited.
			amount: BalanceOf<T>,
		},
		/// Incentives have been claimed from the reward pool.
		IncentivesClaimed {
			/// The account that received the incentives.
			beneficiary: AccountIdOf<T>,
			// The amount of incentives withdrawn.
			amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unable to claim rewards for the current era.
		CannotClaimRewardsForCurrentEra,
		/// The smart contract is already registered.
		ContractAlreadyRegistered,
		/// The smart contract is not registered.
		ContractNotRegistered,
		/// The caller is not the beneficiary authorized to claim rewards.
		NotAuthorizedToClaimRewards,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub starting_era: EraNumber,
		#[serde(skip)]
		pub _config: core::marker::PhantomData<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { starting_era: 0, _config: core::marker::PhantomData }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let era_duration = T::EraDuration::get();
			let era_info = EraInfo::<T> {
				next_era_start: era_duration
					.saturating_add(BlockNumberFor::<T>::zero())
					.saturated_into(),
				contract_fee_total: BalanceOf::<T>::zero(),
				total_fee_amount: BalanceOf::<T>::zero(),
			};
			EraInformation::<T>::insert(self.starting_era, era_info);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let now = now.saturated_into();
			let mut current_era = CurrentEra::<T>::get();
			let mut current_era_info = EraInformation::<T>::get(current_era);
			let mut consumed_weight = T::DbWeight::get().reads(2);

			// Check if is new era, if not don't do anything.
			if !current_era_info.is_new_era(now) {
				return consumed_weight;
			}
			// If is new era, update the new era information.
			current_era.saturating_inc();
			current_era_info
				.set_next_era_start(now.saturating_add(T::EraDuration::get().saturated_into()));
			current_era_info.reset_contract_fee();
			current_era_info.resent_total_fee();
			// Update storage items
			CurrentEra::<T>::put(current_era);
			EraInformation::<T>::insert(current_era, current_era_info);
			consumed_weight = T::DbWeight::get().writes(2);
			Self::deposit_event(Event::<T>::NewEra { era: current_era });
			consumed_weight
		}
		// TODO: Implement on_idle for handling rewards not claimed after X period
		// fn on_idle(_block: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
		// }
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new contract for builder incentives.
		///
		/// # Parameters
		/// - `beneficiary`: The account that will be the beneficiary of the contract incentives.
		/// - `contract`: The smart contract's account to be registered.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn register_contract(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			contract: T::AccountId,
		) -> DispatchResult {
			// TODO: Check if the caller is the contract owner or the contract itself.
			ensure_signed(origin)?;
			// TODO: Deposit to register, manager vs beneficiary?
			ensure!(
				!RegisteredContracts::<T>::contains_key(&contract),
				Error::<T>::ContractAlreadyRegistered,
			);
			// TODO: Max number of registered contracts and check it doesn't
			// ExceededMaxNumberOfContracts
			RegisteredContracts::<T>::insert(&contract, &beneficiary);
			Self::deposit_event(Event::<T>::ContractRegistered { beneficiary, contract });
			Ok(())
		}

		/// Claims rewards for a specific contract for a given era.
		///
		/// Parameters:
		/// - `contract`: The account of the smart contract for which rewards are being claimed.
		/// - `era_to_claim`: The era for which rewards are being claimed. Must be the current era.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			contract: T::AccountId,
			era_to_claim: EraNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let beneficiary = RegisteredContracts::<T>::get(&contract)
				.ok_or(Error::<T>::ContractNotRegistered)?;
			// TODO: Think if has to be called by the beneficiary, or anyone can call it.
			ensure!(beneficiary == who, Error::<T>::NotAuthorizedToClaimRewards);
			ensure!(
				CurrentEra::<T>::get() > era_to_claim,
				Error::<T>::CannotClaimRewardsForCurrentEra
			);
			// TODO: If already claimed, or is 0, so no need to calculate anything else.
			// TODO: Check if is too late to claim rewards
			let contract_fees = crate::ContractUsagePerEra::<T>::take(&contract, era_to_claim);
			let era_info = crate::EraInformation::<T>::get(era_to_claim);
			let calculated_rewards = Self::calculate_contract_share(
				contract_fees,
				era_info.contract_fee_total,
				era_info.total_fee_amount,
			);
			// Transfer the funds to the beneficiary
			<T as pallet::Config>::Currency::transfer(
				&Self::get_pallet_account(),
				&beneficiary,
				calculated_rewards,
				AllowDeath,
			)?;
			// Reset the contract fees for the era.
			crate::ContractUsagePerEra::<T>::remove(&contract, era_to_claim);
			Self::deposit_event(Event::IncentivesClaimed {
				beneficiary,
				amount: calculated_rewards,
			});
			Ok(())
		}

		/// Deposit funds into the  reward pool.
		///
		/// Parameters:
		/// - 'amount': Amount to be send.
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::zero())]
		pub fn deposit_funds(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<T as pallet::Config>::Currency::transfer(
				&who,
				&Self::get_pallet_account(),
				amount,
				AllowDeath,
			)?;
			let current_era = CurrentEra::<T>::get();
			crate::EraInformation::<T>::mutate_exists(current_era, |maybe_era_info| {
				if let Some(era_info) = maybe_era_info {
					era_info.add_total_fee(amount);
				}
			});
			Self::deposit_event(Event::IncentivesDeposited { source: who, amount });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the pallet.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		pub fn get_pallet_account() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		/// Calculate the share of the total fees that should be allocated to a specific contract.
		///
		/// The share is calculated as a proportion of the fees generated by the contract relative
		/// to the total fees generated by all contracts.
		///
		/// # Arguments
		/// - `contract_fees`: The amount of fees generated by the specific contract.
		/// - `total_contract_fees`: The total amount of fees generated by all contracts.
		/// - `total_fees`: The total amount of general fees to be distributed among the contracts.
		fn calculate_contract_share(
			contract_fees: BalanceOf<T>,
			total_contract_fees: BalanceOf<T>,
			total_fees: BalanceOf<T>,
		) -> BalanceOf<T> {
			if total_contract_fees.is_zero() || contract_fees.is_zero() {
				return BalanceOf::<T>::zero();
			}
			let proportion = Permill::from_rational(contract_fees, total_contract_fees);
			proportion * total_fees
		}

		pub fn update_incentives(amount: BalanceOf<T>) {
			let current_era = CurrentEra::<T>::get();
			crate::EraInformation::<T>::mutate_exists(current_era, |maybe_era_info| {
				if let Some(era_info) = maybe_era_info {
					era_info.add_total_fee(amount);
				}
			});
		}
	}
}
