use crate::{AccountId, Balance, PopApiError::UnknownStatusCode, RuntimeCall, *};
use ink::prelude::vec::Vec;
use primitives::AssetId;
use scale::{Compact, Encode};

type Result<T> = core::result::Result<T, FungiblesError>;

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management

/// 1. PSP-22 Interface:
/// - total_supply
/// - balance_of
/// - allowance
/// - transfer
/// - transfer_from
/// - approve
/// - increase_allowance
/// - decrease_allowance

/// Returns the total token supply for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// The total supply of the token, or an error if the operation fails.
pub fn total_supply(id: AssetId) -> Result<Balance> {
	Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::TotalSupply(id)))?)
}

/// Returns the account balance for the specified `owner` for a given asset ID. Returns `0` if
/// the account is non-existent.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `owner` - The account whose balance is being queried.
///
/// # Returns
/// The balance of the specified account, or an error if the operation fails.
pub fn balance_of(id: AssetId, owner: AccountId) -> Result<Balance> {
	Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::BalanceOf(id, owner)))?)
}

/// Returns the amount which `spender` is still allowed to withdraw from `owner` for a given
/// asset ID. Returns `0` if no allowance has been set.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `owner` - The account that owns the tokens.
/// * `spender` - The account that is allowed to spend the tokens.
///
/// # Returns
/// The remaining allowance, or an error if the operation fails.
pub fn allowance(id: AssetId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::Allowance(id, owner, spender)))?)
}

/// Create a new token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `admin` - The account that will administer the asset.
/// * `min_balance` - The minimum balance required for accounts holding this asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the creation fails.
// pub fn create(id: AssetId, admin: impl Into<MultiAddress<AccountId, ()>>, min_balance: Balance) -> Result<()> {
pub fn create(
	id: AssetId,
	admin: impl Into<MultiAddress<AccountId, ()>>,
	min_balance: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Assets(AssetsCall::Create {
		id: id.into(),
		admin: admin.into(),
		min_balance,
	}))?)
}

/// Transfers `value` amount of tokens from the caller's account to account `to`, with additional
/// `data` in unspecified format.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `to` - The recipient account.
/// * `value` - The number of tokens to transfer.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the transfer fails.
// #[allow(unused_variables)]
// pub fn transfer(
// 	id: AssetId,
// 	to: impl Into<MultiAddress<AccountId, ()>>,
// 	value: Balance,
// ) -> Result<()> {
// 	todo!()
// 	// TODO: transfer or transfer_keep_alive
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::Transfer {
// 	// 	id: id.into(),
// 	// 	target: target.into(),
// 	// 	amount: Compact(amount),
// 	// }))?)
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::TransferKeepAlive {
// 	// 	id: id.into(),
// 	// 	target: target.into(),
// 	// 	amount: Compact(amount),
// 	// }))?)
// }

/// Transfers `value` tokens on the behalf of `from` to the account `to` with additional `data`
/// in unspecified format. This can be used to allow a contract to transfer tokens on ones behalf
/// and/or to charge fees in sub-currencies, for example.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `from` - The account from which the tokens are transferred.
/// * `to` - The recipient account.
/// * `value` - The number of tokens to transfer.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the transfer fails.
// pub fn transfer_from(
// 	id: AssetId,
// 	from: impl Into<MultiAddress<AccountId, ()>>,
// 	to: impl Into<MultiAddress<AccountId, ()>>,
// 	value: Balance,
// ) -> Result<()> {
//todo!()
// TODO: depending on `from` and `to`, decide whether to mint, burn or transfer_approved.
// Ok(dispatch(RuntimeCall::Assets(AssetsCall::Mint {
// 	id: id.into(),
// 	beneficiary: beneficiary.into(),
// 	amount: Compact(amount),
// }))?)
// Ok(dispatch(RuntimeCall::Assets(AssetsCall::Burn {
// 	id: id.into(),
// 	who: who.into(),
// 	amount: Compact(amount),
// }))?)
// Ok(dispatch(RuntimeCall::Assets(AssetsCall::TransferApproved {
// 	id: id.into(),
// 	owner: from.into(),
// 	destination: to.into(),
// 	amount: Compact(value),
// }))?)
// }

/// Mint assets of a particular class.
pub fn mint(
	id: AssetId,
	beneficiary: impl Into<MultiAddress<AccountId, ()>>,
	amount: Balance,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Assets(AssetsCall::Mint {
		id: id.into(),
		beneficiary: beneficiary.into(),
		amount: Compact(amount),
	}))?)
}

/// Approves an account to spend a specified number of tokens on behalf of the caller.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to approve.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the approval fails.
// #[allow(unused_variables)]
// fn approve(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
// 	todo!()
// 	// TODO: read allowance and increase or decrease.
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
// 	// 	id: id.into(),
// 	// 	delegate: spender.into(),
// 	// 	amount: Compact(value),
// 	// }))?)
// }

/// Increases the allowance of a spender.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to increase the allowance by.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn increase_allowance(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
// 		id: id.into(),
// 		delegate: spender.into(),
// 		amount: Compact(value),
// 	}))?)
// }

