use ink::{prelude::vec::Vec, scale::Decode};

use crate::{
	constants::{ASSETS, DECODING_FAILED},
	primitives::{AccountId, AssetId, Balance},
	Result, StatusCode,
};

pub mod fungibles;

/// [Pallet Assets](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/assets/src/lib.rs):
/// 1. Dispatchables
/// 2. Read state functions
///
/// 1. Dispatchables within pallet assets (TrustBackedAssets instance):
/// - create
/// - start_destroy
/// - destroy_accounts
/// - destroy_approvals
/// - finish_destroy
/// - mint
/// - burn
/// - transfer
/// - transfer_keep_alive
const TRANSFER_KEEP_ALIVE: u8 = 9;
/// - set_metadata
/// - clear_metadata
/// - approve_transfer
const APPROVE_TRANSFER: u8 = 22;
/// - cancel_approval
const CANCEL_APPROVAL: u8 = 23;
/// - transfer_approved
const TRANSFER_APPROVED: u8 = 25;

/// Helper method to build a dispatch call `ChainExtensionMethod` for `ASSET` module
///
/// - `dispatchable`: The index of the dispatchable functions in `ASSET` module
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_dispatch(ASSETS, dispatchable)
}

/// Helper method to build a dispatch call `ChainExtensionMethod` for `ASSET` module
///
/// - `state_query`: The index of the runtime state query
fn build_read_state(state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_read_state(ASSETS, state_query)
}

/// Issue a new class of fungible assets from a public origin.
// pub(crate) fn create(
// 	id: AssetId,
// 	admin: impl Into<MultiAddress<AccountId, ()>>,
// 	min_balance: Balance,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Create {
// 		id: id.into(),
// 		admin: admin.into(),
// 		min_balance,
// 	}))
// }
//
// /// Start the process of destroying a fungible asset class.
// pub(crate) fn start_destroy(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::StartDestroy { id: id.into() }))
// }
//
// /// Destroy all accounts associated with a given asset.
// pub(crate) fn destroy_accounts(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::DestroyAccounts { id: id.into() }))
// }
//
// /// Destroy all approvals associated with a given asset up to the max (see runtime configuration Assets `RemoveItemsLimit`).
// pub(crate) fn destroy_approvals(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::DestroyApprovals { id: id.into() }))
// }
//
// /// Complete destroying asset and unreserve currency.
// pub(crate) fn finish_destroy(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::FinishDestroy { id: id.into() }))
// }

// /// Mint assets of a particular class.
// pub(crate) fn mint(
// 	id: AssetId,
// 	beneficiary: impl Into<MultiAddress<AccountId, ()>>,
// 	amount: Balance,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Mint {
// 		id: id.into(),
// 		beneficiary: beneficiary.into(),
// 		amount: Compact(amount),
// 	}))
// }
//
// /// Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
// pub(crate) fn burn(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>, amount: Balance) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Burn {
// 		id: id.into(),
// 		who: who.into(),
// 		amount: Compact(amount),
// 	}))
// }

/// Move some assets from the sender account to another, keeping the sender account alive.
#[inline]
pub fn transfer_keep_alive(id: AssetId, target: AccountId, amount: Balance) -> Result<()> {
	// E.D. is always respected with transferring tokens via the API.
	build_dispatch(TRANSFER_KEEP_ALIVE)
		.input::<(AssetId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, target, amount))
}

// /// Set the metadata for an asset.
// pub(crate) fn set_metadata(
// 	id: AssetId,
// 	name: Vec<u8>,
// 	symbol: Vec<u8>,
// 	decimals: u8,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::SetMetadata { id: id.into(), name, symbol, decimals }))
// }

// /// Clear the metadata for an asset.
// pub(crate) fn clear_metadata(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::ClearMetadata { id: id.into() }))
// }

/// Approve an amount of asset for transfer by a delegated third-party account.
#[inline]
pub fn approve_transfer(id: AssetId, delegate: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(APPROVE_TRANSFER)
		.input::<(AssetId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, delegate, amount))
}

/// Cancel all of some asset approved for delegated transfer by a third-party account.
#[inline]
pub fn cancel_approval(id: AssetId, delegate: AccountId) -> Result<()> {
	build_dispatch(CANCEL_APPROVAL)
		.input::<(AssetId, AccountId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, delegate))
}

/// Transfer some asset balance from a previously delegated account to some third-party
/// account.
#[inline]
pub fn transfer_approved(
	id: AssetId,
	from: AccountId,
	to: AccountId,
	amount: Balance,
) -> Result<()> {
	build_dispatch(TRANSFER_APPROVED)
		.input::<(AssetId, AccountId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, from, to, amount))
}

/// 2. Read state functions:
/// - total_supply
const TOTAL_SUPPLY: u8 = 0;
/// - balance_of
const BALANCE_OF: u8 = 1;
/// - allowance
const ALLOWANCE: u8 = 2;
/// - token_name
const TOKEN_NAME: u8 = 3;
/// - token_symbol
const TOKEN_SYMBOL: u8 = 4;
/// - token_decimals
const TOKEN_DECIMALS: u8 = 5;
/// - asset_exists

#[inline]
pub fn total_supply(id: AssetId) -> Result<Balance> {
	build_read_state(TOTAL_SUPPLY)
		.input::<AssetId>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id))
		.and_then(|v| Balance::decode(&mut &v[..]).map_err(|_e| StatusCode(DECODING_FAILED)))
}

#[inline]
pub fn balance_of(id: AssetId, owner: AccountId) -> Result<Balance> {
	build_read_state(ASSETS, BALANCE_OF)
		.input::<(AssetId, AccountId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, owner))
		.and_then(|v| Balance::decode(&mut &v[..]).map_err(|_e| StatusCode(DECODING_FAILED)))
}

#[inline]
pub fn allowance(id: AssetId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	build_read_state(ALLOWANCE)
		.input::<(AssetId, AccountId, AccountId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, owner, spender))
		.and_then(|v| Balance::decode(&mut &v[..]).map_err(|_e| StatusCode(DECODING_FAILED)))
}

#[inline]
pub fn token_name(id: AssetId) -> Result<Vec<u8>> {
	build_read_state(TOKEN_NAME)
		.input::<AssetId>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id))
		.and_then(|v| <Vec<u8>>::decode(&mut &v[..]).map_err(|_e| StatusCode(DECODING_FAILED)))
}
//
#[inline]
pub fn token_symbol(id: AssetId) -> Result<Vec<u8>> {
	build_read_state(TOKEN_SYMBOL)
		.input::<AssetId>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id))
		.and_then(|v| <Vec<u8>>::decode(&mut &v[..]).map_err(|_e| StatusCode(DECODING_FAILED)))
}

#[inline]
pub fn token_decimals(id: AssetId) -> Result<u8> {
	build_read_state(TOKEN_DECIMALS)
		.input::<AssetId>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id))
		.and_then(|v| <u8>::decode(&mut &v[..]).map_err(|_e| StatusCode(DECODING_FAILED)))
}

// pub(crate) fn asset_exists(id: AssetId) -> Result<bool> {
// 	state::read(RuntimeStateKeys::Assets(AssetsKeys::AssetExists(id)))
// }
