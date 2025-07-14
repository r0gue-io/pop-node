use frame_support::{
	assert_noop, assert_ok,
	sp_runtime::{DispatchError::BadOrigin, TokenError},
	traits::{
		fungibles::{approvals::Mutate, metadata::Inspect},
		Get,
	},
};
use pallet_assets::WeightInfo as _;

use super::{weights::WeightInfo as _, *};
use crate::mock::{Assets, *};

type AssetsError = pallet_assets::Error<Test>;
type AssetsWeightInfo = <Test as pallet_assets::Config>::WeightInfo;
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
		.with_asset_balances(vec![(token, from.clone(), value + 1)])
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
			assert_ok!(Assets::approve(token, &from, &spender, value));
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
	use crate::fungibles::approve;

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
				assert_ok!(Assets::approve(token, &owner, &spender, value));
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

#[test]
fn increase_allowance_works() {
	let token = 1;
	let value: Balance = 100 * UNIT;
	let owner = ALICE;
	let spender = BOB;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), UNIT), (spender.clone(), ED::get())])
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			for origin in vec![root(), none()] {
				assert_noop!(
					increase_allowance::<Test, ()>(origin, token, spender.clone(), value),
					BadOrigin.with_weight(WeightInfo::approve(0, 0))
				);
			}
			// Check error works for `Assets::approve_transfer()`.
			assert_noop!(
				increase_allowance::<Test, ()>(
					signed(owner.clone()),
					TokenId::MAX,
					spender.clone(),
					value
				),
				AssetsError::Unknown.with_weight(AssetsWeightInfo::approve_transfer())
			);
			assert_eq!(0, Assets::allowance(token, &owner, &spender));
			assert_ok!(increase_allowance::<Test, ()>(
				signed(owner.clone()),
				token,
				spender.clone(),
				value
			));
			assert_eq!(Assets::allowance(token, &owner, &spender), value);
			// Additive.
			assert_ok!(increase_allowance::<Test, ()>(
				signed(owner.clone()),
				token,
				spender.clone(),
				value
			));
			assert_eq!(Assets::allowance(token, &owner, &spender), value * 2);
		});
}

#[test]
fn decrease_allowance_works() {
	let token = 1;
	let value: Balance = 100 * UNIT;
	let owner = ALICE;
	let spender = BOB;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), UNIT), (spender.clone(), ED::get())])
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			for origin in vec![root(), none()] {
				assert_noop!(
					decrease_allowance::<Test, ()>(origin, token, spender.clone(), 0),
					BadOrigin.with_weight(WeightInfo::approve(0, 0))
				);
			}
			assert_ok!(Assets::approve(token, &owner, &spender, value));
			assert_eq!(Assets::allowance(token, &owner, &spender), value);
			// Check error works for `Assets::cancel_approval()`. No error test for
			// `approve_transfer` because it is not possible.
			assert_ok!(Assets::freeze_asset(signed(owner.clone()), token.into()));
			assert_noop!(
				decrease_allowance::<Test, ()>(
					signed(owner.clone()),
					token,
					spender.clone(),
					value / 2
				),
				AssetsError::AssetNotLive.with_weight(WeightInfo::approve(0, 1))
			);
			assert_ok!(Assets::thaw_asset(signed(owner.clone()), token.into()));
			// Owner balance is not changed if decreased by zero.
			assert_eq!(
				decrease_allowance::<Test, ()>(signed(owner.clone()), token, spender.clone(), 0),
				Ok((0, Some(WeightInfo::approve(0, 0)).into()))
			);
			assert_eq!(Assets::allowance(token, &owner, &spender), value);
			// "Unapproved" error is returned if the current allowance is less than amount to
			// decrease with.
			assert_noop!(
				decrease_allowance::<Test, ()>(
					signed(owner.clone()),
					token,
					spender.clone(),
					value * 2
				),
				AssetsError::Unapproved
			);
			// Decrease allowance successfully.
			assert_eq!(
				decrease_allowance::<Test, ()>(
					signed(owner.clone()),
					token,
					spender.clone(),
					value / 2
				),
				Ok((value / 2, Some(WeightInfo::approve(1, 1)).into()))
			);
			assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
		});
}

