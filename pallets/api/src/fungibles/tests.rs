use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::WithPostDispatchInfo,
	sp_runtime::{traits::Zero, DispatchError::BadOrigin},
	traits::fungibles::{
		approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect, Inspect,
	},
};

use crate::{
	fungibles::{
		weights::WeightInfo as WeightInfoTrait, AssetsInstanceOf, AssetsWeightInfoOf,
		AssetsWeightInfoTrait, Config, Read::*, ReadResult,
	},
	mock::*,
	Read,
};

const TOKEN: u32 = 42;

type AssetsError = pallet_assets::Error<Test, AssetsInstanceOf<Test>>;
type AssetsWeightInfo = AssetsWeightInfoOf<Test>;
type Event = crate::fungibles::Event<Test>;
type WeightInfo = <Test as Config>::WeightInfo;

mod encoding_read_result {
	use super::*;

	#[test]
	fn total_supply() {
		let total_supply = 1_000_000 * UNIT;
		assert_eq!(ReadResult::TotalSupply::<Test>(total_supply).encode(), total_supply.encode());
	}

	#[test]
	fn balance_of() {
		let balance = 100 * UNIT;
		assert_eq!(ReadResult::BalanceOf::<Test>(balance).encode(), balance.encode());
	}

	#[test]
	fn allowance() {
		let allowance = 100 * UNIT;
		assert_eq!(ReadResult::Allowance::<Test>(allowance).encode(), allowance.encode());
	}

	#[test]
	fn token_name() {
		let name = vec![42, 42, 42, 42, 42];
		assert_eq!(ReadResult::TokenName::<Test>(name.clone()).encode(), name.encode());
	}

	#[test]
	fn token_symbol() {
		let symbol = vec![42, 42, 42, 42, 42];
		assert_eq!(ReadResult::TokenSymbol::<Test>(symbol.clone()).encode(), symbol.encode());
	}

	#[test]
	fn token_decimals() {
		let decimals = 42;
		assert_eq!(ReadResult::TokenDecimals::<Test>(decimals).encode(), decimals.encode());
	}

	#[test]
	fn token_exists() {
		let exists = true;
		assert_eq!(ReadResult::TokenExists::<Test>(exists).encode(), exists.encode());
	}
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = ALICE;
		let to = BOB;

		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::transfer(origin, token, to, value), BadOrigin);
		}
		// Check error works for `Assets::transfer_keep_alive()`.
		assert_noop!(Fungibles::transfer(signed(from), token, to, value), AssetsError::Unknown);
		assets::create_and_mint_to(from, token, from, value * 2);
		let balance_before_transfer = Assets::balance(token, &to);
		assert_ok!(Fungibles::transfer(signed(from), token, to, value));
		let balance_after_transfer = Assets::balance(token, &to);
		assert_eq!(balance_after_transfer, balance_before_transfer + value);
		System::assert_last_event(
			Event::Transfer { token, from: Some(from), to: Some(to), value }.into(),
		);
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = ALICE;
		let to = BOB;
		let spender = CHARLIE;

		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::transfer_from(origin, token, from, to, value), BadOrigin);
		}
		// Check error works for `Assets::transfer_approved()`.
		assert_noop!(
			Fungibles::transfer_from(signed(spender), token, from, to, value),
			AssetsError::Unknown
		);
		// Approve `spender` to transfer up to `value`.
		assets::create_mint_and_approve(spender, token, from, value * 2, spender, value);
		// Successfully call transfer from.
		let from_balance_before_transfer = Assets::balance(token, &from);
		let to_balance_before_transfer = Assets::balance(token, &to);
		assert_ok!(Fungibles::transfer_from(signed(spender), token, from, to, value));
		let from_balance_after_transfer = Assets::balance(token, &from);
		let to_balance_after_transfer = Assets::balance(token, &to);
		// Check that `to` has received the `value` tokens from `from`.
		assert_eq!(to_balance_after_transfer, to_balance_before_transfer + value);
		assert_eq!(from_balance_after_transfer, from_balance_before_transfer - value);
		System::assert_last_event(
			Event::Transfer { token, from: Some(from), to: Some(to), value }.into(),
		);
	});
}

