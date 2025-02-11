use codec::Encode;
use frame_support::{assert_ok, dispatch::GetDispatchInfo, weights::Weight};
use frame_system::{EventRecord, Phase};
use mock::{RuntimeCall, RuntimeEvent};
use pallet_collective::Event as CollectiveEvent;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};

use super::*;
use crate as pallet_motion;
use crate::{mock::*, Event as MotionEvent};

fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
	EventRecord { phase: Phase::Initialization, event, topics: vec![] }
}

struct Proposal {
	len: u32,
	weight: Weight,
	hash: H256,
}

enum MotionType {
	SimpleMajority,
	SuperMajority,
	Unanimous,
}

// sets up collective proposal with `threshold` and `motion_type`.
fn setup_proposal(threshold: u32, motion_type: MotionType) -> Proposal {
	// Inner call (requires sudo). Will be wrapped by pallet_motion.
	let inner_call =
		RuntimeCall::Balances(pallet_balances::Call::force_set_balance { who: 5, new_free: 5 });

	// Setup motion with specified origin type
	let motion = match motion_type {
		MotionType::SimpleMajority =>
			RuntimeCall::Motion(pallet_motion::Call::simple_majority { call: Box::new(inner_call) }),
		MotionType::SuperMajority =>
			RuntimeCall::Motion(pallet_motion::Call::super_majority { call: Box::new(inner_call) }),
		MotionType::Unanimous =>
			RuntimeCall::Motion(pallet_motion::Call::unanimous { call: Box::new(inner_call) }),
	};

	let proposal_len: u32 = motion.using_encoded(|p| p.len() as u32);
	let proposal_weight = motion.get_dispatch_info().call_weight;
	let hash = BlakeTwo256::hash_of(&motion);

	assert_ok!(Council::propose(
		RuntimeOrigin::signed(1),
		threshold,
		Box::new(motion.clone()),
		proposal_len
	));

	Proposal { len: proposal_len, weight: proposal_weight, hash }
}

