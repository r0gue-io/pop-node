pub(crate) use INonfungibles::*;

use super::*;

sol!("src/nonfungibles/precompiles/interfaces/v0/INonfungibles.sol");

/// The nonfungibles precompile offers a streamlined interface for interacting with nonfungible
/// tokens. The goal is to provide a simplified, consistent API that adheres to standards in the
/// smart contract space.
pub struct Nonfungibles<const FIXED: u16, T, I = ()>(PhantomData<(T, I)>);
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
		_address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		match input {
			INonfungiblesCalls::approve_0(approve_0Call {
				collection,
				operator,
				approved,
				deadline,
			}) => {
				// TODO: Implement real weight
				let charged = env.charge(Weight::default())?;

				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				// Successfully approves a collection.
				let deadline: Option<BlockNumberFor<T, I>> =
					if *deadline > 0 { Some((*deadline).into()) } else { None };
				approve::<T, I>(
					to_runtime_origin(env.caller()),
					collection_id,
					env.to_account_id(&(*operator.0).into()),
					None,
					*approved,
					deadline,
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

				deposit_event(
					env,
					CollectionApproval {
						operator: *operator,
						approved: *approved,
						collection: *collection,
						owner,
					},
				)?;
				Ok(approve_0Call::abi_encode_returns(&approve_0Return {}))
			},
			INonfungiblesCalls::approve_1(approve_1Call {
				collection,
				item,
				operator,
				approved,
				deadline,
			}) => {
				// TODO: Implement real weight
				let charged = env.charge(Weight::default())?;

				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				let item_id: ItemIdOf<T, I> = (*item).into();

				// Successfully approves.
				let deadline: Option<BlockNumberFor<T, I>> =
					if *deadline > 0 { Some((*deadline).into()) } else { None };
				super::approve::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					env.to_account_id(&(*operator.0).into()),
					Some(item_id),
					*approved,
					deadline,
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

				deposit_event(
					env,
					ItemApproval { operator: *operator, approved: *approved, item: *item, owner },
				)?;
				Ok(approve_1Call::abi_encode_returns(&approve_1Return {}))
			},
			INonfungiblesCalls::transfer(transferCall { collection, to, item }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				let owner = <AddressMapper<T>>::to_address(env.caller().account_id()?).0.into();
				// Successfully transfers an item.
				super::transfer::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					env.to_account_id(&(*to.0).into()),
					(*item).into(),
				)?;

				deposit_event(env, Transfer { from: owner, to: *to, item: *item })?;
				Ok(transferCall::abi_encode_returns(&transferReturn {}))
			},
			INonfungiblesCalls::create(createCall { admin, config }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				let collection_id: u32 =
					super::next_collection_id::<T, I>().unwrap_or_default().into();
				super::create::<T, I>(
					to_runtime_origin(env.caller()),
					env.to_account_id(&(*admin.0).into()),
					decode_bytes::<CollectionConfigFor<T, I>>(config)?,
				)?;

				Ok(createCall::abi_encode_returns(&collection_id))
			},
			INonfungiblesCalls::destroy(destroyCall { collection, witness }) => {
				// TODO: Implement real weight
				let charged = env.charge(Weight::default())?;

				super::destroy::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					decode_bytes(witness)?,
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

				Ok(destroyCall::abi_encode_returns(&destroyReturn {}))
			},
			INonfungiblesCalls::setAttribute_0(setAttribute_0Call {
				collection,
				namespace,
				key,
				value,
			}) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				set_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					None,
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
					BoundedVec::truncate_from(value.to_vec()),
				)?;

				deposit_event(
					env,
					CollectionAttributeSet {
						key: key.clone(),
						data: value.clone(),
						collection: *collection,
					},
				)?;
				Ok(setAttribute_0Call::abi_encode_returns(&setAttribute_0Return {}))
			},
			INonfungiblesCalls::setAttribute_1(setAttribute_1Call {
				collection,
				item,
				namespace,
				key,
				value,
			}) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				set_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					Some((*item).into()),
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
					BoundedVec::truncate_from(value.to_vec()),
				)?;

				deposit_event(
					env,
					ItemAttributeSet { key: key.clone(), data: value.clone(), item: *item },
				)?;
				Ok(setAttribute_1Call::abi_encode_returns(&setAttribute_1Return {}))
			},
			INonfungiblesCalls::clearAttribute_0(clearAttribute_0Call {
				collection,
				namespace,
				key,
			}) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				super::clear_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					None,
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
				)?;

				Ok(clearAttribute_0Call::abi_encode_returns(&clearAttribute_0Return {}))
			},
			INonfungiblesCalls::clearAttribute_1(clearAttribute_1Call {
				collection,
				item,
				namespace,
				key,
			}) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				clear_attribute::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					Some((*item).into()),
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
				)?;

				Ok(clearAttribute_1Call::abi_encode_returns(&clearAttribute_1Return {}))
			},
			INonfungiblesCalls::setMetadata_0(setMetadata_0Call { collection, item, data }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				set_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*item).into(),
					BoundedVec::truncate_from(data.to_vec()),
				)?;

				Ok(setMetadata_0Call::abi_encode_returns(&setMetadata_0Return {}))
			},
			INonfungiblesCalls::setMetadata_1(setMetadata_1Call { collection, data }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				set_collection_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					BoundedVec::truncate_from(data.to_vec()),
				)?;

				Ok(setMetadata_1Call::abi_encode_returns(&setMetadata_1Return {}))
			},
			INonfungiblesCalls::clearMetadata_0(clearMetadata_0Call { collection }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				clear_collection_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
				)?;

				Ok(clearMetadata_0Call::abi_encode_returns(&clearMetadata_0Return {}))
			},
			INonfungiblesCalls::clearMetadata_1(clearMetadata_1Call { collection, item }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				clear_metadata::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*item).into(),
				)?;

				Ok(clearMetadata_1Call::abi_encode_returns(&clearMetadata_1Return {}))
			},
			INonfungiblesCalls::setMaxSupply(setMaxSupplyCall { collection, maxSupply }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				super::set_max_supply::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*maxSupply).into(),
				)?;

				Ok(setMaxSupplyCall::abi_encode_returns(&setMaxSupplyReturn {}))
			},
			INonfungiblesCalls::approveItemAttributes(approveItemAttributesCall {
				collection,
				item,
				delegate,
			}) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				super::approve_item_attributes::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*item).into(),
					env.to_account_id(&(*delegate.0).into()),
				)?;

				Ok(approveItemAttributesCall::abi_encode_returns(&approveItemAttributesReturn {}))
			},
			INonfungiblesCalls::cancelItemAttributesApproval(
				cancelItemAttributesApprovalCall { collection, item, delegate, witness },
			) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				super::cancel_item_attributes_approval::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*item).into(),
					env.to_account_id(&(*delegate.0).into()),
					decode_bytes(witness)?,
				)?;

				Ok(cancelItemAttributesApprovalCall::abi_encode_returns(
					&cancelItemAttributesApprovalReturn {},
				))
			},
			INonfungiblesCalls::clearAllApprovals(clearAllApprovalsCall { collection, item }) => {
				// TODO: Implement real weight
				env.charge(Weight::default())?;

				super::clear_all_transfer_approvals::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*item).into(),
				)?;

				Ok(clearAllApprovalsCall::abi_encode_returns(&clearAllApprovalsReturn {}))
			},
			INonfungiblesCalls::clearCollectionApprovals(clearCollectionApprovalsCall {
				collection,
				limit,
			}) => {
				// TODO: Implement real weight
				let charged = env.charge(Weight::default())?;

				match super::clear_collection_approvals::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					*limit,
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
				Ok(clearCollectionApprovalsCall::abi_encode_returns(
					&clearCollectionApprovalsReturn {},
				))
			},
			INonfungiblesCalls::mint(mintCall { collection, to, item, witness }) => {
				env.charge(Weight::default())?;

				super::mint::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					env.to_account_id(&(*to.0).into()),
					(*item).into(),
					Some(decode_bytes::<MintWitness<ItemIdOf<T, I>, DepositBalanceOf<T, I>>>(
						&witness,
					)?),
				)?;

				Ok(mintCall::abi_encode_returns(&mintReturn {}))
			},
			INonfungiblesCalls::burn(burnCall { collection, item }) => {
				env.charge(Weight::default())?;

				super::burn::<T, I>(
					to_runtime_origin(env.caller()),
					(*collection).into(),
					(*item).into(),
				)?;

				Ok(burnCall::abi_encode_returns(&burnReturn {}))
			},
			INonfungiblesCalls::balanceOf(balanceOfCall { collection, owner }) => {
				env.charge(Weight::default())?;

				let balance = super::balance_of::<T, I>(
					(*collection).into(),
					env.to_account_id(&(*owner.0).into()),
				);
				Ok(balanceOfCall::abi_encode_returns(&balance))
			},
			INonfungiblesCalls::ownerOf(ownerOfCall { collection, item }) => {
				env.charge(Weight::default())?;

				let owner = match super::owner_of::<T, I>((*collection).into(), (*item).into()) {
					Some(owner) => owner,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No owner found for item".to_string(),
						})),
				};
				let owner = <AddressMapper<T>>::to_address(&owner).0.into();
				Ok(ownerOfCall::abi_encode_returns(&owner))
			},
			INonfungiblesCalls::allowance_0(allowance_0Call { collection, owner, operator }) => {
				env.charge(Weight::default())?;

				let is_approved = crate::nonfungibles::allowance::<T, I>(
					(*collection).into(),
					env.to_account_id(&(*owner.0).into()),
					env.to_account_id(&(*operator.0).into()),
					None,
				);
				Ok(allowance_0Call::abi_encode_returns(&is_approved))
			},
			INonfungiblesCalls::allowance_1(allowance_1Call {
				collection,
				owner,
				operator,
				item,
			}) => {
				env.charge(Weight::default())?;

				let is_approved = crate::nonfungibles::allowance::<T, I>(
					(*collection).into(),
					env.to_account_id(&(*owner.0).into()),
					env.to_account_id(&(*operator.0).into()),
					Some((*item).into()),
				);
				Ok(allowance_1Call::abi_encode_returns(&is_approved))
			},
			INonfungiblesCalls::totalSupply(totalSupplyCall { collection }) => {
				env.charge(Weight::default())?;

				let total = super::total_supply::<T, I>((*collection).into());
				Ok(totalSupplyCall::abi_encode_returns(&total))
			},
			INonfungiblesCalls::itemMetadata(itemMetadataCall { collection, item }) => {
				env.charge(Weight::default())?;

				let collection_id: CollectionIdOf<T, I> = (*collection).into();
				let item_id: ItemIdOf<T, I> = (*item).into();
				let metadata = match super::item_metadata::<T, I>(collection_id, item_id) {
					Some(metadata) => metadata,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No metadata found for item".to_string(),
						})),
				};
				let item_metadata = String::from_utf8_lossy(&metadata).into();
				Ok(itemMetadataCall::abi_encode_returns(&item_metadata))
			},
			INonfungiblesCalls::getAttribute_0(getAttribute_0Call {
				collection,
				namespace,
				key,
			}) => {
				env.charge(Weight::default())?;

				let attribute = match super::get_attribute::<T, I>(
					(*collection).into(),
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
				let result = String::from_utf8_lossy(&attribute).into();
				Ok(getAttribute_0Call::abi_encode_returns(&result))
			},
			INonfungiblesCalls::getAttribute_1(getAttribute_1Call {
				collection,
				item,
				namespace,
				key,
			}) => {
				env.charge(Weight::default())?;

				let attribute = match super::get_attribute::<T, I>(
					(*collection).into(),
					Some((*item).into()),
					decode_bytes::<AttributeNamespace<AccountIdOf<T>>>(namespace)?,
					BoundedVec::truncate_from(key.to_vec()),
				) {
					Some(value) => value,
					None =>
						return Err(Error::Revert(Revert {
							reason: "Nonfungibles: No attribute found".to_string(),
						})),
				};
				let result = String::from_utf8_lossy(&attribute).into();
				Ok(getAttribute_1Call::abi_encode_returns(&result))
			},
		}
	}
}