mod approve {
	use super::*;

	#[test]
	fn ensure_signed_works() {
		new_test_ext().execute_with(|| {
			let value: Balance = 100 * UNIT;
			let token = TOKEN;
			let spender = BOB;

			for origin in vec![root(), none()] {
				assert_noop!(
					Fungibles::approve(origin, token, spender, value),
					BadOrigin.with_weight(WeightInfo::approve(0, 0))
				);
			}
		});
	}

	#[test]
	fn ensure_error_cases_from_pallet_assets_work() {
		new_test_ext().execute_with(|| {
			let value: Balance = 100 * UNIT;
			let token = TOKEN;
			let owner = ALICE;
			let spender = BOB;

			for origin in vec![root(), none()] {
				assert_noop!(
					Fungibles::approve(origin, token, spender, value),
					BadOrigin.with_weight(WeightInfo::approve(0, 0))
				);
			}
			// Check error works for `Assets::approve_transfer()` in `Greater` match arm.
			assert_noop!(
				Fungibles::approve(signed(owner), token, spender, value),
				AssetsError::Unknown.with_weight(WeightInfo::approve(1, 0))
			);
			assets::create_mint_and_approve(owner, token, owner, value, spender, value);
			// Check error works for `Assets::cancel_approval()` in `Less` match arm.
			assert_ok!(Assets::freeze_asset(signed(owner), token));
			assert_noop!(
				Fungibles::approve(signed(owner), token, spender, value / 2),
				AssetsError::AssetNotLive.with_weight(WeightInfo::approve(0, 1))
			);
			assert_ok!(Assets::thaw_asset(signed(owner), token));
			// No error test for `approve_transfer` in `Less` arm because it is not possible.
		});
	}

