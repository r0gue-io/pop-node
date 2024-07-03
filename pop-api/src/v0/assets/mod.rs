use ink::{prelude::vec::Vec, scale::Compact};

use crate::{state::read, Balance, RuntimeCall, *};
use primitives::{AssetId, MultiAddress};

pub mod fungibles;

type Result<T> = core::result::Result<T, StatusCode>;

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
/// - force_transfer
/// - freeze
/// - thaw
/// - freeze_asset
/// - thaw_asset
/// - transfer_ownership
/// - set_team
/// - set_metadata
/// - clear_metadata
/// - approve_transfer
/// - cancel_approval
/// - force_cancel_approval
/// - transfer_approved
/// - touch
/// - refund
/// - set_min_balance
/// - touch_other
/// - refund_other
/// - block

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
	dispatch(RuntimeCall::Assets(AssetsCall::TransferKeepAlive {
		id: id.into(),
		target: target.into(),
		amount: Compact(amount),
	}))
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
//
// /// Move some assets from one account to another. Sender should be the Admin of the asset `id`.
// pub(crate) fn force_transfer(
// 	id: AssetId,
// 	source: impl Into<MultiAddress<AccountId, ()>>,
// 	dest: impl Into<MultiAddress<AccountId, ()>>,
// 	amount: Balance,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::ForceTransfer {
// 		id: id.into(),
// 		source: source.into(),
// 		dest: dest.into(),
// 		amount: Compact(amount),
// 	}))
// }
//
// /// Disallow further unprivileged transfers of an asset `id` from an account `who`. `who`
// /// must already exist as an entry in `Account`s of the asset. If you want to freeze an
// /// account that does not have an entry, use `touch_other` first.
// pub(crate) fn freeze(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Freeze { id: id.into(), who: who.into() }))
// }
//
// /// Allow unprivileged transfers to and from an account again.
// pub(crate) fn thaw(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Thaw { id: id.into(), who: who.into() }))
// }
//
// /// Disallow further unprivileged transfers for the asset class.
// pub(crate) fn freeze_asset(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::FreezeAsset { id: id.into() }))
// }
//
// /// Allow unprivileged transfers for the asset again.
// pub(crate) fn thaw_asset(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::ThawAsset { id: id.into() }))
// }
//
// /// Change the Owner of an asset.
// pub(crate) fn transfer_ownership(id: AssetId, owner: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::TransferOwnership {
// 		id: id.into(),
// 		owner: owner.into(),
// 	}))
// }
//
// /// Change the Issuer, Admin and Freezer of an asset.
// pub(crate) fn set_team(
// 	id: AssetId,
// 	issuer: impl Into<MultiAddress<AccountId, ()>>,
// 	admin: impl Into<MultiAddress<AccountId, ()>>,
// 	freezer: impl Into<MultiAddress<AccountId, ()>>,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::SetTeam {
// 		id: id.into(),
// 		issuer: issuer.into(),
// 		admin: admin.into(),
// 		freezer: freezer.into(),
// 	}))
// }

/// Set the metadata for an asset.
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
	dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
		id: id.into(),
		delegate: delegate.into(),
		amount: Compact(amount),
	}))
}

/// Cancel all of some asset approved for delegated transfer by a third-party account.
#[inline]
pub(crate) fn cancel_approval(
	id: AssetId,
	delegate: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	dispatch(RuntimeCall::Assets(AssetsCall::CancelApproval {
		id: id.into(),
		delegate: delegate.into(),
	}))
}

// /// Cancel all of some asset approved for delegated transfer by a third-party account.
// pub(crate) fn force_cancel_approval(
// 	id: AssetId,
// 	owner: impl Into<MultiAddress<AccountId, ()>>,
// 	delegate: impl Into<MultiAddress<AccountId, ()>>,
// ) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::ForceCancelApproval {
// 		id: id.into(),
// 		owner: owner.into(),
// 		delegate: delegate.into(),
// 	}))
// }

/// Transfer some asset balance from a previously delegated account to some third-party
/// account.
#[inline]
pub(crate) fn transfer_approved(
	id: AssetId,
	owner: impl Into<MultiAddress<AccountId, ()>>,
	destination: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	dispatch(RuntimeCall::Assets(AssetsCall::TransferApproved {
		id: id.into(),
		owner: owner.into(),
		destination: destination.into(),
		amount: Compact(amount),
	}))
}
//
// /// Create an asset account for non-provider assets.
// pub(crate) fn touch(id: AssetId) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Touch { id: id.into() }))
// }
//
// /// Return the deposit (if any) of an asset account or a consumer reference (if any) of an
// /// account.
// pub(crate) fn refund(id: AssetId, allow_burn: bool) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Refund { id: id.into(), allow_burn }))
// }
//
// /// Sets the minimum balance of an asset.
// pub(crate) fn set_min_balance(id: AssetId, min_balance: Balance) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::SetMinBalance {
// 		id: id.into(),
// 		min_balance: Compact(min_balance),
// 	}))
// }
//
// /// Create an asset account for `who`.
// pub(crate) fn touch_other(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::TouchOther { id: id.into(), who: who.into() }))
// }
//
// /// Return the deposit (if any) of a target asset account. Useful if you are the depositor.
// pub(crate) fn refund_other(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::RefundOther { id: id.into(), who: who.into() }))
// }
//
// /// Disallow further unprivileged transfers of an asset `id` to and from an account `who`.
// pub(crate) fn block(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
// 	dispatch(RuntimeCall::Assets(AssetsCall::Block { id: id.into(), who: who.into() }))
// }

