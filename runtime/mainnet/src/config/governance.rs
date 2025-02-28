use frame_support::{
	parameter_types,
	traits::{EitherOfDiverse, NeverEnsureOrigin},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_collective::EnsureProportionAtLeast;
use parachains_common::BlockNumber;
use pop_runtime_common::DAYS;
use sp_core::crypto::Ss58Codec;

use crate::{
	config::system::RuntimeBlockWeights, weights, AccountId, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeOrigin,
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

// Multisig account for sudo, generated from the following signatories:
// - 15VPagCVayS6XvT5RogPYop3BJTJzwqR2mCGR1kVn3w58ygg
// - 142zako1kfvrpQ7pJKYR8iGUD58i4wjb78FUsmJ9WcXmkM5z
// - 15k9niqckMg338cFBoz9vWFGwnCtwPBquKvqJEfHApijZkDz
// - 14G3CUFnZUBnHZUhahexSZ6AgemaW9zMHBnGccy3df7actf4
// - Threshold 2
const SUDO_ADDRESS: &str = "15NMV2JX1NeMwarQiiZvuJ8ixUcvayFDcu1F9Wz1HNpSc8gP";

parameter_types! {
	pub CouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxProposalWeight: Weight = sp_runtime::Perbill::from_percent(80) * RuntimeBlockWeights::get().max_block;
	pub SudoAddress: AccountId = AccountId::from_ss58check(SUDO_ADDRESS).expect("sudo address is valid SS58");
}

/// Instance of pallet_collective representing Pop's council.
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
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
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
	type WeightInfo = weights::pallet_motion::WeightInfo<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_sudo::WeightInfo<Runtime>;
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
		fn default_votes_match_the_expected() {
			let seats = 5;
			let range = 0..6;

			for ayes in range.clone() {
				for nays in range.clone() {
					if ayes + nays <= seats {
						if ayes * 2 > seats {
							// Prime is nay.
							// More than half ayes condition holds over prime.
							assert_eq!(
								<Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(None, ayes, nays, seats),
								true
							);
						} else {
							// Prime is aye.
							// More than half ayes condition is not met, prime vote is default.
							assert_eq!(
								<Runtime as pallet_collective::Config<CouncilCollective>>::DefaultVote::default_vote(Some(true), ayes, nays, seats),
								true
							);
						}
					}
				}
			}
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

	mod sudo {
		use super::*;

		#[test]
		fn sudo_account_matches() {
			// Doesn't use SUDO_ADDRESS constant on purpose.
			assert_eq!(
				SudoAddress::get(),
				AccountId::from_ss58check("15NMV2JX1NeMwarQiiZvuJ8ixUcvayFDcu1F9Wz1HNpSc8gP")
					.expect("sudo address is valid SS58")
			);
		}
		#[test]
		fn sudo_does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_sudo::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}
