//! Benchmarking setup for pallet_api::nonfungibles

use frame_benchmarking::{account, v2::*};
use sp_runtime::traits::Zero;

use super::{
	AccountIdOf, BalanceOf, Call, CollectionIdOf, Config, NftsInstanceOf, NftsOf, Pallet, Read,
};
use crate::Read as _;

const SEED: u32 = 1;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	// Storage: ???
	fn total_supply() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn balance_of() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn allowance() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn owner_of() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn get_attribute() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn collection() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn next_collection_id() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	fn item_metadata() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}
}
