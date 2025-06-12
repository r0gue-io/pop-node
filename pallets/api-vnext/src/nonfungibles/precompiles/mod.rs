pub use erc721::{Erc721, IERC721};
use pallet_revive::precompiles::{
	alloy::sol_types::{Revert, SolCall},
	AddressMatcher::Fixed,
};
use INonfungibles::*;

use super::*;

mod erc721;

sol!("src/nonfungibles/precompiles/interfaces/INonfungibles.sol");

/// The nonfungibles precompile offers a streamlined interface for interacting with nonfungible
/// tokens. The goal is to provide a simplified, consistent API that adheres to standards in the
/// smart contract space.
pub struct Nonfungibles<const FIXED: u16, T, I>(PhantomData<(T, I)>);
impl<
		const FIXED: u16,
		T: frame_system::Config
			+ Config<I, CollectionId: Default + From<u32> + Into<u32>, ItemId: Default + From<u32>>
			+ pallet_revive::Config,
		I: 'static,
	> Precompile for Nonfungibles<FIXED, T, I>
{
	type Interface = INonfungiblesCalls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Fixed(NonZero::new(FIXED).expect("expected non-zero precompile address"));

	fn call(
		address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		use INonfungibles::{INonfungiblesCalls::*, *};
		match input {
			approveTransfer(approveTransferCall {
				collection,
				operator,
				item,
				approved,
				deadline,
			}) => {
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();

				// Successfully approves.
				let deadline: Option<BlockNumberFor<T, I>> =
					if *deadline > 0 { Some((*deadline).into()) } else { None };
				super::approve::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					env.to_account_id(&(*operator.0).into()),
					Some(item_id),
					*approved,
					deadline,
				)
				.map_err(|e| e.error)?;
				deposit_event(
					env,
					address,
					ItemApproval { operator: *operator, approved: *approved, item: *item, owner },
				);
				Ok(approveTransferCall::abi_encode_returns(&()))
			},
			approveCollection(approveCollectionCall {
				collection,
				operator,
				approved,
				deadline,
			}) => {
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				let collection_id: CollectionIdOf<T, I> = (*collection).into();

				// Successfully approves a collection.
				let deadline: Option<BlockNumberFor<T, I>> =
					if *deadline > 0 { Some((*deadline).into()) } else { None };
				super::approve::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					env.to_account_id(&(*operator.0).into()),
					None,
					*approved,
					deadline,
				)
				.map_err(|e| e.error)?;
				deposit_event(
					env,
					address,
					CollectionApproval {
						operator: *operator,
						approved: *approved,
						collection: *collection,
						owner,
					},
				);
				Ok(approveCollectionCall::abi_encode_returns(&()))
			},
			transfer(transferCall { collection, to, item }) => {
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();

				// Successfully transfers an item.
				super::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					env.to_account_id(&(*to.0).into()),
					item_id,
				)?;
				deposit_event(env, address, Transfer { from: owner, to: *to, item: *item });
				Ok(transferCall::abi_encode_returns(&()))
			},
			create(createCall { admin, config }) => {
				let collection_id: u32 =
					super::next_collection_id::<T, I>().unwrap_or_default().into();

				// Successfully creates a collection.
				super::create::<T, I>(
					to_runtime_origin(env.caller()),
					env.to_account_id(&(*admin.0).into()),
					decode_bytes::<CollectionConfigFor<T, I>>(config)?,
				)?;
				Ok(createCall::abi_encode_returns(&(collection_id,)))
			},
			destroy(destroyCall { collection, witness }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();

				// Successfully destroys a collection.
				super::destroy::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					decode_bytes(witness)?,
				)
				.map_err(|e| e.error)?;
				Ok(destroyCall::abi_encode_returns(&()))
			},
			setItemAttribute(setItemAttributeCall { collection, item, namespace, key, value }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();
				super::set_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					Some(item_id),
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
					BoundedVec::truncate_from(value.to_vec()),
				)?;
				deposit_event(
					env,
					address,
					ItemAttributeSet { key: key.clone(), data: value.clone(), item: *item },
				);
				Ok(destroyCall::abi_encode_returns(&()))
			},
			setCollectionAttribute(setCollectionAttributeCall {
				collection,
				namespace,
				key,
				value,
			}) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				super::set_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					None,
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
					BoundedVec::truncate_from(value.to_vec()),
				)?;
				deposit_event(
					env,
					address,
					CollectionAttributeSet {
						key: key.clone(),
						data: value.clone(),
						collection: *collection,
					},
				);
				Ok(setCollectionAttributeCall::abi_encode_returns(&()))
			},
			clearAttribute(_) => {
				unimplemented!()
			},
			setMetadata(_) => {
				unimplemented!()
			},
			clearMetadata(_) => {
				unimplemented!()
			},
			setMaxSupply(_) => {
				unimplemented!()
			},
			approveItemAttributes(_) => {
				unimplemented!()
			},
			cancelItemAttributesApproval(_) => {
				unimplemented!()
			},
			clearCollectionApprovals(_) => {
				unimplemented!()
			},
			mint(_) => {
				unimplemented!()
			},
			burn(_) => {
				unimplemented!()
			},
			balanceOf(_) => {
				unimplemented!()
			},
			ownerOf(_) => {
				unimplemented!()
			},
			allowance(_) => {
				unimplemented!()
			},
			totalSupply(_) => {
				unimplemented!()
			},
			getItemAttributes(getItemAttributesCall { collection, item, namespace, key }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();
				let attribute = match super::get_attributes::<T, I>(
					collection_id,
					Some(item_id),
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
				) {
					Some(value) => value,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No attribute found".to_string(),
						})),
				};
				Ok(getItemAttributesCall::abi_encode_returns(&(String::from_utf8_lossy(
					&attribute,
				),)))
			},
			getCollectionAttributes(getCollectionAttributesCall { collection, namespace, key }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let attribute = match super::get_attributes::<T, I>(
					collection_id,
					None,
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
				) {
					Some(value) => value,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No attribute found".to_string(),
						})),
				};
				Ok(getItemAttributesCall::abi_encode_returns(&(String::from_utf8_lossy(
					&attribute,
				),)))
			},
			itemMetadata(_) => {
				unimplemented!()
			},
			_ => unimplemented!(),
		}
	}
}

impl<const FIXED: u16, T: Config<I>, I: 'static> Nonfungibles<FIXED, T, I> {
	pub const fn address() -> [u8; 20] {
		fixed_address(FIXED)
	}
}
