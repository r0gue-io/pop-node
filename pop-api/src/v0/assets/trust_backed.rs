use crate::{Balance, PopApiError::UnknownStatusCode, RuntimeCall, *};
use ink::prelude::vec::Vec;
use primitives::{AssetId, MultiAddress};
use scale::{Compact, Encode};

type Result<T> = core::result::Result<T, Error>;

/// https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/assets/src/lib.rs
/// 
/// Extrinsics within pallet assets (TrustBackedAssets Instance) that can be used via the pop api on Pop Network:
/// 1. 	create
/// 2.	start_destroy
/// 3.	destroy_accounts
/// 4.	destroy_approvals
/// 5.	finish_destroy
/// 6.	mint
/// 7.	burn
/// 8.	transfer
/// 9.	transfer_keep_alive
/// 10.	force_transfer
/// 11. freeze
/// 12.	thaw
/// 13. freeze_asset
/// 14. thaw_asset
/// 15.	transfer_ownership
/// 16. set_team
/// 17.	set_metadata
/// 18.	clear_metadata
/// 19.	approve_transfer
/// 20.	cancel_approval
/// 21.	force_cancel_approval
/// 22.	transfer_approved
/// 23.	touch
/// 24.	refund
/// 25.	set_min_balance
/// 26. touch_other
/// 27.	refund_other
/// 28. block


/// Issue a new class of fungible assets from a public origin.
pub fn create(
	id: AssetId,
	admin: impl Into<MultiAddress<AccountId, ()>>,
	min_balance: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Create {
		id: id.into(),
		admin: admin.into(),
		min_balance: Compact(min_balance),
	}))?)
}

/// Start the process of destroying a fungible asset class.
pub fn start_destroy(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::StartDestroy {
		id: id.into(),
	}))?)
}

/// Destroy all accounts associated with a given asset.
pub fn destroy_accounts(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::DestroyAccounts {
		id: id.into(),
	}))?)
}

/// Destroy all approvals associated with a given asset up to the max (see runtime configuration TrustBackedAssets `RemoveItemsLimit`).
pub fn destroy_approvals(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::DestroyApprovals {
		id: id.into(),
	}))?)
}

/// Complete destroying asset and unreserve currency.
pub fn finish_destroy(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::FinishDestroy {
		id: id.into(),
	}))?)
}

/// Mint assets of a particular class.
pub fn mint(
	id: AssetId,
	beneficiary: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Mint {
		id: id.into(),
		beneficiary: beneficiary.into(),
		amount: Compact(amount),
	}))?)
}

/// Reduce the balance of `who` by as much as possible up to `amount` assets of `id`.
pub fn burn(
	id: AssetId,
	who: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Burn {
		id: id.into(),
		who: who.into(),
		amount: Compact(amount),
	}))?)
}

/// Move some assets from the sender account to another.
pub fn transfer(
	id: AssetId,
	target: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Transfer {
		id: id.into(),
		target: target.into(),
		amount: Compact(amount),
	}))?)
}

/// Move some assets from the sender account to another, keeping the sender account alive.
pub fn transfer_keep_alive(
	id: AssetId,
	target: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::TransferKeepAlive {
		id: id.into(),
		target: target.into(),
		amount: Compact(amount),
	}))?)
}

/// Move some assets from one account to another. Sender should be the Admin of the asset `id`.
pub fn force_transfer(
	id: AssetId,
	source: impl Into<MultiAddress<AccountId, ()>>,
	dest: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::ForceTransfer {
		id: id.into(),
		source: source.into(),
		dest: dest.into(),
		amount: Compact(amount),
	}))?)
}

/// Disallow further unprivileged transfers of an asset `id` from an account `who`. `who`
/// must already exist as an entry in `Account`s of the asset. If you want to freeze an
/// account that does not have an entry, use `touch_other` first.
pub fn freeze(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Freeze {
		id: id.into(),
		who: who.into(),
	}))?)
}

/// Allow unprivileged transfers to and from an account again.
pub fn thaw(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Thaw {
		id: id.into(),
		who: who.into(),
	}))?)
}

/// Disallow further unprivileged transfers for the asset class.
pub fn freeze_asset(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::FreezeAsset {
		id: id.into(),
	}))?)
}

/// Allow unprivileged transfers for the asset again.
pub fn thaw_asset(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::ThawAsset {
		id: id.into(),
	}))?)
}

/// Change the Owner of an asset.
pub fn transfer_ownership(
	id: AssetId,
	owner: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::TransferOwnership {
		id: id.into(),
		owner: owner.into(),
	}))?)
}

