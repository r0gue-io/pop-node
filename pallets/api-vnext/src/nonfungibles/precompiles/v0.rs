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

#[cfg(test)]
mod tests {
	use frame_support::{assert_ok, sp_runtime::DispatchError, BoundedVec};
	use pallet_nfts::{
		AttributeNamespace, CollectionConfig, CollectionConfigFor, CollectionSettings, MintSettings,
	};
	use pallet_revive::{
		precompiles::alloy::sol_types::{SolInterface, SolType, SolValue},
		test_utils::{ALICE, BOB},
		DepositLimit, Weight,
	};

	use super::*;
	use crate::{
		bare_call, fixed_address,
		mock::{AccountId, ExtBuilder, RuntimeOrigin, Test, NONFUNGIBLES},
		nonfungibles::mint,
		prefixed_address, AccountIdOf,
	};

	const ADDRESS: [u8; 20] = fixed_address(NONFUNGIBLES);

	#[test]
	fn approve_0_works() {
		let collection_id: u32 = 0;
		let item_id = 0;
		let owner = ALICE;
		let operator = BOB;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000)])
			.build()
			.execute_with(|| {
				create_collection_and_mint(owner.clone(), collection_id, item_id);
				// Successfully approved.
				assert_ok!(call_precompile::<()>(
					&owner,
					collection_id,
					&INonfungiblesCalls::approve_0(INonfungibles::approve_0Call {
						collection: collection_id.into(),
						operator: to_address(&operator).0.into(),
						approved: true,
						deadline: 0
					})
				));
				assert!(allowance::<Test, ()>(
					collection_id,
					owner.clone(),
					operator.clone(),
					None
				));
				let event = INonfungibles::CollectionApproval {
					collection: collection_id.into(),
					owner: to_address(&owner).0.into(),
					operator: to_address(&operator).0.into(),
					approved: true,
				};
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn approve_1_works() {
		let item_id: u32 = 0;
		let collection_id: u32 = 0;
		let owner = ALICE;
		let operator = BOB;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000)])
			.build()
			.execute_with(|| {
				create_collection_and_mint(owner.clone(), collection_id, item_id);
				// Successfully approved.
				assert_ok!(call_precompile::<()>(
					&owner,
					collection_id,
					&INonfungiblesCalls::approve_1(INonfungibles::approve_1Call {
						collection: collection_id.into(),
						item: item_id.into(),
						operator: to_address(&operator).0.into(),
						approved: true,
						deadline: 0
					})
				));
				assert!(allowance::<Test, ()>(
					collection_id,
					owner.clone(),
					operator.clone(),
					Some(item_id)
				));
				let event = INonfungibles::ItemApproval {
					item: item_id.into(),
					owner: to_address(&owner).0.into(),
					operator: to_address(&operator).0.into(),
					approved: true,
				};
				assert_last_event(ADDRESS, event);
			});
	}

	#[test]
	fn transfer_works() {}

	#[test]
	fn create_works() {}

	#[test]
	fn destroy_works() {}

	#[test]
	fn set_attribute_0_works() {}

	#[test]
	fn set_attribute_1_works() {}

	#[test]
	fn clear_attribute_0_works() {}

	#[test]
	fn clear_attribute_1_works() {}

	#[test]
	fn set_metadata_0_works() {}

	#[test]
	fn set_metadata_1_works() {}

	#[test]
	fn clear_metadata_0_works() {}

	#[test]
	fn clear_metadata_1_works() {}

	#[test]
	fn set_max_supply_works() {}

	#[test]
	fn approve_item_attributes_works() {}

	#[test]
	fn cancel_item_attributes_works() {}

	#[test]
	fn clear_all_approvals_works() {}

	#[test]
	fn clear_collection_approvals_works() {}

	#[test]
	fn mint_works() {}

	#[test]
	fn burn_works() {}

	#[test]
	fn balance_of_works() {}

	#[test]
	fn owner_of_works() {}

	#[test]
	fn allowance_0_works() {}

	#[test]
	fn allowance_1_works() {}

	#[test]
	fn total_supply_works() {}

	#[test]
	fn item_metadata_works() {}

	#[test]
	fn get_attribute_0_works() {}

	#[test]
	fn get_attribute_1_works() {}

	fn create_collection_and_mint(owner: AccountIdOf<Test>, collection_id: u32, item_id: u32) {
		create_collection(owner.clone());
		assert_ok!(mint::<Test, ()>(
			RuntimeOrigin::signed(owner.clone()),
			collection_id,
			owner,
			item_id,
			None,
		));
	}

	fn create_collection(owner: AccountIdOf<Test>) {
		assert_ok!(super::create::<Test, ()>(
			RuntimeOrigin::signed(owner.clone()),
			owner,
			default_collection_config(),
		));
	}

	fn default_collection_config() -> CollectionConfigFor<Test> {
		CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}

	fn set_attribute(collection_id: u32, item_id: Option<u32>, key: &str, value: &str) {
		assert_ok!(crate::nonfungibles::set_attribute::<Test, ()>(
			RuntimeOrigin::signed(ALICE),
			collection_id,
			item_id,
			AttributeNamespace::CollectionOwner,
			BoundedVec::truncate_from(key.as_bytes().to_vec()),
			BoundedVec::truncate_from(value.as_bytes().to_vec()),
		));
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		origin: &AccountId,
		token: u32,
		input: &INonfungiblesCalls,
	) -> Result<Output, DispatchError> {
		let address = prefixed_address(NONFUNGIBLES, token);
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
