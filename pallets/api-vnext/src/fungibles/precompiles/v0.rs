pub(crate) use IFungibles::*;

use super::*;

sol!("src/fungibles/precompiles/interfaces/v0/IFungibles.sol");

/// The fungibles precompile offers a streamlined interface for interacting with fungible
/// tokens. The goal is to provide a simplified, consistent API that adheres to standards in
/// the smart contract space.
pub struct Fungibles<const FIXED: u16, T, I = ()>(PhantomData<(T, I)>);
impl<
		const FIXED: u16,
		T: frame_system::Config
			+ pallet_assets::Config<I, AssetId: Default + From<u32> + Into<u32>>
			+ pallet_revive::Config
			+ Config<I>,
		I: 'static,
	> Precompile for Fungibles<FIXED, T, I>
where
	U256: UintTryFrom<<T as pallet_assets::Config<I>>::Balance>
		+ UintTryTo<<T as pallet_assets::Config<I>>::Balance>,
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
				env.charge(<T as Config<I>>::WeightInfo::transfer())?;
				let from = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				transfer::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				let event = Transfer { token: *token, from, to: *to, value: *value };
				deposit_event(env, address, event);
				Ok(transferCall::abi_encode_returns(&transferReturn {}))
			},
			IFungiblesCalls::transferFrom(transferFromCall { token, from, to, value }) => {
				env.charge(<T as Config<I>>::WeightInfo::transfer_from())?;

				transfer_from::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*from.0).into()),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				let event = Transfer { token: *token, from: *from, to: *to, value: *value };
				deposit_event(env, address, event);
				Ok(transferFromCall::abi_encode_returns(&transferFromReturn {}))
			},
			IFungiblesCalls::approve(approveCall { token, spender, value }) => {
				let charged = env.charge(<T as Config<I>>::WeightInfo::approve(1, 1))?;
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				match approve::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
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

				let event = Approval { token: *token, owner, spender: *spender, value: *value };
				deposit_event(env, address, event);
				Ok(approveCall::abi_encode_returns(&approveReturn {}))
			},
			IFungiblesCalls::increaseAllowance(increaseAllowanceCall { token, spender, value }) => {
				let charged = env.charge(<T as Config<I>>::WeightInfo::approve(1, 0))?;
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let value = U256::saturating_from(
					increase_allowance::<T, I>(
						to_runtime_origin(env.caller()),
						(*token).into(),
						env.to_account_id(&(*spender.0).into()),
						value.saturating_to(),
					)
					.map_err(|e| {
						// Adjust weight
						if let Some(actual_weight) = e.post_info.actual_weight {
							// TODO: replace with `env.adjust_gas(charged, result.weight);` once
							// #8693 lands
							env.gas_meter_mut()
								.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
						}
						e.error
					})?,
				);

				let spender = *spender;
				deposit_event(env, address, Approval { token: *token, owner, spender, value });
				Ok(increaseAllowanceCall::abi_encode_returns(&value))
			},
			IFungiblesCalls::decreaseAllowance(decreaseAllowanceCall { token, spender, value }) => {
				let charged = env.charge(<T as Config<I>>::WeightInfo::approve(1, 1))?;
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let value = match decrease_allowance::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
				) {
					Ok((value, weight)) => {
						// Adjust weight
						if let Some(actual_weight) = weight {
							// TODO: replace with `env.adjust_gas(charged, result.weight);` once
							// #8693 lands
							env.gas_meter_mut()
								.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
						}
						U256::saturating_from(value)
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

				let spender = *spender;
				deposit_event(env, address, Approval { token: *token, owner, spender, value });
				Ok(decreaseAllowanceCall::abi_encode_returns(&value))
			},
			IFungiblesCalls::create(createCall { admin, minBalance }) => {
				env.charge(<T as Config<I>>::WeightInfo::create())?;
				let creator = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				let id = create::<T, I>(
					to_runtime_origin(env.caller()),
					env.to_account_id(&(*admin.0).into()),
					minBalance.saturating_to(),
				)?
				.into();

				deposit_event(env, address, Created { id, creator, admin: *admin });
				Ok(createCall::abi_encode_returns(&id))
			},
			IFungiblesCalls::startDestroy(startDestroyCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::start_destroy())?;

				start_destroy::<T, I>(to_runtime_origin(env.caller()), (*token).into())?;

				Ok(startDestroyCall::abi_encode_returns(&startDestroyReturn {}))
			},
			IFungiblesCalls::setMetadata(setMetadataCall { token, name, symbol, decimals }) => {
				env.charge(<T as Config<I>>::WeightInfo::set_metadata())?;

				set_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					name.as_bytes().to_vec(),
					symbol.as_bytes().to_vec(),
					*decimals,
				)?;

				Ok(setMetadataCall::abi_encode_returns(&setMetadataReturn {}))
			},
			IFungiblesCalls::clearMetadata(clearMetadataCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::clear_metadata())?;

				clear_metadata::<T, I>(to_runtime_origin(env.caller()), (*token).into())?;

				Ok(clearMetadataCall::abi_encode_returns(&clearMetadataReturn {}))
			},
			IFungiblesCalls::mint(mintCall { token, account, value }) => {
				env.charge(<T as Config<I>>::WeightInfo::mint())?;

				mint::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*account.0).into()),
					value.saturating_to(),
				)?;

				let from = Address::default();
				let to = *account;
				deposit_event(env, address, Transfer { token: *token, from, to, value: *value });
				Ok(mintCall::abi_encode_returns(&mintReturn {}))
			},
			IFungiblesCalls::burn(burnCall { token, account, value }) => {
				let charged = env.charge(<T as Config<I>>::WeightInfo::burn())?;

				burn::<T, I>(
					to_runtime_origin(env.caller()),
					(*token).into(),
					env.to_account_id(&(*account.0).into()),
					value.saturating_to(),
				)
				.map_err(|e| {
					// Adjust weight
					if let Some(actual_weight) = e.post_info.actual_weight {
						// TODO: replace with `env.adjust_gas(charged, result.weight);` once
						// #8693 lands
						env.gas_meter_mut()
							.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
					}
					e.error
				})?;

				let to = Address::default();
				let event = Transfer { token: *token, from: *account, to, value: *value };
				deposit_event(env, address, event);
				Ok(burnCall::abi_encode_returns(&burnReturn {}))
			},
			IFungiblesCalls::totalSupply(totalSupplyCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::total_supply())?;

				let total_supply = U256::saturating_from(total_supply::<T, I>((*token).into()));

				Ok(totalSupplyCall::abi_encode_returns(&total_supply))
			},
			IFungiblesCalls::balanceOf(balanceOfCall { token, owner }) => {
				env.charge(<T as Config<I>>::WeightInfo::balance_of())?;

				let account = env.to_account_id(&(*owner.0).into());
				let balance = U256::saturating_from(balance::<T, I>((*token).into(), &account));

				Ok(balanceOfCall::abi_encode_returns(&balance))
			},
			IFungiblesCalls::allowance(allowanceCall { token, owner, spender }) => {
				env.charge(<T as Config<I>>::WeightInfo::allowance())?;

				let owner = env.to_account_id(&(*owner.0).into());
				let spender = env.to_account_id(&(*spender.0).into());
				let allowance = allowance::<T, I>((*token).into(), &owner, &spender);
				let remaining = U256::saturating_from(allowance);

				Ok(allowanceCall::abi_encode_returns(&remaining))
			},
			IFungiblesCalls::name(nameCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_name())?;

				let result = name::<T, I>((*token).into());
				let result = String::from_utf8_lossy(result.as_slice()).into();

				Ok(nameCall::abi_encode_returns(&result))
			},
			IFungiblesCalls::symbol(symbolCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_symbol())?;

				let result = symbol::<T, I>((*token).into());
				let result = String::from_utf8_lossy(result.as_slice()).into();

				Ok(nameCall::abi_encode_returns(&result))
			},
			IFungiblesCalls::decimals(decimalsCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::metadata_decimals())?;

				let result = decimals::<T, I>((*token).into());

				Ok(decimalsCall::abi_encode_returns(&result))
			},
			IFungiblesCalls::exists(existsCall { token }) => {
				env.charge(<T as Config<I>>::WeightInfo::exists())?;

				let result = exists::<T, I>((*token).into());

				Ok(existsCall::abi_encode_returns(&result))
			},
		}
	}
}