impl<const FIXED: u16, T: pallet_nfts::Config<I>, I: 'static> Nonfungibles<FIXED, T, I> {
	/// The address of the precompile.
	pub const fn address() -> [u8; 20] {
		fixed_address(FIXED)
	}
}

// #[cfg(test)]
// mod tests {
// 	use frame_support::{assert_ok, traits::ConstU32};
// 	use mock::{ExtBuilder, RuntimeEvent as Event, Test, TestCall};
// 	use pallet_nfts::{Instance1, Pallet as Nfts};
// 	use pallet_revive::precompiles::{ExtWithInfo, ExtWithInfoExt, Output};

// 	use super::*;
// 	use crate::tests::{accounts, mock};
// 	type TheFungibles = Nonfungibles<1, Test, Instance1>;

// 	#[test]
// 	fn approve_transfer_works() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			let collection = create_collection(accounts::alice());
// 			let item_id = 0u32;

// 			// Mint an NFT to Alice
// 			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

// 			// Approve Bob to transfer Alice's NFT
// 			let mut ext = ExtBuilder::build_ext();
// 			ext.setup(|_| {});
// 			ext.set_caller(accounts::alice());

// 			let call = INonfungiblesCalls::approveTransfer(approveTransferCall {
// 				collection,
// 				operator: Address::from(accounts::bob()),
// 				item: item_id,
// 				approved: true,
// 				deadline: 0,
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Verify Bob is approved to transfer Alice's NFT
// 			let approved = super::allowance::<Test, Instance1>(
// 				collection,
// 				accounts::alice(),
// 				accounts::bob(),
// 				Some(item_id),
// 			);
// 			assert!(approved);
// 		});
// 	}

