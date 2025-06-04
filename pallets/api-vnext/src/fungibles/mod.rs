use core::cmp::Ordering::{Equal, Greater, Less};

use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
	pallet_prelude::{DispatchError, Zero},
	sp_runtime::Saturating,
	traits::fungibles::{approvals::Inspect as _, Inspect as _},
	weights::Weight,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use pallet_assets::{Config, NextAssetId};
use AddressMatcher::Fixed;

use super::*;

pub mod precompiles;

type AssetIdOf<T, I> = <T as Config<I>>::AssetId;
type BalanceOf<T, I> = <T as Config<I>>::Balance;

fn approve<T: Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	spender: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResultWithPostInfo {
	// TODO: weights
	let owner = ensure_signed(origin.clone()).map_err(|e| e.with_weight(Weight::zero()))?;
	let current_allowance = <Assets<T, I>>::allowance(asset.clone(), &owner, &spender);

	let weight = match value.cmp(&current_allowance) {
		// If the new value is equal to the current allowance, do nothing.
		Equal => Weight::zero(),
		// If the new value is greater than the current allowance, approve the difference
		// because `approve_transfer` works additively (see `pallet-assets`).
		Greater => {
			<Assets<T, I>>::approve_transfer(
				origin,
				asset.into(),
				T::Lookup::unlookup(spender),
				value.saturating_sub(current_allowance),
			)
			.map_err(|e| e.with_weight(Weight::zero()))?;
			Weight::zero()
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
			.map_err(|e| e.with_weight(Weight::zero()))?;
			if value.is_zero() {
				Weight::zero()
			} else {
				<Assets<T, I>>::approve_transfer(origin, asset.into(), spender_source, value)?;
				Weight::zero()
			}
		},
	};
	Ok(Some(weight).into())
}

fn burn<T: Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	account: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResultWithPostInfo {
	let current_balance = <Assets<T, I>>::balance(asset.clone(), &account);
	if current_balance < value {
		return Err(pallet_assets::Error::<T, I>::BalanceLow
			// TODO: weight: <T as Config>::WeightInfo::balance_of()
			.with_weight(Weight::zero()));
	}
	<Assets<T, I>>::burn(origin, asset.into(), T::Lookup::unlookup(account.clone()), value)?;
	Ok(().into())
}

fn clear_metadata<T: Config<I>, I>(origin: OriginFor<T>, asset: AssetIdOf<T, I>) -> DispatchResult {
	<Assets<T, I>>::clear_metadata(origin, asset.into())
}

fn create<T: Config<I, AssetId: Default>, I>(
	origin: OriginFor<T>,
	admin: AccountIdOf<T>,
	min_balance: BalanceOf<T, I>,
) -> Result<AssetIdOf<T, I>, DispatchError> {
	let id = NextAssetId::<T, I>::get().unwrap_or_default();
	<Assets<T, I>>::create(origin, id.clone().into(), T::Lookup::unlookup(admin), min_balance)?;
	Ok(id)
}

fn exists<T: Config<I>, I>(asset: AssetIdOf<T, I>) -> bool {
	<Assets<T, I>>::asset_exists(asset)
}

fn mint<T: Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	account: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::mint(origin, asset.into(), T::Lookup::unlookup(account), value)
}

fn set_metadata<T: Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> DispatchResult {
	<Assets<T, I>>::set_metadata(origin, asset.into(), name, symbol, decimals)
}

fn start_destroy<T: Config<I>, I>(origin: OriginFor<T>, asset: AssetIdOf<T, I>) -> DispatchResult {
	<Assets<T, I>>::start_destroy(origin, asset.into())
}

fn transfer<T: Config<I>, I>(
	origin: OriginFor<T>,
	asset: AssetIdOf<T, I>,
	to: AccountIdOf<T>,
	value: BalanceOf<T, I>,
) -> DispatchResult {
	<Assets<T, I>>::transfer(origin, asset.into(), T::Lookup::unlookup(to.clone()), value)
}

fn transfer_from<T: Config<I>, I>(
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

// TODO: replace with type in pallet_assets once available in next release
pub struct InlineAssetIdExtractor;
impl AssetIdExtractor for InlineAssetIdExtractor {
	type AssetId = u32;

	fn asset_id_from_address(addr: &[u8; 20]) -> Result<Self::AssetId, Error> {
		let bytes: [u8; 4] = addr[0..4].try_into().expect("slice is 4 bytes; qed");
		let index = u32::from_be_bytes(bytes);
		Ok(index)
	}
}
/// Mean of extracting the asset id from the precompile address.
pub trait AssetIdExtractor {
	type AssetId;
	/// Extracts the asset id from the address.
	fn asset_id_from_address(address: &[u8; 20]) -> Result<Self::AssetId, Error>;
}
