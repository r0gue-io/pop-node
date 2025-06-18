#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::{convert::Into, marker::PhantomData, num::NonZero};

use frame_support::{dispatch::RawOrigin, sp_runtime::traits::StaticLookup};
#[cfg(feature = "fungibles")]
pub use fungibles::precompiles::{Erc20, Fungibles};
pub use pallet_revive::precompiles::alloy::primitives::U256;
use pallet_revive::{
	evm::{H160, H256},
	precompiles::{
		alloy::{sol, sol_types::SolEvent},
		AddressMatcher, Error, Ext, Precompile,
	},
	AddressMapper as _, Config, Origin,
};
#[cfg(test)]
use {
	frame_support::{
		pallet_prelude::{DispatchError, Weight},
		sp_runtime::traits::Bounded,
	},
	frame_system::pallet_prelude::OriginFor,
	pallet_revive::{precompiles::alloy::sol_types::SolValue, BalanceOf, DepositLimit, MomentOf},
};

#[cfg(feature = "fungibles")]
pub mod fungibles;
#[cfg(test)]
mod mock;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AddressMapper<T> = <T as Config>::AddressMapper;
type Assets<T, I> = pallet_assets::Pallet<T, I>;

// A bare call to a contract.
#[cfg(test)]
fn bare_call<
	T: Config,
	O: SolValue
		+ From<<O::SolType as pallet_revive::precompiles::alloy::sol_types::SolType>::RustType>,
>(
	origin: OriginFor<T>,
	dest: H160,
	value: BalanceOf<T>,
	gas_limit: Weight,
	storage_deposit_limit: DepositLimit<BalanceOf<T>>,
	data: Vec<u8>,
) -> Result<O, DispatchError>
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
	.result?;
	assert!(!result.did_revert());
	Ok(decode::<O>(&result.data))
}

// A direct call to a precompile.
#[cfg(all(test, feature = "runtime-benchmarks"))]
fn call_precompile<
	P: Precompile<T = mock::Test>,
	O: SolValue
		+ From<<O::SolType as pallet_revive::precompiles::alloy::sol_types::SolType>::RustType>,
>(
	ext: &mut impl pallet_revive::precompiles::ExtWithInfo<T = mock::Test>,
	address: &[u8; 20],
	input: &P::Interface,
) -> Result<O, Error> {
	pallet_revive::precompiles::run::precompile::<P, _>(ext, address, input).map(|o| decode(&o))
}

#[cfg(test)]
fn decode<
	T: SolValue
		+ From<<T::SolType as pallet_revive::precompiles::alloy::sol_types::SolType>::RustType>,
>(
	data: &[u8],
) -> T {
	T::abi_decode(data).expect("unable to decode")
}

fn deposit_event<T: Config>(
	_env: &mut impl Ext<T = T>,
	address: impl Into<H160>,
	event: impl SolEvent,
) {
	// TODO: ensure that env.deposit_event impl is correct
	let topics = topics(&event);
	let data = event.encode_data();
	// env.deposit_event(topics, data);
	// TODO: charge gas
	// Workaround to emit events explicitly as precompile, as env.deposit uses derived address of
	// dummy contract when using `CallSetup` within tests
	let revive_event =
		pallet_revive::Event::ContractEmitted { contract: address.into(), data, topics };
	<frame_system::Pallet<T>>::deposit_event(<T as Config>::RuntimeEvent::from(revive_event))
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

#[cfg(test)]
fn to_address(account: &<mock::Test as frame_system::Config>::AccountId) -> H160 {
	pallet_revive::AccountId32Mapper::<mock::Test>::to_address(account)
}

// TODO: verify
fn topics(event: &impl SolEvent) -> Vec<H256> {
	event.encode_topics().into_iter().map(|t| (*t.0).into()).collect()
}

/// Creates a new `RuntimeOrigin` from an ['Origin'].
// TODO: upstream?
pub fn to_runtime_origin<T: Config>(o: Origin<T>) -> T::RuntimeOrigin {
	match o {
		Origin::Root => RawOrigin::Root.into(),
		Origin::Signed(account) => RawOrigin::Signed(account).into(),
	}
}