/// 2. Read state functions
/// - total_supply
/// - balance_of
/// - allowance
/// - asset_exists
/// - token_name
/// - token_symbol
/// - token_decimals
//
#[inline]
pub(crate) fn total_supply(id: AssetId) -> Result<Balance> {
	read(RuntimeStateKeys::Assets(AssetsKeys::TotalSupply(id)))
}

#[inline]
pub(crate) fn balance_of(id: AssetId, owner: AccountId) -> Result<Balance> {
	read(RuntimeStateKeys::Assets(AssetsKeys::BalanceOf(id, owner)))
}

#[inline]
pub(crate) fn allowance(id: AssetId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	read(RuntimeStateKeys::Assets(AssetsKeys::Allowance(id, owner, spender)))
}
// pub(crate) fn asset_exists(id: AssetId) -> Result<bool> {
// 	state::read(RuntimeStateKeys::Assets(AssetsKeys::AssetExists(id)))
// }

#[inline]
pub(crate) fn token_name(id: AssetId) -> Result<Vec<u8>> {
	read(RuntimeStateKeys::Assets(AssetsKeys::TokenName(id)))
}
//
#[inline]
pub(crate) fn token_symbol(id: AssetId) -> Result<Vec<u8>> {
	read(RuntimeStateKeys::Assets(AssetsKeys::TokenSymbol(id)))
}

#[inline]
pub(crate) fn token_decimals(id: AssetId) -> Result<u8> {
	read(RuntimeStateKeys::Assets(AssetsKeys::TokenDecimals(id)))
}