/// Decreases the allowance of a spender.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to decrease the allowance by.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// #[allow(unused_variables)]
// fn decrease_allowance(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
// 	todo!()
// 	// TODO: cancel_approval + approve_transfer
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::CancelApproval {
// 	// 	id: id.into(),
// 	// 	delegate: delegate.into(),
// 	// }))?)
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
// 	// 	id: id.into(),
// 	// 	delegate: spender.into(),
// 	// 	amount: Compact(value),
// 	// }))?)
// }

/// 2. PSP-22 Metadata Interface:
/// - token_name
/// - token_symbol
/// - token_decimals

/// Returns the token name for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// The name of the token as a byte vector, or an error if the operation fails.
// #[allow(unused_variables)]
// pub fn token_name(id: AssetId) -> Result<Option<Vec<u8>>> {
// 	todo!()
// 	// Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::TokenName(id)))?)
// }

/// Returns the token symbol for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
///  The symbol of the token as a byte vector, or an error if the operation fails.
// #[allow(unused_variables)]
// fn token_symbol(id: AssetId) -> Result<Option<Vec<u8>>> {
// 	todo!()
// }

/// Returns the token decimals for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
///  The number of decimals of the token as a byte vector, or an error if the operation fails.
// #[allow(unused_variables)]
// fn token_decimals(id: AssetId) -> Result<Option<Vec<u8>>> {
// 	todo!()
// }

/// 3. Asset Management:
/// - create
/// - start_destroy
/// - destroy_accounts
/// - destroy_approvals
/// - finish_destroy
/// - set_metadata
/// - clear_metadata

/// Start the process of destroying a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn start_destroy(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::StartDestroy {
// 		id: id.into(),
// 	}))?)
// }

/// Destroy all accounts associated with a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn destroy_accounts(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::DestroyAccounts {
// 		id: id.into(),
// 	}))?)
// }

/// Destroy all approvals associated with a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn destroy_approvals(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::DestroyApprovals {
// 		id: id.into(),
// 	}))?)
// }

/// Complete the process of destroying a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn finish_destroy(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::FinishDestroy {
// 		id: id.into(),
// 	}))?)
// }

/// Set the metadata for a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
pub fn set_metadata(id: AssetId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8) -> Result<()> {
	Ok(dispatch(RuntimeCall::Assets(AssetsCall::SetMetadata {
		id: id.into(),
		name,
		symbol,
		decimals,
	}))?)
}

/// Clear the metadata for a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn clear_metadata(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::ClearMetadata {
// 		id: id.into(),
// 	}))?)
// }

pub fn asset_exists(id: AssetId) -> Result<bool> {
	Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::AssetExists(id)))?)
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

#[allow(warnings, unused)]
#[derive(Encode)]
pub(crate) enum AssetsCall {
	#[codec(index = 0)]
	Create { id: AssetIdParameter, admin: MultiAddress<AccountId, ()>, min_balance: Balance },
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
	// TODO: ED or not
	// #[codec(index = 8)]
	// Transfer { id: AssetIdParameter, target: MultiAddress<AccountId, ()>, amount: BalanceParameter },
	#[codec(index = 9)]
	TransferKeepAlive {
		id: AssetIdParameter,
		target: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
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
	#[codec(index = 25)]
	TransferApproved {
		id: AssetIdParameter,
		owner: MultiAddress<AccountId, ()>,
		destination: MultiAddress<AccountId, ()>,
		amount: BalanceParameter,
	},
}

// TODO: remove unnecessary errors
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) enum AssetsError {
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

impl From<PopApiError> for AssetsError {
	fn from(error: PopApiError) -> Self {
		match error {
			PopApiError::Assets(e) => e,
			_ => panic!("Expected AssetsError"),
		}
	}
}

impl TryFrom<u32> for AssetsError {
	type Error = PopApiError;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use AssetsError::*;
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FungiblesError {
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
	InsufficientBalance,
	/// The asset ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
}

impl From<balances::Error> for FungiblesError {
	fn from(error: balances::Error) -> Self {
		match error {
			balances::Error::InsufficientBalance => FungiblesError::InsufficientBalance,
			_ => panic!("Unexpected pallet assets error. This error is unknown to pallet assets"),
		}
	}
}

impl From<AssetsError> for FungiblesError {
	fn from(error: AssetsError) -> Self {
		match error {
			AssetsError::InUse => FungiblesError::InUse,
			_ => panic!("Unexpected pallet assets error. This error is unknown to pallet assets"),
		}
	}
}

impl From<PopApiError> for FungiblesError {
	fn from(error: PopApiError) -> Self {
		match error {
			PopApiError::Assets(e) => e.into(),
			// PopApiError::Balances(e) => todo!("balances: {:?}", e),
			PopApiError::Balances(e) => e.into(),
			// PopApiError::Contracts(_e) => todo!("contracts"),
			// PopApiError::SystemCallFiltered => 100,
			// PopApiError::UnknownStatusCode(u) => u,
			_ => panic!("Unexpected pallet assets error. This error is unknown to pallet assets"),
		}
	}
}
