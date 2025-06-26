//! The fungibles API offers a streamlined interface for interacting with fungible tokens. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

use core::cmp::Ordering::{Equal, Greater, Less};

use frame_support::{
	dispatch::{
		DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo,
	},
	pallet_prelude::{CheckedSub, DispatchError, Zero},
	sp_runtime::Saturating,
	traits::fungibles::{approvals::Inspect as _, Inspect as _},
	weights::Weight,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
pub use pallet::*;
use pallet_assets::NextAssetId;
pub use precompiles::{Erc20, Fungibles};
use weights::WeightInfo;
use AddressMatcher::Fixed;

use super::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
/// The fungibles precompiles offer a streamlined interface for interacting with fungible tokens.
pub mod precompiles;
#[cfg(test)]
mod tests;
pub mod weights;

type AssetIdOf<T, I> = <T as pallet_assets::Config<I>>::AssetId;
type BalanceOf<T, I> = <T as pallet_assets::Config<I>>::Balance;
type WeightOf<T, I> = <T as Config<I>>::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::weights::WeightInfo;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// Weight information for precompiles in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);
}

fn approve<T: Config<I> + pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	spender: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResultWithPostInfo {
	let owner = ensure_signed(origin.clone())
		.map_err(|e| e.with_weight(<WeightOf<T, I>>::approve(0, 0)))?;
	let current_allowance = <Assets<T, I>>::allowance(asset.clone(), &owner, &spender);

	let weight = match value.cmp(&current_allowance) {
		// If the new value is equal to the current allowance, do nothing.
		Equal => <WeightOf<T, I>>::approve(0, 0),
		// If the new value is greater than the current allowance, approve the difference
		// because `approve_transfer` works additively (see `pallet-assets`).
		Greater => {
			<Assets<T, I>>::approve_transfer(
				origin,
				asset.into(),
				T::Lookup::unlookup(spender),
				value.saturating_sub(current_allowance),
			)
			.map_err(|e| e.with_weight(<WeightOf<T, I>>::approve(1, 0)))?;
			<WeightOf<T, I>>::approve(1, 0)
		},
		// If the new value is less than the current allowance, cancel the approval and
		// set the new value.
		Less => {
			let spender_source = T::Lookup::unlookup(spender);
			<Assets<T, I>>::cancel_approval(
				origin.clone(),
				asset.clone().into(),
				spender_source.clone(),
			)
			.map_err(|e| e.with_weight(<WeightOf<T, I>>::approve(0, 1)))?;
			if value.is_zero() {
				<WeightOf<T, I>>::approve(0, 1)
			} else {
				<Assets<T, I>>::approve_transfer(origin, asset.into(), spender_source, value)?;
				<WeightOf<T, I>>::approve(1, 1)
			}
		},
	};
	Ok(Some(weight).into())
}

fn burn<T: Config<I> + pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	account: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResultWithPostInfo {
	let current_balance = <Assets<T, I>>::balance(asset.clone(), &account);
	if current_balance < value {
		return Err(pallet_assets::Error::<T, I>::BalanceLow
			.with_weight(<T as Config<I>>::WeightInfo::balance_of()));
	}
	<Assets<T, I>>::burn(origin, asset.into(), T::Lookup::unlookup(account.clone()), value)?;
	Ok(().into())
}

fn clear_metadata<T: pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::clear_metadata(origin, asset.into())
}

fn create<T: pallet_assets::Config<I, AssetId: Default>, I>(
	origin: OriginFor<T>,
	admin: AccountIdOf<T>,
	min_balance: BalanceOf<T, I>,
) -> Result<AssetIdOf<T, I>, DispatchError> {
	let id = NextAssetId::<T, I>::get().unwrap_or_default();
	<Assets<T, I>>::create(origin, id.clone().into(), T::Lookup::unlookup(admin), min_balance)?;
	Ok(id)
}

fn decrease_allowance<T: Config<I> + pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	spender: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> Result<(BalanceOf<T, I>, Option<Weight>), DispatchErrorWithPostInfo> {
	let owner = ensure_signed(origin.clone())
		.map_err(|e| e.with_weight(<WeightOf<T, I>>::approve(0, 0)))?;
	if value.is_zero() {
		return Ok((value, Some(<WeightOf<T, I>>::approve(0, 0))));
	}
	let current_allowance = <Assets<T, I>>::allowance(asset.clone(), &owner, &spender);
	let spender_source = T::Lookup::unlookup(spender.clone());
	let asset_param: <T as pallet_assets::Config<I>>::AssetIdParameter = asset.clone().into();

	// Cancel the approval and approve `new_allowance` if difference is more than zero.
	let new_allowance = current_allowance
		.checked_sub(&value)
		.ok_or(pallet_assets::Error::<T, I>::Unapproved)?;
	<Assets<T, I>>::cancel_approval(origin.clone(), asset_param.clone(), spender_source.clone())
		.map_err(|e| e.with_weight(<WeightOf<T, I>>::approve(0, 1)))?;
	let weight = if new_allowance.is_zero() {
		<WeightOf<T, I>>::approve(0, 1)
	} else {
		<Assets<T, I>>::approve_transfer(origin, asset_param, spender_source, new_allowance)?;
		<WeightOf<T, I>>::approve(1, 1)
	};
	Ok((new_allowance, Some(weight).into()))
}

fn exists<T: pallet_assets::Config<I>, I>(asset: AssetIdOf<T, I>) -> bool {
	<Assets<T, I>>::asset_exists(asset)
}

fn increase_allowance<T: Config<I> + pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	spender: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> Result<BalanceOf<T, I>, DispatchErrorWithPostInfo> {
	let owner = ensure_signed(origin.clone())
		.map_err(|e| e.with_weight(<WeightOf<T, I>>::approve(0, 0)))?;
	<Assets<T, I>>::approve_transfer(
		origin,
		asset.clone().into(),
		T::Lookup::unlookup(spender.clone()),
		value,
	)
	.map_err(|e| {
		e.with_weight(
			<<T as pallet_assets::Config<I>>::WeightInfo as pallet_assets::WeightInfo>::approve_transfer(),
		)
	})?;
	Ok(<Assets<T, I>>::allowance(asset, &owner, &spender))
}

fn mint<T: pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	account: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::mint(origin, asset.into(), T::Lookup::unlookup(account), value)
}

fn set_metadata<T: pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> DispatchResult {
	<Assets<T, I>>::set_metadata(origin, asset.into(), name, symbol, decimals)
}

fn start_destroy<T: pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::start_destroy(origin, asset.into())
}

fn total_supply<T: pallet_assets::Config<I>, I>(asset: AssetIdOf<T, I>) -> T::Balance {
	<Assets<T, I>>::total_supply(asset)
}

fn transfer<T: pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	to: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::transfer(origin, asset.into(), T::Lookup::unlookup(to.clone()), value)
}

fn transfer_from<T: pallet_assets::Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	from: AccountIdOf<T>,
	to: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::transfer_approved(
		origin,
		asset.into(),
		T::Lookup::unlookup(from),
		T::Lookup::unlookup(to),
		value,
	)
}
