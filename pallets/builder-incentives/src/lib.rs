#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub mod contract_fee_handler;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::types::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero},
		Permill,
	};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_transaction_payment::Config + pallet_contracts::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// Number of blocks of an era.
		#[pallet::constant]
		type EraDuration: Get<BlockNumberFor<Self>>;
		/// Describes smart contract in the context required by dApp staking.
		type SmartContract: Parameter
			+ Member
			+ MaxEncodedLen
			+ SmartContractHandle<Self::AccountId>;
		/// The pallet's id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		// Weight information for dispatchables in this pallet.
		// type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Regsitered Contracts.
	#[pallet::storage]
	pub type RegisteredContracts<T> = StorageMap<_, Twox64Concat, AccountIdOf<T>, AccountIdOf<T>>;

	// TODO: Account ID instead of SmartContract
	/// Contract usage per Era.
	#[pallet::storage]
	pub(super) type ContractUsage<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountIdOf<T>,
		Twox64Concat,
		EraNumber,
		BalanceOf<T>,
		ValueQuery,
	>;

	/// General information about the eras.
	#[pallet::storage]
	pub type ErasInfo<T> = StorageMap<_, Twox64Concat, EraNumber, EraInfo<T>, ValueQuery>;

	/// The current era.
	#[pallet::storage]
	pub(super) type CurrentEra<T> = StorageValue<_, EraNumber, ValueQuery>;

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has successfully set a new value.
		IncentivesClaimed {
			/// The beneficiary of the incentives.
			beneficiary: AccountIdOf<T>,
			// The rewared amount.
			value: BalanceOf<T>,
		},
		/// A smart contract has been registered for dApp staking
		ContractRegistered { beneficiary: T::AccountId, smart_contract: T::AccountId },
		/// A contract has been called.
		ContractCalled {
			/// The contract which has been called.
			contract: AccountIdOf<T>,
		},
		/// New rewards into the Pallet.
		IncentivesAdded {
			/// The beneficiary of the incentives.
			who: AccountIdOf<T>,
			// The rewared amount.
			value: BalanceOf<T>,
		},
		/// New era has started.
		NewEra { era: EraNumber },
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Smart contract already registered.
		ContractAlreadyExists,
		/// Smart contract is not registered.
		ContractIsNotRegistered,
		/// Not the beneficiary to claim rewards.
		NotTheBeneficiary,
		/// Can not claim rewards for the current era.
		CanNotClaimRewardsCurrentEra,
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
			ErasInfo::<T>::insert(self.starting_era, era_info);
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let now = now.saturated_into();
			let mut current_era = CurrentEra::<T>::get();
			let mut current_era_info = ErasInfo::<T>::get(current_era);
			let mut consumed_weight = T::DbWeight::get().reads(2);

			// Nothing to do if it's not new era
			if !current_era_info.is_new_era(now) {
				return consumed_weight;
			}
			// If is new era, update the era info
			current_era.saturating_inc();
			current_era_info
				.set_next_era_start(now.saturating_add(T::EraDuration::get().saturated_into()));
			current_era_info.reset_contract_fee();
			current_era_info.resent_total_fee();
			// Update storage items
			CurrentEra::<T>::put(current_era);
			ErasInfo::<T>::insert(current_era, current_era_info);
			consumed_weight = T::DbWeight::get().writes(2);
			Self::deposit_event(Event::<T>::NewEra { era: current_era });
			consumed_weight
		}
		// TODO: Implement on_idle for handling rewards not claimed after X period
		// fn on_idle(_block: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
		// 	// Clean up if rewards not claimed
		// 	T::DbWeight::get().reads(0)
		// }
	}

	/// The dispatchable functions available.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Used to register a new contract for builder incentives.
		///
		/// If successful, smart contract will be assigned a simple, unique numerical identifier.
		/// Owner is set to be initial beneficiary & manager of the dApp.
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn register(
			origin: OriginFor<T>,
			beneficiary: T::AccountId,
			smart_contract: T::AccountId,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// Deposit to register? Manager to register?
			ensure!(
				!RegisteredContracts::<T>::contains_key(&smart_contract),
				Error::<T>::ContractAlreadyExists,
			);
			// Check it doesn't ExceededMaxNumberOfContracts
			RegisteredContracts::<T>::insert(&smart_contract, &beneficiary);
			Self::deposit_event(Event::<T>::ContractRegistered { beneficiary, smart_contract });
			Ok(())
		}

		/// Send a payment to the pallet.
		///
		/// Parameters:
		/// - 'amount': Amount to be send.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn contribute(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Transfer the funds to the pallet's account
			<T as pallet::Config>::Currency::transfer(
				&who,
				&Self::get_pallet_account(),
				amount,
				AllowDeath,
			)?;
			let current_era = CurrentEra::<T>::get();
			crate::ErasInfo::<T>::mutate_exists(current_era, |maybe_era_info| {
				if let Some(era_info) = maybe_era_info {
					era_info.add_total_fee(amount);
				}
			});
			Self::deposit_event(Event::IncentivesAdded { who, value: amount });
			Ok(())
		}

		/// Send the rewards for the contract (has to be called by the beneficiary?).
		///
		/// Parameters:
		/// - 'amount': Amount to be send.
		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			smart_contract: T::AccountId,
			era_to_claim: EraNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let beneficiary = RegisteredContracts::<T>::get(&smart_contract)
				.ok_or(Error::<T>::ContractIsNotRegistered)?;
			// has to be called by the beneficiary? why?
			ensure!(beneficiary == who, Error::<T>::NotTheBeneficiary);
			ensure!(
				CurrentEra::<T>::get() == era_to_claim,
				Error::<T>::CanNotClaimRewardsCurrentEra
			);
			let contract_fees = crate::ContractUsage::<T>::take(&smart_contract, era_to_claim);
			let era_info = crate::ErasInfo::<T>::get(era_to_claim);
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
			// Reset values
			crate::ContractUsage::<T>::insert(
				&smart_contract,
				era_to_claim,
				BalanceOf::<T>::zero(),
			);
			Self::deposit_event(Event::IncentivesClaimed {
				beneficiary,
				value: calculated_rewards,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of the pallet.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache
		/// the value and only call this once.
		fn get_pallet_account() -> T::AccountId {
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
	}
}