#[test]
fn create_works() {
	let creator = ALICE;
	let admin = BOB;
	let min_balance = 100;
	ExtBuilder::new()
		.with_balances(vec![(creator.clone(), UNIT)])
		.build()
		.execute_with(|| {
			for origin in vec![root(), none()] {
				assert_noop!(create::<Test, ()>(origin, admin.clone(), min_balance), BadOrigin);
			}
			let id = NextAssetId::<Test>::get().unwrap_or_default();
			assert!(!Assets::asset_exists(id));
			assert_ok!(create::<Test, ()>(signed(creator), admin, min_balance));
			assert!(Assets::asset_exists(id));
		});
}

#[test]
fn start_destroy_works() {
	let token = 1;
	let creator = ALICE;
	ExtBuilder::new()
		.with_assets(vec![(token, creator.clone(), false, 1)])
		.build()
		.execute_with(|| {
			// Check error works for `Assets::start_destroy()`.
			assert_noop!(
				start_destroy::<Test, ()>(signed(creator.clone()), TokenId::MAX),
				AssetsError::Unknown
			);
			assert_ok!(start_destroy::<Test, ()>(signed(creator.clone()), token));
			// Check that the token is not live after starting the destroy process.
			assert_noop!(
				Assets::mint(signed(creator.clone()), token.into(), creator.into(), 10 * UNIT),
				TokenError::UnknownAsset
			);
		});
}

#[test]
fn set_metadata_works() {
	let token = 1;
	let creator = ALICE;
	let name = b"name".to_vec();
	let symbol = b"symbol".to_vec();
	let decimals = 42;
	ExtBuilder::new()
		.with_balances(vec![(creator.clone(), UNIT)])
		.with_assets(vec![(token, creator.clone(), false, 1)])
		.build()
		.execute_with(|| {
			// Check error works for `Assets::set_metadata()`.
			assert_noop!(
				set_metadata::<Test, ()>(
					signed(creator.clone()),
					TokenId::MAX,
					name.clone(),
					symbol.clone(),
					decimals
				),
				AssetsError::Unknown
			);
			assert_ok!(set_metadata::<Test, ()>(
				signed(creator),
				token,
				name.clone(),
				symbol.clone(),
				decimals
			));
			assert_eq!(Assets::name(token), name);
			assert_eq!(Assets::symbol(token), symbol);
			assert_eq!(Assets::decimals(token), decimals);
		});
}

#[test]
fn clear_metadata_works() {
	let token = 1;
	let creator = ALICE;
	ExtBuilder::new()
		.with_assets(vec![(token, creator.clone(), false, 1)])
		.with_asset_metadata(vec![(token, b"name".to_vec(), b"symbol".to_vec(), 42)])
		.build()
		.execute_with(|| {
			// Check error works for `Assets::clear_metadata()`.
			assert_noop!(
				clear_metadata::<Test, ()>(signed(creator.clone()), TokenId::MAX),
				AssetsError::Unknown
			);
			assert_ok!(clear_metadata::<Test, ()>(signed(creator), token));
			assert!(Assets::name(token).is_empty());
			assert!(Assets::symbol(token).is_empty());
			assert!(Assets::decimals(token).is_zero());
		});
}

#[test]
fn mint_works() {
	let token = 1;
	let value = 100 * UNIT;
	let from = ALICE;
	let to = BOB;
	ExtBuilder::new()
		.with_balances(vec![(from.clone(), ED::get()), (to.clone(), ED::get())])
		.with_assets(vec![(token, from.clone(), false, 1)])
		.build()
		.execute_with(|| {
			// Check error works for `Assets::mint()`.
			assert_noop!(
				mint::<Test, ()>(signed(from.clone()), TokenId::MAX, to.clone(), value),
				TokenError::UnknownAsset
			);
			let balance_before_mint = Assets::balance(token, &to);
			assert_ok!(mint::<Test, ()>(signed(from), token, to.clone(), value));
			let balance_after_mint = Assets::balance(token, &to);
			assert_eq!(balance_after_mint, balance_before_mint + value);
		});
}