// Parameters to extrinsics representing an asset id (`AssetIdParameter`) and a balance amount (`Balance`) are expected
// to be compact encoded. The pop api handles that for the developer.
//
// reference: https://substrate.stackexchange.com/questions/1873/what-is-the-meaning-of-palletcompact-in-pallet-development
//
// Asset id that is compact encoded.
type AssetIdParameter = Compact<AssetId>;
// Balance amount that is compact encoded.
type BalanceParameter = Compact<Balance>;
//
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub(crate) enum AssetsCall {
	// 	#[codec(index = 0)]
	// 	Create { id: AssetIdParameter, admin: MultiAddress<AccountId, ()>, min_balance: Balance },
	// #[codec(index = 2)]
	// StartDestroy { id: AssetIdParameter },
	// #[codec(index = 3)]
	// DestroyAccounts { id: AssetIdParameter },
	// #[codec(index = 4)]
	// DestroyApprovals { id: AssetIdParameter },
	// #[codec(index = 5)]
	// FinishDestroy { id: AssetIdParameter },
	// #[codec(index = 6)]
	// Mint {
	// 	id: AssetIdParameter,
	// 	beneficiary: MultiAddress<AccountId, ()>,
	// 	amount: BalanceParameter,
	// },
	// #[codec(index = 7)]
	// Burn { id: AssetIdParameter, who: MultiAddress<AccountId, ()>, amount: BalanceParameter },
	// #[codec(index = 8)]
	// Transfer { id: AssetIdParameter, target: MultiAddress<AccountId, ()>, amount: BalanceParameter },
	#[codec(index = 9)]
	TransferKeepAlive {
		id: AssetIdParameter,
		target: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	// #[codec(index = 10)]
	// ForceTransfer {
	// 	id: AssetIdParameter,
	// 	source: MultiAddress<AccountId, ()>,
	// 	dest: MultiAddress<AccountId, ()>,
	// 	amount: BalanceParameter,
	// },
	// #[codec(index = 11)]
	// Freeze { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	// #[codec(index = 12)]
	// Thaw { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	// #[codec(index = 13)]
	// FreezeAsset { id: AssetIdParameter },
	// #[codec(index = 14)]
	// ThawAsset { id: AssetIdParameter },
	// #[codec(index = 15)]
	// TransferOwnership { id: AssetIdParameter, owner: MultiAddress<AccountId, ()> },
	// #[codec(index = 16)]
	// SetTeam {
	// 	id: AssetIdParameter,
	// 	issuer: MultiAddress<AccountId, ()>,
	// 	admin: MultiAddress<AccountId, ()>,
	// 	freezer: MultiAddress<AccountId, ()>,
	// },
	// #[codec(index = 17)]
	// SetMetadata { id: AssetIdParameter, name: Vec<u8>, symbol: Vec<u8>, decimals: u8 },
	// #[codec(index = 18)]
	// ClearMetadata { id: AssetIdParameter },
	#[codec(index = 22)]
	ApproveTransfer {
		id: AssetIdParameter,
		delegate: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	#[codec(index = 23)]
	CancelApproval { id: AssetIdParameter, delegate: MultiAddress<AccountId, ()> },
	// #[codec(index = 24)]
	// ForceCancelApproval {
	// 	id: AssetIdParameter,
	// 	owner: MultiAddress<AccountId, ()>,
	// 	delegate: MultiAddress<AccountId, ()>,
	// },
	#[codec(index = 25)]
	TransferApproved {
		id: AssetIdParameter,
		owner: MultiAddress<AccountId, ()>,
		destination: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	// 	// #[codec(index = 26)]
	// 	// Touch { id: AssetIdParameter },
	// 	// #[codec(index = 27)]
	// 	// Refund { id: AssetIdParameter, allow_burn: bool },
	// 	// #[codec(index = 28)]
	// 	// SetMinBalance { id: AssetIdParameter, min_balance: BalanceParameter },
	// 	// #[codec(index = 29)]
	// 	// TouchOther { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	// 	// #[codec(index = 30)]
	// 	// RefundOther { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	// 	// #[codec(index = 31)]
	// 	// Block { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	// }

	// // TODO: Not being used atm but necessary if we want to provide access to the
	// //  rest of the pallet, outside of the use cases.
	// #[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, scale::Decode)]
	// #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	// pub enum AssetsError {
	// 	/// Account balance must be greater than or equal to the transfer amount.
	// 	BalanceLow,
	// 	/// The account to alter does not exist.
	// 	NoAccount,
	// 	/// The signing account has no permission to do the operation.
	// 	NoPermission,
	// 	/// The given asset ID is unknown.
	// 	Unknown,
	// 	/// The origin account is frozen.
	// 	Frozen,
	// 	/// The asset ID is already taken.
	// 	InUse,
	// 	/// Invalid witness data given.
	// 	BadWitness,
	// 	/// Minimum balance should be non-zero.
	// 	MinBalanceZero,
	// 	/// Unable to increment the consumer reference counters on the account. Either no provider
	// 	/// reference exists to allow a non-zero balance of a non-self-sufficient asset, or one
	// 	/// fewer then the maximum number of consumers has been reached.
	// 	UnavailableConsumer,
	// 	/// Invalid metadata given.
	// 	BadMetadata,
	// 	/// No approval exists that would allow the transfer.
	// 	Unapproved,
	// 	/// The source account would not survive the transfer and it needs to stay alive.
	// 	WouldDie,
	// 	/// The asset-account already exists.
	// 	AlreadyExists,
	// 	/// The asset-account doesn't have an associated deposit.
	// 	NoDeposit,
	// 	/// The operation would result in funds being burned.
	// 	WouldBurn,
	// 	/// The asset is a live asset and is actively being used. Usually emit for operations such
	// 	/// as `start_destroy` which require the asset to be in a destroying state.
	// 	LiveAsset,
	// 	/// The asset is not live, and likely being destroyed.
	// 	AssetNotLive,
	// 	/// The asset status is not the expected status.
	// 	IncorrectStatus,
	// 	/// The asset should be frozen before the given operation.
	// 	NotFrozen,
	// 	/// Callback action resulted in error.
	// 	CallbackFailed,
}

//
// impl TryFrom<u32> for AssetsError {
// 	type Error = Error;
//
// 	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
// 		use AssetsError::*;
// 		match status_code {
// 			0 => Ok(BalanceLow),
// 			1 => Ok(NoAccount),
// 			2 => Ok(NoPermission),
// 			3 => Ok(Unknown),
// 			4 => Ok(Frozen),
// 			5 => Ok(InUse),
// 			6 => Ok(BadWitness),
// 			7 => Ok(MinBalanceZero),
// 			8 => Ok(UnavailableConsumer),
// 			9 => Ok(BadMetadata),
// 			10 => Ok(Unapproved),
// 			11 => Ok(WouldDie),
// 			12 => Ok(AlreadyExists),
// 			13 => Ok(NoDeposit),
// 			14 => Ok(WouldBurn),
// 			15 => Ok(LiveAsset),
// 			16 => Ok(AssetNotLive),
// 			17 => Ok(IncorrectStatus),
// 			18 => Ok(NotFrozen),
// 			_ => todo!(),
// 		}
// 	}
// }