#[test]
fn simple_majority_works() {
	new_test_ext().execute_with(|| {
		let proposal = setup_proposal(2, MotionType::SimpleMajority);

		let hash = proposal.hash;
		let proposal_len = proposal.len;
		let proposal_weight = proposal.weight;

		assert_ok!(Council::vote(RuntimeOrigin::signed(1), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(2), hash, 0, true));

		System::set_block_number(3);

		assert_ok!(Council::close(
			RuntimeOrigin::signed(4),
			hash,
			0,
			proposal_weight,
			proposal_len
		));

		assert_eq!(
			System::events(),
			vec![
				record(RuntimeEvent::Council(CollectiveEvent::Proposed {
					account: 1,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 2
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 1,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 2,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Approved { proposal_hash: hash })),
				record(RuntimeEvent::Balances(pallet_balances::Event::Rescinded { amount: 45 })),
				record(RuntimeEvent::Balances(pallet_balances::Event::BalanceSet {
					who: 5,
					free: 5,
				})),
				record(RuntimeEvent::Motion(MotionEvent::DispatchSimpleMajority {
					motion_result: Ok(())
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Ok(())
				}))
			]
		);
	});
}

#[test]
fn super_majority_works() {
	new_test_ext().execute_with(|| {
		let proposal = setup_proposal(3, MotionType::SuperMajority);

		let hash = proposal.hash;
		let proposal_len = proposal.len;
		let proposal_weight = proposal.weight;

		assert_ok!(Council::vote(RuntimeOrigin::signed(1), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(2), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(3), hash, 0, true));

		System::set_block_number(3);

		assert_ok!(Council::close(
			RuntimeOrigin::signed(4),
			hash,
			0,
			proposal_weight,
			proposal_len
		));

		assert_eq!(
			System::events(),
			vec![
				record(RuntimeEvent::Council(CollectiveEvent::Proposed {
					account: 1,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 3
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 1,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 2,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 3,
					proposal_hash: hash,
					voted: true,
					yes: 3,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 3,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Approved { proposal_hash: hash })),
				record(RuntimeEvent::Balances(pallet_balances::Event::Rescinded { amount: 45 })),
				record(RuntimeEvent::Balances(pallet_balances::Event::BalanceSet {
					who: 5,
					free: 5,
				})),
				record(RuntimeEvent::Motion(MotionEvent::DispatchSuperMajority {
					motion_result: Ok(())
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Ok(())
				}))
			]
		);
	});
}

#[test]
fn unanimous_works() {
	new_test_ext().execute_with(|| {
		let proposal = setup_proposal(4, MotionType::Unanimous);

		let hash = proposal.hash;
		let proposal_len = proposal.len;
		let proposal_weight = proposal.weight;

		assert_ok!(Council::vote(RuntimeOrigin::signed(1), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(2), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(3), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(4), hash, 0, true));

		System::set_block_number(3);

		assert_ok!(Council::close(
			RuntimeOrigin::signed(4),
			hash,
			0,
			proposal_weight,
			proposal_len
		));

		assert_eq!(
			System::events(),
			vec![
				record(RuntimeEvent::Council(CollectiveEvent::Proposed {
					account: 1,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 4
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 1,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 2,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 3,
					proposal_hash: hash,
					voted: true,
					yes: 3,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 4,
					proposal_hash: hash,
					voted: true,
					yes: 4,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 4,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Approved { proposal_hash: hash })),
				record(RuntimeEvent::Balances(pallet_balances::Event::Rescinded { amount: 45 })),
				record(RuntimeEvent::Balances(pallet_balances::Event::BalanceSet {
					who: 5,
					free: 5,
				})),
				record(RuntimeEvent::Motion(MotionEvent::DispatchUnanimous {
					motion_result: Ok(())
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Ok(())
				}))
			]
		);
	});
}

#[cfg(feature = "simple-majority")]
#[test]
fn simple_majority_fails() {
	new_test_ext().execute_with(|| {
		let threshold = 1;
		let proposal = setup_proposal(threshold, MotionType::SimpleMajority);

		let hash = proposal.hash;

		System::set_block_number(3);

		// No votes or closing necessary. A proposal with threshold 1 is automatically executed

		assert_eq!(
			System::events(),
			vec![record(RuntimeEvent::Council(CollectiveEvent::Executed {
				proposal_hash: hash,
				result: Err(sp_runtime::DispatchError::BadOrigin)
			}))]
		);
	});
}

#[test]
fn super_majority_fails() {
	new_test_ext().execute_with(|| {
		let threshold = 2;
		let proposal = setup_proposal(threshold, MotionType::SuperMajority);

		let hash = proposal.hash;
		let proposal_len = proposal.len;
		let proposal_weight = proposal.weight;

		assert_ok!(Council::vote(RuntimeOrigin::signed(1), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(2), hash, 0, true));

		System::set_block_number(3);

		assert_ok!(Council::close(
			RuntimeOrigin::signed(4),
			hash,
			0,
			proposal_weight,
			proposal_len
		));

		assert_eq!(
			System::events(),
			vec![
				record(RuntimeEvent::Council(CollectiveEvent::Proposed {
					account: 1,
					proposal_index: 0,
					proposal_hash: hash,
					threshold,
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 1,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 2,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Approved { proposal_hash: hash })),
				record(RuntimeEvent::Council(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Err(sp_runtime::DispatchError::BadOrigin)
				}))
			]
		);
	});
}

#[test]
fn unanimous_fails() {
	new_test_ext().execute_with(|| {
		let threshold = 3;
		let proposal = setup_proposal(threshold, MotionType::Unanimous);

		let hash = proposal.hash;
		let proposal_len = proposal.len;
		let proposal_weight = proposal.weight;

		assert_ok!(Council::vote(RuntimeOrigin::signed(1), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(2), hash, 0, true));
		assert_ok!(Council::vote(RuntimeOrigin::signed(3), hash, 0, true));

		System::set_block_number(3);

		assert_ok!(Council::close(
			RuntimeOrigin::signed(4),
			hash,
			0,
			proposal_weight,
			proposal_len
		));

		assert_eq!(
			System::events(),
			vec![
				record(RuntimeEvent::Council(CollectiveEvent::Proposed {
					account: 1,
					proposal_index: 0,
					proposal_hash: hash,
					threshold,
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 1,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 2,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Voted {
					account: 3,
					proposal_hash: hash,
					voted: true,
					yes: 3,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 3,
					no: 0
				})),
				record(RuntimeEvent::Council(CollectiveEvent::Approved { proposal_hash: hash })),
				record(RuntimeEvent::Council(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Err(sp_runtime::DispatchError::BadOrigin)
				}))
			]
		);
	});
}