// 	#[test]
// 	fn transfer_works() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			let collection = create_collection(accounts::alice());
// 			let item_id = 0u32;

// 			// Mint an NFT to Alice
// 			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

// 			// Transfer from Alice to Bob
// 			let mut ext = ExtBuilder::build_ext();
// 			ext.setup(|_| {});
// 			ext.set_caller(accounts::alice());

// 			let call = INonfungiblesCalls::transfer(transferCall {
// 				collection,
// 				to: Address::from(accounts::bob()),
// 				item: item_id,
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Verify Bob is now the owner
// 			let owner = super::owner_of::<Test, Instance1>(collection, item_id);
// 			assert_eq!(owner, Some(accounts::bob()));
// 		});
// 	}

// 	#[test]
// 	fn create_and_mint_works() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			// Create a collection
// 			let mut ext = ExtBuilder::build_ext();
// 			ext.setup(|_| {});
// 			ext.set_caller(accounts::alice());

// 			let config =
// 				super::decode_bytes::<CollectionConfigFor<Test, Instance1>>(&abi::encode(&[
// 					alloy::sol_types::SolValue::Bytes(Vec::new()),
// 				]))
// 				.unwrap();

// 			let call = INonfungiblesCalls::create(createCall {
// 				admin: Address::from(accounts::alice()),
// 				config: abi::encode(&[alloy::sol_types::SolValue::Bytes(Vec::new())]),
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Extract collection ID from the result
// 			let collection_id = 0u32; // First collection created

