use frame_support::sp_runtime::traits::AtLeast32Bit;
use pallet_nfts::Item;
use pallet_revive::precompiles::alloy::{
	primitives::{Address, FixedBytes},
	sol_types::{Revert, SolCall},
};
use AddressMatcher::Prefix;
use IERC721::*;

use super::*;

sol!("src/nonfungibles/precompiles/interfaces/IERC721.sol");

/// Precompile providing an interface of the ERC-721 standard as defined in the ERC.
pub struct Erc721<const PREFIX: u16, T, I>(PhantomData<(T, I)>);
impl<
		const PREFIX: u16,
		T: frame_system::Config
			+ Config<I, CollectionId: AtLeast32Bit, ItemId: AtLeast32Bit>
			+ pallet_revive::Config,
		I: 'static,
	> Precompile for Erc721<PREFIX, T, I>
{
	type Interface = IERC721Calls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Prefix(NonZero::new(PREFIX).expect("expected non-zero precompile address"));

	fn call(
		address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		use IERC721::{IERC721Calls::*, *};

		let collection_id = InlineCollectionIdExtractor::collection_id_from_address(address)?;

		let mut transfer_from =
			|transferFromCall { from, to, tokenId }: transferFromCall| -> Result<(), Error> {
				let item_id: ItemIdOf<T, I> = tokenId.saturating_to::<u32>().into();
				super::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id.into(),
					env.to_account_id(&(*to.0).into()),
					item_id,
				)?;
				deposit_event::<T>(env, address, Transfer { from, to, tokenId });
				Ok(())
			};

		let get_attribute = |key: &str| -> Result<Vec<u8>, Error> {
			let collection_id: CollectionIdOf<T, I> = collection_id.into();
			let attribute = match Nfts::<T, I>::collection_attribute(&collection_id, key.as_bytes())
			{
				Some(value) => value,
				None =>
					return Err(Error::Revert(Revert {
						reason: "ERC721: No attributefound".to_string(),
					})),
			};
			// TODO: improve
			let result = String::from_utf8_lossy(attribute.as_slice());
			Ok(nameCall::abi_encode_returns(&(result,)))
		};

		match input {
			// IERC721
			balanceOf(balanceOfCall { owner }) => {
				// TODO: charge based on benchmarked weight
				let owner = env.to_account_id(&(*owner.0).into());
				let balance =
					U256::saturating_from(super::balance_of::<T, I>(collection_id.into(), owner));
				Ok(balanceOfCall::abi_encode_returns(&(balance,)))
			},
			ownerOf(ownerOfCall { tokenId }) => {
				let collection_id: CollectionIdOf<T, I> = collection_id.into();
				let item_id: ItemIdOf<T, I> = tokenId.saturating_to::<u32>().into();

				let owner = super::owner_of::<T, I>(collection_id, item_id).unwrap();
				let address: Address = <AddressMapper<T>>::to_address(&owner).0.into();
				Ok(ownerOfCall::abi_encode_returns(&(address,)))
			},
			// TODO: checkOnERC721Received, reference: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/ERC721.sol#L135C21-L135C42
			safeTransferFrom_0(safeTransferFrom_0Call { from, to, tokenId, data: _ }) => {
				transfer_from(transferFromCall { from: *from, to: *to, tokenId: *tokenId })?;
				Ok(safeTransferFrom_0Call::abi_encode_returns(&()))
			},
			// TODO: checkOnERC721Received, reference: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/ERC721.sol#L135C21-L135C42
			safeTransferFrom_1(safeTransferFrom_1Call { from, to, tokenId }) => {
				transfer_from(transferFromCall { from: *from, to: *to, tokenId: *tokenId })?;
				Ok(safeTransferFrom_1Call::abi_encode_returns(&()))
			},
			transferFrom(transferFromCall { from, to, tokenId }) => {
				transfer_from(transferFromCall { from: *from, to: *to, tokenId: *tokenId })?;
				Ok(transferFromCall::abi_encode_returns(&()))
			},
			approve(approveCall { to, tokenId }) => {
				// TODO: charge based on benchmarked weight
				let collection_id: CollectionIdOf<T, I> = collection_id.into();
				let item_id: ItemIdOf<T, I> = tokenId.saturating_to::<u32>().into();

				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				let item_details = match Item::<T, I>::get(collection_id, item_id) {
					Some(details) => details,
					None =>
						return Err(Error::Revert(Revert {
							reason: "ERC721: Item not found".to_string(),
						})),
				};

				// Only a single account can be approved at a time, so approving the zero address
				// clears previous approvals.
				if !item_details.approvals.is_empty() {
					return Err(Error::Revert(Revert {
						reason: "ERC721: token already approved".to_string(),
					}));
				}
				if *to == Address(FixedBytes::default()) {
					super::clear_all_transfer_approvals::<T, I>(
						to_runtime_origin(env.caller()),
						collection_id,
						item_id,
					)?;
				} else {
					super::approve::<T, I>(
						to_runtime_origin(env.caller()),
						collection_id.into(),
						env.to_account_id(&(*to.0).into()),
						Some(item_id),
						true,
						None,
					)
					.map_err(|e| e.error)?;

					deposit_event(
						env,
						address,
						Approval { owner, approved: *to, tokenId: *tokenId },
					);
				}
				Ok(approveCall::abi_encode_returns(&()))
			},
			setApprovalForAll(setApprovalForAllCall { operator, approved }) => {
				// TODO: charge based on benchmarked weight
				let collection_id: CollectionIdOf<T, I> = collection_id.into();
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				super::approve::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					env.to_account_id(&(*operator.0).into()),
					None,
					*approved,
					None,
				)
				.map_err(|e| e.error)?;

				deposit_event(
					env,
					address,
					ApprovalForAll { owner, operator: *operator, approved: *approved },
				);
				Ok(setApprovalForAllCall::abi_encode_returns(&()))
			},
			getApproved(getApprovedCall { tokenId }) => {
				// TODO: charge based on benchmarked weight
				let collection_id: CollectionIdOf<T, I> = collection_id.into();
				let item_id: ItemIdOf<T, I> = tokenId.saturating_to::<u32>().into();

				let item_details = match Item::<T, I>::get(collection_id, item_id) {
					Some(details) => details,
					None =>
						return Err(Error::Revert(Revert {
							reason: "ERC721: Item not found".to_string(),
						})),
				};
				let accounts = item_details.approvals.first_key_value();
				match accounts {
					Some((approved, _)) => {
						let address: Address = <AddressMapper<T>>::to_address(&approved).0.into();
						Ok(getApprovedCall::abi_encode_returns(&(address,)))
					},
					None => Err(Error::Revert(Revert {
						reason: "ERC721: No approval found".to_string(),
					})),
				}
			},
			isApprovedForAll(isApprovedForAllCall { owner, operator }) => {
				let collection_id: CollectionIdOf<T, I> = collection_id.into();
				let approved = super::allowance::<T, I>(
					collection_id,
					env.to_account_id(&(*owner.0).into()),
					env.to_account_id(&(*operator.0).into()),
					None,
				);
				Ok(isApprovedForAllCall::abi_encode_returns(&(approved,)))
			},
			// IERC721Burnable
			burn(burnCall { tokenId }) => {
				let item_id: ItemIdOf<T, I> = tokenId.saturating_to::<u32>().into();

				// Clear all transfer approvals before burning a collection item.
				// Reference: https://github.com/binodnp/openzeppelin-solidity/blob/master/contracts/token/ERC721/ERC721.sol#L260
				super::clear_all_transfer_approvals::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id.into(),
					item_id,
				)?;
				super::burn::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id.into(),
					item_id,
				)?;
				Ok(burnCall::abi_encode_returns(&()))
			},
			// IERC721Metadata
			// Reference: https://wiki.polkadot.network/learn/learn-nft-pallets/
			name(_) => get_attribute("name"),
			symbol(_) => get_attribute("symbol"),
			tokenURI(_) => get_attribute("image"),
		}
	}
}

