//! The fungibles pallet offers a streamlined interface for interacting with fungible assets. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

use frame_support::traits::fungibles::Inspect;
pub use pallet::*;
use pallet_assets::WeightInfo as AssetsWeightInfoTrait;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::AssetId;
type AssetsOf<T> = pallet_assets::Pallet<T, AssetsInstanceOf<T>>;
type AssetsInstanceOf<T> = <T as Config>::AssetsInstance;
type AssetsWeightInfoOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::WeightInfo;
type BalanceOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::StaticLookup;
	use sp_std::vec::Vec;

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		#[codec(index = 0)]
		Owner(AccountIdOf<T>),
		#[codec(index = 1)]
		CollectionOwner(AccountIdOf<T>),
		#[codec(index = 2)]
		Attribute(AccountIdOf<T>),
		/// TODO: Handle the system attribute and custom attribute.
		#[codec(index = 3)]
		CustomAttribute(AccountIdOf<T>),
		#[codec(index = 4)]
		CollectionAttribute(AccountIdOf<T>),
		#[codec(index = 5)]
		Transferable(AccountIdOf<T>),
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet assets it is tightly coupled to.
		type AssetsInstance;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when allowance by `owner` to `spender` changes.
		Approval {
			/// The asset.
			asset: AssetIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
			/// The new allowance amount.
			value: BalanceOf<T>,
		},
		/// Event emitted when an asset transfer occurs.
		Transfer {
			/// The asset.
			asset: AssetIdOf<T>,
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The amount transferred (or minted/burned).
			value: BalanceOf<T>,
		},
		/// Event emitted when an asset is created.
		Create {
			/// The asset identifier.
			id: AssetIdOf<T>,
			/// The creator of the asset.
			creator: AccountIdOf<T>,
			/// The administrator of the asset.
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`.
		///
		/// # Parameters
		/// - `asset` - The asset to transfer.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			AssetsOf::<T>::transfer_keep_alive(
				origin,
				asset.clone().into(),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { asset, from: Some(from), to: Some(to), value });
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
