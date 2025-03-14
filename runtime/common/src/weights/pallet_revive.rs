
//! Autogenerated weights for `pallet_revive`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.0.0
//! DATE: 2025-02-25, STEPS: `2`, REPEAT: `1`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `tux`, CPU: `12th Gen Intel(R) Core(TM) i7-12700H`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("./scripts/pop-raw.json")`, DB CACHE: 1024

// Executed Command:
// ./target/release/pop-node
// benchmark
// pallet
// --chain=./scripts/pop-raw.json
// --genesis-builder-preset=pop-mainnet
// --pallet=*
// --extrinsic=*
// --steps=2
// --repeat=1
// --output=./runtime/mainnet/src/weights
// --exclude-pallets=pallet_collator_selection,pallet_xcm_benchmarks::generic,pallet_xcm_benchmarks::fungible,pallet_xcm

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_revive`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_revive::WeightInfo for WeightInfo<T> {
	/// Storage: `Revive::DeletionQueueCounter` (r:1 w:0)
	/// Proof: `Revive::DeletionQueueCounter` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `Measured`)
	fn on_process_deletion_queue_batch() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `1594`
		// Minimum execution time: 2_932_000 picoseconds.
		Weight::from_parts(2_932_000, 0)
			.saturating_add(Weight::from_parts(0, 1594))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `k` is `[0, 1024]`.
	fn on_initialize_per_trie_key(_k: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `297 + k * (69 ±0)`
		//  Estimated: `71574`
		// Minimum execution time: 16_595_000 picoseconds.
		Weight::from_parts(1_176_157_000, 0)
			.saturating_add(Weight::from_parts(0, 71574))
			.saturating_add(T::DbWeight::get().reads(1026))
			.saturating_add(T::DbWeight::get().writes(1026))
	}
	/// Storage: `Revive::AddressSuffix` (r:2 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Revive::ContractInfoOf` (r:1 w:1)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Revive::CodeInfoOf` (r:1 w:0)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:1 w:0)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	/// The range of component `c` is `[0, 262144]`.
	fn call_with_code_per_byte(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1326`
		//  Estimated: `7266`
		// Minimum execution time: 102_280_000 picoseconds.
		Weight::from_parts(102_330_000, 0)
			.saturating_add(Weight::from_parts(0, 7266))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:2 w:2)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(157), added: 2632, mode: `Measured`)
	/// Storage: `Revive::AddressSuffix` (r:1 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Revive::ContractInfoOf` (r:1 w:1)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:0 w:1)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// The range of component `c` is `[0, 262144]`.
	/// The range of component `i` is `[0, 262144]`.
	fn instantiate_with_code(c: u32, i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `227`
		//  Estimated: `6167`
		// Minimum execution time: 205_153_000 picoseconds.
		Weight::from_parts(205_153_000, 0)
			.saturating_add(Weight::from_parts(0, 6167))
			// Standard Error: 735
			.saturating_add(Weight::from_parts(349, 0).saturating_mul(c.into()))
			// Standard Error: 735
			.saturating_add(Weight::from_parts(3_916, 0).saturating_mul(i.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(6))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:1 w:0)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// Storage: `Revive::AddressSuffix` (r:1 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Revive::ContractInfoOf` (r:1 w:1)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(157), added: 2632, mode: `Measured`)
	/// The range of component `i` is `[0, 262144]`.
	fn instantiate(_i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1117`
		//  Estimated: `4582`
		// Minimum execution time: 306_525_000 picoseconds.
		Weight::from_parts(1_218_621_000, 0)
			.saturating_add(Weight::from_parts(0, 4582))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(4))
	}
	/// Storage: `Revive::AddressSuffix` (r:2 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Revive::ContractInfoOf` (r:1 w:1)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Revive::CodeInfoOf` (r:1 w:0)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:1 w:0)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// Storage: `Timestamp::Now` (r:1 w:0)
	/// Proof: `Timestamp::Now` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	fn call() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1326`
		//  Estimated: `7266`
		// Minimum execution time: 167_414_000 picoseconds.
		Weight::from_parts(167_414_000, 0)
			.saturating_add(Weight::from_parts(0, 7266))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(157), added: 2632, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:0 w:1)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// The range of component `c` is `[0, 262144]`.
	fn upload_code(_c: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `3574`
		// Minimum execution time: 61_955_000 picoseconds.
		Weight::from_parts(69_767_000, 0)
			.saturating_add(Weight::from_parts(0, 3574))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(157), added: 2632, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:0 w:1)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	fn remove_code() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `281`
		//  Estimated: `3746`
		// Minimum execution time: 45_994_000 picoseconds.
		Weight::from_parts(45_994_000, 0)
			.saturating_add(Weight::from_parts(0, 3746))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Revive::ContractInfoOf` (r:1 w:1)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Revive::CodeInfoOf` (r:2 w:2)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	fn set_code() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `523`
		//  Estimated: `6463`
		// Minimum execution time: 29_004_000 picoseconds.
		Weight::from_parts(29_004_000, 0)
			.saturating_add(Weight::from_parts(0, 6463))
			.saturating_add(T::DbWeight::get().reads(3))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: `Revive::AddressSuffix` (r:1 w:1)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(157), added: 2632, mode: `Measured`)
	fn map_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `3574`
		// Minimum execution time: 45_776_000 picoseconds.
		Weight::from_parts(45_776_000, 0)
			.saturating_add(Weight::from_parts(0, 3574))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: `Balances::Holds` (r:1 w:1)
	/// Proof: `Balances::Holds` (`max_values`: None, `max_size`: Some(157), added: 2632, mode: `Measured`)
	/// Storage: `Revive::AddressSuffix` (r:0 w:1)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	fn unmap_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `55`
		//  Estimated: `3520`
		// Minimum execution time: 35_463_000 picoseconds.
		Weight::from_parts(35_463_000, 0)
			.saturating_add(Weight::from_parts(0, 3520))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn dispatch_as_fallback_account() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 7_712_000 picoseconds.
		Weight::from_parts(7_712_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `r` is `[0, 1600]`.
	fn noop_host_fn(_r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_868_000 picoseconds.
		Weight::from_parts(266_939_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_caller() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 464_000 picoseconds.
		Weight::from_parts(464_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 348_000 picoseconds.
		Weight::from_parts(348_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Revive::ContractInfoOf` (r:1 w:0)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	fn seal_is_contract() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `306`
		//  Estimated: `3771`
		// Minimum execution time: 7_242_000 picoseconds.
		Weight::from_parts(7_242_000, 0)
			.saturating_add(Weight::from_parts(0, 3771))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Revive::ContractInfoOf` (r:1 w:0)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	fn seal_code_hash() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `403`
		//  Estimated: `3868`
		// Minimum execution time: 8_461_000 picoseconds.
		Weight::from_parts(8_461_000, 0)
			.saturating_add(Weight::from_parts(0, 3868))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	fn seal_own_code_hash() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 552_000 picoseconds.
		Weight::from_parts(552_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Revive::ContractInfoOf` (r:1 w:0)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Revive::CodeInfoOf` (r:1 w:0)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	fn seal_code_size() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `470`
		//  Estimated: `3935`
		// Minimum execution time: 12_553_000 picoseconds.
		Weight::from_parts(12_553_000, 0)
			.saturating_add(Weight::from_parts(0, 3935))
			.saturating_add(T::DbWeight::get().reads(2))
	}
	fn seal_caller_is_origin() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 511_000 picoseconds.
		Weight::from_parts(511_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_caller_is_root() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 452_000 picoseconds.
		Weight::from_parts(452_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_address() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 367_000 picoseconds.
		Weight::from_parts(367_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_weight_left() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_108_000 picoseconds.
		Weight::from_parts(1_108_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_balance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `103`
		//  Estimated: `0`
		// Minimum execution time: 5_183_000 picoseconds.
		Weight::from_parts(5_183_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Revive::AddressSuffix` (r:1 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	fn seal_balance_of() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `264`
		//  Estimated: `3729`
		// Minimum execution time: 9_165_000 picoseconds.
		Weight::from_parts(9_165_000, 0)
			.saturating_add(Weight::from_parts(0, 3729))
			.saturating_add(T::DbWeight::get().reads(2))
	}
	/// Storage: `Revive::ImmutableDataOf` (r:1 w:0)
	/// Proof: `Revive::ImmutableDataOf` (`max_values`: None, `max_size`: Some(4118), added: 6593, mode: `Measured`)
	/// The range of component `n` is `[1, 4096]`.
	fn seal_get_immutable_data(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `233 + n * (1 ±0)`
		//  Estimated: `7799`
		// Minimum execution time: 6_432_000 picoseconds.
		Weight::from_parts(9_483_000, 0)
			.saturating_add(Weight::from_parts(0, 7799))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Revive::ImmutableDataOf` (r:0 w:1)
	/// Proof: `Revive::ImmutableDataOf` (`max_values`: None, `max_size`: Some(4118), added: 6593, mode: `Measured`)
	/// The range of component `n` is `[1, 4096]`.
	fn seal_set_immutable_data(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_953_000 picoseconds.
		Weight::from_parts(6_304_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn seal_value_transferred() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 442_000 picoseconds.
		Weight::from_parts(442_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_minimum_balance() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 412_000 picoseconds.
		Weight::from_parts(412_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_block_number() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 394_000 picoseconds.
		Weight::from_parts(394_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `System::BlockHash` (r:1 w:0)
	/// Proof: `System::BlockHash` (`max_values`: None, `max_size`: Some(44), added: 2519, mode: `Measured`)
	fn seal_block_hash() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `30`
		//  Estimated: `3495`
		// Minimum execution time: 4_345_000 picoseconds.
		Weight::from_parts(4_345_000, 0)
			.saturating_add(Weight::from_parts(0, 3495))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	fn seal_now() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 368_000 picoseconds.
		Weight::from_parts(368_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_weight_to_fee() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_699_000 picoseconds.
		Weight::from_parts(2_699_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 262140]`.
	fn seal_input(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 641_000 picoseconds.
		Weight::from_parts(32_465_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 262140]`.
	fn seal_return(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 687_000 picoseconds.
		Weight::from_parts(42_841_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Revive::AddressSuffix` (r:1 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Revive::DeletionQueueCounter` (r:1 w:1)
	/// Proof: `Revive::DeletionQueueCounter` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `Measured`)
	/// Storage: `Revive::CodeInfoOf` (r:33 w:33)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::DeletionQueue` (r:0 w:1)
	/// Proof: `Revive::DeletionQueue` (`max_values`: None, `max_size`: Some(142), added: 2617, mode: `Measured`)
	/// Storage: `Revive::ImmutableDataOf` (r:0 w:1)
	/// Proof: `Revive::ImmutableDataOf` (`max_values`: None, `max_size`: Some(4118), added: 6593, mode: `Measured`)
	/// The range of component `n` is `[0, 32]`.
	fn seal_terminate(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `316 + n * (85 ±0)`
		//  Estimated: `85707`
		// Minimum execution time: 22_235_000 picoseconds.
		Weight::from_parts(149_904_000, 0)
			.saturating_add(Weight::from_parts(0, 85707))
			.saturating_add(T::DbWeight::get().reads(35))
			.saturating_add(T::DbWeight::get().writes(36))
	}
	/// The range of component `t` is `[0, 4]`.
	/// The range of component `n` is `[0, 512]`.
	fn seal_deposit_event(t: u32, n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 6_110_000 picoseconds.
		Weight::from_parts(3_869_500, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 264_354
			.saturating_add(Weight::from_parts(575_625, 0).saturating_mul(t.into()))
			// Standard Error: 2_065
			.saturating_add(Weight::from_parts(4_375, 0).saturating_mul(n.into()))
	}
	/// The range of component `i` is `[0, 262144]`.
	fn seal_debug_message(_i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 538_000 picoseconds.
		Weight::from_parts(242_838_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn get_storage_empty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `749`
		//  Estimated: `749`
		// Minimum execution time: 11_667_000 picoseconds.
		Weight::from_parts(11_667_000, 0)
			.saturating_add(Weight::from_parts(0, 749))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn get_storage_full() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `10759`
		//  Estimated: `10759`
		// Minimum execution time: 47_008_000 picoseconds.
		Weight::from_parts(47_008_000, 0)
			.saturating_add(Weight::from_parts(0, 10759))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_storage_empty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `749`
		//  Estimated: `749`
		// Minimum execution time: 11_090_000 picoseconds.
		Weight::from_parts(11_090_000, 0)
			.saturating_add(Weight::from_parts(0, 749))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn set_storage_full() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `10759`
		//  Estimated: `10759`
		// Minimum execution time: 42_072_000 picoseconds.
		Weight::from_parts(42_072_000, 0)
			.saturating_add(Weight::from_parts(0, 10759))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[0, 512]`.
	/// The range of component `o` is `[0, 512]`.
	fn seal_set_storage(_n: u32, o: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250 + o * (1 ±0)`
		//  Estimated: `250 + o * (1 ±0)`
		// Minimum execution time: 10_971_000 picoseconds.
		Weight::from_parts(14_195_000, 0)
			.saturating_add(Weight::from_parts(0, 250))
			// Standard Error: 1_434
			.saturating_add(Weight::from_parts(1_634, 0).saturating_mul(o.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(o.into()))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[0, 512]`.
	fn seal_clear_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250 + n * (1 ±0)`
		//  Estimated: `765`
		// Minimum execution time: 10_556_000 picoseconds.
		Weight::from_parts(11_494_000, 0)
			.saturating_add(Weight::from_parts(0, 765))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[0, 512]`.
	fn seal_get_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250 + n * (1 ±0)`
		//  Estimated: `765`
		// Minimum execution time: 10_138_000 picoseconds.
		Weight::from_parts(11_182_000, 0)
			.saturating_add(Weight::from_parts(0, 765))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[0, 512]`.
	fn seal_contains_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250 + n * (1 ±0)`
		//  Estimated: `765`
		// Minimum execution time: 9_260_000 picoseconds.
		Weight::from_parts(11_577_000, 0)
			.saturating_add(Weight::from_parts(0, 765))
			.saturating_add(T::DbWeight::get().reads(1))
	}
	/// Storage: `Skipped::Metadata` (r:0 w:0)
	/// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `n` is `[0, 512]`.
	fn seal_take_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `250 + n * (1 ±0)`
		//  Estimated: `765`
		// Minimum execution time: 10_868_000 picoseconds.
		Weight::from_parts(12_186_000, 0)
			.saturating_add(Weight::from_parts(0, 765))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn set_transient_storage_empty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_029_000 picoseconds.
		Weight::from_parts(2_029_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn set_transient_storage_full() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 2_362_000 picoseconds.
		Weight::from_parts(2_362_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn get_transient_storage_empty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_000_000 picoseconds.
		Weight::from_parts(3_000_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn get_transient_storage_full() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_935_000 picoseconds.
		Weight::from_parts(1_935_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn rollback_transient_storage() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_413_000 picoseconds.
		Weight::from_parts(1_413_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 512]`.
	/// The range of component `o` is `[0, 512]`.
	fn seal_set_transient_storage(n: u32, o: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_926_000 picoseconds.
		Weight::from_parts(3_627_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 226
			.saturating_add(Weight::from_parts(583, 0).saturating_mul(n.into()))
			// Standard Error: 226
			.saturating_add(Weight::from_parts(1_068, 0).saturating_mul(o.into()))
	}
	/// The range of component `n` is `[0, 512]`.
	fn seal_clear_transient_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_980_000 picoseconds.
		Weight::from_parts(4_586_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 512]`.
	fn seal_get_transient_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_474_000 picoseconds.
		Weight::from_parts(3_748_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 512]`.
	fn seal_contains_transient_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 3_180_000 picoseconds.
		Weight::from_parts(3_557_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 512]`.
	fn seal_take_transient_storage(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 4_343_000 picoseconds.
		Weight::from_parts(4_924_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Revive::AddressSuffix` (r:1 w:0)
	/// Proof: `Revive::AddressSuffix` (`max_values`: None, `max_size`: Some(32), added: 2507, mode: `Measured`)
	/// Storage: `Revive::ContractInfoOf` (r:1 w:0)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `Revive::CodeInfoOf` (r:1 w:0)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:1 w:0)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	/// The range of component `t` is `[0, 1]`.
	/// The range of component `i` is `[0, 262144]`.
	fn seal_call(t: u32, _i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1289 + t * (206 ±0)`
		//  Estimated: `4754 + t * (2480 ±0)`
		// Minimum execution time: 44_630_000 picoseconds.
		Weight::from_parts(47_789_499, 0)
			.saturating_add(Weight::from_parts(0, 4754))
			// Standard Error: 1_237_550
			.saturating_add(Weight::from_parts(506_500, 0).saturating_mul(t.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(t.into())))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 2480).saturating_mul(t.into()))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:0)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:1 w:0)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	fn seal_delegate_call() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1061`
		//  Estimated: `4526`
		// Minimum execution time: 30_817_000 picoseconds.
		Weight::from_parts(30_817_000, 0)
			.saturating_add(Weight::from_parts(0, 4526))
			.saturating_add(T::DbWeight::get().reads(2))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	/// Storage: `Revive::PristineCode` (r:1 w:0)
	/// Proof: `Revive::PristineCode` (`max_values`: None, `max_size`: Some(262180), added: 264655, mode: `Measured`)
	/// Storage: `Revive::ContractInfoOf` (r:1 w:1)
	/// Proof: `Revive::ContractInfoOf` (`max_values`: None, `max_size`: Some(1779), added: 4254, mode: `Measured`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `Measured`)
	/// The range of component `i` is `[0, 262144]`.
	fn seal_instantiate(_i: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1240`
		//  Estimated: `4735`
		// Minimum execution time: 129_935_000 picoseconds.
		Weight::from_parts(1_073_307_000, 0)
			.saturating_add(Weight::from_parts(0, 4735))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// The range of component `n` is `[0, 262144]`.
	fn seal_hash_sha2_256(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_128_000 picoseconds.
		Weight::from_parts(293_190_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 262144]`.
	fn seal_hash_keccak_256(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_199_000 picoseconds.
		Weight::from_parts(932_486_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 262144]`.
	fn seal_hash_blake2_256(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 933_000 picoseconds.
		Weight::from_parts(356_621_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 262144]`.
	fn seal_hash_blake2_128(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 1_097_000 picoseconds.
		Weight::from_parts(359_191_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// The range of component `n` is `[0, 261889]`.
	fn seal_sr25519_verify(_n: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 60_484_000 picoseconds.
		Weight::from_parts(909_445_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_ecdsa_recover() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_547_148_000 picoseconds.
		Weight::from_parts(9_547_148_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	fn seal_ecdsa_to_eth_address() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 13_870_000 picoseconds.
		Weight::from_parts(13_870_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	fn seal_set_code_hash() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `297`
		//  Estimated: `3762`
		// Minimum execution time: 17_692_000 picoseconds.
		Weight::from_parts(17_692_000, 0)
			.saturating_add(Weight::from_parts(0, 3762))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `Measured`)
	fn lock_delegate_dependency() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `335`
		//  Estimated: `3800`
		// Minimum execution time: 10_996_000 picoseconds.
		Weight::from_parts(10_996_000, 0)
			.saturating_add(Weight::from_parts(0, 3800))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: `Revive::CodeInfoOf` (r:1 w:1)
	/// Proof: `Revive::CodeInfoOf` (`max_values`: None, `max_size`: Some(96), added: 2571, mode: `MaxEncodedLen`)
	fn unlock_delegate_dependency() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `335`
		//  Estimated: `3561`
		// Minimum execution time: 9_950_000 picoseconds.
		Weight::from_parts(9_950_000, 0)
			.saturating_add(Weight::from_parts(0, 3561))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// The range of component `r` is `[0, 5000]`.
	fn instr(_r: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 10_644_000 picoseconds.
		Weight::from_parts(366_804_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
	}
}