	// Non-additive, sets new value.
	#[test]
	fn approve_works() {
		new_test_ext().execute_with(|| {
			let value: Balance = 100 * UNIT;
			let token = TOKEN;
			let owner = ALICE;
			let spender = BOB;

			// Approves a value to spend that is higher than the current allowance.
			assets::create_and_mint_to(owner, token, owner, value);
			assert_eq!(Assets::allowance(token, &owner, &spender), 0);
			assert_eq!(
				Fungibles::approve(signed(owner), token, spender, value),
				Ok(Some(WeightInfo::approve(1, 0)).into())
			);
			assert_eq!(Assets::allowance(token, &owner, &spender), value);
			System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
			// Approves a value to spend that is lower than the current allowance.
			assert_eq!(
				Fungibles::approve(signed(owner), token, spender, value / 2),
				Ok(Some(WeightInfo::approve(1, 1)).into())
			);
			assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
			System::assert_last_event(
				Event::Approval { token, owner, spender, value: value / 2 }.into(),
			);
			// Approves a value to spend that is equal to the current allowance.
			assert_eq!(
				Fungibles::approve(signed(owner), token, spender, value / 2),
				Ok(Some(WeightInfo::approve(0, 0)).into())
			);
			assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
			System::assert_last_event(
				Event::Approval { token, owner, spender, value: value / 2 }.into(),
			);
			// Sets allowance to zero.
			assert_eq!(
				Fungibles::approve(signed(owner), token, spender, 0),
				Ok(Some(WeightInfo::approve(0, 1)).into())
			);
			assert_eq!(Assets::allowance(token, &owner, &spender), 0);
			System::assert_last_event(Event::Approval { token, owner, spender, value: 0 }.into());
		});
	}
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let spender = BOB;

		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::increase_allowance(origin, token, spender, value),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		// Check error works for `Assets::approve_transfer()`.
		assert_noop!(
			Fungibles::increase_allowance(signed(owner), token, spender, value),
			AssetsError::Unknown.with_weight(AssetsWeightInfo::approve_transfer())
		);
		assets::create_and_mint_to(owner, token, owner, value);
		assert_eq!(0, Assets::allowance(token, &owner, &spender));
		assert_ok!(Fungibles::increase_allowance(signed(owner), token, spender, value));
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
		// Additive.
		assert_ok!(Fungibles::increase_allowance(signed(owner), token, spender, value));
		assert_eq!(Assets::allowance(token, &owner, &spender), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let spender = BOB;

		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::decrease_allowance(origin, token, spender, 0),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		// Check error works for `Assets::cancel_approval()`. No error test for `approve_transfer`
		// because it is not possible.
		assert_noop!(
			Fungibles::decrease_allowance(signed(owner), token, spender, value / 2),
			AssetsError::Unknown.with_weight(WeightInfo::approve(0, 1))
		);
		assets::create_mint_and_approve(owner, token, owner, value, spender, value);
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		// Owner balance is not changed if decreased by zero.
		assert_eq!(
			Fungibles::decrease_allowance(signed(owner), token, spender, 0),
			Ok(Some(WeightInfo::approve(0, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		// Decrease allowance successfully.
		assert_eq!(
			Fungibles::decrease_allowance(signed(owner), token, spender, value / 2),
			Ok(Some(WeightInfo::approve(1, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Saturating if current allowance is decreased more than the owner balance.
		assert_eq!(
			Fungibles::decrease_allowance(signed(owner), token, spender, value),
			Ok(Some(WeightInfo::approve(0, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), 0);
		System::assert_last_event(Event::Approval { token, owner, spender, value: 0 }.into());
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let id = TOKEN;
		let creator = ALICE;
		let admin = ALICE;

		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::create(origin, id, admin, 100), BadOrigin);
		}
		assert!(!Assets::asset_exists(id));
		assert_ok!(Fungibles::create(signed(creator), id, admin, 100));
		assert!(Assets::asset_exists(id));
		System::assert_last_event(Event::Create { id, creator, admin }.into());
		// Check error works for `Assets::create()`.
		assert_noop!(Fungibles::create(signed(creator), id, admin, 100), AssetsError::InUse);
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;

		// Check error works for `Assets::start_destroy()`.
		assert_noop!(Fungibles::start_destroy(signed(ALICE), token), AssetsError::Unknown);
		assert_ok!(Assets::create(signed(ALICE), token, ALICE, 1));
		assert_ok!(Fungibles::start_destroy(signed(ALICE), token));
		// Check that the token is not live after starting the destroy process.
		assert_noop!(
			Assets::mint(signed(ALICE), token, ALICE, 10 * UNIT),
			AssetsError::AssetNotLive
		);
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42;

		// Check error works for `Assets::set_metadata()`.
		assert_noop!(
			Fungibles::set_metadata(signed(ALICE), token, name.clone(), symbol.clone(), decimals),
			AssetsError::Unknown
		);
		assert_ok!(Assets::create(signed(ALICE), token, ALICE, 1));
		assert_ok!(Fungibles::set_metadata(
			signed(ALICE),
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
	new_test_ext().execute_with(|| {
		let token = TOKEN;

		// Check error works for `Assets::clear_metadata()`.
		assert_noop!(Fungibles::clear_metadata(signed(ALICE), token), AssetsError::Unknown);
		assets::create_and_set_metadata(ALICE, token, vec![42], vec![42], 42);
		assert_ok!(Fungibles::clear_metadata(signed(ALICE), token));
		assert!(Assets::name(token).is_empty());
		assert!(Assets::symbol(token).is_empty());
		assert!(Assets::decimals(token).is_zero());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = ALICE;
		let to = BOB;

		// Check error works for `Assets::mint()`.
		assert_noop!(
			Fungibles::mint(signed(from), token, to, value),
			sp_runtime::TokenError::UnknownAsset
		);
		assert_ok!(Assets::create(signed(from), token, from, 1));
		let balance_before_mint = Assets::balance(token, &to);
		assert_ok!(Fungibles::mint(signed(from), token, to, value));
		let balance_after_mint = Assets::balance(token, &to);
		assert_eq!(balance_after_mint, balance_before_mint + value);
		System::assert_last_event(
			Event::Transfer { token, from: None, to: Some(to), value }.into(),
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let from = BOB;
		let total_supply = value * 2;

		// Check error works for `Assets::burn()`.
		assert_noop!(Fungibles::burn(signed(owner), token, from, value), AssetsError::Unknown);
		assets::create_and_mint_to(owner, token, from, total_supply);
		assert_eq!(Assets::total_supply(TOKEN), total_supply);
		let balance_before_burn = Assets::balance(token, &from);
		assert_ok!(Fungibles::burn(signed(owner), token, from, value));
		assert_eq!(Assets::total_supply(TOKEN), total_supply - value);
		let balance_after_burn = Assets::balance(token, &from);
		assert_eq!(balance_after_burn, balance_before_burn - value);
		System::assert_last_event(
			Event::Transfer { token, from: Some(from), to: None, value }.into(),
		);
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let total_supply = INIT_AMOUNT;
		assert_eq!(
			Fungibles::read(TotalSupply(TOKEN)),
			ReadResult::TotalSupply(Default::default())
		);
		assets::create_and_mint_to(ALICE, TOKEN, ALICE, total_supply);
		assert_eq!(Fungibles::read(TotalSupply(TOKEN)), ReadResult::TotalSupply(total_supply));
		assert_eq!(
			Fungibles::read(TotalSupply(TOKEN)).encode(),
			Assets::total_supply(TOKEN).encode(),
		);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let value = 1_000 * UNIT;
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }),
			ReadResult::BalanceOf(Default::default())
		);
		assets::create_and_mint_to(ALICE, TOKEN, ALICE, value);
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }),
			ReadResult::BalanceOf(value)
		);
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }).encode(),
			Assets::balance(TOKEN, ALICE).encode(),
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let value = 1_000 * UNIT;
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }),
			ReadResult::Allowance(Default::default())
		);
		assets::create_mint_and_approve(ALICE, TOKEN, ALICE, value * 2, BOB, value);
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }),
			ReadResult::Allowance(value)
		);
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }).encode(),
			Assets::allowance(TOKEN, &ALICE, &BOB).encode(),
		);
	});
}

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;
		assert_eq!(Fungibles::read(TokenName(TOKEN)), ReadResult::TokenName(Default::default()));
		assert_eq!(
			Fungibles::read(TokenSymbol(TOKEN)),
			ReadResult::TokenSymbol(Default::default())
		);
		assert_eq!(
			Fungibles::read(TokenDecimals(TOKEN)),
			ReadResult::TokenDecimals(Default::default())
		);
		assets::create_and_set_metadata(ALICE, TOKEN, name.clone(), symbol.clone(), decimals);
		assert_eq!(Fungibles::read(TokenName(TOKEN)), ReadResult::TokenName(name));
		assert_eq!(Fungibles::read(TokenSymbol(TOKEN)), ReadResult::TokenSymbol(symbol));
		assert_eq!(Fungibles::read(TokenDecimals(TOKEN)), ReadResult::TokenDecimals(decimals));
		assert_eq!(Fungibles::read(TokenName(TOKEN)).encode(), Assets::name(TOKEN).encode());
		assert_eq!(Fungibles::read(TokenSymbol(TOKEN)).encode(), Assets::symbol(TOKEN).encode());
		assert_eq!(
			Fungibles::read(TokenDecimals(TOKEN)).encode(),
			Assets::decimals(TOKEN).encode(),
		);
	});
}

