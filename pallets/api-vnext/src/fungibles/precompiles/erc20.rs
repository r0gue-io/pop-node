use alloc::{string::String, vec::Vec};

use frame_support::{
	sp_runtime::traits::AtLeast32Bit,
	traits::fungibles::{approvals::Inspect as _, metadata::Inspect as _},
};
use pallet_assets::precompiles::{AssetIdExtractor, InlineAssetIdExtractor};
use pallet_revive::{precompiles::RuntimeCosts, AddressMapper as _};
use AddressMatcher::Prefix;
use IERC20::*;

use super::{
	deposit_event, prefixed_address, sol, to_runtime_origin, weights::WeightInfo, AddressMapper,
	AddressMatcher, Assets, Config, Error, Ext, NonZero, PhantomData, Precompile, SolCall,
	UintTryFrom, UintTryTo, U256,
};

sol!("src/fungibles/precompiles/interfaces/IERC20.sol");

/// Precompile providing an interface of the ERC-20 standard as defined in the ERC.
pub struct Erc20<const PREFIX: u16, T, I = ()>(PhantomData<(T, I)>);
impl<
		const PREFIX: u16,
		T: frame_system::Config
			+ pallet_assets::Config<I, AssetId: AtLeast32Bit>
			+ pallet_revive::Config
			+ Config<I>,
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
				env.charge(<T as Config<I>>::WeightInfo::total_supply())?;

				let total_supply = U256::saturating_from(super::total_supply::<T, I>(token));

				Ok(totalSupplyCall::abi_encode_returns(&total_supply))
			},
			balanceOf(balanceOfCall { account }) => {
				env.charge(<T as Config<I>>::WeightInfo::balance_of())?;

				let account = env.to_account_id(&(*account.0).into());
				let balance = U256::saturating_from(super::balance::<T, I>(token, &account));

				Ok(balanceOfCall::abi_encode_returns(&balance))
			},
			transfer(transferCall { to, value }) => {
				env.charge(<T as Config<I>>::WeightInfo::transfer())?;
				let from = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				super::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event::<T>(env, address, Transfer { from, to: *to, value: *value });
				Ok(transferCall::abi_encode_returns(&true))
			},
			allowance(allowanceCall { owner, spender }) => {
				env.charge(<T as Config<I>>::WeightInfo::allowance())?;

				let owner = env.to_account_id(&(*owner.0).into());
				let spender = env.to_account_id(&(*spender.0).into());
				let remaining =
					U256::saturating_from(super::allowance::<T, I>(token, &owner, &spender));

				Ok(allowanceCall::abi_encode_returns(&remaining))
			},
			approve(approveCall { spender, value }) => {
				let charged = env.charge(<T as Config<I>>::WeightInfo::approve(1, 1))?;
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let result = super::approve::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;

				// Adjust weight
				if let Some(actual_weight) = result.actual_weight {
					// TODO: replace with `env.adjust_gas(charged, result.weight);` once #8693 lands
					env.gas_meter_mut()
						.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
				}

				let event = Approval { owner, spender: *spender, value: *value };
				deposit_event(env, address, event);
				Ok(approveCall::abi_encode_returns(&true))
			},
			transferFrom(transferFromCall { from, to, value }) => {
				env.charge(<T as Config<I>>::WeightInfo::transfer_from())?;

				super::transfer_from::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*from.0).into()),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event(env, address, Transfer { from: *from, to: *to, value: *value });
				Ok(transferFromCall::abi_encode_returns(&true))
			},
			// IERC20Metadata
			name(_) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_name())?;

				let result = <Assets<T, I>>::name(token);
				let result = String::from_utf8_lossy(result.as_slice()).into();

				Ok(nameCall::abi_encode_returns(&result))
			},
			symbol(_) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_symbol())?;

				let result = <Assets<T, I>>::symbol(token);
				let result = String::from_utf8_lossy(result.as_slice()).into();

				Ok(symbolCall::abi_encode_returns(&result))
			},
			decimals(_) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_decimals())?;

				let result = <Assets<T, I>>::decimals(token);

				Ok(decimalsCall::abi_encode_returns(&result))
			},
		}
	}
}

impl<const PREFIX: u16, T: pallet_assets::Config<I>, I: 'static> Erc20<PREFIX, T, I> {
	/// The address of the precompile.
	pub fn address(id: u32) -> [u8; 20] {
		prefixed_address(PREFIX, id)
	}
}

#[cfg(test)]
mod tests {
	use frame_support::{
		assert_ok,
		sp_runtime::{app_crypto::sp_core::bytes::to_hex, DispatchError},
	};
	use pallet_revive::{
		precompiles::alloy::sol_types::{SolInterface, SolType, SolValue},
		test_utils::{ALICE, BOB, CHARLIE},
	};
	use IERC20Calls::*;
	use IERC20::{Approval, Transfer};

	use super::*;
	use crate::{
		assert_last_event, bare_call,
		fungibles::approve,
		mock::{Assets, ExtBuilder, RuntimeOrigin, Test, ERC20},
		to_address, DepositLimit, Weight,
	};

	type AccountId = <Test as frame_system::Config>::AccountId;

