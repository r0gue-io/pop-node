use frame_support::{
	parameter_types,
	traits::{EitherOfDiverse, NeverEnsureOrigin},
};
use frame_system::EnsureRoot;
use pallet_collective::EnsureProportionAtLeast;
use sp_core::crypto::Ss58Codec;

use crate::{
	config::system::RuntimeBlockWeights, AccountId, BlockNumber, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, Weight, DAYS,
};

// Type aliases for council origins.
type AtLeastThreeFourthsOfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
>;
type UnanimousCouncilVote = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
>;

/// SUDO account set at genesis.
const SUDO_ADDRESS: &str = "5FPL3ZLqUk6MyBoZrQZ1Co29WAteX6T6N68TZ6jitHvhpyuD";

parameter_types! {
	pub CouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = sp_runtime::Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
	pub SudoAddress: AccountId = AccountId::from_ss58check(SUDO_ADDRESS).expect("sudo address is valid SS58");
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Consideration = ();
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type DisapproveOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type MaxMembers = CouncilMaxMembers;
	type MaxProposalWeight = MaxProposalWeight;
	type MaxProposals = CouncilMaxProposals;
	type MotionDuration = CouncilMotionDuration;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_motion::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	// Simple majority is disabled.
	type SimpleMajorityOrigin = NeverEnsureOrigin<()>;
	// At least 3/4 of the council vote is needed.
	type SuperMajorityOrigin = AtLeastThreeFourthsOfCouncil;
	// A unanimous council vote is needed.
	type UnanimousOrigin = UnanimousCouncilVote;
	type WeightInfo = pallet_motion::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}