#[test]
fn token_exists_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Fungibles::read(TokenExists(TOKEN)), ReadResult::TokenExists(false));
		assert_ok!(Assets::create(signed(ALICE), TOKEN, ALICE, 1));
		assert_eq!(Fungibles::read(TokenExists(TOKEN)), ReadResult::TokenExists(true));
		assert_eq!(
			Fungibles::read(TokenExists(TOKEN)).encode(),
			Assets::asset_exists(TOKEN).encode(),
		);
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn root() -> RuntimeOrigin {
	RuntimeOrigin::root()
}

fn none() -> RuntimeOrigin {
	RuntimeOrigin::none()
}

// Helper functions for interacting with pallet-assets.
mod assets {
	use super::*;

	pub(super) fn create_and_mint_to(
		owner: AccountId,
		token: TokenId,
		to: AccountId,
		value: Balance,
	) {
		assert_ok!(Assets::create(signed(owner), token, owner, 1));
		assert_ok!(Assets::mint(signed(owner), token, to, value));
	}

	pub(super) fn create_mint_and_approve(
		owner: AccountId,
		token: TokenId,
		to: AccountId,
		mint: Balance,
		spender: AccountId,
		approve: Balance,
	) {
		create_and_mint_to(owner, token, to, mint);
		assert_ok!(Assets::approve_transfer(signed(to), token, spender, approve,));
	}

