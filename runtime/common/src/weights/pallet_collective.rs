
//! Autogenerated weights for `pallet_collective`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 46.0.0
//! DATE: 2025-02-28, STEPS: `5`, REPEAT: `5`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `tux`, CPU: `12th Gen Intel(R) Core(TM) i7-12700H`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("pop")`, DB CACHE: `1024`

// Executed Command:
// ./target/release//pop-node
// benchmark
// pallet
// --template=./scripts/templates/runtime-weight-template.hbs
// --chain=pop
// --wasm-execution=compiled
// --pallet=pallet_collective
// --extrinsic=*
// --steps=5
// --repeat=5
// --output=./runtime/mainnet/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `pallet_collective`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::WeightInfo for WeightInfo<T> {
	/// Storage: `Council::Members` (r:1 w:1)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:0)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:100 w:100)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Prime` (r:0 w:1)
	/// Proof: `Council::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[0, 100]`.
	/// The range of component `n` is `[0, 100]`.
	/// The range of component `p` is `[0, 100]`.
	fn set_members(m: u32, _n: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + m * (3232 ±0) + p * (3190 ±0)`
		//  Estimated: `146005 + m * (1975 ±161) + p * (3657 ±161)`
		// Minimum execution time: 15_045_000 picoseconds.
		Weight::from_parts(15_365_000, 146005)
			// Standard Error: 764_699
			.saturating_add(Weight::from_parts(7_121_499, 0).saturating_mul(m.into()))
			// Standard Error: 764_699
			.saturating_add(Weight::from_parts(9_777_422, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
			.saturating_add(T::DbWeight::get().writes(2_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
			.saturating_add(Weight::from_parts(0, 1975).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 3657).saturating_mul(p.into()))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `31 + m * (32 ±0)`
		//  Estimated: `1517 + m * (32 ±0)`
		// Minimum execution time: 13_649_000 picoseconds.
		Weight::from_parts(12_629_237, 1517)
			// Standard Error: 788
			.saturating_add(Weight::from_parts(1_117, 0).saturating_mul(b.into()))
			// Standard Error: 8_109
			.saturating_add(Weight::from_parts(27_522, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:0)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[1, 100]`.
	fn propose_execute(b: u32, m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `31 + m * (32 ±0)`
		//  Estimated: `3497 + m * (32 ±0)`
		// Minimum execution time: 16_274_000 picoseconds.
		Weight::from_parts(15_969_682, 3497)
			// Standard Error: 433
			.saturating_add(Weight::from_parts(834, 0).saturating_mul(b.into()))
			// Standard Error: 4_455
			.saturating_add(Weight::from_parts(22_394, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalCount` (r:1 w:1)
	/// Proof: `Council::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[2, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn propose_proposed(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `552 + m * (32 ±0) + p * (35 ±0)`
		//  Estimated: `3562 + m * (33 ±0) + p * (38 ±0)`
		// Minimum execution time: 22_359_000 picoseconds.
		Weight::from_parts(23_798_830, 3562)
			// Standard Error: 2_332
			.saturating_add(Weight::from_parts(2_120, 0).saturating_mul(b.into()))
			// Standard Error: 24_276
			.saturating_add(Weight::from_parts(50_762, 0).saturating_mul(m.into()))
			// Standard Error: 23_983
			.saturating_add(Weight::from_parts(191_197, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(4_u64))
			.saturating_add(Weight::from_parts(0, 33).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 38).saturating_mul(p.into()))
	}
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:1 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[5, 100]`.
	fn vote(m: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `770 + m * (64 ±0)`
		//  Estimated: `4235 + m * (64 ±0)`
		// Minimum execution time: 25_040_000 picoseconds.
		Weight::from_parts(27_142_455, 4235)
			// Standard Error: 10_870
			.saturating_add(Weight::from_parts(27_042, 0).saturating_mul(m.into()))
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(Weight::from_parts(0, 64).saturating_mul(m.into()))
	}
	/// Storage: `Council::Voting` (r:1 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:0 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `388 + m * (64 ±0) + p * (36 ±0)`
		//  Estimated: `3709 + m * (65 ±0) + p * (37 ±0)`
		// Minimum execution time: 25_954_000 picoseconds.
		Weight::from_parts(25_124_810, 3709)
			// Standard Error: 11_597
			.saturating_add(Weight::from_parts(35_929, 0).saturating_mul(m.into()))
			// Standard Error: 11_200
			.saturating_add(Weight::from_parts(201_885, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 65).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 37).saturating_mul(p.into()))
	}
	/// Storage: `Council::Voting` (r:1 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_early_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1024 + m * (64 ±0) + p * (38 ±0)`
		//  Estimated: `3980 + b * (1 ±0) + m * (65 ±1) + p * (42 ±1)`
		// Minimum execution time: 36_398_000 picoseconds.
		Weight::from_parts(35_703_279, 3980)
			// Standard Error: 2_455
			.saturating_add(Weight::from_parts(1_540, 0).saturating_mul(b.into()))
			// Standard Error: 26_142
			.saturating_add(Weight::from_parts(38_385, 0).saturating_mul(m.into()))
			// Standard Error: 25_244
			.saturating_add(Weight::from_parts(291_917, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 65).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 42).saturating_mul(p.into()))
	}
	/// Storage: `Council::Voting` (r:1 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Prime` (r:1 w:0)
	/// Proof: `Council::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:0 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_disapproved(m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `503 + m * (48 ±0) + p * (36 ±0)`
		//  Estimated: `3802 + m * (49 ±0) + p * (37 ±0)`
		// Minimum execution time: 27_900_000 picoseconds.
		Weight::from_parts(27_683_021, 3802)
			// Standard Error: 10_652
			.saturating_add(Weight::from_parts(20_720, 0).saturating_mul(m.into()))
			// Standard Error: 10_287
			.saturating_add(Weight::from_parts(247_505, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 37).saturating_mul(p.into()))
	}
	/// Storage: `Council::Voting` (r:1 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Members` (r:1 w:0)
	/// Proof: `Council::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Prime` (r:1 w:0)
	/// Proof: `Council::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// The range of component `b` is `[2, 1024]`.
	/// The range of component `m` is `[4, 100]`.
	/// The range of component `p` is `[1, 100]`.
	fn close_approved(b: u32, m: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1044 + m * (64 ±0) + p * (38 ±0)`
		//  Estimated: `4000 + b * (1 ±0) + m * (65 ±1) + p * (42 ±1)`
		// Minimum execution time: 38_238_000 picoseconds.
		Weight::from_parts(41_951_337, 4000)
			// Standard Error: 20_397
			.saturating_add(Weight::from_parts(16_966, 0).saturating_mul(m.into()))
			// Standard Error: 19_696
			.saturating_add(Weight::from_parts(286_553, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
			.saturating_add(Weight::from_parts(0, 65).saturating_mul(m.into()))
			.saturating_add(Weight::from_parts(0, 42).saturating_mul(p.into()))
	}
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::ProposalOf` (r:0 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `p` is `[1, 100]`.
	fn disapprove_proposal(p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `187 + p * (32 ±0)`
		//  Estimated: `1673 + p * (32 ±0)`
		// Minimum execution time: 14_139_000 picoseconds.
		Weight::from_parts(15_197_701, 1673)
			// Standard Error: 4_418
			.saturating_add(Weight::from_parts(165_635, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(p.into()))
	}
	/// Storage: `Council::ProposalOf` (r:1 w:1)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::CostOf` (r:1 w:0)
	/// Proof: `Council::CostOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Proposals` (r:1 w:1)
	/// Proof: `Council::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Council::Voting` (r:0 w:1)
	/// Proof: `Council::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `d` is `[0, 1]`.
	/// The range of component `p` is `[1, 100]`.
	fn kill(d: u32, p: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1436 + p * (36 ±0)`
		//  Estimated: `4820 + d * (78 ±49) + p * (37 ±0)`
		// Minimum execution time: 20_188_000 picoseconds.
		Weight::from_parts(21_698_958, 4820)
			// Standard Error: 625_111
			.saturating_add(Weight::from_parts(1_090_769, 0).saturating_mul(d.into()))
			// Standard Error: 8_385
			.saturating_add(Weight::from_parts(204_084, 0).saturating_mul(p.into()))
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
			.saturating_add(Weight::from_parts(0, 78).saturating_mul(d.into()))
			.saturating_add(Weight::from_parts(0, 37).saturating_mul(p.into()))
	}
	/// Storage: `Council::ProposalOf` (r:1 w:0)
	/// Proof: `Council::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Council::CostOf` (r:1 w:0)
	/// Proof: `Council::CostOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
	fn release_proposal_cost() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `874`
		//  Estimated: `4339`
		// Minimum execution time: 14_960_000 picoseconds.
		Weight::from_parts(15_310_000, 4339)
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
}