/// Change the Issuer, Admin and Freezer of an asset.
pub fn set_team(
	id: AssetId,
	issuer: impl Into<MultiAddress<AccountId, ()>>,
	admin: impl Into<MultiAddress<AccountId, ()>>,
	freezer: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::SetTeam {
		id: id.into(),
		issuer: issuer.into(),
		admin: admin.into(),
		freezer: freezer.into(),
	}))?)
}

/// Set the metadata for an asset.
pub fn set_metadata(id: AssetId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::SetMetadata {
		id: id.into(),
		name,
		symbol,
		decimals,
	}))?)
}

/// Clear the metadata for an asset.
pub fn clear_metadata(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::ClearMetadata {
		id: id.into(),
	}))?)
}

/// Approve an amount of asset for transfer by a delegated third-party account.
pub fn approve_transfer(
	id: AssetId,
	delegate: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::ApproveTransfer {
		id: id.into(),
		delegate: delegate.into(),
		amount: Compact(amount),
	}))?)
}

/// Cancel all of some asset approved for delegated transfer by a third-party account.
pub fn cancel_approval(
	id: AssetId,
	delegate: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::CancelApproval {
		id: id.into(),
		delegate: delegate.into(),
	}))?)
}

/// Cancel all of some asset approved for delegated transfer by a third-party account.
pub fn force_cancel_approval(
	id: AssetId,
	owner: impl Into<MultiAddress<AccountId, ()>>,
	delegate: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::ForceCancelApproval {
		id: id.into(),
		owner: owner.into(),
		delegate: delegate.into(),
	}))?)
}

/// Transfer some asset balance from a previously delegated account to some third-party
/// account.
pub fn transfer_approved(
	id: AssetId,
	owner: impl Into<MultiAddress<AccountId, ()>>,
	destination: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::TransferApproved {
		id: id.into(),
		owner: owner.into(),
		destination: destination.into(),
		amount: Compact(amount),
	}))?)
}

/// Create an asset account for non-provider assets.
pub fn touch(id: AssetId) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Touch {
		id: id.into(),
	}))?)
}

/// Return the deposit (if any) of an asset account or a consumer reference (if any) of an
/// account.
pub fn refund(id: AssetId, allow_burn: bool) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Refund {
		id: id.into(),
		allow_burn,
	}))?)
}

/// Sets the minimum balance of an asset.
pub fn set_min_balance(id: AssetId, min_balance: Balance) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::SetMinBalance {
		id: id.into(),
		min_balance: Compact(min_balance),
	}))?)
}

/// Create an asset account for `who`.
pub fn touch_other(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::TouchOther {
		id: id.into(),
		who: who.into(),
	}))?)
}

/// Return the deposit (if any) of a target asset account. Useful if you are the depositor.
pub fn refund_other(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::RefundOther {
		id: id.into(),
		who: who.into(),
	}))?)
}

/// Disallow further unprivileged transfers of an asset `id` to and from an account `who`.
pub fn block(id: AssetId, who: impl Into<MultiAddress<AccountId, ()>>) -> Result<()> {
	Ok(dispatch(RuntimeCall::TrustBackedAssets(TrustBackedAssetsCalls::Block {
		id: id.into(),
		who: who.into(),
	}))?)
}