// 			// Mint an NFT
// 			let item_id = 0u32;
// 			let call = INonfungiblesCalls::mint(mintCall {
// 				collection: collection_id,
// 				item: item_id,
// 				owner: Address::from(accounts::bob()),
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Verify Bob is now the owner
// 			let owner = super::owner_of::<Test, Instance1>(collection_id, item_id);
// 			assert_eq!(owner, Some(accounts::bob()));
// 		});
// 	}

// 	#[test]
// 	fn set_and_get_attributes_works() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			let collection = create_collection(accounts::alice());
// 			let item_id = 0u32;

// 			// Mint an NFT to Alice
// 			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

// 			// Set item attribute
// 			let mut ext = ExtBuilder::build_ext();
// 			ext.setup(|_| {});
// 			ext.set_caller(accounts::alice());

// 			let key = "key".as_bytes().to_vec();
// 			let value = "value".as_bytes().to_vec();

// 			let namespace = AttributeNamespace::Pallet;
// 			let encoded_namespace = abi::encode(&[alloy::sol_types::SolValue::Uint(0u32.into())]);

// 			let call = INonfungiblesCalls::setItemAttribute(setItemAttributeCall {
// 				collection,
// 				item: item_id,
// 				namespace: encoded_namespace.clone(),
// 				key: key.clone(),
// 				value: value.clone(),
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Get item attribute
// 			let call = INonfungiblesCalls::getItemAttributes(getItemAttributesCall {
// 				collection,
// 				item: item_id,
// 				namespace: encoded_namespace,
// 				key,
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call).unwrap();

