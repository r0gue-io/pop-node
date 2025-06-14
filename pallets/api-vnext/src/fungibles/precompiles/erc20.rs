use alloc::{string::String, vec::Vec};

use frame_support::{
	sp_runtime::traits::AtLeast32Bit,
	traits::fungibles::{approvals::Inspect as _, metadata::Inspect as _},
};
use pallet_revive::AddressMapper as _;
use AddressMatcher::Prefix;
use IERC20::*;

use super::{
	deposit_event, prefixed_address, sol, to_runtime_origin, AddressMapper, AddressMatcher,
	AssetIdExtractor, Assets, Config, Error, Ext, InlineAssetIdExtractor, NonZero, PhantomData,
	Precompile, SolCall, UintTryFrom, UintTryTo, U256,
};

sol!("src/fungibles/precompiles/interfaces/IERC20.sol");

/// Precompile providing an interface of the ERC-20 standard as defined in the ERC.
pub struct Erc20<const PREFIX: u16, T, I>(PhantomData<(T, I)>);
impl<
		const PREFIX: u16,
		T: frame_system::Config + Config<I, AssetId: AtLeast32Bit> + pallet_revive::Config,
		I: 'static,
	> Precompile for Erc20<PREFIX, T, I>
where
	U256: UintTryFrom<T::Balance> + UintTryTo<T::Balance>,
{
	type Interface = IERC20Calls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Prefix(NonZero::new(PREFIX).expect("expected non-zero precompile address"));

	fn call(
		address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		use IERC20::{IERC20Calls::*, *};

		let token = InlineAssetIdExtractor::asset_id_from_address(address)?.into();

		match input {
			// IERC20
			totalSupply(_) => {
				// TODO: charge based on benchmarked weight
				let total_supply = U256::saturating_from(<Assets<T, I>>::total_supply(token));
				Ok(totalSupplyCall::abi_encode_returns(&(total_supply,)))
			},
			balanceOf(balanceOfCall { account }) => {
				// TODO: charge based on benchmarked weight
				let account = env.to_account_id(&(*account.0).into());
				let balance = U256::saturating_from(<Assets<T, I>>::balance(token, account));
				Ok(balanceOfCall::abi_encode_returns(&(balance,)))
			},
			transfer(transferCall { to, value }) => {
				// TODO: charge based on benchmarked weight
				let from = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				super::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event::<T>(env, address, Transfer { from, to: *to, value: *value });
				Ok(transferCall::abi_encode_returns(&(true,)))
			},
			allowance(allowanceCall { owner, spender }) => {
				// TODO: charge based on benchmarked weight
				let owner = env.to_account_id(&(*owner.0).into());
				let spender = env.to_account_id(&(*spender.0).into());
				let remaining =
					U256::saturating_from(<Assets<T, I>>::allowance(token, &owner, &spender));
				Ok(allowanceCall::abi_encode_returns(&(remaining,)))
			},
			approve(approveCall { spender, value }) => {
				// TODO: charge based on benchmarked weight
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				super::approve::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;

				let event = Approval { owner, spender: *spender, value: *value };
				deposit_event(env, address, event);
				Ok(approveCall::abi_encode_returns(&(true,)))
			},
			transferFrom(transferFromCall { from, to, value }) => {
				// TODO: charge based on benchmarked weight

				super::transfer_from::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*from.0).into()),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event(env, address, Transfer { from: *from, to: *to, value: *value });
				Ok(transferFromCall::abi_encode_returns(&(true,)))
			},
			// IERC20Metadata
			name(_) => {
				// TODO: charge based on benchmarked weight
				let result = <Assets<T, I>>::name(token);
				// TODO: improve
				let result = String::from_utf8_lossy(result.as_slice());
				Ok(nameCall::abi_encode_returns(&(result,)))
			},
			symbol(_) => {
				// TODO: charge based on benchmarked weight
				let result = <Assets<T, I>>::symbol(token);
				// TODO: improve
				let result = String::from_utf8_lossy(result.as_slice());
				Ok(symbolCall::abi_encode_returns(&(result,)))
			},
			decimals(_) => {
				// TODO: charge based on benchmarked weight
				let result = <Assets<T, I>>::decimals(token);
				Ok(decimalsCall::abi_encode_returns(&(result,)))
			},
		}
	}
}

impl<const PREFIX: u16, T: Config<I>, I: 'static> Erc20<PREFIX, T, I> {
	pub fn address(id: u32) -> [u8; 20] {
		prefixed_address(PREFIX, id)
	}
}

#[cfg(test)]
mod tests {
	use frame_support::{assert_ok, sp_runtime::app_crypto::sp_core::bytes::to_hex};
	use pallet_assets::Instance1;
	use pallet_revive::{
		precompiles::{
			alloy::sol_types::{SolType, SolValue},
			ExtWithInfo,
		},
		test_utils::{ALICE, ALICE_ADDR, BOB, BOB_ADDR},
		Origin,
	};
	use IERC20Calls::*;
	use IERC20::{Approval, Transfer};