	pub(super) fn create_and_set_metadata(
		owner: AccountId,
		token: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) {
		assert_ok!(Assets::create(signed(owner), token, owner, 1));
		assert_ok!(Assets::set_metadata(signed(owner), token, name, symbol, decimals));
	}
}

mod read_weights {
	use frame_support::weights::Weight;

	use super::*;
	use crate::fungibles::{weights::WeightInfo, Config};

	struct ReadWeightInfo {
		total_supply: Weight,
		balance_of: Weight,
		allowance: Weight,
		token_name: Weight,
		token_symbol: Weight,
		token_decimals: Weight,
		token_exists: Weight,
	}

	impl ReadWeightInfo {
		fn new() -> Self {
			Self {
				total_supply: Fungibles::weight(&TotalSupply(TOKEN)),
				balance_of: Fungibles::weight(&BalanceOf { token: TOKEN, owner: ALICE }),
				allowance: Fungibles::weight(&Allowance {
					token: TOKEN,
					owner: ALICE,
					spender: BOB,
				}),
				token_name: Fungibles::weight(&TokenName(TOKEN)),
				token_symbol: Fungibles::weight(&TokenSymbol(TOKEN)),
				token_decimals: Fungibles::weight(&TokenDecimals(TOKEN)),
				token_exists: Fungibles::weight(&TokenExists(TOKEN)),
			}
		}
	}

	#[test]
	fn ensure_read_matches_benchmarks() {
		let ReadWeightInfo {
			allowance,
			balance_of,
			token_decimals,
			token_name,
			token_symbol,
			total_supply,
			token_exists,
		} = ReadWeightInfo::new();

		assert_eq!(total_supply, <Test as Config>::WeightInfo::total_supply());
		assert_eq!(balance_of, <Test as Config>::WeightInfo::balance_of());
		assert_eq!(allowance, <Test as Config>::WeightInfo::allowance());
		assert_eq!(token_name, <Test as Config>::WeightInfo::token_name());
		assert_eq!(token_symbol, <Test as Config>::WeightInfo::token_symbol());
		assert_eq!(token_decimals, <Test as Config>::WeightInfo::token_decimals());
		assert_eq!(token_exists, <Test as Config>::WeightInfo::token_exists());
	}

	// These types read from the `AssetMetadata` storage.
	#[test]
	fn ensure_asset_metadata_variants_match() {
		let ReadWeightInfo { token_decimals, token_name, token_symbol, .. } = ReadWeightInfo::new();

		assert_eq!(token_decimals, token_name);
		assert_eq!(token_decimals, token_symbol);
	}

	// These types read from the `Assets` storage.
	#[test]
	fn ensure_asset_variants_match() {
		let ReadWeightInfo { total_supply, token_exists, .. } = ReadWeightInfo::new();

		assert_eq!(total_supply, token_exists);
	}

