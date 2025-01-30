use frame_support::{parameter_types, traits::EitherOfDiverse, weights::Weight};
use frame_system::EnsureRoot;
use parachains_common::BlockNumber;
use pop_runtime_common::DAYS;

use crate::{
	AccountId, Council, Runtime, RuntimeBlockWeights, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};

type MoreThanHalfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
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

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type AddOrigin = MoreThanHalfCouncil;
	type MaxMembers = CouncilMaxMembers;
	type MembershipChanged = Council;
	type MembershipInitialized = Council;
	type PrimeOrigin = MoreThanHalfCouncil;
	type RemoveOrigin = MoreThanHalfCouncil;
	type ResetOrigin = MoreThanHalfCouncil;
	type RuntimeEvent = RuntimeEvent;
	type SwapOrigin = MoreThanHalfCouncil;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

impl pallet_motion::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type SimpleMajorityOrigin =
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>;
	type SuperMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type UnanimousOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	type WeightInfo = ();
}
