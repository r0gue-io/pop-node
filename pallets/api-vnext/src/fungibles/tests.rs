use frame_support::{
	assert_noop, assert_ok,
	sp_runtime::DispatchError::BadOrigin,
	traits::{fungibles::approvals::Mutate, Get},
};

use super::*;
use crate::mock::{Assets, *};

type AssetsError = pallet_assets::Error<Test>;
type ED = ExistentialDeposit;
type WeightInfo = <Test as Config>::WeightInfo;

#[test]
fn transfer_works() {
	let token = 1;
	let value = 100 * UNIT;
	let from = ALICE;
	let to = BOB;
	ExtBuilder::new()
		.with_balances(vec![(from.clone(), ED::get()), (to.clone(), ED::get())])
		.with_assets(vec![(token, from.clone(), false, 1)])
		.with_asset_balances(vec![(token, from.clone(), value)])
		.build()
		.execute_with(|| {
			for origin in vec![root(), none()] {
				assert_noop!(transfer::<Test, ()>(origin, 0, to.clone(), value), BadOrigin);
			}
			// Check error works for `Assets::transfer_keep_alive()`.
			assert_noop!(
				transfer::<Test, ()>(signed(from.clone()), TokenId::MAX, to.clone(), value),
				AssetsError::Unknown
			);
			let balance_before_transfer = Assets::balance(token, &to);
			assert_ok!(transfer::<Test, ()>(signed(from), token, to.clone(), value));
			let balance_after_transfer = Assets::balance(token, &to);
			assert_eq!(balance_after_transfer, balance_before_transfer + value);
		});
}

#[test]
fn transfer_from_works() {
	let token = 1;
	let value = 100 * UNIT;
	let from = ALICE;
	let to = BOB;
	let spender = CHARLIE;
	ExtBuilder::new()
		.with_balances(vec![(from.clone(), UNIT), (to.clone(), ED::get())])
		.with_assets(vec![(token, from.clone(), false, 1)])
		.with_asset_balances(vec![(token, from.clone(), value * 2)])
		.build()
		.execute_with(|| {
			for origin in vec![root(), none()] {
				assert_noop!(
					transfer_from::<Test, ()>(origin, token, from.clone(), to.clone(), value),
					BadOrigin
				);
			}
			// Check error works for `Assets::transfer_approved()`.
			assert_noop!(
				transfer_from::<Test, ()>(
					signed(spender.clone()),
					TokenId::MAX,
					from.clone(),
					to.clone(),
					value
				),
				AssetsError::Unknown
			);
			// Approve `spender` to transfer up to `value`.
			approve(token, &from, &spender, value);
			// Successfully call transfer from.
			let from_balance_before_transfer = Assets::balance(token, &from);
			let to_balance_before_transfer = Assets::balance(token, &to);
			assert_ok!(transfer_from::<Test, ()>(
				signed(spender),
				token,
				from.clone(),
				to.clone(),
				value
			));
			let from_balance_after_transfer = Assets::balance(token, &from);
			let to_balance_after_transfer = Assets::balance(token, &to);
			// Check that `to` has received the `value` tokens from `from`.
			assert_eq!(to_balance_after_transfer, to_balance_before_transfer + value);
			assert_eq!(from_balance_after_transfer, from_balance_before_transfer - value);
		});
}

mod approve {
	use super::*;
	use crate::fungibles::{approve, weights::WeightInfo as _};

	#[test]
	fn ensure_signed_works() {
		let token = 1;
		let value = 100 * UNIT;
		let spender = BOB;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.build()
			.execute_with(|| {
				for origin in vec![root(), none()] {
					assert_noop!(
						approve::<Test, ()>(origin, token, spender.clone(), value),
						BadOrigin.with_weight(WeightInfo::approve(0, 0))
					);
				}
			});
	}

	#[test]
	fn ensure_error_cases_from_pallet_assets_work() {
		let token = 1;
		let value = 100 * UNIT;
		let owner = ALICE;
		let spender = BOB;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), UNIT), (spender.clone(), ED::get())])
			.with_assets(vec![(token, owner.clone(), false, 1)])
			.build()
			.execute_with(|| {
				// Check error works for `Assets::approve_transfer()` in `Greater` match arm.
				assert_noop!(
					approve::<Test, ()>(
						signed(owner.clone()),
						TokenId::MAX,
						spender.clone(),
						value
					),
					AssetsError::Unknown.with_weight(WeightInfo::approve(1, 0))
				);
				super::approve(token, &owner, &spender, value);
				// Check error works for `Assets::cancel_approval()` in `Less` match arm.
				assert_ok!(Assets::freeze_asset(signed(owner.clone()), token.into()));
				assert_noop!(
					approve::<Test, ()>(signed(owner.clone()), token, spender, value / 2),
					AssetsError::AssetNotLive.with_weight(WeightInfo::approve(0, 1))
				);
				assert_ok!(Assets::thaw_asset(signed(owner), token.into()));
				// No error test for `approve_transfer` in `Less` arm because it is not possible.
			});
	}

	// Non-additive, sets new value.
	#[test]
	fn approve_works() {
		let token = 1;
		let value: Balance = 100 * UNIT;
		let owner = ALICE;
		let spender = BOB;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), UNIT), (spender.clone(), ED::get())])
			.with_assets(vec![(token, owner.clone(), false, 1)])
			.build()
			.execute_with(|| {
				// Approves a value to spend that is higher than the current allowance.
				assert_eq!(Assets::allowance(token, &owner, &spender), 0);
				assert_eq!(
					approve::<Test, ()>(signed(owner.clone()), token, spender.clone(), value),
					Ok(Some(WeightInfo::approve(1, 0)).into())
				);
				assert_eq!(Assets::allowance(token, &owner, &spender), value);
				// Approves a value to spend that is lower than the current allowance.
				assert_eq!(
					approve::<Test, ()>(signed(owner.clone()), token, spender.clone(), value / 2),
					Ok(Some(WeightInfo::approve(1, 1)).into())
				);
				assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
				// Approves a value to spend that is equal to the current allowance.
				assert_eq!(
					approve::<Test, ()>(signed(owner.clone()), token, spender.clone(), value / 2),
					Ok(Some(WeightInfo::approve(0, 0)).into())
				);
				assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
				// Sets allowance to zero.
				assert_eq!(
					approve::<Test, ()>(signed(owner.clone()), token, spender.clone(), 0),
					Ok(Some(WeightInfo::approve(0, 1)).into())
				);
				assert_eq!(Assets::allowance(token, &owner, &spender), 0);
			});
	}
}

fn approve(token: TokenId, owner: &AccountId, spender: &AccountId, amount: Balance) {
	assert_ok!(Assets::approve(token, owner, spender, amount));
}
