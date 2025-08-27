use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU128, ConstU16, ConstU64},
	weights::Weight,
};
use frame_system::{self as system, limits::BlockWeights};
use sp_runtime::{BuildStorage, Perbill};

use super::*;
pub(crate) use crate as pallet_motion;

type Block = frame_system::mocking::MockBlock<Test>;

parameter_types! {
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::default();
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system::{Pallet, Call, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		Motion: pallet_motion,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
	type AccountData = pallet_balances::AccountData<u128>;
	type Block = Block;
	type BlockHashCount = ConstU64<250>;
	type Nonce = u64;
	type PalletInfo = PalletInfo;
	type SS58Prefix = ConstU16<42>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = u128;
	type ExistentialDeposit = ConstU128<1>;
	type FreezeIdentifier = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
}

parameter_types! {
	pub CouncilMotionDuration: u64 = 7;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type Consideration = ();
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type DisapproveOrigin = frame_system::EnsureRoot<u64>;
	type KillOrigin = frame_system::EnsureRoot<u64>;
	type MaxMembers = CouncilMaxMembers;
	type MaxProposalWeight = MaxProposalWeight;
	type MaxProposals = CouncilMaxProposals;
	type MotionDuration = CouncilMotionDuration;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = frame_system::EnsureRoot<u64>;
	type WeightInfo = ();
}

impl pallet_motion::Config for Test {
	type RuntimeCall = RuntimeCall;
	type SimpleMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<u64, CouncilCollective, 1, 2>;
	type SuperMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<u64, CouncilCollective, 2, 3>;
	type UnanimousOrigin = pallet_collective::EnsureProportionAtLeast<u64, CouncilCollective, 1, 1>;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig {
		balances: pallet_balances::GenesisConfig::<Test> {
			balances: vec![(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)],
			..Default::default()
		},
		council: pallet_collective::GenesisConfig {
			members: vec![1, 2, 3, 4],
			phantom: Default::default(),
		},
	}
	.build_storage()
	.unwrap()
	.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