#[test]
fn burn_works() {
	let token = 1;
	let value = 100 * UNIT;
	let owner = ALICE;
	let from = BOB;
	let total_supply = value * 2;
	ExtBuilder::new()
		.with_balances(vec![(from.clone(), ED::get())])
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.with_asset_balances(vec![(token, from.clone(), total_supply)])
		.build()
		.execute_with(|| {
			// "BalanceLow" error is returned if token is not created.
			assert_noop!(
				burn::<Test, ()>(signed(owner.clone()), TokenId::MAX, from.clone(), value),
				AssetsError::BalanceLow.with_weight(WeightInfo::balance_of())
			);
			assert_eq!(Assets::total_supply(token), total_supply);
			// Check error works for `Assets::burn()`.
			assert_ok!(Assets::freeze_asset(signed(owner.clone()), token.into()));
			assert_noop!(
				burn::<Test, ()>(signed(owner.clone()), token, from.clone(), value),
				AssetsError::AssetNotLive
			);
			assert_ok!(Assets::thaw_asset(signed(owner.clone()), token.into()));
			// "BalanceLow" error is returned if the balance is less than amount to burn.
			assert_noop!(
				burn::<Test, ()>(signed(owner.clone()), token, from.clone(), total_supply * 2),
				AssetsError::BalanceLow.with_weight(WeightInfo::balance_of())
			);
			let balance_before_burn = Assets::balance(token, &from);
			assert_ok!(burn::<Test, ()>(signed(owner), token, from.clone(), value));
			assert_eq!(Assets::total_supply(token), total_supply - value);
			let balance_after_burn = Assets::balance(token, &from);
			assert_eq!(balance_after_burn, balance_before_burn - value);
		});
}

#[test]
fn total_supply_works() {
	let token = 1;
	let total_supply = 100 * UNIT;
	ExtBuilder::new()
		.with_assets(vec![(token, ALICE, false, 1)])
		.with_asset_balances(vec![(token, ALICE, total_supply)])
		.build()
		.execute_with(|| {
			assert_eq!(super::total_supply::<Test, ()>(TokenId::MAX), 0);
			assert_eq!(super::total_supply::<Test, ()>(token), total_supply);
		});
}

#[test]
fn balance_of_works() {
	let token = 1;
	let owner = ALICE;
	let value = 100 * UNIT;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.with_asset_balances(vec![(token, owner.clone(), value)])
		.build()
		.execute_with(|| {
			assert_eq!(balance::<Test, ()>(TokenId::MAX, &owner), 0);
			assert_eq!(balance::<Test, ()>(token, &owner), value);
		});
}

#[test]
fn allowance_works() {
	let token = 1;
	let value: Balance = 100 * UNIT;
	let owner = ALICE;
	let spender = BOB;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), UNIT), (spender.clone(), ED::get())])
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			assert_eq!(allowance::<Test, ()>(token, &owner, &spender), 0);
			assert_ok!(Assets::approve(token, &owner, &spender, value));
			assert_eq!(allowance::<Test, ()>(token, &owner, &spender), value);
		});
}

#[test]
fn metadata_works() {
	let token = 1;
	let creator = ALICE;
	let name = b"name".to_vec();
	let symbol = b"symbol".to_vec();
	let decimals = 42;
	ExtBuilder::new()
		.with_balances(vec![(creator.clone(), UNIT)])
		.with_assets(vec![(token, creator.clone(), false, 1)])
		.with_asset_metadata(vec![(token, name.clone(), symbol.clone(), decimals)])
		.build()
		.execute_with(|| {
			assert!(super::name::<Test, ()>(TokenId::MAX).is_empty());
			assert!(super::symbol::<Test, ()>(TokenId::MAX).is_empty());
			assert_eq!(super::decimals::<Test, ()>(TokenId::MAX), 0);

			assert_eq!(super::name::<Test, ()>(token), name);
			assert_eq!(super::symbol::<Test, ()>(token), symbol);
			assert_eq!(super::decimals::<Test, ()>(token), decimals);
		});
}

#[test]
fn exists_works() {
	let token = 1;
	let owner = ALICE;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			assert_eq!(exists::<Test, ()>(TokenId::MAX), false);
			assert_eq!(exists::<Test, ()>(token), true);
		});
}
