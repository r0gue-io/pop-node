pub use erc20::{Erc20, IERC20};
use pallet_revive::precompiles::alloy::{
	primitives::{ruint::UintTryTo, Address},
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
	U256: UintTryTo<<T as Config<I>>::Balance>,
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
		use IFungibles::IFungiblesCalls::*;
		match input {
			create(createCall { admin, minBalance }) => {
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
			mint(mintCall { id, account, value }) => {
				// TODO: charge based on benchmarked weight

				self::mint::<T, I>(
					to_runtime_origin(env.caller()),
					(*id).into(),
					env.to_account_id(&(*account.0).into()),
					value.saturating_to(),
				)?;

				let from = Address::default();
				let to = *account;
				deposit_event(env, address, Transfer { id: *id, from, to, value: *value });
				Ok(mintCall::abi_encode_returns(&()))
			},
			transfer(transferCall { id, to, value }) => {
				// TODO: charge based on benchmarked weight
				let from = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				self::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					(*id).into(),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event(env, address, Transfer { id: *id, from, to: *to, value: *value });
				Ok(transferCall::abi_encode_returns(&()))
			},
			approve(approveCall { id, spender, value }) => {
				// TODO: charge based on benchmarked weight
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				self::approve::<T, I>(
					to_runtime_origin(env.caller()),
					(*id).into(),
					env.to_account_id(&(*spender.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;

				deposit_event(env, address, Approval { id: *id, owner, spender: *spender, value: *value });
				Ok(approveCall::abi_encode_returns(&()))
			},
			transferFrom(transferFromCall { id, from, to, value }) => {
				// TODO: charge based on benchmarked weight

				transfer_from::<T, I>(
					to_runtime_origin(env.caller()),
					(*id).into(),
					env.to_account_id(&(*from.0).into()),
					env.to_account_id(&(*to.0).into()),
					value.saturating_to(),
				)?;

				deposit_event(env, address, Transfer { id: *id, from: *from, to: *to, value: *value });
				Ok(transferFromCall::abi_encode_returns(&()))
			},
			burn(burnCall { id, account, value }) => {
				// TODO: charge based on benchmarked weight

				self::burn::<T, I>(
					to_runtime_origin(env.caller()),
					(*id).into(),
					env.to_account_id(&(*account.0).into()),
					value.saturating_to(),
				) // TODO: adjust weight
				.map_err(|e| e.error)?;

				let from = account;
				let to = Address::default();
				deposit_event(env, address, Transfer { id: *id, from: *from, to, value: *value });
				Ok(burnCall::abi_encode_returns(&()))
			},
			startDestroy(startDestroyCall { id }) => {
				// TODO: charge based on benchmarked weight
				start_destroy::<T, I>(to_runtime_origin(env.caller()), (*id).into())?;
				Ok(startDestroyCall::abi_encode_returns(&()))
			},
			exists(existsCall { id }) => {
				let result = self::exists::<T, I>((*id).into());
				Ok(existsCall::abi_encode_returns(&(result,)))
			},
			setMetadata(setMetadataCall { id, name, symbol, decimals }) => {
				// TODO: charge based on benchmarked weight
				set_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*id).into(),
					name.as_bytes().to_vec(),
					symbol.as_bytes().to_vec(),
					*decimals,
				)?;
				Ok(setMetadataCall::abi_encode_returns(&()))
			},
			clearMetadata(clearMetadataCall { id }) => {
				// TODO: charge based on benchmarked weight
				clear_metadata::<T, I>(to_runtime_origin(env.caller()), (*id).into())?;
				Ok(clearMetadataCall::abi_encode_returns(&()))
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
	use frame_support::{assert_ok, traits::fungibles::Inspect, weights::Weight, BoundedVec};
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
	fn mint_works() {
		let id = 1;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(id, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::balance(id, ALICE), 0);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::mint(mintCall {
						id,
						account: ALICE_ADDR.0.into(),
						value: U256::from(value)
					})
				));

				assert_eq!(Assets::balance(id, ALICE), value);
				let from = Address::default();
				let to = ALICE_ADDR.0.into();
				let event = Transfer { id, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			},
		);
	}

	#[test]
	fn transfer_works() {
		let id = 1;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(id, ALICE, true, 1)])
			.with_asset_balances(vec![(id, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::balance(id, ALICE), endowment);
				assert_eq!(Assets::balance(id, BOB), 0);

				let value = endowment / 2;
				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::transfer(transferCall {
						id,
						to: BOB_ADDR.0.into(),
						value: U256::from(value)
					})
				));

				assert_eq!(Assets::balance(id, ALICE), endowment - value);
				assert_eq!(Assets::balance(id, BOB), value);

				let from = ALICE_ADDR.0.into();
				let to = BOB_ADDR.0.into();
				let event = Transfer { id, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn approve_works() {
		let id = 1;
		let value = 10_000_000;
		ExtBuilder::new().with_assets(vec![(id, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::allowance(id, &ALICE, &BOB), 0);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::approve(approveCall {
						id,
						spender: BOB_ADDR.0.into(),
						value: U256::from(value),
					})
				));

				assert_eq!(Assets::allowance(id, &ALICE, &BOB), value);
				let owner = ALICE_ADDR.0.into();
				let spender = BOB_ADDR.0.into();
				let event = Approval { id, owner, spender, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			},
		);
	}

	#[test]
	fn transfer_from_works() {
		let id = 1;
		let endowment = 10_000_000;
		let value = endowment / 2;
		ExtBuilder::new()
			.with_assets(vec![(id, ALICE, true, 1)])
			.with_asset_balances(vec![(id, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				assert_eq!(Assets::balance(id, ALICE), endowment);
				assert_eq!(Assets::balance(id, BOB), 0);
				assert_ok!(approve::<Test, Instance1>(
					RuntimeOrigin::signed(ALICE),
					id,
					BOB,
					value
				));
				call_setup.set_origin(Origin::Signed(BOB));

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::transferFrom(transferFromCall {
						id,
						from: ALICE_ADDR.0.into(),
						to: BOB_ADDR.0.into(),
						value: U256::from(value),
					})
				));

				assert_eq!(Assets::balance(id, ALICE), endowment - value);
				assert_eq!(Assets::balance(id, BOB), value);
				let from = ALICE_ADDR.0.into();
				let to = BOB_ADDR.0.into();
				let event = Transfer { id, from, to, value: U256::from(value) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn burn_works() {
		let id = 1;
		let endowment = 10_000_000;
		ExtBuilder::new()
			.with_assets(vec![(id, ALICE, false, 1)])
			.with_asset_balances(vec![(id, ALICE, endowment)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Assets::balance(id, ALICE), endowment);

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&IFungiblesCalls::burn(burnCall {
						id,
						account: ALICE_ADDR.0.into(),
						value: U256::from(endowment),
					}),
				));

				assert_eq!(Assets::balance(id, ALICE), 0);
				let from = ALICE_ADDR.0.into();
				let to = Address::default();
				let event = Transfer { id, from, to, value: U256::from(endowment) };
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn start_destroy_works() {
		let id = 1;
		ExtBuilder::new().with_assets(vec![(id, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Asset::get(id).unwrap().status, AssetStatus::Live);

				let mut ext = call_setup.ext().0;
				assert_ok!(call_precompile::<()>(&mut ext, &startDestroy(startDestroyCall { id })));

				assert_eq!(Asset::get(id).unwrap().status, AssetStatus::Destroying);
			},
		);
	}

	#[test]
	fn exists_works() {
		let id = 1;
		ExtBuilder::new().with_assets(vec![(id, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert!(Assets::asset_exists(id));

				let mut ext = call_setup.ext().0;
				assert!(call_precompile::<bool>(
					&mut ext,
					&IFungiblesCalls::exists(existsCall { id })
				)
				.unwrap());

				let id = id + 1;
				assert!(!Assets::asset_exists(id));
				assert!(!call_precompile::<bool>(
					&mut ext,
					&IFungiblesCalls::exists(existsCall { id })
				)
				.unwrap());
			},
		);
	}

	#[test]
	fn exists_via_bare_call_works() {
		let id = 1;
		let origin = RuntimeOrigin::signed(ALICE);
		ExtBuilder::new()
			.with_assets(vec![(id, ALICE, false, 1)])
			.build()
			.execute_with(|| {
				assert!(Assets::asset_exists(id));

				let asset_exists = bare_call::<Test, bool>(
					origin.clone(),
					ADDRESS.into(),
					0,
					Weight::MAX,
					Unchecked,
					IFungiblesCalls::exists(existsCall { id }).abi_encode(),
				)
				.unwrap();
				assert!(asset_exists);

				let id = id + 1;
				assert!(!Assets::asset_exists(id));
				let exists = bare_call::<Test, bool>(
					origin,
					ADDRESS.into(),
					0,
					Weight::MAX,
					Unchecked,
					IFungiblesCalls::exists(existsCall { id }).abi_encode(),
				)
				.unwrap();
				assert!(!exists);
			});
	}

	#[test]
	fn set_metadata_works() {
		let id = 1;
		let name = "name".to_string();
		let symbol = "symbol".to_string();
		let decimals = u8::MAX;
		ExtBuilder::new().with_assets(vec![(id, ALICE, false, 1)]).build_with_env(
			|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(Metadata::get(id), AssetMetadata::default());

				assert_ok!(call_precompile::<()>(
					&mut call_setup.ext().0,
					&setMetadata(setMetadataCall {
						id,
						name: name.clone(),
						symbol: symbol.clone(),
						decimals
					})
				));

				assert_eq!(
					Metadata::get(id),
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
		let id = 1;
		let name = b"name".to_vec();
		let symbol = b"symbol".to_vec();
		let decimals = u8::MAX;
		ExtBuilder::new()
			.with_assets(vec![(id, ALICE, false, 1)])
			.with_asset_metadata(vec![(id, name.clone(), symbol.clone(), decimals)])
			.build_with_env(|mut call_setup| {
				call_setup.set_origin(Origin::Signed(ALICE));
				assert_eq!(
					Metadata::get(id),
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
					&clearMetadata(clearMetadataCall { id })
				));

				assert_eq!(Metadata::get(id), AssetMetadata::default());
			});
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		ext: &mut impl ExtWithInfo<T = Test>,
		input: &IFungiblesCalls,
	) -> Result<Output, Error> {
		super::call_precompile::<Fungibles, Output>(ext, &ADDRESS, input)
	}
}
