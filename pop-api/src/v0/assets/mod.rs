use ink::{scale::{Compact, Decode}, env::chain_extension::ChainExtensionMethod, prelude::vec::Vec};

use crate::primitives::{AssetId, MultiAddress};
use crate::{AccountId, Balance, Result, StatusCode};

pub mod fungibles;

// Parameters to extrinsics representing an asset id (`AssetIdParameter`) and a balance amount (`Balance`) are expected
// to be compact encoded. The pop api handles that for the developer.
//
// reference: https://substrate.stackexchange.com/questions/1873/what-is-the-meaning-of-palletcompact-in-pallet-development
//
// Asset id that is compact encoded.
type AssetIdParameter = Compact<AssetId>;
// Balance amount that is compact encoded.
type BalanceParameter = Compact<Balance>;

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
/// - set_metadata
/// - clear_metadata
/// - approve_transfer
/// - cancel_approval
/// - transfer_approved

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

/// Move some assets from the sender account to another.
#[inline]
pub(crate) fn transfer(
	id: AssetId,
	target: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	ChainExtensionMethod::build(0)
		.input::<(u8, u8, AssetIdParameter, MultiAddress<AccountId, ()>, BalanceParameter)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 9, Compact(id), target.into(), Compact(amount)))
}

// /// Move some assets from the sender account to another, keeping the sender account alive.
// pub(crate) fn transfer_keep_alive(
// 	id: AssetId,
// 	target: impl Into<MultiAddress<AccountId, ()>>,
// 	amount: Balance,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::TransferKeepAlive {
// 		id: id.into(),
// 		target: target.into(),
// 		amount: Compact(amount),
// 	}))
// }

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
pub(crate) fn approve_transfer(
	id: AssetId,
	delegate: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	ChainExtensionMethod::build(0)
		.input::<(u8, u8, AssetIdParameter, MultiAddress<AccountId, ()>, BalanceParameter)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 22, Compact(id), delegate.into(), Compact(amount)))
}

/// Cancel all of some asset approved for delegated transfer by a third-party account.
#[inline]
pub(crate) fn cancel_approval(
	id: AssetId,
	delegate: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	ChainExtensionMethod::build(0)
		.input::<(u8, u8, AssetIdParameter, MultiAddress<AccountId, ()>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 23, Compact(id), delegate.into()))
}

/// Transfer some asset balance from a previously delegated account to some third-party
/// account.
#[inline]
pub(crate) fn transfer_approved(
	id: AssetId,
	from: impl Into<MultiAddress<AccountId, ()>>,
	to: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	ChainExtensionMethod::build(0)
		.input::<(
			u8,
			u8,
			AssetIdParameter,
			MultiAddress<AccountId, ()>,
			MultiAddress<AccountId, ()>,
			BalanceParameter,
		)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 25, Compact(id), from.into(), to.into(), Compact(amount)))
}

/// 2. Read state functions
/// - total_supply
/// - balance_of
/// - allowance
/// - asset_exists
/// - token_name
/// - token_symbol
/// - token_decimals

#[inline]
pub(crate) fn total_supply(id: AssetId) -> Result<Balance> {
	ChainExtensionMethod::build(1)
		.input::<(u8, u8, AssetId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 2, id))
		.and_then(|v| Balance::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}

#[inline]
pub(crate) fn balance_of(id: AssetId, owner: AccountId) -> Result<Balance> {
	ChainExtensionMethod::build(1)
		.input::<(u8, u8, AssetId, AccountId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 1, id, owner))
		.and_then(|v| Balance::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}

#[inline]
pub(crate) fn allowance(id: AssetId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	ChainExtensionMethod::build(1)
		.input::<(u8, u8, AssetId, AccountId, AccountId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 0, id, owner, spender))
		.and_then(|v| Balance::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}

#[inline]
pub(crate) fn token_name(id: AssetId) -> Result<Vec<u8>> {
	ChainExtensionMethod::build(1)
		.input::<(u8, u8, AssetId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 5, id))
		.and_then(|v| <Vec<u8>>::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}
//
#[inline]
pub(crate) fn token_symbol(id: AssetId) -> Result<Vec<u8>> {
	ChainExtensionMethod::build(1)
		.input::<(u8, u8, AssetId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 4, id))
		.and_then(|v| <Vec<u8>>::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}

#[inline]
pub(crate) fn token_decimals(id: AssetId) -> Result<u8> {
	ChainExtensionMethod::build(1)
		.input::<(u8, u8, AssetId)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(52, 3, id))
		.and_then(|v| u8::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}

// pub(crate) fn asset_exists(id: AssetId) -> Result<bool> {
// 	state::read(RuntimeStateKeys::Assets(AssetsKeys::AssetExists(id)))
// }
