use frame_support::{
	parameter_types,
	traits::{EitherOfDiverse, NeverEnsureOrigin},
	weights::Weight,
};
use frame_system::EnsureRoot;
use parachains_common::BlockNumber;
use pop_runtime_common::DAYS;

use crate::{
	AccountId, Council, Runtime, RuntimeBlockWeights, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};

type UnanimousCouncilVote = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
>;

type AtLeastThreeFourthsOfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
>;

parameter_types! {
	pub CouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = sp_runtime::Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Consideration = ();
	type DefaultVote = pallet_collective::PrimeDefaultVote;
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

pub type CouncilMembership = pallet_collective::Instance1;
impl pallet_membership::Config<CouncilMembership> for Runtime {
	type AddOrigin = UnanimousCouncilVote;
	type MaxMembers = CouncilMaxMembers;
	type MembershipChanged = Council;
	type MembershipInitialized = Council;
	type PrimeOrigin = AtLeastThreeFourthsOfCouncil;
	type RemoveOrigin = AtLeastThreeFourthsOfCouncil;
	type ResetOrigin = AtLeastThreeFourthsOfCouncil;
	type RuntimeEvent = RuntimeEvent;
	type SwapOrigin = AtLeastThreeFourthsOfCouncil;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

impl pallet_motion::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	// SimpleMajority origin check will always fail.
	// Making it not possible for SimpleMajority to dispatch as root.
	type SimpleMajorityOrigin = NeverEnsureOrigin<()>;
	// SuperMajority origin check will always fail.
	// Making it not possible for SimpleMajority to dispatch as root.
	type SuperMajorityOrigin = NeverEnsureOrigin<()>;
	type UnanimousOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	type WeightInfo = pallet_motion::weights::SubstrateWeight<Runtime>;
}
