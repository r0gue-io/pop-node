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
	#[cfg(not(feature = "runtime-benchmarks"))]
	// Simple majority is disabled.
	type SimpleMajorityOrigin = NeverEnsureOrigin<()>;
	#[cfg(feature = "runtime-benchmarks")]
	// Provide some way to ensure origin such that benchmarks can run.
	type SimpleMajorityOrigin = AtLeastThreeFourthsOfCouncil;
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

/// Contains the impl to set the council members on a runtime upgrade.
pub(crate) mod initiate_council_migration {
	use alloc::{vec, vec::Vec};

	use frame_support::traits::OnRuntimeUpgrade;
	#[cfg(feature = "try-runtime")]
	use sp_runtime::TryRuntimeError;

	use super::*;

	parameter_types! {
		C1: AccountId = AccountId::from_ss58check("13BL7T6bTgeEdfEdZqLCKJZPN8ncyFNxxHRKFb2YMATvyfH4").expect("address is valid SS58");
		C2: AccountId = AccountId::from_ss58check("142zako1kfvrpQ7pJKYR8iGUD58i4wjb78FUsmJ9WcXmkM5z").expect("address is valid SS58");
		C3: AccountId = AccountId::from_ss58check("14G3CUFnZUBnHZUhahexSZ6AgemaW9zMHBnGccy3df7actf4").expect("address is valid SS58");
		C4: AccountId = AccountId::from_ss58check("15VPagCVayS6XvT5RogPYop3BJTJzwqR2mCGR1kVn3w58ygg").expect("address is valid SS58");
		C5: AccountId = AccountId::from_ss58check("15k9niqckMg338cFBoz9vWFGwnCtwPBquKvqJEfHApijZkDz").expect("address is valid SS58");
		Members: Vec<AccountId> = vec![C1::get(), C2::get(), C3::get(), C4::get(), C5::get()];
	}

	/// Populates council with certain members.
	pub struct SetCouncilors;

	impl OnRuntimeUpgrade for SetCouncilors {
		fn on_runtime_upgrade() -> Weight {
			let members_in_storage: Vec<AccountId> =
				pallet_collective::Members::<Runtime, CouncilCollective>::get();

			if !members_in_storage.is_empty() {
				// Members already set. Nothing to do.
				return <Runtime as frame_system::Config>::DbWeight::get().reads(1);
			}

			let members: Vec<AccountId> = Members::get();
			pallet_collective::Members::<Runtime, CouncilCollective>::put(members.clone());
			<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			// Council doesn't exist previous to the upgrade including this migration.
			Ok(Vec::<u8>::new())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
			let expected_members: Vec<AccountId> = Members::get();
			let members_in_storage: Vec<AccountId> =
				pallet_collective::Members::<Runtime, CouncilCollective>::get();

			// Err if there are not 5 members in storage after the migration.
			if members_in_storage.len() != 5 {
				return Err(TryRuntimeError::Other("Migration didn't execute successfully."));
			}
			// Iterate over expected members and check if they are in storage.
			for m in expected_members.iter() {
				if !members_in_storage.contains(m) {
					return Err(TryRuntimeError::Other("Migration didn't execute successfully."));
				}
			}
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		use crate::System;

		fn new_test_ext() -> sp_io::TestExternalities {
			let mut ext = sp_io::TestExternalities::new_empty();
			ext.execute_with(|| System::set_block_number(1));
			ext
		}

		#[test]
		fn members_are_sorted() {
			// The provided member list is already sorted.
			let members = Members::get();
			let mut sorted_members = members.clone();
			sorted_members.sort();
			assert_eq!(members, sorted_members);
		}

		#[test]
		fn migration_does_not_write_in_storage_if_members_is_not_empty() {
			new_test_ext().execute_with(|| {
				// Populate Members with only one member.
				let members: Vec<AccountId> = vec![C1::get()];
				pallet_collective::Members::<Runtime, CouncilCollective>::put(members.clone());

				let resulting_weight = SetCouncilors::on_runtime_upgrade();
				let expected_weight = <Runtime as frame_system::Config>::DbWeight::get().reads(1);
				// Resulting weight from on_runtime_upgrade is only 1 read.
				assert_eq!(resulting_weight, expected_weight);

				// Ensure there is still only 1 member in storage.
				let members_in_storage: Vec<AccountId> =
					pallet_collective::Members::<Runtime, CouncilCollective>::get();
				assert_eq!(members_in_storage.len(), 1);
			})
		}

		#[test]
		fn on_runtime_upgrade_weight_is_within_limits() {
			new_test_ext().execute_with(|| {
				let mut members = Members::get();
				members.sort();
				println!("Sorted Members {:?}", members);
				let resulting_weight = SetCouncilors::on_runtime_upgrade();
				let expected_weight =
					<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1);
				assert_eq!(resulting_weight, expected_weight);
				assert!(resulting_weight.all_lt(pop_runtime_common::MAXIMUM_BLOCK_WEIGHT));
			})
		}
	}
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
			#[cfg(not(feature = "runtime-benchmarks"))]
			assert_eq!(
				TypeId::of::<<Runtime as pallet_motion::Config>::SimpleMajorityOrigin>(),
				TypeId::of::<NeverEnsureOrigin<()>>(),
			);
			#[cfg(feature = "runtime-benchmarks")]
			assert_eq!(
				TypeId::of::<<Runtime as pallet_motion::Config>::SimpleMajorityOrigin>(),
				TypeId::of::<AtLeastThreeFourthsOfCouncil>(),
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
