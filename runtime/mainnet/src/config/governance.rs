use frame_support::{
	parameter_types,
	traits::{EitherOfDiverse, NeverEnsureOrigin},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_collective::EnsureProportionAtLeast;
use parachains_common::BlockNumber;
use pop_runtime_common::DAYS;

use crate::{
	AccountId, Council, Runtime, RuntimeBlockWeights, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};

type UnanimousCouncilVote = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
>;

type AtLeastThreeFourthsOfCouncil = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
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
	// SimpleMajorityOrigin won't ever ensure origin.
	type SimpleMajorityOrigin = NeverEnsureOrigin<()>;
	// At least 3/4 of the council vote is needed.
	type SuperMajorityOrigin = AtLeastThreeFourthsOfCouncil;
	// A unanimous council vote is needed.
	type UnanimousOrigin = UnanimousCouncilVote;
	type WeightInfo = pallet_motion::weights::SubstrateWeight<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use sp_runtime::traits::Get;

	use super::*;

	mod council_collective {
		use frame_system::WeightInfo;
		use pallet_collective::DefaultVote;

		use super::*;

		#[test]
		fn consideration_is_not_configured() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_collective::Config<CouncilCollective>>::Consideration,
				>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn default_vote_is_more_than_majority_then_prime() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote>(
				),
				TypeId::of::<pallet_collective::MoreThanMajorityThenPrimeDefaultVote>(),
			);
		}

		#[test]
		fn default_votes_unanimous_threshold_prime_is_nay() {
			// Prime does not exist. (prime vote defaults to false)
			let prime = None;
			let seats: u32 = 5;
			// We want unanimous consensus.
			let threshold = seats;

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 0;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold will be met and the proposal approved.
			assert!(aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 2;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, false);
			// Abstentions will be added to nay votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);
		}

		#[test]
		fn default_votes_unanimous_threshold_prime_is_aye() {
			// Prime is aye.
			let prime = Some(true);
			let seats: u32 = 5;
			// We want unanimous consensus.
			let threshold = seats;

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 0;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold will be met and the proposal approved.
			assert!(aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 2;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 1;
			let nay_votes: u32 = 0;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(aye_votes + abstentions >= threshold);
		}

		#[test]
		fn default_votes_super_majority_threshold_prime_is_nay() {
			// Prime does not exist. (prime vote defaults to false)
			let prime = None;
			let seats: u32 = 5;
			// We want super majority consensus.
			let threshold = 4;

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold will be met and the proposal approved.
			assert!(aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold will be met and the proposal approved.
			assert!(aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 2;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, false);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);
		}

		#[test]
		fn default_votes_super_majority_threshold_prime_is_aye() {
			// Prime is aye.
			let prime = Some(true);
			let seats: u32 = 5;
			// We want super majority consensus.
			let threshold = 4;

			let aye_votes: u32 = 3;
			let nay_votes: u32 = 0;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold will be met and the proposal approved.
			assert!(aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 2;
			let nay_votes: u32 = 2;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 1;
			let nay_votes: u32 = 1;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold will be met and the proposal approved.
			assert!(aye_votes + abstentions >= threshold);

			let aye_votes: u32 = 1;
			let nay_votes: u32 = 0;
			let abstentions = seats - (aye_votes + nay_votes);

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, true);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(aye_votes + abstentions >= threshold);
		}

		#[test]
		fn default_votes_more_than_than_majority_for_super_majority_is_disapproved() {
			let seats: u32 = 5;
			// We want super majority consensus.
			let threshold = 4;
			let aye_votes: u32 = 2;
			let nay_votes: u32 = 2;
			let abstentions = seats - (aye_votes + nay_votes);
			// Prime does not exist or prime vote is nay.
			let prime = None;

			let default = <Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(prime, aye_votes, nay_votes, seats);
			assert_eq!(default, false);
			// Abstentions will be added to aye votes.
			// Threshold won't be met and the proposal disapproved.
			assert!(!aye_votes + abstentions >= threshold);
		}

		#[test]
		fn disapprove_origin_ensures_root() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_collective::Config<CouncilCollective>>::DisapproveOrigin,
				>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[test]
		fn kill_origin_ensures_root() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_collective::Config<CouncilCollective>>::KillOrigin>(
				),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[test]
		fn number_of_councilors_is_limited() {
			assert_eq!(
				<<Runtime as pallet_collective::Config<CouncilCollective>>::MaxMembers as Get<
					u32,
				>>::get(),
				100,
			);
		}

		#[test]
		fn proposal_weight_is_limited() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_collective::Config<CouncilCollective>>::MaxProposalWeight,
				>(),
				TypeId::of::<MaxProposalWeight>(),
			);

			assert_eq!(
				<<Runtime as pallet_collective::Config<CouncilCollective>>::MaxProposalWeight as Get<Weight>>::get(),
				sp_runtime::Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block,
			);
		}

		#[test]
		fn authorize_upgrade_does_not_saturate_weight_limit() {
			let authorize_upgrade_weight =
				<<Runtime as frame_system::Config>::SystemWeightInfo>::authorize_upgrade();
			let max_proposal_weight = <<Runtime as pallet_collective::Config<CouncilCollective>>::MaxProposalWeight as Get<Weight>>::get();
			assert!(authorize_upgrade_weight.all_lt(max_proposal_weight));
		}

		#[test]
		fn number_of_proposals_is_limited() {
			assert_eq!(
				<<Runtime as pallet_collective::Config<CouncilCollective>>::MaxProposals as Get<
					u32,
				>>::get(),
				100,
			);
		}

		#[test]
		fn motion_duration_is_7_days() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_collective::Config<CouncilCollective>>::MotionDuration,
				>(),
				TypeId::of::<CouncilMotionDuration>(),
			);

			assert_eq!(
				<<Runtime as pallet_collective::Config<CouncilCollective>>::MotionDuration as Get<BlockNumber>>::get(),
				7 * DAYS,
			);
		}

		#[test]
		fn proposals_are_runtime_calls() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_collective::Config<CouncilCollective>>::Proposal>(),
				TypeId::of::<RuntimeCall>(),
			);
		}

		#[test]
		fn set_members_origin_ensures_root() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_collective::Config<CouncilCollective>>::SetMembersOrigin,
				>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_collective::Config<CouncilCollective>>::WeightInfo>(
				),
				TypeId::of::<()>(),
			);
		}
	}

	mod membership {
		use super::*;

		#[test]
		fn add_origin_requires_unanimous_vote() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::AddOrigin>(
				),
				TypeId::of::<UnanimousCouncilVote>(),
			);

			assert_eq!(
				TypeId::of::<UnanimousCouncilVote>(),
				TypeId::of::<
					EitherOfDiverse<
						EnsureRoot<AccountId>,
						EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
					>,
				>(),
			);
		}

		#[test]
		fn number_of_members_is_100() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::MaxMembers>(
				),
				TypeId::of::<CouncilMaxMembers>(),
			);

			assert_eq!(CouncilMaxMembers::get(), 100);
		}

		#[test]
		fn council_handles_membership_changes() {
			assert_eq!(
				TypeId::of::<
					<Runtime as pallet_membership::Config<CouncilMembership>>::MembershipChanged,
				>(),
				TypeId::of::<Council>(),
			);
		}

		#[test]
		fn council_handles_membership_initialization() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::MembershipInitialized>(),
				TypeId::of::<Council>(),
			);
		}

		#[test]
		fn prime_origin_ensures_at_least_three_fourths() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::PrimeOrigin>(
				),
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
			);
			assert_eq!(
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
				TypeId::of::<
					EitherOfDiverse<
						EnsureRoot<AccountId>,
						EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
					>,
				>(),
			);
		}

		#[test]
		fn remove_origin_ensures_at_least_three_fourths() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::RemoveOrigin>(
				),
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
			);
		}

		#[test]
		fn reset_origin_ensures_at_least_three_fourths() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::ResetOrigin>(
				),
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
			);
		}

		#[test]
		fn swap_origin_ensures_at_least_three_fourths() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::SwapOrigin>(
				),
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_membership::Config<CouncilMembership>>::WeightInfo>(
				),
				TypeId::of::<()>(),
			);
		}
	}

	mod motion {
		use super::*;

		#[test]
		fn simple_majority_is_never_origin() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_motion::Config>::SimpleMajorityOrigin>(),
				TypeId::of::<NeverEnsureOrigin<()>>(),
			);
		}

		#[test]
		fn super_majority_ensures_ensures_at_least_three_fourths() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_motion::Config>::SuperMajorityOrigin>(),
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
			);
		}

		#[test]
		fn unanimous_origin_ensures_unanimous_vote() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_motion::Config>::UnanimousOrigin>(),
				TypeId::of::<UnanimousCouncilVote>(),
			);
		}

		#[test]
		fn default_weights_are_not_used() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_motion::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}
