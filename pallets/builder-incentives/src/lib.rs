#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub mod contract_fee_handler;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::types::*;
	use contract_fee_handler::BalanceOf;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_transaction_payment::Config + pallet_contracts::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Describes smart contract in the context required by dApp staking.
		type SmartContract: Parameter
			+ Member
			+ MaxEncodedLen
			+ SmartContractHandle<Self::AccountId>;
		// Weight information for dispatchables in this pallet.
		// type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Regsitered Contracts.
	#[pallet::storage]
	pub type RegisteredContracts<T: Config> =
		StorageMap<_, Twox64Concat, T::SmartContract, AccountIdOf<T>>;

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
		/// A smart contract has been registered for dApp staking
		ContractRegistered { owner: T::AccountId, smart_contract: T::SmartContract },
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
			_origin: OriginFor<T>,
			owner: T::AccountId,
			smart_contract: T::SmartContract,
		) -> DispatchResult {
			// Deposit to register? Manager to register?
			ensure!(
				!RegisteredContracts::<T>::contains_key(&smart_contract),
				Error::<T>::ContractAlreadyExists,
			);
			// Check it doesn't ExceededMaxNumberOfContracts
			RegisteredContracts::<T>::insert(&smart_contract, &owner);
			Self::deposit_event(Event::<T>::ContractRegistered { owner, smart_contract });
			Ok(())
		}
		/// A contract has been called.
		///
		/// Parameters:
		/// - 'contract': The address of the contracte.
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn contract_call(origin: OriginFor<T>, contract: AccountIdOf<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			//let who = ensure_signed(origin)?;
			Self::deposit_event(Event::IncentivesClaimed { beneficiary: contract });
			Ok(())
		}
	}
}
