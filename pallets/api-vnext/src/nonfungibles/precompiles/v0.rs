pub use erc721::{Erc721, IERC721};
use pallet_revive::precompiles::{
	alloy::sol_types::{Revert, SolCall},
	AddressMatcher::Fixed,
};
use INonfungibles::*;

use super::*;

sol!("src/nonfungibles/precompiles/interfaces/v0/INonfungibles.sol");

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
			clearAttribute(clearAttributeCall { collection, item, namespace, key }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id =
					if let Some(item_value) = item { Some((*item_value).into()) } else { None };

				super::clear_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					item_id,
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
				)?;

				if let Some(item_value) = item {
					deposit_event(
						env,
						address,
						ItemAttributeCleared { key: key.clone(), item: *item_value },
					);
				} else {
					deposit_event(
						env,
						address,
						CollectionAttributeCleared { key: key.clone(), collection: *collection },
					);
				}

				Ok(clearAttributeCall::abi_encode_returns(&()))
			},
			setMetadata(setMetadataCall { collection, item, data }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id =
					if let Some(item_value) = item { Some((*item_value).into()) } else { None };

				super::set_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					item_id,
					BoundedVec::truncate_from(data.to_vec()),
				)?;

				if let Some(item_value) = item {
					deposit_event(
						env,
						address,
						ItemMetadataSet { data: data.clone(), item: *item_value },
					);
				} else {
					deposit_event(
						env,
						address,
						CollectionMetadataSet { data: data.clone(), collection: *collection },
					);
				}

				Ok(setMetadataCall::abi_encode_returns(&()))
			},
			clearMetadata(clearMetadataCall { collection, item }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id =
					if let Some(item_value) = item { Some((*item_value).into()) } else { None };

				super::clear_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					item_id,
				)?;

				if let Some(item_value) = item {
					deposit_event(env, address, ItemMetadataCleared { item: *item_value });
				} else {
					deposit_event(
						env,
						address,
						CollectionMetadataCleared { collection: *collection },
					);
				}

				Ok(clearMetadataCall::abi_encode_returns(&()))
			},
			setMaxSupply(setMaxSupplyCall { collection, max_supply }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();

				super::set_collection_max_supply::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					(*max_supply).into(),
				)?;

				deposit_event(
					env,
					address,
					CollectionMaxSupplySet { max_supply: *max_supply, collection: *collection },
				);

				Ok(setMaxSupplyCall::abi_encode_returns(&()))
			},
			approveItemAttributes(approveItemAttributesCall { collection, operator }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();

				super::set_accept_ownership::<T, I>(
					to_runtime_origin(env.to_account_id(&(*operator.0).into())),
					collection_id,
				)?;

				deposit_event(
					env,
					address,
					ItemAttributesApprovalSet { operator: *operator, collection: *collection },
				);

				Ok(approveItemAttributesCall::abi_encode_returns(&()))
			},
			cancelItemAttributesApproval(cancelItemAttributesApprovalCall {
				collection,
				witness,
			}) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();

				super::cancel_approval::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					decode_bytes(witness)?,
				)?;

				deposit_event(
					env,
					address,
					ItemAttributesApprovalCancelled { collection: *collection },
				);

				Ok(cancelItemAttributesApprovalCall::abi_encode_returns(&()))
			},
			clearCollectionApprovals(clearCollectionApprovalsCall { collection }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				super::clear_all_transfer_approvals::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
				)?;

				deposit_event(
					env,
					address,
					CollectionApprovalsCleared { collection: *collection, owner },
				);

				Ok(clearCollectionApprovalsCall::abi_encode_returns(&()))
			},
			mint(mintCall { collection, item, owner }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();

				super::mint::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					item_id,
					env.to_account_id(&(*owner.0).into()),
					None,
				)?;

				deposit_event(env, address, Mint { to: *owner, item: *item });

				Ok(mintCall::abi_encode_returns(&()))
			},
			burn(burnCall { collection, item }) => {
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();
				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();

				super::burn::<T, I>(to_runtime_origin(env.caller()), collection_id, item_id)?;

				deposit_event(env, address, Burn { from: owner, item: *item });

				Ok(burnCall::abi_encode_returns(&()))
			},
			balanceOf(balanceOfCall { owner }) => {
				// Charge the weight for this call.
				T::ReviveCallRuntimeCost::charge_weight(
					RuntimeCosts::BalanceOf,
					env.remaining_gas()?,
				)?;

				let account_id = env.to_account_id(&(*owner.0).into());
				let balance = super::balance_of::<T, I>(account_id);

				Ok(balanceOfCall::abi_encode_returns(&(balance.into(),)))
			},
			ownerOf(ownerOfCall { collection, item }) => {
				// Charge the weight for this call.
				T::ReviveCallRuntimeCost::charge_weight(
					RuntimeCosts::OwnerOf,
					env.remaining_gas()?,
				)?;

				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();

				let owner = match super::owner_of::<T, I>(collection_id, item_id) {
					Some(owner) => owner,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No owner found for item".to_string(),
						})),
				};

				let address = <AddressMapper<T>>::from_address(owner);

				Ok(ownerOfCall::abi_encode_returns(&(address,)))
			},
			allowance(allowanceCall { owner, operator, item }) => {
				// Charge the weight for this call.
				T::ReviveCallRuntimeCost::charge_weight(
					RuntimeCosts::Allowance,
					env.remaining_gas()?,
				)?;

				let collection_id: CollectionIdOf<T, I> = (*item >> 128).into();
				let item_id: ItemIdOf<T, I> = (*item & 0xFFFFFFFF).into();

				let owner_account_id = env.to_account_id(&(*owner.0).into());
				let operator_account_id = env.to_account_id(&(*operator.0).into());

				let is_approved = super::allowance::<T, I>(
					collection_id,
					item_id,
					owner_account_id,
					operator_account_id,
				);

				Ok(allowanceCall::abi_encode_returns(&(is_approved,)))
			},
			totalSupply(totalSupplyCall { collection }) => {
				// Charge the weight for this call.
				T::ReviveCallRuntimeCost::charge_weight(
					RuntimeCosts::TotalSupply,
					env.remaining_gas()?,
				)?;

				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let total = super::total_supply::<T, I>(collection_id);

				Ok(totalSupplyCall::abi_encode_returns(&(total.into(),)))
			},
			itemMetadata(itemMetadataCall { collection, item }) => {
				// Charge the weight for this call.
				T::ReviveCallRuntimeCost::charge_weight(
					RuntimeCosts::ItemMetadata,
					env.remaining_gas()?,
				)?;

				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();

				let metadata = match super::item_metadata::<T, I>(collection_id, item_id) {
					Some(metadata) => metadata,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No metadata found for item".to_string(),
						})),
				};

				Ok(itemMetadataCall::abi_encode_returns(&(String::from_utf8_lossy(&metadata),)))
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

