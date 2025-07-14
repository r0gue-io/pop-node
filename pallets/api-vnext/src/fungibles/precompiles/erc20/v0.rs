use alloc::{string::String, vec::Vec};

use frame_support::sp_runtime::traits::AtLeast32Bit;
use pallet_assets::precompiles::{AssetIdExtractor, InlineAssetIdExtractor};
use pallet_revive::{precompiles::RuntimeCosts, AddressMapper as _};
use AddressMatcher::Prefix;
use IERC20::*;

use super::{super::super::*, U256, *};

sol!(
	#![sol(extra_derives(Debug, PartialEq))]
	"src/fungibles/precompiles/interfaces/v0/IERC20.sol"
);

/// Precompile providing an interface of the ERC-20 standard as defined in the ERC.
pub struct Erc20<const PREFIX: u16, T, I = ()>(PhantomData<(T, I)>);
impl<
		const PREFIX: u16,
		T: frame_system::Config
			+ pallet_assets::Config<
				I,
				AssetId: AtLeast32Bit,
				Balance: TryConvert<U256, Error = Error>,
			> + pallet_revive::Config
			+ Config<I>,
		I: 'static,
	> Precompile for Erc20<PREFIX, T, I>
where
	U256: TryConvert<<T as pallet_assets::Config<I>>::Balance, Error = Error>,
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
		let token = InlineAssetIdExtractor::asset_id_from_address(address)?.into();

		match input {
			// IERC20
			IERC20Calls::totalSupply(_) => {
				env.charge(<T as Config<I>>::WeightInfo::total_supply())?;

				let total_supply = total_supply::<T, I>(token).try_convert()?;

				Ok(totalSupplyCall::abi_encode_returns(&total_supply))
			},
			IERC20Calls::balanceOf(balanceOfCall { account }) => {
				env.charge(<T as Config<I>>::WeightInfo::balance_of())?;

				let account = env.to_account_id(&(*account.0).into());
				let balance = balance::<T, I>(token, &account).try_convert()?;

				Ok(balanceOfCall::abi_encode_returns(&balance))
			},
			IERC20Calls::transfer(transferCall { to, value }) => {
				env.charge(<T as Config<I>>::WeightInfo::transfer())?;
				let origin = env.caller();
				let account = origin.account_id()?;
				let from = <AddressMapper<T>>::to_address(&account).0.into();
				ensure!(!to.is_zero(), ERC20InvalidReceiver { receiver: *to });
				ensure!(!value.is_zero(), ERC20InsufficientValue);

				let balance = balance::<T, I>(token.clone(), &account).try_convert()?;

				transfer::<T, I>(
					to_runtime_origin(origin),
					token,
					env.to_account_id(&(*to.0).into()),
					(*value).try_convert()?,
				)
				.map_err(|e| Self::map_transfer_err(e, &from, value, &balance))?;

				deposit_event(env, Transfer { from, to: *to, value: *value })?;
				Ok(transferCall::abi_encode_returns(&true))
			},
			IERC20Calls::allowance(allowanceCall { owner, spender }) => {
				env.charge(<T as Config<I>>::WeightInfo::allowance())?;

				let owner = env.to_account_id(&(*owner.0).into());
				let spender = env.to_account_id(&(*spender.0).into());
				let remaining = allowance::<T, I>(token, &owner, &spender).try_convert()?;

				Ok(allowanceCall::abi_encode_returns(&remaining))
			},
			IERC20Calls::approve(approveCall { spender, value }) => {
				let charged = env.charge(<T as Config<I>>::WeightInfo::approve(1, 1))?;
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				ensure!(!spender.is_zero(), ERC20InvalidSpender { spender: *spender });
				ensure!(!value.is_zero(), ERC20InsufficientValue);

				match approve::<T, I>(
					to_runtime_origin(env.caller()),
					token,
					env.to_account_id(&(*spender.0).into()),
					(*value).try_convert()?,
				) {
					Ok(result) => {
						// Adjust weight
						if let Some(actual_weight) = result.actual_weight {
							// TODO: replace with `env.adjust_gas(charged, result.weight);` once
							// #8693 lands
							env.gas_meter_mut()
								.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
						}
					},
					Err(e) => {
						// Adjust weight
						if let Some(actual_weight) = e.post_info.actual_weight {
							// TODO: replace with `env.adjust_gas(charged, result.weight);` once
							// #8693 lands
							env.gas_meter_mut()
								.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
						}
						return Err(e.error.into())
					},
				};

				deposit_event(env, Approval { owner, spender: *spender, value: *value })?;
				Ok(approveCall::abi_encode_returns(&true))
			},
			IERC20Calls::transferFrom(transferFromCall { from, to, value }) => {
				env.charge(<T as Config<I>>::WeightInfo::transfer_from())?;
				let origin = env.caller();
				let account = origin.account_id()?;
				ensure!(!from.is_zero(), ERC20InvalidSender { sender: *from });
				ensure!(!to.is_zero(), ERC20InvalidReceiver { receiver: *to });
				ensure!(!value.is_zero(), ERC20InsufficientValue);

				let owner = env.to_account_id(&(*from.0).into());
				let spender = <AddressMapper<T>>::to_address(&account).0.into();
				let allowance = allowance::<T, I>(token.clone(), &owner, &account).try_convert()?;

				transfer_from::<T, I>(
					to_runtime_origin(origin),
					token,
					owner,
					env.to_account_id(&(*to.0).into()),
					(*value).try_convert()?,
				)
				.map_err(|e| Self::map_transfer_from_err(e, &spender, value, &allowance))?;

				deposit_event(env, Transfer { from: *from, to: *to, value: *value })?;
				Ok(transferFromCall::abi_encode_returns(&true))
			},
			// IERC20Metadata
			IERC20Calls::name(_) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_name())?;

				let result = name::<T, I>(token);
				let result = String::from_utf8_lossy(result.as_slice()).into();

				Ok(nameCall::abi_encode_returns(&result))
			},
			IERC20Calls::symbol(_) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_symbol())?;

				let result = symbol::<T, I>(token);
				let result = String::from_utf8_lossy(result.as_slice()).into();

				Ok(symbolCall::abi_encode_returns(&result))
			},
			IERC20Calls::decimals(_) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_decimals())?;

				let result = decimals::<T, I>(token);

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

	// Maps select, domain-specific dispatch errors to ERC20 errors. Anything not mapped results
	// in a `Error::Error(ExecError::DispatchError)` which results in trap rather than a revert.
	fn map_transfer_err(e: DispatchError, from: &Address, value: &U256, balance: &U256) -> Error {
		match e {
			DispatchError::Module(ModuleError { index, error, .. })
				if Some(index as usize) ==
					T::PalletInfo::index::<pallet_assets::Pallet<T, I>>() =>
			{
				use pallet_assets::{Error, Error::*};
				Error::<T, I>::decode(&mut error.as_slice()).ok().and_then(|error| match error {
					BalanceLow => Some(
						IERC20::ERC20InsufficientBalance {
							sender: *from,
							balance: *balance,
							needed: *value,
						}
						.into(),
					),
					_ => None,
				})
			},
			_ => None,
		}
		.unwrap_or_else(|| e.into())
	}

	// Maps select, domain-specific dispatch errors to ERC20 errors. Anything not mapped results
	// in a `Error::Error(ExecError::DispatchError)` which results in trap rather than a revert.
	fn map_transfer_from_err(
		e: DispatchError,
		spender: &Address,
		value: &U256,
		allowance: &U256,
	) -> Error {
		match e {
			DispatchError::Module(ModuleError { index, error, .. })
				if Some(index as usize) ==
					T::PalletInfo::index::<pallet_assets::Pallet<T, I>>() =>
			{
				use pallet_assets::{Error, Error::*};
				Error::<T, I>::decode(&mut error.as_slice()).ok().and_then(|error| match error {
					Unapproved => Some(
						IERC20::ERC20InsufficientAllowance {
							spender: *spender,
							allowance: *allowance,
							needed: *value,
						}
						.into(),
					),
					_ => None,
				})
			},
			_ => None,
		}
		.unwrap_or_else(|| e.into())
	}
}

