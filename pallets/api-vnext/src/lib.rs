#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::{convert::Into, marker::PhantomData, num::NonZero};

use frame_support::{
	dispatch::RawOrigin,
	pallet_prelude::DispatchError,
	sp_runtime::{traits::StaticLookup, ArithmeticError},
};
#[cfg(any(test, feature = "runtime-benchmarks"))]
use pallet_revive::precompiles::alloy::sol_types::{SolType, SolValue};
use pallet_revive::{
	precompiles::{
		alloy::{
			primitives::{self as alloy, IntoLogData},
			sol,
			sol_types::SolEvent,
		},
		AddressMatcher, Error, Ext, Precompile,
	},
	AddressMapper as _, Origin, H256,
};
#[cfg(feature = "runtime-benchmarks")]
use {
	frame_support::{pallet_prelude::IsType, traits::fungible::Inspect},
	pallet_revive::evm::U256,
	pallet_revive::precompiles::run::{CallSetup, WasmModule},
};
#[cfg(test)]
use {
	frame_support::{pallet_prelude::Weight, sp_runtime::traits::Bounded},
	frame_system::pallet_prelude::OriginFor,
	pallet_revive::{
		precompiles::alloy::sol_types::{Revert, SolError},
		BalanceOf, DepositLimit, MomentOf, H160,
	},
};

mod errors;
#[cfg(feature = "fungibles")]
pub mod fungibles;
#[cfg(test)]
mod mock;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AddressMapper<T> = <T as pallet_revive::Config>::AddressMapper;
type Assets<T, I> = pallet_assets::Pallet<T, I>;

// A bare call to a contract.
#[cfg(test)]
fn bare_call<
	T: pallet_revive::Config,
	O: SolValue
		+ From<<O::SolType as pallet_revive::precompiles::alloy::sol_types::SolType>::RustType>,
>(
	origin: OriginFor<T>,
	dest: H160,
	value: BalanceOf<T>,
	gas_limit: Weight,
	storage_deposit_limit: DepositLimit<BalanceOf<T>>,
	data: Vec<u8>,
) -> Result<O, pallet_revive::precompiles::Error>
where
	BalanceOf<T>: Into<pallet_revive::evm::U256> + TryFrom<pallet_revive::evm::U256> + Bounded,
	MomentOf<T>: Into<pallet_revive::evm::U256>,
	T::Hash: frame_support::traits::IsType<H256>,
{
	let result = pallet_revive::Pallet::<T>::bare_call(
		origin,
		dest,
		value,
		gas_limit,
		storage_deposit_limit,
		data,
	)
	.result
	.map_err(|e| Error::Error(e.into()))?;
	match result.did_revert() {
		true => {
			let revert = Revert::abi_decode(&result.data).expect("revert data is invalid");
			Err(Error::Revert(revert))
		},
		false => Ok(decode::<O>(&result.data)),
	}
}

// A direct call to a precompile.
#[cfg(feature = "runtime-benchmarks")]
fn call_precompile<
	P: Precompile<T = E::T>,
	E: pallet_revive::precompiles::ExtWithInfo<
		T: pallet_revive::Config<
			Currency: frame_support::traits::fungible::Inspect<
				<E::T as frame_system::Config>::AccountId,
				Balance: Into<U256> + TryFrom<U256>,
			>,
			Hash: frame_support::traits::IsType<H256>,
			Time: frame_support::traits::Time<Moment: Into<U256>>,
		>,
	>,
	O: SolValue
		+ From<<O::SolType as pallet_revive::precompiles::alloy::sol_types::SolType>::RustType>,
>(
	ext: &mut E,
	address: &[u8; 20],
	input: &P::Interface,
) -> Result<O, Error> {
	pallet_revive::precompiles::run::precompile::<P, _>(ext, address, input).map(|o| decode(&o))
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
fn decode<T: SolValue + From<<T::SolType as SolType>::RustType>>(data: &[u8]) -> T {
	T::abi_decode(data).expect("unable to decode")
}

fn deposit_event<T: pallet_revive::Config>(
	env: &mut impl Ext<T = T>,
	event: impl SolEvent + IntoLogData,
) -> Result<(), Error> {
	// Source: https://github.com/paritytech/polkadot-sdk/blob/e1026d7ee22a593cf566a99484eee02a03ecc236/substrate/frame/assets/src/precompiles.rs#L152
	let (topics, data) = event.into_log_data().split();
	let topics = topics.into_iter().map(|v| H256(v.0)).collect::<Vec<_>>();
	env.gas_meter_mut()
		.charge(pallet_revive::precompiles::RuntimeCosts::DepositEvent {
			num_topic: topics.len() as u32,
			len: topics.len() as u32,
		})?;
	env.deposit_event(topics, data.to_vec());
	Ok(())
}

#[cfg(test)]
fn assert_last_event(address: impl Into<H160>, event: impl SolEvent) {
	mock::System::assert_last_event(
		pallet_revive::Event::ContractEmitted {
			contract: address.into(),
			data: event.encode_data(),
			topics: topics(&event),
		}
		.into(),
	)
}

const fn fixed_address(n: u16) -> [u8; 20] {
	AddressMatcher::Fixed(NonZero::new(n).unwrap()).base_address()
}

fn prefixed_address(n: u16, prefix: u32) -> [u8; 20] {
	let mut address = AddressMatcher::Prefix(NonZero::new(n).unwrap()).base_address();
	address[..4].copy_from_slice(&prefix.to_be_bytes());
	address
}

#[cfg(feature = "runtime-benchmarks")]
fn set_up_call<
	T: pallet_revive::Config<
		Currency: Inspect<
			<T as frame_system::Config>::AccountId,
			Balance: Into<U256> + TryFrom<U256>,
		>,
		Hash: IsType<H256>,
		Time: frame_support::traits::Time<Moment: Into<U256>>,
	>,
>() -> CallSetup<T> {
	CallSetup::<T>::new(WasmModule::dummy())
}

#[cfg(test)]
fn to_address(account: &<mock::Test as frame_system::Config>::AccountId) -> H160 {
	pallet_revive::AccountId32Mapper::<mock::Test>::to_address(account)
}

#[cfg(test)]
fn topics(event: &impl SolEvent) -> Vec<H256> {
	event.encode_topics().into_iter().map(|t| (*t.0).into()).collect()
}

/// Creates a new `RuntimeOrigin` from an ['Origin'].
pub fn to_runtime_origin<T: pallet_revive::Config>(o: Origin<T>) -> T::RuntimeOrigin {
	match o {
		Origin::Root => RawOrigin::Root.into(),
		Origin::Signed(account) => RawOrigin::Signed(account).into(),
	}
}

/// Extension trait for ergonomic conversion between types.
pub trait TryConvert<T> {
	/// The type returned in the event of a conversion error.
	type Error;

	/// Attempt to convert to target type.
	fn try_convert(self) -> Result<T, Self::Error>;
}

impl TryConvert<alloy::U256> for u128 {
	type Error = DispatchError;

	fn try_convert(self) -> Result<alloy::U256, Self::Error> {
		self.try_into().map_err(|_| DispatchError::from(ArithmeticError::Overflow))
	}
}

impl<T: TryFrom<alloy::U256>> TryConvert<T> for alloy::U256 {
	type Error = DispatchError;

	fn try_convert(self) -> Result<T, Self::Error> {
		self.try_into().map_err(|_| DispatchError::from(ArithmeticError::Overflow))
	}
}