	// Proof size is based on `MaxEncodedLen`, not hardware.
	// This test ensures that the data structure sizes do not change with upgrades.
	#[test]
	fn ensure_expected_proof_size_does_not_change() {
		let ReadWeightInfo {
			allowance,
			balance_of,
			token_decimals,
			token_name,
			token_symbol,
			total_supply,
			token_exists,
		} = ReadWeightInfo::new();

		// These values come from `weights.rs`.
		assert_eq!(allowance.proof_size(), 3613);
		assert_eq!(balance_of.proof_size(), 3599);
		assert_eq!(token_name.proof_size(), 3605);
		assert_eq!(token_symbol.proof_size(), 3605);
		assert_eq!(token_decimals.proof_size(), 3605);
		assert_eq!(total_supply.proof_size(), 3675);
		assert_eq!(token_exists.proof_size(), 3675);
	}
}

mod ensure_codec_indexes {
	use super::{Encode, RuntimeCall, *};
	use crate::{fungibles, fungibles::Call::*, mock::RuntimeCall::Fungibles};

	#[test]
	fn ensure_read_variant_indexes() {
		[
			// explicit u8 to help Rust infer the type
			(TotalSupply::<Test>(Default::default()), 0u8, "TotalSupply"),
			(
				BalanceOf::<Test> { token: Default::default(), owner: Default::default() },
				1,
				"BalanceOf",
			),
			(
				Allowance::<Test> {
					token: Default::default(),
					owner: Default::default(),
					spender: Default::default(),
				},
				2,
				"Allowance",
			),
			(TokenName::<Test>(Default::default()), 8, "TokenName"),
			(TokenSymbol::<Test>(Default::default()), 9, "TokenSymbol"),
			(TokenDecimals::<Test>(Default::default()), 10, "TokenDecimals"),
			(TokenExists::<Test>(Default::default()), 18, "TokenExists"),
		]
		.iter()
		.for_each(|(encoded_variant, expected_index, name)| {
			assert_eq!(
				encoded_variant.encode()[0],
				*expected_index,
				"{name} variant index changed"
			);
		})
	}

	#[test]
	fn ensure_dispatchable_indexes() {
		use fungibles::Call::*;

		[
			(
				Fungibles(transfer {
					token: Default::default(),
					to: Default::default(),
					value: Default::default(),
				}),
				3u8, // explicit u8 to help Rust infer the type
				"transfer",
			),
			(
				Fungibles(transfer_from {
					token: Default::default(),
					from: Default::default(),
					to: Default::default(),
					value: Default::default(),
				}),
				4,
				"transfer_from",
			),
			(
				Fungibles(approve {
					token: Default::default(),
					spender: Default::default(),
					value: Default::default(),
				}),
				5,
				"approve",
			),
			(
				Fungibles(increase_allowance {
					token: Default::default(),
					spender: Default::default(),
					value: Default::default(),
				}),
				6,
				"increase_allowance",
			),
			(
				Fungibles(decrease_allowance {
					token: Default::default(),
					spender: Default::default(),
					value: Default::default(),
				}),
				7,
				"decrease_allowance",
			),
			(
				Fungibles(create {
					id: Default::default(),
					admin: Default::default(),
					min_balance: Default::default(),
				}),
				11,
				"create",
			),
			(Fungibles(start_destroy { token: Default::default() }), 12, "start_destroy"),
			(
				Fungibles(set_metadata {
					token: Default::default(),
					name: Default::default(),
					symbol: Default::default(),
					decimals: Default::default(),
				}),
				16,
				"set_metadata",
			),
			(Fungibles(clear_metadata { token: Default::default() }), 17, "clear_metadata"),
			(
				Fungibles(mint {
					token: Default::default(),
					account: Default::default(),
					value: Default::default(),
				}),
				19,
				"mint",
			),
			(
				Fungibles(burn {
					token: Default::default(),
					account: Default::default(),
					value: Default::default(),
				}),
				20,
				"burn",
			),
		]
		.iter()
		.for_each(|(encoded_variant, expected_index, name)| {
			assert_eq!(
				encoded_variant.encode()[1],
				*expected_index,
				"{name} dispatchable index changed"
			);
		})
	}
}