	#[test]
	fn total_supply_works() {
		let token = 1;
		let total_supply = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_balances(vec![(token, ALICE, total_supply)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<U256>(&ALICE, token, &totalSupply(totalSupplyCall {}))
						.unwrap(),
					U256::from(total_supply)
				);
			});
	}

	#[test]
	fn balance_of_works() {
		let token = 1;
		let account = ALICE;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_balances(vec![(token, account.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<U256>(
						&BOB,
						token,
						&balanceOf(balanceOfCall { account: to_address(&account).0.into() })
					)
					.unwrap(),
					U256::from(endowment)
				);
			});
	}

	#[test]
	fn transfer_works() {
		let token = 1;
		let origin = ALICE;
		let endowment = 10_000_000;
		let to = BOB;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, true, 1)])
			.with_asset_balances(vec![(token, origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_eq!(Assets::balance(token, &origin), endowment);
				assert_eq!(Assets::balance(token, &to), 0);

				let value = endowment / 2;
				assert!(call_precompile::<bool>(
					&origin,
					token,
					&IERC20Calls::transfer(IERC20::transferCall {
						to: to_address(&to).0.into(),
						value: U256::from(value)
					})
				)
				.unwrap());

				assert_eq!(Assets::balance(token, &origin), endowment - value);
				assert_eq!(Assets::balance(token, &to), value);
				let from = to_address(&origin).0.into();
				let to = to_address(&to).0.into();
				let event = Transfer { from, to, value: U256::from(value) };
				assert_last_event(prefixed_address(ERC20, token), event);
			});
	}

	#[test]
	fn allowance_works() {
		let token = 1;
		let owner = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.build()
			.execute_with(|| {
				assert_ok!(approve::<Test, ()>(
					RuntimeOrigin::signed(owner.clone()),
					token,
					BOB,
					value
				));

				assert_eq!(
					call_precompile::<U256>(
						&BOB,
						token,
						&allowance(allowanceCall {
							owner: to_address(&owner).0.into(),
							spender: to_address(&spender).0.into(),
						})
					)
					.unwrap(),
					U256::from(value)
				);
			});
	}

	#[test]
	fn approve_works() {
		let token = 1;
		let origin = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.build()
			.execute_with(|| {
				assert_eq!(Assets::allowance(token, &origin, &spender), 0);

				assert!(call_precompile::<bool>(
					&origin,
					token,
					&IERC20Calls::approve(IERC20::approveCall {
						spender: to_address(&spender).0.into(),
						value: U256::from(value)
					})
				)
				.unwrap());

				assert_eq!(Assets::allowance(token, &origin, &spender), value);
				let owner = to_address(&origin).0.into();
				let spender = to_address(&spender).0.into();
				let event = Approval { owner, spender, value: U256::from(value) };
				assert_last_event(prefixed_address(ERC20, token), event);
			});
	}

	#[test]
	fn transfer_from_works() {
		let token = 1;
		let origin = BOB;
		let from = ALICE;
		let endowment = 10_000_000;
		let to = CHARLIE;
		let value = endowment / 2;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, true, 1)])
			.with_asset_balances(vec![(token, from.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_eq!(Assets::balance(token, &from), endowment);
				assert_eq!(Assets::balance(token, &to), 0);
				assert_ok!(approve::<Test, ()>(
					RuntimeOrigin::signed(from.clone()),
					token,
					origin.clone(),
					value
				));

				assert!(call_precompile::<bool>(
					&origin,
					token,
					&transferFrom(IERC20::transferFromCall {
						from: to_address(&from).0.into(),
						to: to_address(&to).0.into(),
						value: U256::from(value),
					})
				)
				.unwrap());

				assert_eq!(Assets::balance(token, &from), endowment - value);
				assert_eq!(Assets::balance(token, &to), value);
				let from = to_address(&from).0.into();
				let to = to_address(&to).0.into();
				let event = Transfer { from, to, value: U256::from(value) };
				assert_last_event(prefixed_address(ERC20, token), event);
			});
	}

	#[test]
	fn name_works() {
		let token = 1;
		let _name = "name";
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_metadata(vec![(token, _name.as_bytes().to_vec(), b"symbol".to_vec(), 10)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<String>(&ALICE, token, &name(nameCall {})).unwrap().as_str(),
					_name
				);
			});
	}

	#[test]
	fn symbol_works() {
		let token = 1;
		let _symbol = "symbol";
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), _symbol.as_bytes().to_vec(), 10)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<String>(&ALICE, token, &symbol(symbolCall {}))
						.unwrap()
						.as_str(),
					_symbol
				);
			});
	}

	#[test]
	fn decimals_works() {
		let token = 1;
		let _decimals = u8::MAX;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), b"symbol".to_vec(), _decimals)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<u16>(&ALICE, token, &decimals(decimalsCall {})).unwrap()
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
		origin: &AccountId,
		token: u32,
		input: &IERC20Calls,
	) -> Result<Output, DispatchError> {
		let address = prefixed_address(ERC20, token);
		bare_call::<Test, Output>(
			RuntimeOrigin::signed(origin.clone()),
			address.into(),
			0,
			Weight::MAX,
			DepositLimit::Balance(u128::MAX),
			input.abi_encode(),
		)
	}
}
