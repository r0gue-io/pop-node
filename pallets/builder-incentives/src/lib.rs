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
		traits::{
			fungible::Inspect, tokens::Preservation, Currency, ExistenceRequirement::KeepAlive,
			ReservableCurrency,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero};

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
	pub type RegisteredContracts<T: Config> =
		StorageMap<_, Twox64Concat, T::SmartContract, AccountIdOf<T>>;

	/// Contract usage per Era.
	#[pallet::storage]
	pub(super) type ContractUsage<T> = StorageMap<_, Twox64Concat, AccountIdOf<T>, BalanceOf<T>>;

	/// Unclaimed rewards of the beneficiary.
	#[pallet::storage]
	pub(super) type UnclaimedRewards<T> = StorageMap<_, Twox64Concat, AccountIdOf<T>, BalanceOf<T>>;

	/// General information about the current era.
	#[pallet::storage]
	pub type CurrentEraInfo<T: Config> = StorageValue<_, EraInfo<BalanceOf<T>>, ValueQuery>;

	/// Historic information about the eras.
	// #[pallet::storage]
	// pub type HistoricEraInfo<T: Config> = StorageMap<_, Twox64Concat, EraNumber, EraPaymentData>;

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
		/// A smart contract has been registered for dApp staking
		ContractRegistered { beneficiary: T::AccountId, smart_contract: T::SmartContract },
		/// A contract has been called.
		ContractCalled {
			/// The contract which has been called.
			contract: AccountIdOf<T>,
		},
	}

	/// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Smart contract already registered.
		ContractAlreadyExists,
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
			let account_id = Pallet::<T>::get_pallet_account();
			let era_duration = T::EraDuration::get();
			// Prepare current Era
			let era_info = EraInfo {
				era: self.starting_era,
				next_era_start: era_duration
					.saturating_add(BlockNumberFor::<T>::zero())
					.saturated_into(),
				amount: <T as pallet::Config>::Currency::total_balance(&account_id),
			};

			CurrentEraInfo::<T>::put(era_info);
		}
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
			smart_contract: T::SmartContract,
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
				KeepAlive,
			)?;
			CurrentEraInfo::<T>::mutate(|era| {
				era.amount += amount;
			});
			//Self::deposit_event(Event::IncentivesClaimed { beneficiary: contract });
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
	}
}
