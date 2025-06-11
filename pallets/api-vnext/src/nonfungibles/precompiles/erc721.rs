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
			|transferFromCall { from, to, tokenId }: transferFromCall| -> Result<Vec<u8>, Error> {
				let item_id: ItemIdOf<T, I> = tokenId.saturating_to::<u32>().into();

				super::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id.into(),
					env.to_account_id(&(*to.0).into()),
					item_id,
				)?;
				deposit_event::<T>(env, address, Transfer { from, to, tokenId });
				Ok(transferFromCall::abi_encode_returns(&()))
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
			safeTransferFrom_0(safeTransferFrom_0Call { from, to, tokenId, data: _ }) =>
				transfer_from(transferFromCall { from: *from, to: *to, tokenId: *tokenId }),
			// TODO: checkOnERC721Received, reference: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC721/ERC721.sol#L135C21-L135C42
			safeTransferFrom_1(safeTransferFrom_1Call { from, to, tokenId }) =>
				transfer_from(transferFromCall { from: *from, to: *to, tokenId: *tokenId }),
			transferFrom(transferFromCall { from, to, tokenId }) =>
				transfer_from(transferFromCall { from: *from, to: *to, tokenId: *tokenId }),
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
			// IERC721Mintable
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
	pub fn address(id: u32) -> [u8; 20] {
		prefixed_address(PREFIX, id)
	}
}