// Encoding of custom errors via `Error(String)`.
impl_from_sol_error! {
	IERC20::ERC20InsufficientAllowance,
	IERC20::ERC20InsufficientBalance,
}

#[cfg(test)]
mod tests {
	use frame_support::{
		assert_ok, sp_runtime::app_crypto::sp_core::bytes::to_hex,
		traits::fungibles::approvals::Inspect,
	};
	use pallet_revive::{
		precompiles::alloy::{
			primitives::Address,
			sol_types::{SolInterface, SolType, SolValue},
		},
		test_utils::{ALICE, BOB, CHARLIE},
	};
	use IERC20::{Approval, Transfer};

	use super::*;
	use crate::{
		assert_last_event, bare_call,
		fungibles::approve,
		mock::{Assets, ExtBuilder, RuntimeOrigin, Test, ERC20, UNIT},
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
				let call = IERC20Calls::totalSupply(totalSupplyCall {});
				assert_eq!(
					call_precompile::<U256>(&ALICE, token, &call).unwrap(),
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
						&IERC20Calls::balanceOf(balanceOfCall {
							account: to_address(&account).0.into()
						})
					)
					.unwrap(),
					U256::from(endowment)
				);
			});
	}

	#[test]
	fn transfer_reverts_with_invalid_receiver() {
		let token = 1;
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let call = transferCall { to: Address::default(), value: U256::ZERO };
			let transfer = IERC20Calls::transfer(call);
			assert_revert!(
				call_precompile::<()>(&origin, token, &transfer),
				ERC20InvalidReceiver { receiver: Address::default() }
			);
		});
	}

	#[test]
	fn transfer_reverts_with_zero_value() {
		let token = 1;
		let origin = ALICE;
		let to = [255; 20].into();
		ExtBuilder::new().build().execute_with(|| {
			let call = transferCall { to, value: U256::ZERO };
			let transfer = IERC20Calls::transfer(call);
			assert_revert!(
				call_precompile::<()>(&origin, token, &transfer),
				ERC20InsufficientValue
			);
		});
	}

	#[test]
	fn transfer_reverts_with_insufficient_balance() {
		let token = 1;
		let origin = ALICE;
		let endowment = 10_000_000;
		let to = BOB;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, true, 1)])
			.with_asset_balances(vec![(token, origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				let value = U256::from(endowment + 1);
				let call = transferCall { to: to_address(&to).0.into(), value };
				let transfer = IERC20Calls::transfer(call);
				assert_revert!(
					call_precompile::<()>(&origin, token, &transfer),
					ERC20InsufficientBalance {
						sender: to_address(&origin).0.into(),
						balance: U256::from(endowment),
						needed: value
					}
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
			.with_balances(vec![(owner.clone(), UNIT)])
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
						&IERC20Calls::allowance(allowanceCall {
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
	fn approve_reverts_with_invalid_spender() {
		let token = 1;
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let call = approveCall { spender: Address::default(), value: U256::ZERO };
			let approve = IERC20Calls::approve(call);
			assert_revert!(
				call_precompile::<()>(&origin, token, &approve),
				ERC20InvalidSpender { spender: Address::default() }
			);
		});
	}

	#[test]
	fn approve_reverts_with_zero_value() {
		let token = 1;
		let origin = ALICE;
		let spender = [255; 20].into();
		ExtBuilder::new().build().execute_with(|| {
			let call = approveCall { spender, value: U256::ZERO };
			let approve = IERC20Calls::approve(call);
			assert_revert!(call_precompile::<()>(&origin, token, &approve), ERC20InsufficientValue);
		});
	}

	#[test]
	fn approve_works() {
		let token = 1;
		let origin = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), UNIT)])
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
	fn transfer_from_reverts_with_invalid_sender() {
		let token = 1;
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let call = transferFromCall {
				from: Address::default(),
				to: Address::default(),
				value: U256::ZERO,
			};
			let transfer_from = IERC20Calls::transferFrom(call);
			assert_revert!(
				call_precompile::<()>(&origin, token, &transfer_from),
				ERC20InvalidSender { sender: Address::default() }
			);
		});
	}

	#[test]
	fn transfer_from_reverts_with_invalid_receiver() {
		let token = 1;
		let origin = ALICE;
		let from = [255; 20].into();
		ExtBuilder::new().build().execute_with(|| {
			let call = transferFromCall { from, to: Address::default(), value: U256::ZERO };
			let transfer_from = IERC20Calls::transferFrom(call);
			assert_revert!(
				call_precompile::<()>(&origin, token, &transfer_from),
				ERC20InvalidReceiver { receiver: Address::default() }
			);
		});
	}

	#[test]
	fn transfer_from_reverts_with_zero_value() {
		let token = 1;
		let origin = ALICE;
		let from = [255; 20].into();
		let to = [1; 20].into();
		ExtBuilder::new().build().execute_with(|| {
			let call = transferFromCall { from, to, value: U256::ZERO };
			let transfer_from = IERC20Calls::transferFrom(call);
			assert_revert!(
				call_precompile::<()>(&origin, token, &transfer_from),
				ERC20InsufficientValue
			);
		});
	}

	#[test]
	fn transfer_from_reverts_with_insufficient_allowance() {
		let token = 1;
		let origin = BOB;
		let from = ALICE;
		let endowment = 10_000_000;
		let to = CHARLIE;
		let allowance = endowment / 2;
		ExtBuilder::new()
			.with_balances(vec![(from.clone(), UNIT)])
			.with_assets(vec![(token, CHARLIE, true, 1)])
			.with_asset_balances(vec![(token, from.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_ok!(approve::<Test, ()>(
					RuntimeOrigin::signed(from.clone()),
					token,
					origin.clone(),
					allowance
				));

				let value = U256::from(allowance + 1);
				let call = transferFromCall {
					from: to_address(&from).0.into(),
					to: to_address(&to).0.into(),
					value,
				};
				let transfer_from = IERC20Calls::transferFrom(call);
				assert_revert!(
					call_precompile::<()>(&origin, token, &transfer_from),
					ERC20InsufficientAllowance {
						spender: to_address(&origin).0.into(),
						allowance: U256::from(allowance),
						needed: value
					}
				);
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
			.with_balances(vec![(from.clone(), UNIT)])
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
					&IERC20Calls::transferFrom(IERC20::transferFromCall {
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
					call_precompile::<String>(&ALICE, token, &IERC20Calls::name(nameCall {}))
						.unwrap()
						.as_str(),
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
					call_precompile::<String>(&ALICE, token, &IERC20Calls::symbol(symbolCall {}))
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
					call_precompile::<u16>(&ALICE, token, &IERC20Calls::decimals(decimalsCall {}))
						.unwrap() as u8,
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
	) -> Result<Output, Error> {
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