impl<const FIXED: u16, T: pallet_assets::Config<I>, I: 'static> Fungibles<FIXED, T, I> {
	/// The address of the precompile.
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
	use pallet_assets::{AssetDetails, AssetMetadata, AssetStatus};
	use pallet_revive::{
		precompiles::alloy::sol_types::{SolInterface, SolType},
		test_utils::{ALICE, BOB, CHARLIE},
	};

	use super::{
		IFungibles::IFungiblesCalls::{clearMetadata, setMetadata, startDestroy},
		*,
	};

	const ADDRESS: [u8; 20] = fixed_address(FUNGIBLES);

	type AccountId = <Test as frame_system::Config>::AccountId;
	type Asset = pallet_assets::Asset<Test>;
	type Metadata = pallet_assets::Metadata<Test>;

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
				assert_ok!(call_precompile::<()>(
					&origin,
					&IFungiblesCalls::transfer(transferCall {
						token,
						to: to_address(&to).0.into(),
						value: U256::from(value)
					})
				));

				assert_eq!(Assets::balance(token, &origin), endowment - value);
				assert_eq!(Assets::balance(token, &to), value);

				let from = to_address(&origin).0.into();
				let to = to_address(&to).0.into();
				let event = Transfer { token, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
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

				assert_ok!(call_precompile::<()>(
					&origin,
					&IFungiblesCalls::transferFrom(transferFromCall {
						token,
						from: to_address(&from).0.into(),
						to: to_address(&to).0.into(),
						value: U256::from(value),
					})
				));

				assert_eq!(Assets::balance(token, &from), endowment - value);
				assert_eq!(Assets::balance(token, &to), value);
				let from = to_address(&from).0.into();
				let to = to_address(&to).0.into();
				let event = Transfer { token, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
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

				assert_ok!(call_precompile::<()>(
					&origin,
					&IFungiblesCalls::approve(approveCall {
						token,
						spender: to_address(&spender).0.into(),
						value: U256::from(value),
					})
				));

				assert_eq!(Assets::allowance(token, &origin, &spender), value);
				let owner = to_address(&origin).0.into();
				let spender = to_address(&spender).0.into();
				let event = Approval { token, owner, spender, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn increase_allowance_works() {
		let token = 1;
		let origin = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), UNIT)])
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.build()
			.execute_with(|| {
				assert_ok!(Assets::approve_transfer(
					RuntimeOrigin::signed(origin.clone()),
					token.into(),
					AccountIdLookup::unlookup(spender.clone()),
					value,
				));
				assert_eq!(Assets::allowance(token, &origin, &spender), value);

				// Double the allowance.
				let allowance = call_precompile::<U256>(
					&origin,
					&IFungiblesCalls::increaseAllowance(increaseAllowanceCall {
						token,
						spender: to_address(&spender).0.into(),
						value: U256::from(value),
					}),
				)
				.unwrap();

				assert_eq!(allowance, U256::from(value * 2));
				assert_eq!(Assets::allowance(token, &origin, &spender), value * 2);
				let owner = to_address(&origin).0.into();
				let spender = to_address(&spender).0.into();
				let event = Approval { token, owner, spender, value: U256::from(allowance) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn decrease_allowance_works() {
		let token = 1;
		let origin = ALICE;
		let spender = BOB;
		let value = 10_000_000;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), UNIT)])
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.build()
			.execute_with(|| {
				assert_ok!(Assets::approve_transfer(
					RuntimeOrigin::signed(origin.clone()),
					token.into(),
					AccountIdLookup::unlookup(spender.clone()),
					value,
				));
				assert_eq!(Assets::allowance(token, &origin, &spender), value);

				// Halve the allowance.
				let allowance = call_precompile::<U256>(
					&origin,
					&IFungiblesCalls::decreaseAllowance(decreaseAllowanceCall {
						token,
						spender: to_address(&spender).0.into(),
						value: U256::from(value / 2),
					}),
				)
				.unwrap();

				assert_eq!(allowance, U256::from(value / 2));
				assert_eq!(Assets::allowance(token, &origin, &spender), value / 2);
				let owner = to_address(&origin).0.into();
				let spender = to_address(&spender).0.into();
				let event = Approval { token, owner, spender, value: U256::from(allowance) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn create_works() {
		let token = 0;
		let origin = ALICE;
		let admin = BOB;
		let min_balance = 1;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), UNIT)])
			.build()
			.execute_with(|| {
				assert!(!Assets::asset_exists(token));

				assert_eq!(
					call_precompile::<u32>(
						&origin,
						&IFungiblesCalls::create(createCall {
							admin: to_address(&admin).0.into(),
							minBalance: U256::from(min_balance),
						}),
					)
					.unwrap(),
					token
				);

				assert!(Assets::asset_exists(token));
				assert_eq!(
					Asset::get(token).unwrap(),
					AssetDetails {
						owner: origin.clone(),
						issuer: admin.clone(),
						admin: admin.clone(),
						freezer: admin.clone(),
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

				let creator = to_address(&origin).0.into();
				let admin = to_address(&admin).0.into();
				let event = Created { id: token, creator, admin };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn start_destroy_works() {
		let token = 1;
		let origin = ALICE;
		ExtBuilder::new()
			.with_assets(vec![(token, origin.clone(), false, 1)])
			.build()
			.execute_with(|| {
				assert_eq!(Asset::get(token).unwrap().status, AssetStatus::Live);

				assert_ok!(call_precompile::<()>(
					&origin,
					&startDestroy(startDestroyCall { token })
				));

				assert_eq!(Asset::get(token).unwrap().status, AssetStatus::Destroying);
			});
	}

	#[test]
	fn set_metadata_works() {
		let token = 1;
		let origin = ALICE;
		let name = "name".to_string();
		let symbol = "symbol".to_string();
		let decimals = u8::MAX;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), UNIT)])
			.with_assets(vec![(token, origin.clone(), false, 1)])
			.build()
			.execute_with(|| {
				assert_eq!(Metadata::get(token), AssetMetadata::default());

				assert_ok!(call_precompile::<()>(
					&origin,
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
			});
	}

	#[test]
	fn clear_metadata_works() {
		let token = 1;
		let origin = ALICE;
		let name = b"name".to_vec();
		let symbol = b"symbol".to_vec();
		let decimals = u8::MAX;
		ExtBuilder::new()
			.with_assets(vec![(token, origin.clone(), false, 1)])
			.with_asset_metadata(vec![(token, name.clone(), symbol.clone(), decimals)])
			.build()
			.execute_with(|| {
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
					&origin,
					&clearMetadata(clearMetadataCall { token })
				));

				assert_eq!(Metadata::get(token), AssetMetadata::default());
			});
	}

	#[test]
	fn mint_works() {
		let token = 1;
		let origin = ALICE;
		let value = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, origin.clone(), false, 1)])
			.build()
			.execute_with(|| {
				assert_eq!(Assets::balance(token, &origin), 0);

				assert_ok!(call_precompile::<()>(
					&origin,
					&IFungiblesCalls::mint(mintCall {
						token,
						account: to_address(&origin).0.into(),
						value: U256::from(value)
					})
				));

				assert_eq!(Assets::balance(token, &origin), value);
				let from = Address::default();
				let to = to_address(&origin).0.into();
				let event = Transfer { token, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn burn_works() {
		let token = 1;
		let origin = ALICE;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, origin.clone(), false, 1)])
			.with_asset_balances(vec![(token, origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_eq!(Assets::balance(token, &origin), endowment);

				assert_ok!(call_precompile::<()>(
					&origin,
					&IFungiblesCalls::burn(burnCall {
						token,
						account: to_address(&origin).0.into(),
						value: U256::from(endowment),
					}),
				));

				assert_eq!(Assets::balance(token, &origin), 0);
				let from = to_address(&origin).0.into();
				let to = Address::default();
				let event = Transfer { token, from, to, value: U256::from(endowment) };
				assert_last_event(ADDRESS, event);
			});
	}

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
					call_precompile::<U256>(
						&BOB,
						&IFungiblesCalls::totalSupply(totalSupplyCall { token })
					)
					.unwrap(),
					U256::from(total_supply)
				);
			});
	}

	#[test]
	fn balance_of_works() {
		let token = 1;
		let owner = ALICE;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_balances(vec![(token, owner.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<U256>(
						&BOB,
						&IFungiblesCalls::balanceOf(balanceOfCall {
							token,
							owner: to_address(&owner).0.into()
						})
					)
					.unwrap(),
					U256::from(endowment)
				);

				assert_eq!(Assets::balance(token, owner), endowment);
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
					spender.clone(),
					value
				));

				assert_eq!(
					call_precompile::<U256>(
						&BOB,
						&IFungiblesCalls::allowance(allowanceCall {
							token,
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
	fn name_works() {
		let token = 1;
		let name = "name".to_string();
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_metadata(vec![(token, name.as_bytes().into(), b"symbol".to_vec(), u8::MAX)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<String>(&ALICE, &IFungiblesCalls::name(nameCall { token }))
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
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), symbol.as_bytes().into(), u8::MAX)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<String>(
						&ALICE,
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
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.with_asset_metadata(vec![(token, b"name".to_vec(), b"symbol".to_vec(), decimals)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<u16>(
						&ALICE,
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
		ExtBuilder::new()
			.with_assets(vec![(token, CHARLIE, false, 1)])
			.build()
			.execute_with(|| {
				assert!(Assets::asset_exists(token));

				assert!(call_precompile::<bool>(
					&ALICE,
					&IFungiblesCalls::exists(existsCall { token })
				)
				.unwrap());

				let token = token + 1;
				assert!(!Assets::asset_exists(token));
				assert!(!call_precompile::<bool>(
					&ALICE,
					&IFungiblesCalls::exists(existsCall { token })
				)
				.unwrap());
			});
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		origin: &AccountId,
		input: &IFungiblesCalls,
	) -> Result<Output, DispatchError> {
		bare_call::<Test, Output>(
			RuntimeOrigin::signed(origin.clone()),
			ADDRESS.into(),
			0,
			Weight::MAX,
			DepositLimit::Balance(u128::MAX),
			input.abi_encode(),
		)
	}
}