impl<const PREFIX: u16, T: Config<I>, I: 'static> Erc721<PREFIX, T, I> {
	pub fn address(collection_id: u32) -> [u8; 20] {
		prefixed_address(PREFIX, collection_id)
	}
}

#[cfg(test)]
mod tests {
	use frame_support::assert_ok;
	use pallet_nfts::{CollectionConfig, CollectionSettings, Instance1, MintSettings};
	use pallet_revive::{
		precompiles::{
			alloy::{primitives::Bytes, sol_types::SolType},
			ExtWithInfo,
		},
		test_utils::{ALICE, ALICE_ADDR, BOB, BOB_ADDR},
	};
	use IERC721Calls::*;

	use super::*;
	use crate::mock::{ExtBuilder, RuntimeOrigin, Test};

	const ERC721: u16 = 100;

	type Erc721 = super::Erc721<ERC721, Test, Instance1>;

	#[test]
	fn balance_of_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			create_collection_and_mint(ALICE, collection_id, item_id);
			assert_eq!(
				call_precompile::<U256>(
					&mut call_setup.ext().0,
					collection_id,
					&balanceOf(balanceOfCall { owner: ALICE_ADDR.0.into() })
				)
				.unwrap(),
				U256::from(1)
			);
			assert_eq!(super::balance_of::<Test, Instance1>(collection_id, ALICE), 1);
		});
	}

	#[test]
	fn owner_of_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			create_collection_and_mint(ALICE, collection_id, item_id);
			let address: Address = ALICE_ADDR.0.into();
			assert_eq!(
				call_precompile::<Address>(
					&mut call_setup.ext().0,
					collection_id,
					&ownerOf(ownerOfCall { tokenId: U256::from(item_id) })
				)
				.unwrap(),
				address
			);
			assert_eq!(super::owner_of::<Test, Instance1>(collection_id, item_id), Some(ALICE));
		});
	}

	#[test]
	fn safe_transfer_from_0_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			create_collection_and_mint(ALICE, collection_id, item_id);
			assert_ok!(super::approve::<Test, Instance1>(
				RuntimeOrigin::signed(ALICE),
				collection_id,
				BOB,
				Some(item_id),
				true,
				None,
			));
			call_setup.set_origin(Origin::Signed(BOB));
			assert_ok!(call_precompile::<()>(
				&mut call_setup.ext().0,
				collection_id,
				&safeTransferFrom_0(safeTransferFrom_0Call {
					from: ALICE_ADDR.0.into(),
					to: BOB_ADDR.0.into(),
					tokenId: U256::from(item_id),
					data: Bytes::default()
				})
			));
			assert_eq!(super::balance_of::<Test, Instance1>(collection_id, BOB), 1);
		});
	}

	#[test]
	fn safe_transfer_from_1_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			create_collection_and_mint(ALICE, collection_id, item_id);
			assert_ok!(super::approve::<Test, Instance1>(
				RuntimeOrigin::signed(ALICE),
				collection_id,
				BOB,
				Some(item_id),
				true,
				None,
			));
			call_setup.set_origin(Origin::Signed(BOB));
			assert_ok!(call_precompile::<()>(
				&mut call_setup.ext().0,
				collection_id,
				&safeTransferFrom_1(safeTransferFrom_1Call {
					from: ALICE_ADDR.0.into(),
					to: BOB_ADDR.0.into(),
					tokenId: U256::from(item_id),
				})
			));
			assert_eq!(super::balance_of::<Test, Instance1>(collection_id, BOB), 1);
		});
	}

	#[test]
	fn transfer_from_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			create_collection_and_mint(ALICE, collection_id, item_id);
			assert_ok!(super::approve::<Test, Instance1>(
				RuntimeOrigin::signed(ALICE),
				collection_id,
				BOB,
				Some(item_id),
				true,
				None,
			));
			call_setup.set_origin(Origin::Signed(BOB));
			assert_ok!(call_precompile::<()>(
				&mut call_setup.ext().0,
				collection_id,
				&transferFrom(transferFromCall {
					from: ALICE_ADDR.0.into(),
					to: BOB_ADDR.0.into(),
					tokenId: U256::from(item_id)
				})
			));
			assert_eq!(super::balance_of::<Test, Instance1>(collection_id, BOB), 1);
		});
	}

	#[test]
	fn approve_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		ExtBuilder::new().build_with_env(|mut call_setup| {
			call_setup.set_origin(Origin::Signed(ALICE));
			create_collection_and_mint(ALICE, collection_id, item_id);
			assert_ok!(call_precompile::<()>(
				&mut call_setup.ext().0,
				collection_id,
				&IERC721Calls::approve(approveCall {
					to: BOB_ADDR.0.into(),
					tokenId: U256::from(item_id)
				})
			));
			assert!(super::allowance::<Test, Instance1>(collection_id, ALICE, BOB, Some(item_id)));
		});
	}

	fn create_collection_and_mint(owner: AccountIdOf<Test>, collection_id: u32, item_id: u32) {
		assert_ok!(super::create::<Test, Instance1>(
			RuntimeOrigin::signed(owner.clone()),
			owner.clone(),
			default_collection_config(),
		));
		assert_ok!(super::mint::<Test, Instance1>(
			RuntimeOrigin::signed(owner.clone()),
			collection_id,
			owner,
			item_id,
			None,
		));
	}

	fn default_collection_config() -> CollectionConfigFor<Test, Instance1> {
		CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		ext: &mut impl ExtWithInfo<T = mock::Test>,
		id: u32,
		input: &IERC721Calls,
	) -> Result<Output, Error> {
		super::call_precompile::<Erc721, Output>(ext, &prefixed_address(ERC721, id), input)
	}
}