// 			// Verify attribute value
// 			let decoded = getItemAttributesCall::abi_decode_returns(&result).unwrap();
// 			assert_eq!(decoded.0, "value");
// 		});
// 	}

// 	#[test]
// 	fn burn_works() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			let collection = create_collection(accounts::alice());
// 			let item_id = 0u32;

// 			// Mint an NFT to Alice
// 			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

// 			// Burn the NFT
// 			let mut ext = ExtBuilder::build_ext();
// 			ext.setup(|_| {});
// 			ext.set_caller(accounts::alice());

// 			let call = INonfungiblesCalls::burn(burnCall { collection, item: item_id });

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Verify NFT no longer exists
// 			let owner = super::owner_of::<Test, Instance1>(collection, item_id);
// 			assert_eq!(owner, None);
// 		});
// 	}

// 	#[test]
// 	fn metadata_works() {
// 		ExtBuilder::default().build().execute_with(|| {
// 			let collection = create_collection(accounts::alice());
// 			let item_id = 0u32;

// 			// Mint an NFT to Alice
// 			assert_ok!(mint_nft(collection, item_id, accounts::alice()));

// 			// Set metadata
// 			let mut ext = ExtBuilder::build_ext();
// 			ext.setup(|_| {});
// 			ext.set_caller(accounts::alice());

// 			let metadata = "metadata".as_bytes().to_vec();

// 			let call = INonfungiblesCalls::setMetadata(setMetadataCall {
// 				collection,
// 				item: Some(item_id),
// 				data: metadata.clone(),
// 			});

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call);
// 			assert_ok!(result);

// 			// Get metadata
// 			let call =
// 				INonfungiblesCalls::itemMetadata(itemMetadataCall { collection, item: item_id });

// 			let result = call_precompile(&mut ext, TheFungibles::address(), &call).unwrap();

// 			// Verify metadata
// 			let decoded = itemMetadataCall::abi_decode_returns(&result).unwrap();
// 			assert_eq!(decoded.0, "metadata");
// 		});
// 	}

// 	// Helper functions
// 	fn create_collection(owner: AccountIdOf<Test>) -> u32 {
// 		let collection_id = super::next_collection_id::<Test, Instance1>().unwrap_or_default();
// 		let config = default_collection_config();
// 		assert_ok!(Nfts::<Test, Instance1>::create(Origin::signed(owner), owner, config,));
// 		collection_id
// 	}

// 	fn mint_nft(collection_id: u32, item_id: u32, owner: AccountIdOf<Test>) -> DispatchResult {
// 		Nfts::<Test, Instance1>::mint(Origin::signed(owner), collection_id, item_id, owner, None)
// 	}

// 	fn default_collection_config() -> CollectionConfigFor<Test, Instance1> {
// 		CollectionConfigFor::<Test, Instance1> {
// 			settings: Default::default(),
// 			max_supply: None,
// 			mint_settings: Default::default(),
// 		}
// 	}

// 	fn call_precompile(
// 		ext: &mut impl ExtWithInfo<T = Test>,
// 		address: [u8; 20],
// 		input: &INonfungiblesCalls,
// 	) -> Result<Output, Error> {
// 		ext.call_precompile(address, 0.into(), input, false)
// 	}
// }