	use super::*;
	use crate::{
		assert_last_event,
		fungibles::approve,
		mock::{Assets, ExtBuilder, RuntimeOrigin, Test},
	};

	const ERC20: u16 = 101;

	type Erc20 = super::Erc20<ERC20, Test, Instance1>;

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
						token,
						&totalSupply(totalSupplyCall {})
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
						token,
						&balanceOf(balanceOfCall { account: ALICE_ADDR.0.into() })
					)
					.unwrap(),
					U256::from(endowment)
				);

				assert_eq!(Assets::balance(token, ALICE), endowment);
			});
	}

	#[test]
	fn transfer_works() {
		let token = u32::MAX;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, BOB, true, 1)])
			.with_asset_balances(vec![(token, BOB, endowment)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(BOB));
				assert_eq!(Assets::balance(token, BOB), endowment);
				assert_eq!(Assets::balance(token, ALICE), 0);

				let value = endowment / 2;
				assert!(call_precompile::<bool>(
					&mut call_setup.ext().0,
					token,
					&IERC20Calls::transfer(IERC20::transferCall {
						to: ALICE_ADDR.0.into(),
						value: U256::from(value)
					})
				)
				.unwrap());

				assert_eq!(Assets::balance(token, BOB), endowment - value);
				assert_eq!(Assets::balance(token, ALICE), value);
				let from = BOB_ADDR.0.into();
				let to = ALICE_ADDR.0.into();
				let event = Transfer { from, to, value: U256::from(value) };
				assert_last_event(prefixed_address(ERC20, token), event);
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
						token,
						&allowance(allowanceCall {
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
	fn approve_works() {
		let token = u32::MAX;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(token, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));

				assert_eq!(Assets::allowance(token, &ALICE, &BOB), 0);

				assert!(call_precompile::<bool>(
					&mut call_setup.ext().0,
					token,
					&IERC20Calls::approve(IERC20::approveCall {
						spender: BOB_ADDR.0.into(),
						value: U256::from(value)
					})
				)
				.unwrap());

				assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
				let owner = ALICE_ADDR.0.into();
				let spender = BOB_ADDR.0.into();
				let event = Approval { owner, spender, value: U256::from(value) };
				assert_last_event(prefixed_address(ERC20, token), event);
			},
		);
	}

	#[test]
	fn transfer_from_works() {
		let token = u32::MAX;
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

				assert!(call_precompile::<bool>(
					&mut call_setup.ext().0,
					token,
					&transferFrom(IERC20::transferFromCall {
						from: ALICE_ADDR.0.into(),
						to: BOB_ADDR.0.into(),
						value: U256::from(value),
					})
				)
				.unwrap());

				assert_eq!(Assets::balance(token, ALICE), endowment - value);
				assert_eq!(Assets::balance(token, BOB), value);
				let from = ALICE_ADDR.0.into();
				let to = BOB_ADDR.0.into();
				let event = Transfer { from, to, value: U256::from(value) };
				assert_last_event(prefixed_address(ERC20, token), event);
			});
	}

	#[test]
	fn name_works() {
		let token = u32::MAX;
		let _name = "name";
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, _name.as_bytes().to_vec(), b"symbol".to_vec(), 10)])
			.build_with_env(|mut call_setup| {
				assert_eq!(
					call_precompile::<String>(&mut call_setup.ext().0, token, &name(nameCall {}))
						.unwrap()
						.as_str(),
					_name
				);
			});
	}

	#[test]
	fn symbol_works() {
		let token = u32::MAX;
		let _symbol = "symbol";
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), _symbol.as_bytes().to_vec(), 10)])
			.build_with_env(|mut call_setup| {
				assert_eq!(
					call_precompile::<String>(
						&mut call_setup.ext().0,
						token,
						&symbol(symbolCall {})
					)
					.unwrap()
					.as_str(),
					_symbol
				);
			});
	}

	#[test]
	fn decimals_works() {
		let token = u32::MAX;
		let _decimals = u8::MAX;
		ExtBuilder::new()
			.with_assets(vec![(token, ALICE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), b"symbol".to_vec(), _decimals)])
			.build_with_env(|mut call_setup| {
				let mut ext = call_setup.ext().0;
				assert_eq!(
					call_precompile::<u16>(&mut ext, token, &decimals(decimalsCall {})).unwrap()
						as u8,
					_decimals
				);
			});
	}

	#[test]
	fn selectors_match_standard() {
		assert_eq!(to_hex(&decimalsCall::SELECTOR, false), "0x313ce567");
		assert_eq!(to_hex(&nameCall::SELECTOR, false), "0x06fdde03");
		assert_eq!(to_hex(&symbolCall::SELECTOR, false), "0x95d89b41");
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		ext: &mut impl ExtWithInfo<T = Test>,
		token: u32,
		input: &IERC20Calls,
	) -> Result<Output, Error> {
		crate::call_precompile::<Erc20, Output>(ext, &prefixed_address(ERC20, token), input)
	}
}
