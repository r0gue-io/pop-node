use alloc::string::String;

pub use erc20::{Erc20, IERC20};
use frame_support::traits::fungibles::metadata::Inspect as _;
use pallet_revive::precompiles::alloy::{
	primitives::{
		ruint::{UintTryFrom, UintTryTo},
		Address,
	},
	sol_types::SolCall,
};
use IFungibles::*;

use super::*;

mod erc20;

sol!("src/fungibles/precompiles/interfaces/IFungibles.sol");

/// The fungibles precompile offers a streamlined interface for interacting with fungible tokens.
/// The goal is to provide a simplified, consistent API that adheres to standards in the smart
/// contract space.
pub struct Fungibles<const FIXED: u16, T, I>(PhantomData<(T, I)>);
impl<
		const FIXED: u16,
		T: frame_system::Config
			+ Config<I, AssetId: Default + From<u32> + Into<u32>>
			+ pallet_revive::Config,
		I: 'static,
	> Precompile for Fungibles<FIXED, T, I>
where
	U256: UintTryFrom<T::Balance> + UintTryTo<<T as Config<I>>::Balance>,
{
	type Interface = IFungiblesCalls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Fixed(NonZero::new(FIXED).expect("expected non-zero precompile address"));

	fn call(
		address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		match input {
			IFungiblesCalls::transfer(transferCall { token, to, value }) => {
				// TODO: charge based on benchmarked weight
				let from = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				self::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event(
					env,
					address,
					Transfer { token: *token, from, to: *to, value: *value },
				);
				Ok(transferCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::transferFrom(transferFromCall { token, from, to, value }) => {
				// TODO: charge based on benchmarked weight

				transfer_from::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*from.0).into()),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				let value = *value;
				deposit_event(
					env,
					address,
					Transfer { token: *token, from: *from, to: *to, value },
				);
				Ok(transferFromCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::approve(approveCall { token, spender, value }) => {
				// TODO: charge based on benchmarked weight
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				self::approve::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;

				let spender = *spender;
				deposit_event(
					env,
					address,
					Approval { token: *token, owner, spender, value: *value },
				);
				Ok(approveCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::increaseAllowance(increaseAllowanceCall { token, spender, value }) => {
				// TODO: charge based on benchmarked weight
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let value = U256::saturating_from(
					self::increase_allowance::<T, I>(
						to_runtime_origin(env.caller()),
						(*token).into(),
						env.to_account_id(&(*spender.0).into()),
						value.saturating_to(),
					) // TODO: adjust weight
					.map_err(|e| e.error)?,
				);

				let spender = *spender;
				deposit_event(env, address, Approval { token: *token, owner, spender, value });
				Ok(increaseAllowanceCall::abi_encode_returns(&(value,)))
			},
			IFungiblesCalls::decreaseAllowance(decreaseAllowanceCall { token, spender, value }) => {
				// TODO: charge based on benchmarked weight
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let (value, weight) = self::decrease_allowance::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;
				let value = U256::saturating_from(value);

				let spender = *spender;
				deposit_event(env, address, Approval { token: *token, owner, spender, value });
				Ok(decreaseAllowanceCall::abi_encode_returns(&(value,)))
			},
			IFungiblesCalls::create(createCall { admin, minBalance }) => {
				// TODO: charge based on benchmarked weight
				let creator = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let id = self::create::<T, I>(
					to_runtime_origin(env.caller()),
					env.to_account_id(&(*admin.0).into()),
					minBalance.saturating_to(),
				)?
				.into();

				deposit_event(env, address, Created { id, creator, admin: *admin });
				Ok(createCall::abi_encode_returns(&(id,)))
			},
			IFungiblesCalls::startDestroy(startDestroyCall { token }) => {
				// TODO: charge based on benchmarked weight
				start_destroy::<T, I>(to_runtime_origin(env.caller()), (*token).into())?;
				Ok(startDestroyCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::setMetadata(setMetadataCall { token, name, symbol, decimals }) => {
				// TODO: charge based on benchmarked weight
				set_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					name.as_bytes().to_vec(),
					symbol.as_bytes().to_vec(),
					*decimals,
				)?;
				Ok(setMetadataCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::clearMetadata(clearMetadataCall { token }) => {
				// TODO: charge based on benchmarked weight
				clear_metadata::<T, I>(to_runtime_origin(env.caller()), (*token).into())?;
				Ok(clearMetadataCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::mint(mintCall { token, account, value }) => {
				// TODO: charge based on benchmarked weight

				self::mint::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*account.0).into()),
					value.saturating_to(),
				)?;

				let from = Address::default();
				let to = *account;
				deposit_event(env, address, Transfer { token: *token, from, to, value: *value });
				Ok(mintCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::burn(burnCall { token, account, value }) => {
				// TODO: charge based on benchmarked weight

				self::burn::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*account.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;

				let from = account;
				let to = Address::default();
				deposit_event(
					env,
					address,
					Transfer { token: *token, from: *from, to, value: *value },
				);
				Ok(burnCall::abi_encode_returns(&()))
			},
			IFungiblesCalls::totalSupply(totalSupplyCall { token }) => {
				// TODO: charge based on benchmarked weight
				let total_supply =
					U256::saturating_from(<Assets<T, I>>::total_supply((*token).into()));
				Ok(totalSupplyCall::abi_encode_returns(&(total_supply,)))
			},
			IFungiblesCalls::balanceOf(balanceOfCall { token, owner }) => {
				// TODO: charge based on benchmarked weight
				let account = env.to_account_id(&(*owner.0).into());
				let balance =
					U256::saturating_from(<Assets<T, I>>::balance((*token).into(), account));
				Ok(balanceOfCall::abi_encode_returns(&(balance,)))
			},
			IFungiblesCalls::allowance(allowanceCall { token, owner, spender }) => {
				// TODO: charge based on benchmarked weight
				let owner = env.to_account_id(&(*owner.0).into());
				let spender = env.to_account_id(&(*spender.0).into());
				let remaining = U256::saturating_from(<Assets<T, I>>::allowance(
					(*token).into(),
					&owner,
					&spender,
				));
				Ok(allowanceCall::abi_encode_returns(&(remaining,)))
			},
			IFungiblesCalls::name(nameCall { token }) => {
				// TODO: charge based on benchmarked weight
				let result = <Assets<T, I>>::name((*token).into());
				// TODO: improve
				let result = String::from_utf8_lossy(result.as_slice());
				Ok(nameCall::abi_encode_returns(&(result,)))
			},
			IFungiblesCalls::symbol(symbolCall { token }) => {
				// TODO: charge based on benchmarked weight
				let result = <Assets<T, I>>::symbol((*token).into());
				// TODO: improve
				let result = String::from_utf8_lossy(result.as_slice());
				Ok(nameCall::abi_encode_returns(&(result,)))
			},
			IFungiblesCalls::decimals(decimalsCall { token }) => {
				// TODO: charge based on benchmarked weight
				let result = <Assets<T, I>>::decimals((*token).into());
				Ok(decimalsCall::abi_encode_returns(&(result,)))
			},
			IFungiblesCalls::exists(existsCall { token }) => {
				// TODO: charge based on benchmarked weight
				let result = self::exists::<T, I>((*token).into());
				Ok(existsCall::abi_encode_returns(&(result,)))
			},
		}
	}
}

impl<const FIXED: u16, T: Config<I>, I: 'static> Fungibles<FIXED, T, I> {
	pub const fn address() -> [u8; 20] {
		fixed_address(FIXED)
	}
}

#[cfg(test)]
mod tests {
	use frame_support::{
		assert_ok, sp_runtime::traits::AccountIdLookup, traits::fungibles::Inspect,
		weights::Weight, BoundedVec,
	};
	use mock::{Assets, ExtBuilder, *};
	use pallet_assets::{AssetDetails, AssetMetadata, AssetStatus, Instance1};
	use pallet_revive::{
		precompiles::{
			alloy::sol_types::{SolInterface, SolType},
			ExtWithInfo,
		},
		test_utils::{ALICE_ADDR, BOB, BOB_ADDR},
		DepositLimit::*,
	};

	use super::{
		IFungibles::IFungiblesCalls::{clearMetadata, setMetadata, startDestroy},
		*,
	};

	const FUNGIBLES: u16 = 100;
	const ADDRESS: [u8; 20] = fixed_address(FUNGIBLES);

	type Asset = pallet_assets::Asset<Test, Instance1>;
	type Fungibles = super::Fungibles<FUNGIBLES, Test, Instance1>;
	type Metadata = pallet_assets::Metadata<Test, Instance1>;

	#[test]
	fn transfer_works() {
		let token = 1;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, true, 1)])
			.with_asset_balances(vec![(token, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::balance(token, ALICE), endowment);
				assert_eq!(Assets::balance(token, BOB), 0);

				let value = endowment / 2;
				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::transfer(transferCall {
						token,
						to: BOB_ADDR.0.into(),
						value: U256::from(value)
					})
				));

				assert_eq!(Assets::balance(token, ALICE), endowment - value);
				assert_eq!(Assets::balance(token, BOB), value);

				let from = ALICE_ADDR.0.into();
				let to = BOB_ADDR.0.into();
				let event = Transfer { token, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn transfer_from_works() {
		let token = 1;
		let endowment = 10_000_000;
		let value = endowment / 2;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, true, 1)])
			.with_asset_balances(vec![(token, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				assert_eq!(Assets::balance(token, ALICE), endowment);
				assert_eq!(Assets::balance(token, BOB), 0);
				assert_ok!(approve::<Test, Instance1>(
					RuntimeOrigin::signed(ALICE),
					token,
					BOB,
					value
				));
				call_setup.set_origin(Origin::Signed(BOB));

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::transferFrom(transferFromCall {
						token,
						from: ALICE_ADDR.0.into(),
						to: BOB_ADDR.0.into(),
						value: U256::from(value),
					})
				));

				assert_eq!(Assets::balance(token, ALICE), endowment - value);
				assert_eq!(Assets::balance(token, BOB), value);
				let from = ALICE_ADDR.0.into();
				let to = BOB_ADDR.0.into();
				let event = Transfer { token, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn approve_works() {
		let token = 1;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::allowance(token, &ALICE, &BOB), 0);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::approve(approveCall {
						token,
						spender: BOB_ADDR.0.into(),
						value: U256::from(value),
					})
				));

				assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
				let owner = ALICE_ADDR.0.into();
				let spender = BOB_ADDR.0.into();
				let event = Approval { token, owner, spender, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			},
		);
	}

	#[test]
	fn increase_allowance_works() {
		let token = 1;
		let origin = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(origin.clone()));
				assert_ok!(Assets::approve_transfer(
					RuntimeOrigin::signed(origin.clone()),
					token.into(),
					AccountIdLookup::unlookup(spender.clone()),
					value,
				));
				assert_eq!(Assets::allowance(token, &origin, &spender), value);

				// Double the allowance.
				let allowance = call_precompile::<U256>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::increaseAllowance(increaseAllowanceCall {
						token,
						spender: BOB_ADDR.0.into(),
						value: U256::from(value),
					}),
				)
				.unwrap();

				assert_eq!(allowance, U256::from(value * 2));
				assert_eq!(Assets::allowance(token, &origin, &spender), value * 2);
				let owner = ALICE_ADDR.0.into();
				let spender = BOB_ADDR.0.into();
				let event = Approval { token, owner, spender, value: U256::from(allowance) };
				assert_last_event(ADDRESS, event);
			},
		);
	}

	#[test]
	fn decrease_allowance_works() {
		let token = 1;
		let origin = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(origin.clone()));
				assert_ok!(Assets::approve_transfer(
					RuntimeOrigin::signed(origin.clone()),
					token.into(),
					AccountIdLookup::unlookup(spender.clone()),
					value,
				));
				assert_eq!(Assets::allowance(token, &origin, &spender), value);

				// Halve the allowance.
				let allowance = call_precompile::<U256>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::decreaseAllowance(decreaseAllowanceCall {
						token,
						spender: BOB_ADDR.0.into(),
						value: U256::from(value / 2),
					}),
				)
				.unwrap();

				assert_eq!(allowance, U256::from(value / 2));
				assert_eq!(Assets::allowance(token, &origin, &spender), value / 2);
				let owner = ALICE_ADDR.0.into();
				let spender = BOB_ADDR.0.into();
				let event = Approval { token, owner, spender, value: U256::from(allowance) };
				assert_last_event(ADDRESS, event);
			},
		);
	}

	#[test]
	fn create_works() {
		let id = 0u32;
		let min_balance = 1;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			call_setup.set_origin(Origin::Signed(ALICE));
			assert!(!Assets::asset_exists(id));

			let mut ext = call_setup.ext().0;
			let result: u32 = call_precompile(
				&mut ext,
				&IFungiblesCalls::create(createCall {
					admin: BOB_ADDR.0.into(),
					minBalance: U256::from(min_balance),
				}),
			)
			.unwrap();

			assert_eq!(result, id);
			assert!(Assets::asset_exists(id));
			assert_eq!(
				Asset::get(id).unwrap(),
				AssetDetails {
					owner: ALICE,
					issuer: BOB,
					admin: BOB,
					freezer: BOB,
					supply: 0,
					deposit: min_balance,
					min_balance,
					is_sufficient: false,
					accounts: 0,
					sufficients: 0,
					approvals: 0,
					status: AssetStatus::Live,
				}
			);

			let event = Created { id, creator: ALICE_ADDR.0.into(), admin: BOB_ADDR.0.into() };
			assert_last_event(ADDRESS, event);
		});
	}

	#[test]
	fn create_via_bare_call_works() {
		let id = 0u32;
		let origin = RuntimeOrigin::signed(ALICE);
		ExtBuilder::new().build().execute_with(|| {
			assert!(!Assets::asset_exists(id));

			let result = bare_call::<Test, u32>(
				origin,
				ADDRESS.into(),
				0,
				Weight::MAX,
				DepositLimit::Balance(u128::MAX),
				IFungiblesCalls::create(createCall {
					admin: BOB_ADDR.0.into(),
					minBalance: U256::from(1),
				})
				.abi_encode(),
			)
			.unwrap();

			assert_eq!(result, id);
			assert!(Assets::asset_exists(id));
			let event = Created { id, creator: ALICE_ADDR.0.into(), admin: BOB_ADDR.0.into() };
			assert_last_event(ADDRESS, event);
		});
	}

	#[test]
	fn start_destroy_works() {
		let token = 1;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Asset::get(token).unwrap().status, AssetStatus::Live);

				let mut ext = call_setup.ext().0;
				assert_ok!(call_precompile::<()>(
					&mut ext,
					&startDestroy(startDestroyCall { token })
				));

				assert_eq!(Asset::get(token).unwrap().status, AssetStatus::Destroying);
			},
		);
	}

	#[test]
	fn set_metadata_works() {
		let token = 1;
		let name = "name".to_string();
		let symbol = "symbol".to_string();
		let decimals = u8::MAX;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Metadata::get(token), AssetMetadata::default());

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&setMetadata(setMetadataCall {
						token,
						name: name.clone(),
						symbol: symbol.clone(),
						decimals
					})
				));

				assert_eq!(
					Metadata::get(token),
					AssetMetadata {
						deposit: 11,
						name: BoundedVec::truncate_from(name.into_bytes()),
						symbol: BoundedVec::truncate_from(symbol.into_bytes()),
						decimals,
						is_frozen: false,
					}
				);
			},
		);
	}

	#[test]
	fn clear_metadata_works() {
		let token = 1;
		let name = b"name".to_vec();
		let symbol = b"symbol".to_vec();
		let decimals = u8::MAX;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, name.clone(), symbol.clone(), decimals)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(
					Metadata::get(token),
					AssetMetadata {
						deposit: 0,
						name: BoundedVec::truncate_from(name),
						symbol: BoundedVec::truncate_from(symbol),
						decimals,
						is_frozen: false,
					}
				);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&clearMetadata(clearMetadataCall { token })
				));

				assert_eq!(Metadata::get(token), AssetMetadata::default());
			});
	}

	#[test]
	fn mint_works() {
		let token = 1;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::balance(token, ALICE), 0);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::mint(mintCall {
						token,
						account: ALICE_ADDR.0.into(),
						value: U256::from(value)
					})
				));

				assert_eq!(Assets::balance(token, ALICE), value);
				let from = Address::default();
				let to = ALICE_ADDR.0.into();
				let event = Transfer { token, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			},
		);
	}

	#[test]
	fn burn_works() {
		let token = 1;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_balances(vec![(token, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::balance(token, ALICE), endowment);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::burn(burnCall {
						token,
						account: ALICE_ADDR.0.into(),
						value: U256::from(endowment),
					}),
				));

				assert_eq!(Assets::balance(token, ALICE), 0);
				let from = ALICE_ADDR.0.into();
				let to = Address::default();
				let event = Transfer { token, from, to, value: U256::from(endowment) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn total_supply_works() {
		let token = u32::MAX;
		let total_supply = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_balances(vec![(token, ALICE, total_supply)])
			.build_with_env(|mut call_setup| {
				assert_eq!(
					call_precompile::<U256>(
						&mut call_setup.ext().0,
						&IFungiblesCalls::totalSupply(totalSupplyCall { token })
					)
					.unwrap(),
					U256::from(total_supply)
				);
			});
	}

	#[test]
	fn balance_of_works() {
		let token = u32::MAX;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_balances(vec![(token, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));

				assert_eq!(
					call_precompile::<U256>(
						&mut call_setup.ext().0,
						&IFungiblesCalls::balanceOf(balanceOfCall {
							token,
							owner: ALICE_ADDR.0.into()
						})
					)
					.unwrap(),
					U256::from(endowment)
				);

				assert_eq!(Assets::balance(token, ALICE), endowment);
			});
	}

	#[test]
	fn allowance_works() {
		let token = u32::MAX;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				assert_ok!(approve::<Test, Instance1>(
					RuntimeOrigin::signed(ALICE),
					token,
					BOB,
					value
				));

				assert_eq!(
					call_precompile::<U256>(
						&mut call_setup.ext().0,
						&IFungiblesCalls::allowance(allowanceCall {
							token,
							owner: ALICE_ADDR.0.into(),
							spender: BOB_ADDR.0.into(),
						})
					)
					.unwrap(),
					U256::from(value)
				);
			},
		);
	}

	#[test]
	fn name_works() {
		let token = 1;
		let name = "name".to_string();
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, name.as_bytes().into(), b"symbol".to_vec(), u8::MAX)])
			.build_with_env(|mut call_setup| {
				let mut ext = call_setup.ext().0;
				assert_eq!(
					call_precompile::<String>(&mut ext, &IFungiblesCalls::name(nameCall { token }))
						.unwrap(),
					name
				);
			});
	}

	#[test]
	fn symbol_works() {
		let token = 1;
		let symbol = "symbol".to_string();
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), symbol.as_bytes().into(), u8::MAX)])
			.build_with_env(|mut call_setup| {
				let mut ext = call_setup.ext().0;
				assert_eq!(
					call_precompile::<String>(
						&mut ext,
						&IFungiblesCalls::symbol(symbolCall { token })
					)
					.unwrap(),
					symbol
				);
			});
	}

	#[test]
	fn decimals_works() {
		let token = 1;
		let decimals = u8::MAX;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), b"symbol".to_vec(), decimals)])
			.build_with_env(|mut call_setup| {
				let mut ext = call_setup.ext().0;
				assert_eq!(
					call_precompile::<u16>(
						&mut ext,
						&IFungiblesCalls::decimals(decimalsCall { token })
					)
					.unwrap() as u8,
					decimals
				);
			});
	}

	#[test]
	fn exists_works() {
		let token = 1;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert!(Assets::asset_exists(token));

				let mut ext = call_setup.ext().0;
				assert!(call_precompile::<bool>(
					&mut ext,
					&IFungiblesCalls::exists(existsCall { token })
				)
				.unwrap());

				let token = token + 1;
				assert!(!Assets::asset_exists(token));
				assert!(!call_precompile::<bool>(
					&mut ext,
					&IFungiblesCalls::exists(existsCall { token })
				)
				.unwrap());
			},
		);
	}

	#[test]
	fn exists_via_bare_call_works() {
		let token = 1;
		let origin = RuntimeOrigin::signed(ALICE);
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.build()
			.execute_with(|| {
				assert!(Assets::asset_exists(token));

				let asset_exists = bare_call::<Test, bool>(
					origin.clone(),
					ADDRESS.into(),
					0,
					Weight::MAX,
					Unchecked,
					IFungiblesCalls::exists(existsCall { token }).abi_encode(),
				)
				.unwrap();
				assert!(asset_exists);

				let token = token + 1;
				assert!(!Assets::asset_exists(token));
				let exists = bare_call::<Test, bool>(
					origin,
					ADDRESS.into(),
					0,
					Weight::MAX,
					Unchecked,
					IFungiblesCalls::exists(existsCall { token }).abi_encode(),
				)
				.unwrap();
				assert!(!exists);
			});
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		ext: &mut impl ExtWithInfo<T = Test>,
		input: &IFungiblesCalls,
	) -> Result<Output, Error> {
		super::call_precompile::<Fungibles, Output>(ext, &ADDRESS, input)
	}
}