#[cfg(test)]
mod tests {
	use frame_support::{assert_ok, traits::ConstU32};
	use mock::{ExtBuilder, RuntimeEvent as Event, Test, TestCall};
	use pallet_nfts::{Instance1, Pallet as Nfts};
	use pallet_revive::precompiles::{ExtWithInfo, ExtWithInfoExt, Output};

	use super::*;
	use crate::tests::{accounts, mock};
	type TheFungibles = Nonfungibles<1, Test, Instance1>;

	#[test]
	fn approve_transfer_works() {
		ExtBuilder::default().build().execute_with(|| {
			let collection = create_collection(accounts::alice());
			let item_id = 0u32;

			// Mint an NFT to Alice
			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

			// Approve Bob to transfer Alice's NFT
			let mut ext = ExtBuilder::build_ext();
			ext.setup(|_| {});
			ext.set_caller(accounts::alice());

			let call = INonfungiblesCalls::approveTransfer(approveTransferCall {
				collection,
				operator: Address::from(accounts::bob()),
				item: item_id,
				approved: true,
				deadline: 0,
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Verify Bob is approved to transfer Alice's NFT
			let approved = super::allowance::<Test, Instance1>(
				collection,
				accounts::alice(),
				accounts::bob(),
				Some(item_id),
			);
			assert!(approved);
		});
	}

	#[test]
	fn transfer_works() {
		ExtBuilder::default().build().execute_with(|| {
			let collection = create_collection(accounts::alice());
			let item_id = 0u32;

			// Mint an NFT to Alice
			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

			// Transfer from Alice to Bob
			let mut ext = ExtBuilder::build_ext();
			ext.setup(|_| {});
			ext.set_caller(accounts::alice());

			let call = INonfungiblesCalls::transfer(transferCall {
				collection,
				to: Address::from(accounts::bob()),
				item: item_id,
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Verify Bob is now the owner
			let owner = super::owner_of::<Test, Instance1>(collection, item_id);
			assert_eq!(owner, Some(accounts::bob()));
		});
	}

	#[test]
	fn create_and_mint_works() {
		ExtBuilder::default().build().execute_with(|| {
			// Create a collection
			let mut ext = ExtBuilder::build_ext();
			ext.setup(|_| {});
			ext.set_caller(accounts::alice());

			let config =
				super::decode_bytes::<CollectionConfigFor<Test, Instance1>>(&abi::encode(&[
					alloy::sol_types::SolValue::Bytes(Vec::new()),
				]))
				.unwrap();

			let call = INonfungiblesCalls::create(createCall {
				admin: Address::from(accounts::alice()),
				config: abi::encode(&[alloy::sol_types::SolValue::Bytes(Vec::new())]),
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Extract collection ID from the result
			let collection_id = 0u32; // First collection created

			// Mint an NFT
			let item_id = 0u32;
			let call = INonfungiblesCalls::mint(mintCall {
				collection: collection_id,
				item: item_id,
				owner: Address::from(accounts::bob()),
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Verify Bob is now the owner
			let owner = super::owner_of::<Test, Instance1>(collection_id, item_id);
			assert_eq!(owner, Some(accounts::bob()));
		});
	}

	#[test]
	fn set_and_get_attributes_works() {
		ExtBuilder::default().build().execute_with(|| {
			let collection = create_collection(accounts::alice());
			let item_id = 0u32;

			// Mint an NFT to Alice
			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

			// Set item attribute
			let mut ext = ExtBuilder::build_ext();
			ext.setup(|_| {});
			ext.set_caller(accounts::alice());

			let key = "key".as_bytes().to_vec();
			let value = "value".as_bytes().to_vec();

			let namespace = AttributeNamespace::Pallet;
			let encoded_namespace = abi::encode(&[alloy::sol_types::SolValue::Uint(0u32.into())]);

			let call = INonfungiblesCalls::setItemAttribute(setItemAttributeCall {
				collection,
				item: item_id,
				namespace: encoded_namespace.clone(),
				key: key.clone(),
				value: value.clone(),
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Get item attribute
			let call = INonfungiblesCalls::getItemAttributes(getItemAttributesCall {
				collection,
				item: item_id,
				namespace: encoded_namespace,
				key,
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call).unwrap();

			// Verify attribute value
			let decoded = getItemAttributesCall::abi_decode_returns(&result).unwrap();
			assert_eq!(decoded.0, "value");
		});
	}

	#[test]
	fn burn_works() {
		ExtBuilder::default().build().execute_with(|| {
			let collection = create_collection(accounts::alice());
			let item_id = 0u32;

			// Mint an NFT to Alice
			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

			// Burn the NFT
			let mut ext = ExtBuilder::build_ext();
			ext.setup(|_| {});
			ext.set_caller(accounts::alice());

			let call = INonfungiblesCalls::burn(burnCall { collection, item: item_id });

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Verify NFT no longer exists
			let owner = super::owner_of::<Test, Instance1>(collection, item_id);
			assert_eq!(owner, None);
		});
	}

	#[test]
	fn metadata_works() {
		ExtBuilder::default().build().execute_with(|| {
			let collection = create_collection(accounts::alice());
			let item_id = 0u32;

			// Mint an NFT to Alice
			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

			// Set metadata
			let mut ext = ExtBuilder::build_ext();
			ext.setup(|_| {});
			ext.set_caller(accounts::alice());

			let metadata = "metadata".as_bytes().to_vec();

			let call = INonfungiblesCalls::setMetadata(setMetadataCall {
				collection,
				item: Some(item_id),
				data: metadata.clone(),
			});

			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
			assert_ok!(result);

			// Get metadata
			let call =
				INonfungiblesCalls::itemMetadata(itemMetadataCall { collection, item: item_id });

			let result = call_precompile(&mut ext, TheFungibles::address(), &call).unwrap();

			// Verify metadata
			let decoded = itemMetadataCall::abi_decode_returns(&result).unwrap();
			assert_eq!(decoded.0, "metadata");
		});
	}

	// Helper functions
	fn create_collection(owner: AccountIdOf<Test>) -> u32 {
		let collection_id = super::next_collection_id::<Test, Instance1>().unwrap_or_default();
		let config = default_collection_config();
		assert_ok!(Nfts::<Test, Instance1>::create(Origin::signed(owner), owner, config,));
		collection_id
	}

	fn mint_nft(collection_id: u32, item_id: u32, owner: AccountIdOf<Test>) -> DispatchResult {
		Nfts::<Test, Instance1>::mint(Origin::signed(owner), collection_id, item_id, owner, None)
	}

	fn default_collection_config() -> CollectionConfigFor<Test, Instance1> {
		CollectionConfigFor::<Test, Instance1> {
			settings: Default::default(),
			max_supply: None,
			mint_settings: Default::default(),
		}
	}

	fn call_precompile(
		ext: &mut impl ExtWithInfo<T = Test>,
		address: [u8; 20],
		input: &INonfungiblesCalls,
	) -> Result<Output, Error> {
		ext.call_precompile(address, 0.into(), input, false)
	}
}