pub fn asset_exists(id: AssetId) -> Result<bool> {
	Ok(state::read(RuntimeStateKeys::TrustBackedAssets(TrustBackedAssetsKeys::AssetExists(id)))?)
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

#[derive(Encode)]
pub(crate) enum TrustBackedAssetsCalls {
	#[codec(index = 0)]
	Create {
		id: AssetIdParameter,
		admin: MultiAddress<AccountId, ()>,
		min_balance: BalanceParameter,
	},
	#[codec(index = 2)]
	StartDestroy { id: AssetIdParameter },
	#[codec(index = 3)]
	DestroyAccounts { id: AssetIdParameter },
	#[codec(index = 4)]
	DestroyApprovals { id: AssetIdParameter },
	#[codec(index = 5)]
	FinishDestroy { id: AssetIdParameter },
	#[codec(index = 6)]
	Mint {
		id: AssetIdParameter,
		beneficiary: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	#[codec(index = 7)]
	Burn { id: AssetIdParameter, who: MultiAddress<AccountId, ()>, amount: BalanceParameter },
	#[codec(index = 8)]
	Transfer { id: AssetIdParameter, target: MultiAddress<AccountId, ()>, amount: BalanceParameter },
	#[codec(index = 9)]
	TransferKeepAlive {
		id: AssetIdParameter,
		target: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	#[codec(index = 10)]
	ForceTransfer {
		id: AssetIdParameter,
		source: MultiAddress<AccountId, ()>,
		dest: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	#[codec(index = 11)]
	Freeze { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	#[codec(index = 12)]
	Thaw { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	#[codec(index = 13)]
	FreezeAsset { id: AssetIdParameter },
	#[codec(index = 14)]
	ThawAsset { id: AssetIdParameter },
	#[codec(index = 15)]
	TransferOwnership { id: AssetIdParameter, owner: MultiAddress<AccountId, ()> },
	#[codec(index = 16)]
	SetTeam {
		id: AssetIdParameter,
		issuer: MultiAddress<AccountId, ()>,
		admin: MultiAddress<AccountId, ()>,
		freezer: MultiAddress<AccountId, ()>,
	},
	#[codec(index = 17)]
	SetMetadata { id: AssetIdParameter, name: Vec<u8>, symbol: Vec<u8>, decimals: u8 },
	#[codec(index = 18)]
	ClearMetadata { id: AssetIdParameter },
	#[codec(index = 22)]
	ApproveTransfer {
		id: AssetIdParameter,
		delegate: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	#[codec(index = 23)]
	CancelApproval { id: AssetIdParameter, delegate: MultiAddress<AccountId, ()> },
	#[codec(index = 24)]
	ForceCancelApproval {
		id: AssetIdParameter,
		owner: MultiAddress<AccountId, ()>,
		delegate: MultiAddress<AccountId, ()>,
	},
	#[codec(index = 25)]
	TransferApproved {
		id: AssetIdParameter,
		owner: MultiAddress<AccountId, ()>,
		destination: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
	#[codec(index = 26)]
	Touch { id: AssetIdParameter },
	#[codec(index = 27)]
	Refund { id: AssetIdParameter, allow_burn: bool },
	#[codec(index = 28)]
	SetMinBalance { id: AssetIdParameter, min_balance: BalanceParameter },
	#[codec(index = 29)]
	TouchOther { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	#[codec(index = 30)]
	RefundOther { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
	#[codec(index = 31)]
	Block { id: AssetIdParameter, who: MultiAddress<AccountId, ()> },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	/// Account balance must be greater than or equal to the transfer amount.
	BalanceLow,
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
	/// The origin account is frozen.
	Frozen,
	/// The asset ID is already taken.
	InUse,
	/// Invalid witness data given.
	BadWitness,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// Unable to increment the consumer reference counters on the account. Either no provider
	/// reference exists to allow a non-zero balance of a non-self-sufficient asset, or one
	/// fewer then the maximum number of consumers has been reached.
	UnavailableConsumer,
	/// Invalid metadata given.
	BadMetadata,
	/// No approval exists that would allow the transfer.
	Unapproved,
	/// The source account would not survive the transfer and it needs to stay alive.
	WouldDie,
	/// The asset-account already exists.
	AlreadyExists,
	/// The asset-account doesn't have an associated deposit.
	NoDeposit,
	/// The operation would result in funds being burned.
	WouldBurn,
	/// The asset is a live asset and is actively being used. Usually emit for operations such
	/// as `start_destroy` which require the asset to be in a destroying state.
	LiveAsset,
	/// The asset is not live, and likely being destroyed.
	AssetNotLive,
	/// The asset status is not the expected status.
	IncorrectStatus,
	/// The asset should be frozen before the given operation.
	NotFrozen,
	/// Callback action resulted in error
	CallbackFailed,
}

impl TryFrom<u32> for Error {
	type Error = PopApiError;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use Error::*;
		match status_code {
			0 => Ok(BalanceLow),
			1 => Ok(NoAccount),
			2 => Ok(NoPermission),
			3 => Ok(Unknown),
			4 => Ok(Frozen),
			5 => Ok(InUse),
			6 => Ok(BadWitness),
			7 => Ok(MinBalanceZero),
			8 => Ok(UnavailableConsumer),
			9 => Ok(BadMetadata),
			10 => Ok(Unapproved),
			11 => Ok(WouldDie),
			12 => Ok(AlreadyExists),
			13 => Ok(NoDeposit),
			14 => Ok(WouldBurn),
			15 => Ok(LiveAsset),
			16 => Ok(AssetNotLive),
			17 => Ok(IncorrectStatus),
			18 => Ok(NotFrozen),
			_ => Err(UnknownStatusCode(status_code)),
		}
	}
}

impl From<PopApiError> for Error {
	fn from(error: PopApiError) -> Self {
		match error {
			PopApiError::TrustBackedAssets(e) => e,
			_ => panic!("Unexpected pallet assets error. This error is unknown to pallet assets"),
		}
	}
}
