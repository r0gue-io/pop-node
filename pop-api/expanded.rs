#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use core::convert::TryInto;
use ink::{prelude::vec::Vec, ChainExtensionInstance};
use primitives::{cross_chain::*, storage_keys::*, AccountId as AccountId32};
pub use sp_runtime::{BoundedVec, MultiAddress, MultiSignature};
use v0::RuntimeCall;
pub use v0::{
    assets, balances, contracts, cross_chain, dispatch_error, nfts,
    relay_chain_block_number, state,
};
pub mod primitives {
    pub use pop_primitives::*;
    pub use sp_runtime::{BoundedVec, MultiAddress};
}
pub mod v0 {
    use crate::{
        primitives::storage_keys::{ParachainSystemKeys, RuntimeStateKeys},
        BlockNumber, PopApiError,
    };
    pub mod assets {
        pub mod fungibles {
            use crate::{
                balances::BalancesError, AccountId, Balance,
                PopApiError::UnknownModuleStatusCode, RuntimeCall, *,
            };
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
                Ok(
                    state::read(
                        RuntimeStateKeys::Assets(AssetsKeys::BalanceOf(id, owner)),
                    )?,
                )
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
            pub fn allowance(
                id: AssetId,
                owner: AccountId,
                spender: AccountId,
            ) -> Result<Balance> {
                Ok(
                    state::read(
                        RuntimeStateKeys::Assets(
                            AssetsKeys::Allowance(id, owner, spender),
                        ),
                    )?,
                )
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
            pub fn transfer(
                id: AssetId,
                to: impl Into<MultiAddress<AccountId, ()>>,
                value: Balance,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Assets(AssetsCall::TransferKeepAlive {
                            id: id.into(),
                            target: to.into(),
                            amount: Compact(value),
                        }),
                    )?,
                )
            }
            /// Transfers `value` tokens on behalf of `from` to account `to` with additional `data`
            /// in unspecified format. If `from` is equal to `None`, tokens will be minted to account `to`. If
            /// `to` is equal to `None`, tokens will be burned from account `from`.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            /// * `from` - The account from which the tokens are transferred.
            /// * `to` - The recipient account.
            /// * `value` - The number of tokens to transfer.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the transfer fails.
            pub fn transfer_from(
                id: AssetId,
                from: Option<impl Into<MultiAddress<AccountId, ()>>>,
                to: Option<impl Into<MultiAddress<AccountId, ()>>>,
                value: Balance,
                _data: &[u8],
            ) -> Result<()> {
                match (from, to) {
                    (None, Some(to)) => mint(id, to, value),
                    (Some(from), Some(to)) => {
                        Ok(
                            dispatch(
                                RuntimeCall::Assets(AssetsCall::TransferApproved {
                                    id: id.into(),
                                    owner: from.into(),
                                    destination: to.into(),
                                    amount: Compact(value),
                                }),
                            )?,
                        )
                    }
                    _ => Ok(()),
                }
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
            /// Increases the allowance of a spender.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            /// * `spender` - The account that is allowed to spend the tokens.
            /// * `value` - The number of tokens to increase the allowance by.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            /// Decreases the allowance of a spender.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            /// * `spender` - The account that is allowed to spend the tokens.
            /// * `value` - The number of tokens to decrease the allowance by.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
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
            /// Returns the token symbol for a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            ///  The symbol of the token as a byte vector, or an error if the operation fails.
            /// Returns the token decimals for a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            ///  The number of decimals of the token as a byte vector, or an error if the operation fails.
            /// 3. Asset Management:
            /// - create
            /// - start_destroy
            /// - destroy_accounts
            /// - destroy_approvals
            /// - finish_destroy
            /// - set_metadata
            /// - clear_metadata
            /// Create a new token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            /// * `admin` - The account that will administer the asset.
            /// * `min_balance` - The minimum balance required for accounts holding this asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the creation fails.
            pub fn create(
                id: AssetId,
                admin: impl Into<MultiAddress<AccountId, ()>>,
                min_balance: Balance,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Assets(AssetsCall::Create {
                            id: id.into(),
                            admin: admin.into(),
                            min_balance,
                        }),
                    )?,
                )
            }
            /// Start the process of destroying a token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            /// Destroy all accounts associated with a token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            /// Destroy all approvals associated with a token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            /// Complete the process of destroying a token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            /// Set the metadata for a token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            pub fn set_metadata(
                id: AssetId,
                name: Vec<u8>,
                symbol: Vec<u8>,
                decimals: u8,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Assets(AssetsCall::SetMetadata {
                            id: id.into(),
                            name,
                            symbol,
                            decimals,
                        }),
                    )?,
                )
            }
            /// Clear the metadata for a token with a given asset ID.
            ///
            /// # Arguments
            /// * `id` - The ID of the asset.
            ///
            /// # Returns
            /// Returns `Ok(())` if successful, or an error if the operation fails.
            pub fn asset_exists(id: AssetId) -> Result<bool> {
                Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::AssetExists(id)))?)
            }
            /// Mint assets of a particular class.
            fn mint(
                id: AssetId,
                beneficiary: impl Into<MultiAddress<AccountId, ()>>,
                amount: Balance,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Assets(AssetsCall::Mint {
                            id: id.into(),
                            beneficiary: beneficiary.into(),
                            amount: Compact(amount),
                        }),
                    )?,
                )
            }
            type AssetIdParameter = Compact<AssetId>;
            type BalanceParameter = Compact<Balance>;
            #[allow(warnings, unused)]
            pub(crate) enum AssetsCall {
                #[codec(index = 0)]
                Create {
                    id: AssetIdParameter,
                    admin: MultiAddress<AccountId, ()>,
                    min_balance: Balance,
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
                Burn {
                    id: AssetIdParameter,
                    who: MultiAddress<AccountId, ()>,
                    amount: BalanceParameter,
                },
                #[codec(index = 9)]
                TransferKeepAlive {
                    id: AssetIdParameter,
                    target: MultiAddress<AccountId, ()>,
                    amount: BalanceParameter,
                },
                #[codec(index = 17)]
                SetMetadata {
                    id: AssetIdParameter,
                    name: Vec<u8>,
                    symbol: Vec<u8>,
                    decimals: u8,
                },
                #[codec(index = 18)]
                ClearMetadata { id: AssetIdParameter },
                #[codec(index = 22)]
                ApproveTransfer {
                    id: AssetIdParameter,
                    delegate: MultiAddress<AccountId, ()>,
                    amount: BalanceParameter,
                },
                #[codec(index = 23)]
                CancelApproval {
                    id: AssetIdParameter,
                    delegate: MultiAddress<AccountId, ()>,
                },
                #[codec(index = 25)]
                TransferApproved {
                    id: AssetIdParameter,
                    owner: MultiAddress<AccountId, ()>,
                    destination: MultiAddress<AccountId, ()>,
                    amount: BalanceParameter,
                },
            }
            #[allow(deprecated)]
            const _: () = {
                #[allow(warnings, unused)]
                #[automatically_derived]
                impl ::scale::Encode for AssetsCall {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                AssetsCall::Create {
                                    ref id,
                                    ref admin,
                                    ref min_balance,
                                } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(admin))
                                        .saturating_add(::scale::Encode::size_hint(min_balance))
                                }
                                AssetsCall::StartDestroy { ref id } => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(id))
                                }
                                AssetsCall::DestroyAccounts { ref id } => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(id))
                                }
                                AssetsCall::DestroyApprovals { ref id } => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(id))
                                }
                                AssetsCall::FinishDestroy { ref id } => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(id))
                                }
                                AssetsCall::Mint { ref id, ref beneficiary, ref amount } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(beneficiary))
                                        .saturating_add(::scale::Encode::size_hint(amount))
                                }
                                AssetsCall::Burn { ref id, ref who, ref amount } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(who))
                                        .saturating_add(::scale::Encode::size_hint(amount))
                                }
                                AssetsCall::TransferKeepAlive {
                                    ref id,
                                    ref target,
                                    ref amount,
                                } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(target))
                                        .saturating_add(::scale::Encode::size_hint(amount))
                                }
                                AssetsCall::SetMetadata {
                                    ref id,
                                    ref name,
                                    ref symbol,
                                    ref decimals,
                                } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(name))
                                        .saturating_add(::scale::Encode::size_hint(symbol))
                                        .saturating_add(::scale::Encode::size_hint(decimals))
                                }
                                AssetsCall::ClearMetadata { ref id } => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(id))
                                }
                                AssetsCall::ApproveTransfer {
                                    ref id,
                                    ref delegate,
                                    ref amount,
                                } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(delegate))
                                        .saturating_add(::scale::Encode::size_hint(amount))
                                }
                                AssetsCall::CancelApproval { ref id, ref delegate } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(delegate))
                                }
                                AssetsCall::TransferApproved {
                                    ref id,
                                    ref owner,
                                    ref destination,
                                    ref amount,
                                } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(id))
                                        .saturating_add(::scale::Encode::size_hint(owner))
                                        .saturating_add(::scale::Encode::size_hint(destination))
                                        .saturating_add(::scale::Encode::size_hint(amount))
                                }
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            AssetsCall::Create {
                                ref id,
                                ref admin,
                                ref min_balance,
                            } => {
                                __codec_dest_edqy.push_byte(0u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(admin, __codec_dest_edqy);
                                ::scale::Encode::encode_to(min_balance, __codec_dest_edqy);
                            }
                            AssetsCall::StartDestroy { ref id } => {
                                __codec_dest_edqy.push_byte(2u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                            }
                            AssetsCall::DestroyAccounts { ref id } => {
                                __codec_dest_edqy.push_byte(3u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                            }
                            AssetsCall::DestroyApprovals { ref id } => {
                                __codec_dest_edqy.push_byte(4u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                            }
                            AssetsCall::FinishDestroy { ref id } => {
                                __codec_dest_edqy.push_byte(5u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                            }
                            AssetsCall::Mint { ref id, ref beneficiary, ref amount } => {
                                __codec_dest_edqy.push_byte(6u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(beneficiary, __codec_dest_edqy);
                                ::scale::Encode::encode_to(amount, __codec_dest_edqy);
                            }
                            AssetsCall::Burn { ref id, ref who, ref amount } => {
                                __codec_dest_edqy.push_byte(7u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(who, __codec_dest_edqy);
                                ::scale::Encode::encode_to(amount, __codec_dest_edqy);
                            }
                            AssetsCall::TransferKeepAlive {
                                ref id,
                                ref target,
                                ref amount,
                            } => {
                                __codec_dest_edqy.push_byte(9u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(target, __codec_dest_edqy);
                                ::scale::Encode::encode_to(amount, __codec_dest_edqy);
                            }
                            AssetsCall::SetMetadata {
                                ref id,
                                ref name,
                                ref symbol,
                                ref decimals,
                            } => {
                                __codec_dest_edqy.push_byte(17u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(name, __codec_dest_edqy);
                                ::scale::Encode::encode_to(symbol, __codec_dest_edqy);
                                ::scale::Encode::encode_to(decimals, __codec_dest_edqy);
                            }
                            AssetsCall::ClearMetadata { ref id } => {
                                __codec_dest_edqy.push_byte(18u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                            }
                            AssetsCall::ApproveTransfer {
                                ref id,
                                ref delegate,
                                ref amount,
                            } => {
                                __codec_dest_edqy.push_byte(22u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(delegate, __codec_dest_edqy);
                                ::scale::Encode::encode_to(amount, __codec_dest_edqy);
                            }
                            AssetsCall::CancelApproval { ref id, ref delegate } => {
                                __codec_dest_edqy.push_byte(23u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(delegate, __codec_dest_edqy);
                            }
                            AssetsCall::TransferApproved {
                                ref id,
                                ref owner,
                                ref destination,
                                ref amount,
                            } => {
                                __codec_dest_edqy.push_byte(25u8 as ::core::primitive::u8);
                                ::scale::Encode::encode_to(id, __codec_dest_edqy);
                                ::scale::Encode::encode_to(owner, __codec_dest_edqy);
                                ::scale::Encode::encode_to(destination, __codec_dest_edqy);
                                ::scale::Encode::encode_to(amount, __codec_dest_edqy);
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for AssetsCall {}
            };
            pub enum AssetsError {
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
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                impl ::scale_info::TypeInfo for AssetsError {
                    type Identity = Self;
                    fn type_info() -> ::scale_info::Type {
                        ::scale_info::Type::builder()
                            .path(
                                ::scale_info::Path::new_with_replace(
                                    "AssetsError",
                                    "pop_api::v0::assets::fungibles",
                                    &[],
                                ),
                            )
                            .type_params(::alloc::vec::Vec::new())
                            .variant(
                                ::scale_info::build::Variants::new()
                                    .variant(
                                        "BalanceLow",
                                        |v| {
                                            v
                                                .index(0usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "Account balance must be greater than or equal to the transfer amount.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "NoAccount",
                                        |v| {
                                            v
                                                .index(1usize as ::core::primitive::u8)
                                                .docs(&["The account to alter does not exist."])
                                        },
                                    )
                                    .variant(
                                        "NoPermission",
                                        |v| {
                                            v
                                                .index(2usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "The signing account has no permission to do the operation.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "Unknown",
                                        |v| {
                                            v
                                                .index(3usize as ::core::primitive::u8)
                                                .docs(&["The given asset ID is unknown."])
                                        },
                                    )
                                    .variant(
                                        "Frozen",
                                        |v| {
                                            v
                                                .index(4usize as ::core::primitive::u8)
                                                .docs(&["The origin account is frozen."])
                                        },
                                    )
                                    .variant(
                                        "InUse",
                                        |v| {
                                            v
                                                .index(5usize as ::core::primitive::u8)
                                                .docs(&["The asset ID is already taken."])
                                        },
                                    )
                                    .variant(
                                        "BadWitness",
                                        |v| {
                                            v
                                                .index(6usize as ::core::primitive::u8)
                                                .docs(&["Invalid witness data given."])
                                        },
                                    )
                                    .variant(
                                        "MinBalanceZero",
                                        |v| {
                                            v
                                                .index(7usize as ::core::primitive::u8)
                                                .docs(&["Minimum balance should be non-zero."])
                                        },
                                    )
                                    .variant(
                                        "UnavailableConsumer",
                                        |v| {
                                            v
                                                .index(8usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "Unable to increment the consumer reference counters on the account. Either no provider",
                                                        "reference exists to allow a non-zero balance of a non-self-sufficient asset, or one",
                                                        "fewer then the maximum number of consumers has been reached.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "BadMetadata",
                                        |v| {
                                            v
                                                .index(9usize as ::core::primitive::u8)
                                                .docs(&["Invalid metadata given."])
                                        },
                                    )
                                    .variant(
                                        "Unapproved",
                                        |v| {
                                            v
                                                .index(10usize as ::core::primitive::u8)
                                                .docs(
                                                    &["No approval exists that would allow the transfer."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "WouldDie",
                                        |v| {
                                            v
                                                .index(11usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "The source account would not survive the transfer and it needs to stay alive.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "AlreadyExists",
                                        |v| {
                                            v
                                                .index(12usize as ::core::primitive::u8)
                                                .docs(&["The asset-account already exists."])
                                        },
                                    )
                                    .variant(
                                        "NoDeposit",
                                        |v| {
                                            v
                                                .index(13usize as ::core::primitive::u8)
                                                .docs(
                                                    &["The asset-account doesn't have an associated deposit."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "WouldBurn",
                                        |v| {
                                            v
                                                .index(14usize as ::core::primitive::u8)
                                                .docs(
                                                    &["The operation would result in funds being burned."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "LiveAsset",
                                        |v| {
                                            v
                                                .index(15usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "The asset is a live asset and is actively being used. Usually emit for operations such",
                                                        "as `start_destroy` which require the asset to be in a destroying state.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "AssetNotLive",
                                        |v| {
                                            v
                                                .index(16usize as ::core::primitive::u8)
                                                .docs(
                                                    &["The asset is not live, and likely being destroyed."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "IncorrectStatus",
                                        |v| {
                                            v
                                                .index(17usize as ::core::primitive::u8)
                                                .docs(&["The asset status is not the expected status."])
                                        },
                                    )
                                    .variant(
                                        "NotFrozen",
                                        |v| {
                                            v
                                                .index(18usize as ::core::primitive::u8)
                                                .docs(
                                                    &["The asset should be frozen before the given operation."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "CallbackFailed",
                                        |v| {
                                            v
                                                .index(19usize as ::core::primitive::u8)
                                                .docs(&["Callback action resulted in error"])
                                        },
                                    ),
                            )
                    }
                }
            };
            #[automatically_derived]
            impl ::core::fmt::Debug for AssetsError {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::write_str(
                        f,
                        match self {
                            AssetsError::BalanceLow => "BalanceLow",
                            AssetsError::NoAccount => "NoAccount",
                            AssetsError::NoPermission => "NoPermission",
                            AssetsError::Unknown => "Unknown",
                            AssetsError::Frozen => "Frozen",
                            AssetsError::InUse => "InUse",
                            AssetsError::BadWitness => "BadWitness",
                            AssetsError::MinBalanceZero => "MinBalanceZero",
                            AssetsError::UnavailableConsumer => "UnavailableConsumer",
                            AssetsError::BadMetadata => "BadMetadata",
                            AssetsError::Unapproved => "Unapproved",
                            AssetsError::WouldDie => "WouldDie",
                            AssetsError::AlreadyExists => "AlreadyExists",
                            AssetsError::NoDeposit => "NoDeposit",
                            AssetsError::WouldBurn => "WouldBurn",
                            AssetsError::LiveAsset => "LiveAsset",
                            AssetsError::AssetNotLive => "AssetNotLive",
                            AssetsError::IncorrectStatus => "IncorrectStatus",
                            AssetsError::NotFrozen => "NotFrozen",
                            AssetsError::CallbackFailed => "CallbackFailed",
                        },
                    )
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for AssetsError {}
            #[automatically_derived]
            impl ::core::clone::Clone for AssetsError {
                #[inline]
                fn clone(&self) -> AssetsError {
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for AssetsError {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for AssetsError {
                #[inline]
                fn eq(&self, other: &AssetsError) -> bool {
                    let __self_tag = ::core::intrinsics::discriminant_value(self);
                    let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                    __self_tag == __arg1_tag
                }
            }
            #[automatically_derived]
            impl ::core::cmp::Eq for AssetsError {
                #[inline]
                #[doc(hidden)]
                #[coverage(off)]
                fn assert_receiver_is_total_eq(&self) -> () {}
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for AssetsError {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                AssetsError::BalanceLow => 0_usize,
                                AssetsError::NoAccount => 0_usize,
                                AssetsError::NoPermission => 0_usize,
                                AssetsError::Unknown => 0_usize,
                                AssetsError::Frozen => 0_usize,
                                AssetsError::InUse => 0_usize,
                                AssetsError::BadWitness => 0_usize,
                                AssetsError::MinBalanceZero => 0_usize,
                                AssetsError::UnavailableConsumer => 0_usize,
                                AssetsError::BadMetadata => 0_usize,
                                AssetsError::Unapproved => 0_usize,
                                AssetsError::WouldDie => 0_usize,
                                AssetsError::AlreadyExists => 0_usize,
                                AssetsError::NoDeposit => 0_usize,
                                AssetsError::WouldBurn => 0_usize,
                                AssetsError::LiveAsset => 0_usize,
                                AssetsError::AssetNotLive => 0_usize,
                                AssetsError::IncorrectStatus => 0_usize,
                                AssetsError::NotFrozen => 0_usize,
                                AssetsError::CallbackFailed => 0_usize,
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            AssetsError::BalanceLow => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(0usize as ::core::primitive::u8);
                            }
                            AssetsError::NoAccount => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(1usize as ::core::primitive::u8);
                            }
                            AssetsError::NoPermission => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(2usize as ::core::primitive::u8);
                            }
                            AssetsError::Unknown => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(3usize as ::core::primitive::u8);
                            }
                            AssetsError::Frozen => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(4usize as ::core::primitive::u8);
                            }
                            AssetsError::InUse => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(5usize as ::core::primitive::u8);
                            }
                            AssetsError::BadWitness => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(6usize as ::core::primitive::u8);
                            }
                            AssetsError::MinBalanceZero => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(7usize as ::core::primitive::u8);
                            }
                            AssetsError::UnavailableConsumer => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(8usize as ::core::primitive::u8);
                            }
                            AssetsError::BadMetadata => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(9usize as ::core::primitive::u8);
                            }
                            AssetsError::Unapproved => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(10usize as ::core::primitive::u8);
                            }
                            AssetsError::WouldDie => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(11usize as ::core::primitive::u8);
                            }
                            AssetsError::AlreadyExists => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(12usize as ::core::primitive::u8);
                            }
                            AssetsError::NoDeposit => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(13usize as ::core::primitive::u8);
                            }
                            AssetsError::WouldBurn => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(14usize as ::core::primitive::u8);
                            }
                            AssetsError::LiveAsset => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(15usize as ::core::primitive::u8);
                            }
                            AssetsError::AssetNotLive => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(16usize as ::core::primitive::u8);
                            }
                            AssetsError::IncorrectStatus => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(17usize as ::core::primitive::u8);
                            }
                            AssetsError::NotFrozen => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(18usize as ::core::primitive::u8);
                            }
                            AssetsError::CallbackFailed => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(19usize as ::core::primitive::u8);
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for AssetsError {}
            };
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Decode for AssetsError {
                    fn decode<__CodecInputEdqy: ::scale::Input>(
                        __codec_input_edqy: &mut __CodecInputEdqy,
                    ) -> ::core::result::Result<Self, ::scale::Error> {
                        match __codec_input_edqy
                            .read_byte()
                            .map_err(|e| {
                                e
                                    .chain(
                                        "Could not decode `AssetsError`, failed to read variant byte",
                                    )
                            })?
                        {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 0usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::BalanceLow)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 1usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::NoAccount)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 2usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::NoPermission)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 3usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::Unknown)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 4usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::Frozen)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 5usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::InUse)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 6usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::BadWitness)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 7usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::MinBalanceZero)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 8usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::UnavailableConsumer)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 9usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::BadMetadata)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 10usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::Unapproved)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 11usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::WouldDie)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 12usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::AlreadyExists)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 13usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::NoDeposit)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 14usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::WouldBurn)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 15usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::LiveAsset)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 16usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::AssetNotLive)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 17usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::IncorrectStatus)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 18usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::NotFrozen)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 19usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(AssetsError::CallbackFailed)
                                })();
                            }
                            _ => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Err(
                                        <_ as ::core::convert::Into<
                                            _,
                                        >>::into(
                                            "Could not decode `AssetsError`, variant doesn't exist",
                                        ),
                                    )
                                })();
                            }
                        }
                    }
                }
            };
            impl From<PopApiError> for AssetsError {
                fn from(error: PopApiError) -> Self {
                    match error {
                        PopApiError::Assets(e) => e,
                        _ => {
                            ::core::panicking::panic_fmt(
                                format_args!("Expected AssetsError"),
                            );
                        }
                    }
                }
            }
            impl TryFrom<u32> for AssetsError {
                type Error = PopApiError;
                fn try_from(
                    status_code: u32,
                ) -> core::result::Result<Self, Self::Error> {
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
                        _ => Err(UnknownModuleStatusCode(status_code)),
                    }
                }
            }
            pub enum FungiblesError {
                /// The asset is not live; either frozen or being destroyed.
                AssetNotLive,
                /// The amount to mint is less than the existential deposit.
                BelowMinimum,
                /// Unspecified dispatch error, providing the index and its error index (if none `0`).
                DispatchError { index: u8, error: u8 },
                /// Not enough allowance to fulfill a request is available.
                InsufficientAllowance,
                /// Not enough balance to fulfill a request is available.
                InsufficientBalance,
                /// The asset ID is already taken.
                InUse,
                /// Minimum balance should be non-zero.
                MinBalanceZero,
                /// Unspecified pallet error, providing pallet index and error index.
                ModuleError { pallet: u8, error: u16 },
                /// The account to alter does not exist.
                NoAccount,
                /// The signing account has no permission to do the operation.
                NoPermission,
                /// The given asset ID is unknown.
                Unknown,
            }
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                impl ::scale_info::TypeInfo for FungiblesError {
                    type Identity = Self;
                    fn type_info() -> ::scale_info::Type {
                        ::scale_info::Type::builder()
                            .path(
                                ::scale_info::Path::new_with_replace(
                                    "FungiblesError",
                                    "pop_api::v0::assets::fungibles",
                                    &[],
                                ),
                            )
                            .type_params(::alloc::vec::Vec::new())
                            .variant(
                                ::scale_info::build::Variants::new()
                                    .variant(
                                        "AssetNotLive",
                                        |v| {
                                            v
                                                .index(0usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "The asset is not live; either frozen or being destroyed.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "BelowMinimum",
                                        |v| {
                                            v
                                                .index(1usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "The amount to mint is less than the existential deposit.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "DispatchError",
                                        |v| {
                                            v
                                                .index(2usize as ::core::primitive::u8)
                                                .fields(
                                                    ::scale_info::build::Fields::named()
                                                        .field(|f| f.ty::<u8>().name("index").type_name("u8"))
                                                        .field(|f| f.ty::<u8>().name("error").type_name("u8")),
                                                )
                                                .docs(
                                                    &[
                                                        "Unspecified dispatch error, providing the index and its error index (if none `0`).",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "InsufficientAllowance",
                                        |v| {
                                            v
                                                .index(3usize as ::core::primitive::u8)
                                                .docs(
                                                    &["Not enough allowance to fulfill a request is available."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "InsufficientBalance",
                                        |v| {
                                            v
                                                .index(4usize as ::core::primitive::u8)
                                                .docs(
                                                    &["Not enough balance to fulfill a request is available."],
                                                )
                                        },
                                    )
                                    .variant(
                                        "InUse",
                                        |v| {
                                            v
                                                .index(5usize as ::core::primitive::u8)
                                                .docs(&["The asset ID is already taken."])
                                        },
                                    )
                                    .variant(
                                        "MinBalanceZero",
                                        |v| {
                                            v
                                                .index(6usize as ::core::primitive::u8)
                                                .docs(&["Minimum balance should be non-zero."])
                                        },
                                    )
                                    .variant(
                                        "ModuleError",
                                        |v| {
                                            v
                                                .index(7usize as ::core::primitive::u8)
                                                .fields(
                                                    ::scale_info::build::Fields::named()
                                                        .field(|f| f.ty::<u8>().name("pallet").type_name("u8"))
                                                        .field(|f| f.ty::<u16>().name("error").type_name("u16")),
                                                )
                                                .docs(
                                                    &[
                                                        "Unspecified pallet error, providing pallet index and error index.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "NoAccount",
                                        |v| {
                                            v
                                                .index(8usize as ::core::primitive::u8)
                                                .docs(&["The account to alter does not exist."])
                                        },
                                    )
                                    .variant(
                                        "NoPermission",
                                        |v| {
                                            v
                                                .index(9usize as ::core::primitive::u8)
                                                .docs(
                                                    &[
                                                        "The signing account has no permission to do the operation.",
                                                    ],
                                                )
                                        },
                                    )
                                    .variant(
                                        "Unknown",
                                        |v| {
                                            v
                                                .index(10usize as ::core::primitive::u8)
                                                .docs(&["The given asset ID is unknown."])
                                        },
                                    ),
                            )
                    }
                }
            };
            #[automatically_derived]
            impl ::core::fmt::Debug for FungiblesError {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    match self {
                        FungiblesError::AssetNotLive => {
                            ::core::fmt::Formatter::write_str(f, "AssetNotLive")
                        }
                        FungiblesError::BelowMinimum => {
                            ::core::fmt::Formatter::write_str(f, "BelowMinimum")
                        }
                        FungiblesError::DispatchError {
                            index: __self_0,
                            error: __self_1,
                        } => {
                            ::core::fmt::Formatter::debug_struct_field2_finish(
                                f,
                                "DispatchError",
                                "index",
                                __self_0,
                                "error",
                                &__self_1,
                            )
                        }
                        FungiblesError::InsufficientAllowance => {
                            ::core::fmt::Formatter::write_str(f, "InsufficientAllowance")
                        }
                        FungiblesError::InsufficientBalance => {
                            ::core::fmt::Formatter::write_str(f, "InsufficientBalance")
                        }
                        FungiblesError::InUse => {
                            ::core::fmt::Formatter::write_str(f, "InUse")
                        }
                        FungiblesError::MinBalanceZero => {
                            ::core::fmt::Formatter::write_str(f, "MinBalanceZero")
                        }
                        FungiblesError::ModuleError {
                            pallet: __self_0,
                            error: __self_1,
                        } => {
                            ::core::fmt::Formatter::debug_struct_field2_finish(
                                f,
                                "ModuleError",
                                "pallet",
                                __self_0,
                                "error",
                                &__self_1,
                            )
                        }
                        FungiblesError::NoAccount => {
                            ::core::fmt::Formatter::write_str(f, "NoAccount")
                        }
                        FungiblesError::NoPermission => {
                            ::core::fmt::Formatter::write_str(f, "NoPermission")
                        }
                        FungiblesError::Unknown => {
                            ::core::fmt::Formatter::write_str(f, "Unknown")
                        }
                    }
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for FungiblesError {}
            #[automatically_derived]
            impl ::core::clone::Clone for FungiblesError {
                #[inline]
                fn clone(&self) -> FungiblesError {
                    let _: ::core::clone::AssertParamIsClone<u8>;
                    let _: ::core::clone::AssertParamIsClone<u16>;
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for FungiblesError {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for FungiblesError {
                #[inline]
                fn eq(&self, other: &FungiblesError) -> bool {
                    let __self_tag = ::core::intrinsics::discriminant_value(self);
                    let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                    __self_tag == __arg1_tag
                        && match (self, other) {
                            (
                                FungiblesError::DispatchError {
                                    index: __self_0,
                                    error: __self_1,
                                },
                                FungiblesError::DispatchError {
                                    index: __arg1_0,
                                    error: __arg1_1,
                                },
                            ) => *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1,
                            (
                                FungiblesError::ModuleError {
                                    pallet: __self_0,
                                    error: __self_1,
                                },
                                FungiblesError::ModuleError {
                                    pallet: __arg1_0,
                                    error: __arg1_1,
                                },
                            ) => *__self_0 == *__arg1_0 && *__self_1 == *__arg1_1,
                            _ => true,
                        }
                }
            }
            #[automatically_derived]
            impl ::core::cmp::Eq for FungiblesError {
                #[inline]
                #[doc(hidden)]
                #[coverage(off)]
                fn assert_receiver_is_total_eq(&self) -> () {
                    let _: ::core::cmp::AssertParamIsEq<u8>;
                    let _: ::core::cmp::AssertParamIsEq<u16>;
                }
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for FungiblesError {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                FungiblesError::AssetNotLive => 0_usize,
                                FungiblesError::BelowMinimum => 0_usize,
                                FungiblesError::DispatchError { ref index, ref error } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(index))
                                        .saturating_add(::scale::Encode::size_hint(error))
                                }
                                FungiblesError::InsufficientAllowance => 0_usize,
                                FungiblesError::InsufficientBalance => 0_usize,
                                FungiblesError::InUse => 0_usize,
                                FungiblesError::MinBalanceZero => 0_usize,
                                FungiblesError::ModuleError { ref pallet, ref error } => {
                                    0_usize
                                        .saturating_add(::scale::Encode::size_hint(pallet))
                                        .saturating_add(::scale::Encode::size_hint(error))
                                }
                                FungiblesError::NoAccount => 0_usize,
                                FungiblesError::NoPermission => 0_usize,
                                FungiblesError::Unknown => 0_usize,
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            FungiblesError::AssetNotLive => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(0usize as ::core::primitive::u8);
                            }
                            FungiblesError::BelowMinimum => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(1usize as ::core::primitive::u8);
                            }
                            FungiblesError::DispatchError { ref index, ref error } => {
                                __codec_dest_edqy
                                    .push_byte(2usize as ::core::primitive::u8);
                                ::scale::Encode::encode_to(index, __codec_dest_edqy);
                                ::scale::Encode::encode_to(error, __codec_dest_edqy);
                            }
                            FungiblesError::InsufficientAllowance => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(3usize as ::core::primitive::u8);
                            }
                            FungiblesError::InsufficientBalance => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(4usize as ::core::primitive::u8);
                            }
                            FungiblesError::InUse => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(5usize as ::core::primitive::u8);
                            }
                            FungiblesError::MinBalanceZero => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(6usize as ::core::primitive::u8);
                            }
                            FungiblesError::ModuleError { ref pallet, ref error } => {
                                __codec_dest_edqy
                                    .push_byte(7usize as ::core::primitive::u8);
                                ::scale::Encode::encode_to(pallet, __codec_dest_edqy);
                                ::scale::Encode::encode_to(error, __codec_dest_edqy);
                            }
                            FungiblesError::NoAccount => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(8usize as ::core::primitive::u8);
                            }
                            FungiblesError::NoPermission => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(9usize as ::core::primitive::u8);
                            }
                            FungiblesError::Unknown => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(10usize as ::core::primitive::u8);
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for FungiblesError {}
            };
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Decode for FungiblesError {
                    fn decode<__CodecInputEdqy: ::scale::Input>(
                        __codec_input_edqy: &mut __CodecInputEdqy,
                    ) -> ::core::result::Result<Self, ::scale::Error> {
                        match __codec_input_edqy
                            .read_byte()
                            .map_err(|e| {
                                e
                                    .chain(
                                        "Could not decode `FungiblesError`, failed to read variant byte",
                                    )
                            })?
                        {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 0usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::AssetNotLive)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 1usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::BelowMinimum)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 2usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::DispatchError {
                                        index: {
                                            let __codec_res_edqy = <u8 as ::scale::Decode>::decode(
                                                __codec_input_edqy,
                                            );
                                            match __codec_res_edqy {
                                                ::core::result::Result::Err(e) => {
                                                    return ::core::result::Result::Err(
                                                        e
                                                            .chain(
                                                                "Could not decode `FungiblesError::DispatchError::index`",
                                                            ),
                                                    );
                                                }
                                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                                    __codec_res_edqy
                                                }
                                            }
                                        },
                                        error: {
                                            let __codec_res_edqy = <u8 as ::scale::Decode>::decode(
                                                __codec_input_edqy,
                                            );
                                            match __codec_res_edqy {
                                                ::core::result::Result::Err(e) => {
                                                    return ::core::result::Result::Err(
                                                        e
                                                            .chain(
                                                                "Could not decode `FungiblesError::DispatchError::error`",
                                                            ),
                                                    );
                                                }
                                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                                    __codec_res_edqy
                                                }
                                            }
                                        },
                                    })
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 3usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(
                                        FungiblesError::InsufficientAllowance,
                                    )
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 4usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(
                                        FungiblesError::InsufficientBalance,
                                    )
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 5usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::InUse)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 6usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::MinBalanceZero)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 7usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::ModuleError {
                                        pallet: {
                                            let __codec_res_edqy = <u8 as ::scale::Decode>::decode(
                                                __codec_input_edqy,
                                            );
                                            match __codec_res_edqy {
                                                ::core::result::Result::Err(e) => {
                                                    return ::core::result::Result::Err(
                                                        e
                                                            .chain(
                                                                "Could not decode `FungiblesError::ModuleError::pallet`",
                                                            ),
                                                    );
                                                }
                                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                                    __codec_res_edqy
                                                }
                                            }
                                        },
                                        error: {
                                            let __codec_res_edqy = <u16 as ::scale::Decode>::decode(
                                                __codec_input_edqy,
                                            );
                                            match __codec_res_edqy {
                                                ::core::result::Result::Err(e) => {
                                                    return ::core::result::Result::Err(
                                                        e
                                                            .chain(
                                                                "Could not decode `FungiblesError::ModuleError::error`",
                                                            ),
                                                    );
                                                }
                                                ::core::result::Result::Ok(__codec_res_edqy) => {
                                                    __codec_res_edqy
                                                }
                                            }
                                        },
                                    })
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 8usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::NoAccount)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 9usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::NoPermission)
                                })();
                            }
                            #[allow(clippy::unnecessary_cast)]
                            __codec_x_edqy if __codec_x_edqy
                                == 10usize as ::core::primitive::u8 => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Ok(FungiblesError::Unknown)
                                })();
                            }
                            _ => {
                                #[allow(clippy::redundant_closure_call)]
                                return (move || {
                                    ::core::result::Result::Err(
                                        <_ as ::core::convert::Into<
                                            _,
                                        >>::into(
                                            "Could not decode `FungiblesError`, variant doesn't exist",
                                        ),
                                    )
                                })();
                            }
                        }
                    }
                }
            };
            impl From<BalancesError> for FungiblesError {
                fn from(error: BalancesError) -> Self {
                    match error {
                        BalancesError::InsufficientBalance => {
                            FungiblesError::InsufficientBalance
                        }
                        _ => {
                            FungiblesError::ModuleError {
                                pallet: 40,
                                error: error as u16,
                            }
                        }
                    }
                }
            }
            impl From<dispatch_error::TokenError> for FungiblesError {
                fn from(error: dispatch_error::TokenError) -> Self {
                    match error {
                        dispatch_error::TokenError::BelowMinimum => {
                            FungiblesError::BelowMinimum
                        }
                        dispatch_error::TokenError::OnlyProvider => {
                            FungiblesError::InsufficientBalance
                        }
                        dispatch_error::TokenError::UnknownAsset => {
                            FungiblesError::Unknown
                        }
                        _ => {
                            FungiblesError::DispatchError {
                                index: 7,
                                error: error as u8,
                            }
                        }
                    }
                }
            }
            impl From<AssetsError> for FungiblesError {
                fn from(error: AssetsError) -> Self {
                    match error {
                        AssetsError::AssetNotLive => FungiblesError::AssetNotLive,
                        AssetsError::BalanceLow => FungiblesError::InsufficientBalance,
                        AssetsError::Unapproved => FungiblesError::InsufficientAllowance,
                        AssetsError::InUse => FungiblesError::InUse,
                        AssetsError::MinBalanceZero => FungiblesError::MinBalanceZero,
                        AssetsError::NoPermission => FungiblesError::NoPermission,
                        AssetsError::NoAccount => FungiblesError::NoAccount,
                        AssetsError::Unknown => FungiblesError::Unknown,
                        _ => {
                            FungiblesError::ModuleError {
                                pallet: 52,
                                error: error as u16,
                            }
                        }
                    }
                }
            }
            impl From<PopApiError> for FungiblesError {
                fn from(error: PopApiError) -> Self {
                    match error {
                        PopApiError::Assets(e) => e.into(),
                        PopApiError::Balances(e) => e.into(),
                        PopApiError::TokenError(e) => e.into(),
                        PopApiError::UnknownModuleStatusCode(e) => {
                            let pallet = (e / 1_000) as u8;
                            let error = (e % 1_000) as u16;
                            FungiblesError::ModuleError {
                                pallet,
                                error,
                            }
                        }
                        PopApiError::UnknownDispatchStatusCode(e) => {
                            let index = (e / 1_000_000) as u8;
                            let error = (3 % 1_000_000) as u8;
                            FungiblesError::DispatchError {
                                index,
                                error,
                            }
                        }
                        _ => ::core::panicking::panic("not yet implemented"),
                    }
                }
            }
        }
    }
    pub mod balances {
        use crate::{
            dispatch, primitives::MultiAddress, v0::RuntimeCall, AccountId, PopApiError,
            PopApiError::UnknownModuleStatusCode,
        };
        type Result<T> = core::result::Result<T, BalancesError>;
        pub fn transfer_keep_alive(
            dest: impl Into<MultiAddress<AccountId, ()>>,
            value: u128,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Balances(BalancesCall::TransferKeepAlive {
                        dest: dest.into(),
                        value,
                    }),
                )?,
            )
        }
        #[allow(dead_code)]
        pub enum BalancesCall {
            #[codec(index = 3)]
            TransferKeepAlive {
                dest: MultiAddress<AccountId, ()>,
                #[codec(compact)]
                value: u128,
            },
        }
        #[allow(deprecated)]
        const _: () = {
            #[allow(dead_code)]
            #[automatically_derived]
            impl ::scale::Encode for BalancesCall {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            BalancesCall::TransferKeepAlive { ref dest, ref value } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(dest))
                                    .saturating_add(
                                        ::scale::Encode::size_hint(
                                            &<<u128 as ::scale::HasCompact>::Type as ::scale::EncodeAsRef<
                                                '_,
                                                u128,
                                            >>::RefType::from(value),
                                        ),
                                    )
                            }
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        BalancesCall::TransferKeepAlive { ref dest, ref value } => {
                            __codec_dest_edqy.push_byte(3u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(dest, __codec_dest_edqy);
                            {
                                ::scale::Encode::encode_to(
                                    &<<u128 as ::scale::HasCompact>::Type as ::scale::EncodeAsRef<
                                        '_,
                                        u128,
                                    >>::RefType::from(value),
                                    __codec_dest_edqy,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for BalancesCall {}
        };
        pub enum BalancesError {
            /// Vesting balance too high to send value.
            VestingBalance,
            /// Account liquidity restrictions prevent withdrawal.
            LiquidityRestrictions,
            /// Balance too low to send value.
            InsufficientBalance,
            /// Value too low to create account due to existential deposit.
            ExistentialDeposit,
            /// Transfer/payment would kill account.
            Expendability,
            /// A vesting schedule already exists for this account.
            ExistingVestingSchedule,
            /// Beneficiary account must pre-exist.
            DeadAccount,
            /// Number of named reserves exceed `MaxReserves`.
            TooManyReserves,
            /// Number of holds exceed `VariantCountOf<T::RuntimeHoldReason>`.
            TooManyHolds,
            /// Number of freezes exceed `MaxFreezes`.
            TooManyFreezes,
            /// The issuance cannot be modified since it is already deactivated.
            IssuanceDeactivated,
            /// The delta cannot be zero.
            DeltaZero,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for BalancesError {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(
                            ::scale_info::Path::new_with_replace(
                                "BalancesError",
                                "pop_api::v0::balances",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            ::scale_info::build::Variants::new()
                                .variant(
                                    "VestingBalance",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .docs(&["Vesting balance too high to send value."])
                                    },
                                )
                                .variant(
                                    "LiquidityRestrictions",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .docs(
                                                &["Account liquidity restrictions prevent withdrawal."],
                                            )
                                    },
                                )
                                .variant(
                                    "InsufficientBalance",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .docs(&["Balance too low to send value."])
                                    },
                                )
                                .variant(
                                    "ExistentialDeposit",
                                    |v| {
                                        v
                                            .index(3usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Value too low to create account due to existential deposit.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "Expendability",
                                    |v| {
                                        v
                                            .index(4usize as ::core::primitive::u8)
                                            .docs(&["Transfer/payment would kill account."])
                                    },
                                )
                                .variant(
                                    "ExistingVestingSchedule",
                                    |v| {
                                        v
                                            .index(5usize as ::core::primitive::u8)
                                            .docs(
                                                &["A vesting schedule already exists for this account."],
                                            )
                                    },
                                )
                                .variant(
                                    "DeadAccount",
                                    |v| {
                                        v
                                            .index(6usize as ::core::primitive::u8)
                                            .docs(&["Beneficiary account must pre-exist."])
                                    },
                                )
                                .variant(
                                    "TooManyReserves",
                                    |v| {
                                        v
                                            .index(7usize as ::core::primitive::u8)
                                            .docs(&["Number of named reserves exceed `MaxReserves`."])
                                    },
                                )
                                .variant(
                                    "TooManyHolds",
                                    |v| {
                                        v
                                            .index(8usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Number of holds exceed `VariantCountOf<T::RuntimeHoldReason>`.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "TooManyFreezes",
                                    |v| {
                                        v
                                            .index(9usize as ::core::primitive::u8)
                                            .docs(&["Number of freezes exceed `MaxFreezes`."])
                                    },
                                )
                                .variant(
                                    "IssuanceDeactivated",
                                    |v| {
                                        v
                                            .index(10usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The issuance cannot be modified since it is already deactivated.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "DeltaZero",
                                    |v| {
                                        v
                                            .index(11usize as ::core::primitive::u8)
                                            .docs(&["The delta cannot be zero."])
                                    },
                                ),
                        )
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for BalancesError {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        BalancesError::VestingBalance => "VestingBalance",
                        BalancesError::LiquidityRestrictions => "LiquidityRestrictions",
                        BalancesError::InsufficientBalance => "InsufficientBalance",
                        BalancesError::ExistentialDeposit => "ExistentialDeposit",
                        BalancesError::Expendability => "Expendability",
                        BalancesError::ExistingVestingSchedule => {
                            "ExistingVestingSchedule"
                        }
                        BalancesError::DeadAccount => "DeadAccount",
                        BalancesError::TooManyReserves => "TooManyReserves",
                        BalancesError::TooManyHolds => "TooManyHolds",
                        BalancesError::TooManyFreezes => "TooManyFreezes",
                        BalancesError::IssuanceDeactivated => "IssuanceDeactivated",
                        BalancesError::DeltaZero => "DeltaZero",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for BalancesError {}
        #[automatically_derived]
        impl ::core::clone::Clone for BalancesError {
            #[inline]
            fn clone(&self) -> BalancesError {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for BalancesError {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for BalancesError {
            #[inline]
            fn eq(&self, other: &BalancesError) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for BalancesError {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for BalancesError {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            BalancesError::VestingBalance => 0_usize,
                            BalancesError::LiquidityRestrictions => 0_usize,
                            BalancesError::InsufficientBalance => 0_usize,
                            BalancesError::ExistentialDeposit => 0_usize,
                            BalancesError::Expendability => 0_usize,
                            BalancesError::ExistingVestingSchedule => 0_usize,
                            BalancesError::DeadAccount => 0_usize,
                            BalancesError::TooManyReserves => 0_usize,
                            BalancesError::TooManyHolds => 0_usize,
                            BalancesError::TooManyFreezes => 0_usize,
                            BalancesError::IssuanceDeactivated => 0_usize,
                            BalancesError::DeltaZero => 0_usize,
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        BalancesError::VestingBalance => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        }
                        BalancesError::LiquidityRestrictions => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        }
                        BalancesError::InsufficientBalance => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        }
                        BalancesError::ExistentialDeposit => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        }
                        BalancesError::Expendability => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                        }
                        BalancesError::ExistingVestingSchedule => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                        }
                        BalancesError::DeadAccount => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                        }
                        BalancesError::TooManyReserves => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                        }
                        BalancesError::TooManyHolds => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                        }
                        BalancesError::TooManyFreezes => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                        }
                        BalancesError::IssuanceDeactivated => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(10usize as ::core::primitive::u8);
                        }
                        BalancesError::DeltaZero => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(11usize as ::core::primitive::u8);
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for BalancesError {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for BalancesError {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    match __codec_input_edqy
                        .read_byte()
                        .map_err(|e| {
                            e
                                .chain(
                                    "Could not decode `BalancesError`, failed to read variant byte",
                                )
                        })?
                    {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 0usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::VestingBalance)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 1usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    BalancesError::LiquidityRestrictions,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 2usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    BalancesError::InsufficientBalance,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 3usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    BalancesError::ExistentialDeposit,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 4usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::Expendability)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 5usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    BalancesError::ExistingVestingSchedule,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 6usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::DeadAccount)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 7usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::TooManyReserves)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 8usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::TooManyHolds)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 9usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::TooManyFreezes)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 10usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    BalancesError::IssuanceDeactivated,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 11usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(BalancesError::DeltaZero)
                            })();
                        }
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Err(
                                    <_ as ::core::convert::Into<
                                        _,
                                    >>::into(
                                        "Could not decode `BalancesError`, variant doesn't exist",
                                    ),
                                )
                            })();
                        }
                    }
                }
            }
        };
        impl TryFrom<u32> for BalancesError {
            type Error = PopApiError;
            fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
                use BalancesError::*;
                match status_code {
                    0 => Ok(VestingBalance),
                    1 => Ok(LiquidityRestrictions),
                    2 => Ok(InsufficientBalance),
                    3 => Ok(ExistentialDeposit),
                    4 => Ok(Expendability),
                    5 => Ok(ExistingVestingSchedule),
                    6 => Ok(DeadAccount),
                    7 => Ok(TooManyReserves),
                    8 => Ok(TooManyHolds),
                    9 => Ok(TooManyFreezes),
                    10 => Ok(IssuanceDeactivated),
                    11 => Ok(DeltaZero),
                    _ => Err(UnknownModuleStatusCode(status_code)),
                }
            }
        }
        impl From<PopApiError> for BalancesError {
            fn from(error: PopApiError) -> Self {
                match error {
                    PopApiError::Balances(e) => e,
                    _ => ::core::panicking::panic("not yet implemented"),
                }
            }
        }
    }
    pub mod contracts {
        use crate::{PopApiError, PopApiError::UnknownModuleStatusCode};
        pub enum Error {
            /// Invalid schedule supplied, e.g. with zero weight of a basic operation.
            InvalidSchedule,
            /// Invalid combination of flags supplied to `seal_call` or `seal_delegate_call`.
            InvalidCallFlags,
            /// The executed contract exhausted its gas limit.
            OutOfGas,
            /// The output buffer supplied to a contract API call was too small.
            OutputBufferTooSmall,
            /// Performing the requested transfer failed. Probably because there isn't enough
            /// free balance in the sender's account.
            TransferFailed,
            /// Performing a call was denied because the calling depth reached the limit
            /// of what is specified in the schedule.
            MaxCallDepthReached,
            /// No contract was found at the specified address.
            ContractNotFound,
            /// The code supplied to `instantiate_with_code` exceeds the limit specified in the
            /// current schedule.
            CodeTooLarge,
            /// No code could be found at the supplied code hash.
            CodeNotFound,
            /// No code info could be found at the supplied code hash.
            CodeInfoNotFound,
            /// A buffer outside of sandbox memory was passed to a contract API function.
            OutOfBounds,
            /// Input passed to a contract API function failed to decode as expected type.
            DecodingFailed,
            /// Contract trapped during execution.
            ContractTrapped,
            /// The size defined in `T::MaxValueSize` was exceeded.
            ValueTooLarge,
            /// Termination of a contract is not allowed while the contract is already
            /// on the call stack. Can be triggered by `seal_terminate`.
            TerminatedWhileReentrant,
            /// `seal_call` forwarded this contracts input. It therefore is no longer available.
            InputForwarded,
            /// The subject passed to `seal_random` exceeds the limit.
            RandomSubjectTooLong,
            /// The amount of topics passed to `seal_deposit_events` exceeds the limit.
            TooManyTopics,
            /// The chain does not provide a chain extension. Calling the chain extension results
            /// in this error. Note that this usually  shouldn't happen as deploying such contracts
            /// is rejected.
            NoChainExtension,
            /// Failed to decode the XCM program.
            XCMDecodeFailed,
            /// A contract with the same AccountId already exists.
            DuplicateContract,
            /// A contract self destructed in its constructor.
            ///
            /// This can be triggered by a call to `seal_terminate`.
            TerminatedInConstructor,
            /// A call tried to invoke a contract that is flagged as non-reentrant.
            /// The only other cause is that a call from a contract into the runtime tried to call back
            /// into `pallet-contracts`. This would make the whole pallet reentrant with regard to
            /// contract code execution which is not supported.
            ReentranceDenied,
            /// Origin doesn't have enough balance to pay the required storage deposits.
            StorageDepositNotEnoughFunds,
            /// More storage was created than allowed by the storage deposit limit.
            StorageDepositLimitExhausted,
            /// Code removal was denied because the code is still in use by at least one contract.
            CodeInUse,
            /// The contract ran to completion but decided to revert its storage changes.
            /// Please note that this error is only returned from extrinsics. When called directly
            /// or via RPC an `Ok` will be returned. In this case the caller needs to inspect the flags
            /// to determine whether a reversion has taken place.
            ContractReverted,
            /// The contract's code was found to be invalid during validation.
            ///
            /// The most likely cause of this is that an API was used which is not supported by the
            /// node. This happens if an older node is used with a new version of ink!. Try updating
            /// your node to the newest available version.
            ///
            /// A more detailed error can be found on the node console if debug messages are enabled
            /// by supplying `-lruntime::contracts=debug`.
            CodeRejected,
            /// An indeterministic code was used in a context where this is not permitted.
            Indeterministic,
            /// A pending migration needs to complete before the extrinsic can be called.
            MigrationInProgress,
            /// Migrate dispatch call was attempted but no migration was performed.
            NoMigrationPerformed,
            /// The contract has reached its maximum number of delegate dependencies.
            MaxDelegateDependenciesReached,
            /// The dependency was not found in the contract's delegate dependencies.
            DelegateDependencyNotFound,
            /// The contract already depends on the given delegate dependency.
            DelegateDependencyAlreadyExists,
            /// Can not add a delegate dependency to the code hash of the contract itself.
            CannotAddSelfAsDelegateDependency,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for Error {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(
                            ::scale_info::Path::new_with_replace(
                                "Error",
                                "pop_api::v0::contracts",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            ::scale_info::build::Variants::new()
                                .variant(
                                    "InvalidSchedule",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Invalid schedule supplied, e.g. with zero weight of a basic operation.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "InvalidCallFlags",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Invalid combination of flags supplied to `seal_call` or `seal_delegate_call`.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "OutOfGas",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .docs(&["The executed contract exhausted its gas limit."])
                                    },
                                )
                                .variant(
                                    "OutputBufferTooSmall",
                                    |v| {
                                        v
                                            .index(3usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The output buffer supplied to a contract API call was too small.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "TransferFailed",
                                    |v| {
                                        v
                                            .index(4usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Performing the requested transfer failed. Probably because there isn't enough",
                                                    "free balance in the sender's account.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "MaxCallDepthReached",
                                    |v| {
                                        v
                                            .index(5usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Performing a call was denied because the calling depth reached the limit",
                                                    "of what is specified in the schedule.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "ContractNotFound",
                                    |v| {
                                        v
                                            .index(6usize as ::core::primitive::u8)
                                            .docs(&["No contract was found at the specified address."])
                                    },
                                )
                                .variant(
                                    "CodeTooLarge",
                                    |v| {
                                        v
                                            .index(7usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The code supplied to `instantiate_with_code` exceeds the limit specified in the",
                                                    "current schedule.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CodeNotFound",
                                    |v| {
                                        v
                                            .index(8usize as ::core::primitive::u8)
                                            .docs(
                                                &["No code could be found at the supplied code hash."],
                                            )
                                    },
                                )
                                .variant(
                                    "CodeInfoNotFound",
                                    |v| {
                                        v
                                            .index(9usize as ::core::primitive::u8)
                                            .docs(
                                                &["No code info could be found at the supplied code hash."],
                                            )
                                    },
                                )
                                .variant(
                                    "OutOfBounds",
                                    |v| {
                                        v
                                            .index(10usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "A buffer outside of sandbox memory was passed to a contract API function.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "DecodingFailed",
                                    |v| {
                                        v
                                            .index(11usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Input passed to a contract API function failed to decode as expected type.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "ContractTrapped",
                                    |v| {
                                        v
                                            .index(12usize as ::core::primitive::u8)
                                            .docs(&["Contract trapped during execution."])
                                    },
                                )
                                .variant(
                                    "ValueTooLarge",
                                    |v| {
                                        v
                                            .index(13usize as ::core::primitive::u8)
                                            .docs(
                                                &["The size defined in `T::MaxValueSize` was exceeded."],
                                            )
                                    },
                                )
                                .variant(
                                    "TerminatedWhileReentrant",
                                    |v| {
                                        v
                                            .index(14usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Termination of a contract is not allowed while the contract is already",
                                                    "on the call stack. Can be triggered by `seal_terminate`.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "InputForwarded",
                                    |v| {
                                        v
                                            .index(15usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "`seal_call` forwarded this contracts input. It therefore is no longer available.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "RandomSubjectTooLong",
                                    |v| {
                                        v
                                            .index(16usize as ::core::primitive::u8)
                                            .docs(
                                                &["The subject passed to `seal_random` exceeds the limit."],
                                            )
                                    },
                                )
                                .variant(
                                    "TooManyTopics",
                                    |v| {
                                        v
                                            .index(17usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The amount of topics passed to `seal_deposit_events` exceeds the limit.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "NoChainExtension",
                                    |v| {
                                        v
                                            .index(18usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The chain does not provide a chain extension. Calling the chain extension results",
                                                    "in this error. Note that this usually  shouldn't happen as deploying such contracts",
                                                    "is rejected.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "XCMDecodeFailed",
                                    |v| {
                                        v
                                            .index(19usize as ::core::primitive::u8)
                                            .docs(&["Failed to decode the XCM program."])
                                    },
                                )
                                .variant(
                                    "DuplicateContract",
                                    |v| {
                                        v
                                            .index(20usize as ::core::primitive::u8)
                                            .docs(
                                                &["A contract with the same AccountId already exists."],
                                            )
                                    },
                                )
                                .variant(
                                    "TerminatedInConstructor",
                                    |v| {
                                        v
                                            .index(21usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "A contract self destructed in its constructor.",
                                                    "",
                                                    "This can be triggered by a call to `seal_terminate`.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "ReentranceDenied",
                                    |v| {
                                        v
                                            .index(22usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "A call tried to invoke a contract that is flagged as non-reentrant.",
                                                    "The only other cause is that a call from a contract into the runtime tried to call back",
                                                    "into `pallet-contracts`. This would make the whole pallet reentrant with regard to",
                                                    "contract code execution which is not supported.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "StorageDepositNotEnoughFunds",
                                    |v| {
                                        v
                                            .index(23usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Origin doesn't have enough balance to pay the required storage deposits.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "StorageDepositLimitExhausted",
                                    |v| {
                                        v
                                            .index(24usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "More storage was created than allowed by the storage deposit limit.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CodeInUse",
                                    |v| {
                                        v
                                            .index(25usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Code removal was denied because the code is still in use by at least one contract.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "ContractReverted",
                                    |v| {
                                        v
                                            .index(26usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The contract ran to completion but decided to revert its storage changes.",
                                                    "Please note that this error is only returned from extrinsics. When called directly",
                                                    "or via RPC an `Ok` will be returned. In this case the caller needs to inspect the flags",
                                                    "to determine whether a reversion has taken place.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CodeRejected",
                                    |v| {
                                        v
                                            .index(27usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The contract's code was found to be invalid during validation.",
                                                    "",
                                                    "The most likely cause of this is that an API was used which is not supported by the",
                                                    "node. This happens if an older node is used with a new version of ink!. Try updating",
                                                    "your node to the newest available version.",
                                                    "",
                                                    "A more detailed error can be found on the node console if debug messages are enabled",
                                                    "by supplying `-lruntime::contracts=debug`.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "Indeterministic",
                                    |v| {
                                        v
                                            .index(28usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "An indeterministic code was used in a context where this is not permitted.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "MigrationInProgress",
                                    |v| {
                                        v
                                            .index(29usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "A pending migration needs to complete before the extrinsic can be called.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "NoMigrationPerformed",
                                    |v| {
                                        v
                                            .index(30usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Migrate dispatch call was attempted but no migration was performed.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "MaxDelegateDependenciesReached",
                                    |v| {
                                        v
                                            .index(31usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The contract has reached its maximum number of delegate dependencies.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "DelegateDependencyNotFound",
                                    |v| {
                                        v
                                            .index(32usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The dependency was not found in the contract's delegate dependencies.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "DelegateDependencyAlreadyExists",
                                    |v| {
                                        v
                                            .index(33usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The contract already depends on the given delegate dependency.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CannotAddSelfAsDelegateDependency",
                                    |v| {
                                        v
                                            .index(34usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Can not add a delegate dependency to the code hash of the contract itself.",
                                                ],
                                            )
                                    },
                                ),
                        )
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for Error {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        Error::InvalidSchedule => "InvalidSchedule",
                        Error::InvalidCallFlags => "InvalidCallFlags",
                        Error::OutOfGas => "OutOfGas",
                        Error::OutputBufferTooSmall => "OutputBufferTooSmall",
                        Error::TransferFailed => "TransferFailed",
                        Error::MaxCallDepthReached => "MaxCallDepthReached",
                        Error::ContractNotFound => "ContractNotFound",
                        Error::CodeTooLarge => "CodeTooLarge",
                        Error::CodeNotFound => "CodeNotFound",
                        Error::CodeInfoNotFound => "CodeInfoNotFound",
                        Error::OutOfBounds => "OutOfBounds",
                        Error::DecodingFailed => "DecodingFailed",
                        Error::ContractTrapped => "ContractTrapped",
                        Error::ValueTooLarge => "ValueTooLarge",
                        Error::TerminatedWhileReentrant => "TerminatedWhileReentrant",
                        Error::InputForwarded => "InputForwarded",
                        Error::RandomSubjectTooLong => "RandomSubjectTooLong",
                        Error::TooManyTopics => "TooManyTopics",
                        Error::NoChainExtension => "NoChainExtension",
                        Error::XCMDecodeFailed => "XCMDecodeFailed",
                        Error::DuplicateContract => "DuplicateContract",
                        Error::TerminatedInConstructor => "TerminatedInConstructor",
                        Error::ReentranceDenied => "ReentranceDenied",
                        Error::StorageDepositNotEnoughFunds => {
                            "StorageDepositNotEnoughFunds"
                        }
                        Error::StorageDepositLimitExhausted => {
                            "StorageDepositLimitExhausted"
                        }
                        Error::CodeInUse => "CodeInUse",
                        Error::ContractReverted => "ContractReverted",
                        Error::CodeRejected => "CodeRejected",
                        Error::Indeterministic => "Indeterministic",
                        Error::MigrationInProgress => "MigrationInProgress",
                        Error::NoMigrationPerformed => "NoMigrationPerformed",
                        Error::MaxDelegateDependenciesReached => {
                            "MaxDelegateDependenciesReached"
                        }
                        Error::DelegateDependencyNotFound => "DelegateDependencyNotFound",
                        Error::DelegateDependencyAlreadyExists => {
                            "DelegateDependencyAlreadyExists"
                        }
                        Error::CannotAddSelfAsDelegateDependency => {
                            "CannotAddSelfAsDelegateDependency"
                        }
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Error {}
        #[automatically_derived]
        impl ::core::clone::Clone for Error {
            #[inline]
            fn clone(&self) -> Error {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Error {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Error {
            #[inline]
            fn eq(&self, other: &Error) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Error {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for Error {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            Error::InvalidSchedule => 0_usize,
                            Error::InvalidCallFlags => 0_usize,
                            Error::OutOfGas => 0_usize,
                            Error::OutputBufferTooSmall => 0_usize,
                            Error::TransferFailed => 0_usize,
                            Error::MaxCallDepthReached => 0_usize,
                            Error::ContractNotFound => 0_usize,
                            Error::CodeTooLarge => 0_usize,
                            Error::CodeNotFound => 0_usize,
                            Error::CodeInfoNotFound => 0_usize,
                            Error::OutOfBounds => 0_usize,
                            Error::DecodingFailed => 0_usize,
                            Error::ContractTrapped => 0_usize,
                            Error::ValueTooLarge => 0_usize,
                            Error::TerminatedWhileReentrant => 0_usize,
                            Error::InputForwarded => 0_usize,
                            Error::RandomSubjectTooLong => 0_usize,
                            Error::TooManyTopics => 0_usize,
                            Error::NoChainExtension => 0_usize,
                            Error::XCMDecodeFailed => 0_usize,
                            Error::DuplicateContract => 0_usize,
                            Error::TerminatedInConstructor => 0_usize,
                            Error::ReentranceDenied => 0_usize,
                            Error::StorageDepositNotEnoughFunds => 0_usize,
                            Error::StorageDepositLimitExhausted => 0_usize,
                            Error::CodeInUse => 0_usize,
                            Error::ContractReverted => 0_usize,
                            Error::CodeRejected => 0_usize,
                            Error::Indeterministic => 0_usize,
                            Error::MigrationInProgress => 0_usize,
                            Error::NoMigrationPerformed => 0_usize,
                            Error::MaxDelegateDependenciesReached => 0_usize,
                            Error::DelegateDependencyNotFound => 0_usize,
                            Error::DelegateDependencyAlreadyExists => 0_usize,
                            Error::CannotAddSelfAsDelegateDependency => 0_usize,
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        Error::InvalidSchedule => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        }
                        Error::InvalidCallFlags => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        }
                        Error::OutOfGas => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        }
                        Error::OutputBufferTooSmall => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        }
                        Error::TransferFailed => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                        }
                        Error::MaxCallDepthReached => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                        }
                        Error::ContractNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                        }
                        Error::CodeTooLarge => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                        }
                        Error::CodeNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                        }
                        Error::CodeInfoNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                        }
                        Error::OutOfBounds => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(10usize as ::core::primitive::u8);
                        }
                        Error::DecodingFailed => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(11usize as ::core::primitive::u8);
                        }
                        Error::ContractTrapped => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(12usize as ::core::primitive::u8);
                        }
                        Error::ValueTooLarge => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(13usize as ::core::primitive::u8);
                        }
                        Error::TerminatedWhileReentrant => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(14usize as ::core::primitive::u8);
                        }
                        Error::InputForwarded => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(15usize as ::core::primitive::u8);
                        }
                        Error::RandomSubjectTooLong => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(16usize as ::core::primitive::u8);
                        }
                        Error::TooManyTopics => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(17usize as ::core::primitive::u8);
                        }
                        Error::NoChainExtension => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(18usize as ::core::primitive::u8);
                        }
                        Error::XCMDecodeFailed => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(19usize as ::core::primitive::u8);
                        }
                        Error::DuplicateContract => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(20usize as ::core::primitive::u8);
                        }
                        Error::TerminatedInConstructor => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(21usize as ::core::primitive::u8);
                        }
                        Error::ReentranceDenied => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(22usize as ::core::primitive::u8);
                        }
                        Error::StorageDepositNotEnoughFunds => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(23usize as ::core::primitive::u8);
                        }
                        Error::StorageDepositLimitExhausted => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(24usize as ::core::primitive::u8);
                        }
                        Error::CodeInUse => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(25usize as ::core::primitive::u8);
                        }
                        Error::ContractReverted => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(26usize as ::core::primitive::u8);
                        }
                        Error::CodeRejected => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(27usize as ::core::primitive::u8);
                        }
                        Error::Indeterministic => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(28usize as ::core::primitive::u8);
                        }
                        Error::MigrationInProgress => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(29usize as ::core::primitive::u8);
                        }
                        Error::NoMigrationPerformed => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(30usize as ::core::primitive::u8);
                        }
                        Error::MaxDelegateDependenciesReached => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(31usize as ::core::primitive::u8);
                        }
                        Error::DelegateDependencyNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(32usize as ::core::primitive::u8);
                        }
                        Error::DelegateDependencyAlreadyExists => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(33usize as ::core::primitive::u8);
                        }
                        Error::CannotAddSelfAsDelegateDependency => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(34usize as ::core::primitive::u8);
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for Error {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for Error {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    match __codec_input_edqy
                        .read_byte()
                        .map_err(|e| {
                            e
                                .chain(
                                    "Could not decode `Error`, failed to read variant byte",
                                )
                        })?
                    {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 0usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InvalidSchedule)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 1usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InvalidCallFlags)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 2usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::OutOfGas)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 3usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::OutputBufferTooSmall)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 4usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TransferFailed)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 5usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MaxCallDepthReached)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 6usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ContractNotFound)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 7usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CodeTooLarge)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 8usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CodeNotFound)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 9usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CodeInfoNotFound)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 10usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::OutOfBounds)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 11usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::DecodingFailed)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 12usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ContractTrapped)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 13usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ValueTooLarge)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 14usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TerminatedWhileReentrant)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 15usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InputForwarded)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 16usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::RandomSubjectTooLong)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 17usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TooManyTopics)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 18usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NoChainExtension)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 19usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::XCMDecodeFailed)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 20usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::DuplicateContract)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 21usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TerminatedInConstructor)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 22usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ReentranceDenied)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 23usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::StorageDepositNotEnoughFunds,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 24usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::StorageDepositLimitExhausted,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 25usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CodeInUse)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 26usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ContractReverted)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 27usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CodeRejected)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 28usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::Indeterministic)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 29usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MigrationInProgress)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 30usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NoMigrationPerformed)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 31usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::MaxDelegateDependenciesReached,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 32usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::DelegateDependencyNotFound,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 33usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::DelegateDependencyAlreadyExists,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 34usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::CannotAddSelfAsDelegateDependency,
                                )
                            })();
                        }
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Err(
                                    <_ as ::core::convert::Into<
                                        _,
                                    >>::into("Could not decode `Error`, variant doesn't exist"),
                                )
                            })();
                        }
                    }
                }
            }
        };
        impl TryFrom<u32> for Error {
            type Error = PopApiError;
            fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
                use Error::*;
                match status_code {
                    0 => Ok(InvalidSchedule),
                    1 => Ok(InvalidCallFlags),
                    2 => Ok(OutOfGas),
                    3 => Ok(OutputBufferTooSmall),
                    4 => Ok(TransferFailed),
                    5 => Ok(MaxCallDepthReached),
                    6 => Ok(ContractNotFound),
                    7 => Ok(CodeTooLarge),
                    8 => Ok(CodeNotFound),
                    9 => Ok(CodeInfoNotFound),
                    10 => Ok(OutOfBounds),
                    11 => Ok(DecodingFailed),
                    12 => Ok(ContractTrapped),
                    13 => Ok(ValueTooLarge),
                    14 => Ok(TerminatedWhileReentrant),
                    15 => Ok(InputForwarded),
                    16 => Ok(RandomSubjectTooLong),
                    17 => Ok(TooManyTopics),
                    18 => Ok(NoChainExtension),
                    19 => Ok(XCMDecodeFailed),
                    20 => Ok(DuplicateContract),
                    21 => Ok(TerminatedInConstructor),
                    22 => Ok(ReentranceDenied),
                    23 => Ok(StorageDepositNotEnoughFunds),
                    24 => Ok(StorageDepositLimitExhausted),
                    25 => Ok(CodeInUse),
                    26 => Ok(ContractReverted),
                    27 => Ok(CodeRejected),
                    28 => Ok(Indeterministic),
                    29 => Ok(MigrationInProgress),
                    30 => Ok(NoMigrationPerformed),
                    31 => Ok(MaxDelegateDependenciesReached),
                    32 => Ok(DelegateDependencyNotFound),
                    33 => Ok(DelegateDependencyAlreadyExists),
                    34 => Ok(CannotAddSelfAsDelegateDependency),
                    _ => Err(UnknownModuleStatusCode(status_code)),
                }
            }
        }
        impl From<PopApiError> for Error {
            fn from(error: PopApiError) -> Self {
                match error {
                    PopApiError::Contracts(e) => e,
                    _ => {
                        ::core::panicking::panic_fmt(
                            format_args!("expected balances error"),
                        );
                    }
                }
            }
        }
    }
    pub mod cross_chain {
        pub mod coretime {
            use crate::{
                primitives::cross_chain::{
                    CrossChainMessage, OnDemand, RelayChainMessage,
                },
                send_xcm,
            };
            /// Send a cross-chain message to place a sport order for instantaneous coretime.
            pub fn place_spot_order(
                max_amount: u128,
                para_id: u32,
            ) -> crate::cross_chain::Result<()> {
                Ok(
                    send_xcm(
                        CrossChainMessage::Relay(
                            RelayChainMessage::OnDemand(OnDemand::PlaceOrderKeepAlive {
                                max_amount,
                                para_id,
                            }),
                        ),
                    )?,
                )
            }
        }
        use crate::{PopApiError::UnknownModuleStatusCode, *};
        type Result<T> = core::result::Result<T, Error>;
        pub enum Error {
            /// The desired destination was unreachable, generally because there is a no way of routing
            /// to it.
            Unreachable,
            /// There was some other issue (i.e. not to do with routing) in sending the message.
            /// Perhaps a lack of space for buffering the message.
            SendFailure,
            /// The message execution fails the filter.
            Filtered,
            /// The message's weight could not be determined.
            UnweighableMessage,
            /// The destination `Location` provided cannot be inverted.
            DestinationNotInvertible,
            /// The assets to be sent are empty.
            Empty,
            /// Could not re-anchor the assets to declare the fees for the destination chain.
            CannotReanchor,
            /// Too many assets have been attempted for transfer.
            TooManyAssets,
            /// Origin is invalid for sending.
            InvalidOrigin,
            /// The version of the `Versioned` value used is not able to be interpreted.
            BadVersion,
            /// The given location could not be used (e.g. because it cannot be expressed in the
            /// desired version of XCM).
            BadLocation,
            /// The referenced subscription could not be found.
            NoSubscription,
            /// The location is invalid since it already has a subscription from us.
            AlreadySubscribed,
            /// Could not check-out the assets for teleportation to the destination chain.
            CannotCheckOutTeleport,
            /// The owner does not own (all) of the asset that they wish to do the operation on.
            LowBalance,
            /// The asset owner has too many locks on the asset.
            TooManyLocks,
            /// The given account is not an identifiable sovereign account for any location.
            AccountNotSovereign,
            /// The operation required fees to be paid which the initiator could not meet.
            FeesNotMet,
            /// A remote lock with the corresponding data could not be found.
            LockNotFound,
            /// The unlock operation cannot succeed because there are still consumers of the lock.
            InUse,
            /// Invalid non-concrete asset.
            InvalidAssetNotConcrete,
            /// Invalid asset, reserve chain could not be determined for it.
            InvalidAssetUnknownReserve,
            /// Invalid asset, do not support remote asset reserves with different fees reserves.
            InvalidAssetUnsupportedReserve,
            /// Too many assets with different reserve locations have been attempted for transfer.
            TooManyReserves,
            /// Local XCM execution incomplete.
            LocalExecutionIncomplete,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for Error {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(
                            ::scale_info::Path::new_with_replace(
                                "Error",
                                "pop_api::v0::cross_chain",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            ::scale_info::build::Variants::new()
                                .variant(
                                    "Unreachable",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The desired destination was unreachable, generally because there is a no way of routing",
                                                    "to it.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "SendFailure",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "There was some other issue (i.e. not to do with routing) in sending the message.",
                                                    "Perhaps a lack of space for buffering the message.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "Filtered",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .docs(&["The message execution fails the filter."])
                                    },
                                )
                                .variant(
                                    "UnweighableMessage",
                                    |v| {
                                        v
                                            .index(3usize as ::core::primitive::u8)
                                            .docs(&["The message's weight could not be determined."])
                                    },
                                )
                                .variant(
                                    "DestinationNotInvertible",
                                    |v| {
                                        v
                                            .index(4usize as ::core::primitive::u8)
                                            .docs(
                                                &["The destination `Location` provided cannot be inverted."],
                                            )
                                    },
                                )
                                .variant(
                                    "Empty",
                                    |v| {
                                        v
                                            .index(5usize as ::core::primitive::u8)
                                            .docs(&["The assets to be sent are empty."])
                                    },
                                )
                                .variant(
                                    "CannotReanchor",
                                    |v| {
                                        v
                                            .index(6usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Could not re-anchor the assets to declare the fees for the destination chain.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "TooManyAssets",
                                    |v| {
                                        v
                                            .index(7usize as ::core::primitive::u8)
                                            .docs(
                                                &["Too many assets have been attempted for transfer."],
                                            )
                                    },
                                )
                                .variant(
                                    "InvalidOrigin",
                                    |v| {
                                        v
                                            .index(8usize as ::core::primitive::u8)
                                            .docs(&["Origin is invalid for sending."])
                                    },
                                )
                                .variant(
                                    "BadVersion",
                                    |v| {
                                        v
                                            .index(9usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The version of the `Versioned` value used is not able to be interpreted.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "BadLocation",
                                    |v| {
                                        v
                                            .index(10usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The given location could not be used (e.g. because it cannot be expressed in the",
                                                    "desired version of XCM).",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "NoSubscription",
                                    |v| {
                                        v
                                            .index(11usize as ::core::primitive::u8)
                                            .docs(&["The referenced subscription could not be found."])
                                    },
                                )
                                .variant(
                                    "AlreadySubscribed",
                                    |v| {
                                        v
                                            .index(12usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The location is invalid since it already has a subscription from us.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CannotCheckOutTeleport",
                                    |v| {
                                        v
                                            .index(13usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Could not check-out the assets for teleportation to the destination chain.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "LowBalance",
                                    |v| {
                                        v
                                            .index(14usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The owner does not own (all) of the asset that they wish to do the operation on.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "TooManyLocks",
                                    |v| {
                                        v
                                            .index(15usize as ::core::primitive::u8)
                                            .docs(&["The asset owner has too many locks on the asset."])
                                    },
                                )
                                .variant(
                                    "AccountNotSovereign",
                                    |v| {
                                        v
                                            .index(16usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The given account is not an identifiable sovereign account for any location.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "FeesNotMet",
                                    |v| {
                                        v
                                            .index(17usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The operation required fees to be paid which the initiator could not meet.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "LockNotFound",
                                    |v| {
                                        v
                                            .index(18usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "A remote lock with the corresponding data could not be found.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "InUse",
                                    |v| {
                                        v
                                            .index(19usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The unlock operation cannot succeed because there are still consumers of the lock.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "InvalidAssetNotConcrete",
                                    |v| {
                                        v
                                            .index(20usize as ::core::primitive::u8)
                                            .docs(&["Invalid non-concrete asset."])
                                    },
                                )
                                .variant(
                                    "InvalidAssetUnknownReserve",
                                    |v| {
                                        v
                                            .index(21usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Invalid asset, reserve chain could not be determined for it.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "InvalidAssetUnsupportedReserve",
                                    |v| {
                                        v
                                            .index(22usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Invalid asset, do not support remote asset reserves with different fees reserves.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "TooManyReserves",
                                    |v| {
                                        v
                                            .index(23usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Too many assets with different reserve locations have been attempted for transfer.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "LocalExecutionIncomplete",
                                    |v| {
                                        v
                                            .index(24usize as ::core::primitive::u8)
                                            .docs(&["Local XCM execution incomplete."])
                                    },
                                ),
                        )
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for Error {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        Error::Unreachable => "Unreachable",
                        Error::SendFailure => "SendFailure",
                        Error::Filtered => "Filtered",
                        Error::UnweighableMessage => "UnweighableMessage",
                        Error::DestinationNotInvertible => "DestinationNotInvertible",
                        Error::Empty => "Empty",
                        Error::CannotReanchor => "CannotReanchor",
                        Error::TooManyAssets => "TooManyAssets",
                        Error::InvalidOrigin => "InvalidOrigin",
                        Error::BadVersion => "BadVersion",
                        Error::BadLocation => "BadLocation",
                        Error::NoSubscription => "NoSubscription",
                        Error::AlreadySubscribed => "AlreadySubscribed",
                        Error::CannotCheckOutTeleport => "CannotCheckOutTeleport",
                        Error::LowBalance => "LowBalance",
                        Error::TooManyLocks => "TooManyLocks",
                        Error::AccountNotSovereign => "AccountNotSovereign",
                        Error::FeesNotMet => "FeesNotMet",
                        Error::LockNotFound => "LockNotFound",
                        Error::InUse => "InUse",
                        Error::InvalidAssetNotConcrete => "InvalidAssetNotConcrete",
                        Error::InvalidAssetUnknownReserve => "InvalidAssetUnknownReserve",
                        Error::InvalidAssetUnsupportedReserve => {
                            "InvalidAssetUnsupportedReserve"
                        }
                        Error::TooManyReserves => "TooManyReserves",
                        Error::LocalExecutionIncomplete => "LocalExecutionIncomplete",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Error {}
        #[automatically_derived]
        impl ::core::clone::Clone for Error {
            #[inline]
            fn clone(&self) -> Error {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Error {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Error {
            #[inline]
            fn eq(&self, other: &Error) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Error {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for Error {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            Error::Unreachable => 0_usize,
                            Error::SendFailure => 0_usize,
                            Error::Filtered => 0_usize,
                            Error::UnweighableMessage => 0_usize,
                            Error::DestinationNotInvertible => 0_usize,
                            Error::Empty => 0_usize,
                            Error::CannotReanchor => 0_usize,
                            Error::TooManyAssets => 0_usize,
                            Error::InvalidOrigin => 0_usize,
                            Error::BadVersion => 0_usize,
                            Error::BadLocation => 0_usize,
                            Error::NoSubscription => 0_usize,
                            Error::AlreadySubscribed => 0_usize,
                            Error::CannotCheckOutTeleport => 0_usize,
                            Error::LowBalance => 0_usize,
                            Error::TooManyLocks => 0_usize,
                            Error::AccountNotSovereign => 0_usize,
                            Error::FeesNotMet => 0_usize,
                            Error::LockNotFound => 0_usize,
                            Error::InUse => 0_usize,
                            Error::InvalidAssetNotConcrete => 0_usize,
                            Error::InvalidAssetUnknownReserve => 0_usize,
                            Error::InvalidAssetUnsupportedReserve => 0_usize,
                            Error::TooManyReserves => 0_usize,
                            Error::LocalExecutionIncomplete => 0_usize,
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        Error::Unreachable => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        }
                        Error::SendFailure => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        }
                        Error::Filtered => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        }
                        Error::UnweighableMessage => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        }
                        Error::DestinationNotInvertible => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                        }
                        Error::Empty => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                        }
                        Error::CannotReanchor => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                        }
                        Error::TooManyAssets => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                        }
                        Error::InvalidOrigin => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                        }
                        Error::BadVersion => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                        }
                        Error::BadLocation => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(10usize as ::core::primitive::u8);
                        }
                        Error::NoSubscription => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(11usize as ::core::primitive::u8);
                        }
                        Error::AlreadySubscribed => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(12usize as ::core::primitive::u8);
                        }
                        Error::CannotCheckOutTeleport => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(13usize as ::core::primitive::u8);
                        }
                        Error::LowBalance => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(14usize as ::core::primitive::u8);
                        }
                        Error::TooManyLocks => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(15usize as ::core::primitive::u8);
                        }
                        Error::AccountNotSovereign => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(16usize as ::core::primitive::u8);
                        }
                        Error::FeesNotMet => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(17usize as ::core::primitive::u8);
                        }
                        Error::LockNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(18usize as ::core::primitive::u8);
                        }
                        Error::InUse => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(19usize as ::core::primitive::u8);
                        }
                        Error::InvalidAssetNotConcrete => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(20usize as ::core::primitive::u8);
                        }
                        Error::InvalidAssetUnknownReserve => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(21usize as ::core::primitive::u8);
                        }
                        Error::InvalidAssetUnsupportedReserve => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(22usize as ::core::primitive::u8);
                        }
                        Error::TooManyReserves => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(23usize as ::core::primitive::u8);
                        }
                        Error::LocalExecutionIncomplete => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(24usize as ::core::primitive::u8);
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for Error {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for Error {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    match __codec_input_edqy
                        .read_byte()
                        .map_err(|e| {
                            e
                                .chain(
                                    "Could not decode `Error`, failed to read variant byte",
                                )
                        })?
                    {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 0usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::Unreachable)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 1usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::SendFailure)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 2usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::Filtered)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 3usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::UnweighableMessage)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 4usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::DestinationNotInvertible)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 5usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::Empty)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 6usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CannotReanchor)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 7usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TooManyAssets)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 8usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InvalidOrigin)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 9usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::BadVersion)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 10usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::BadLocation)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 11usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NoSubscription)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 12usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::AlreadySubscribed)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 13usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CannotCheckOutTeleport)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 14usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::LowBalance)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 15usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TooManyLocks)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 16usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::AccountNotSovereign)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 17usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::FeesNotMet)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 18usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::LockNotFound)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 19usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InUse)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 20usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InvalidAssetNotConcrete)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 21usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::InvalidAssetUnknownReserve,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 22usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::InvalidAssetUnsupportedReserve,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 23usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::TooManyReserves)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 24usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::LocalExecutionIncomplete)
                            })();
                        }
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Err(
                                    <_ as ::core::convert::Into<
                                        _,
                                    >>::into("Could not decode `Error`, variant doesn't exist"),
                                )
                            })();
                        }
                    }
                }
            }
        };
        impl TryFrom<u32> for Error {
            type Error = PopApiError;
            fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
                use Error::*;
                match status_code {
                    0 => Ok(Unreachable),
                    1 => Ok(SendFailure),
                    2 => Ok(Filtered),
                    3 => Ok(UnweighableMessage),
                    4 => Ok(DestinationNotInvertible),
                    5 => Ok(Empty),
                    6 => Ok(CannotReanchor),
                    7 => Ok(TooManyAssets),
                    8 => Ok(InvalidOrigin),
                    9 => Ok(BadVersion),
                    10 => Ok(BadLocation),
                    11 => Ok(NoSubscription),
                    12 => Ok(AlreadySubscribed),
                    13 => Ok(CannotCheckOutTeleport),
                    14 => Ok(LowBalance),
                    15 => Ok(TooManyLocks),
                    16 => Ok(AccountNotSovereign),
                    17 => Ok(FeesNotMet),
                    18 => Ok(LockNotFound),
                    19 => Ok(InUse),
                    20 => Ok(InvalidAssetNotConcrete),
                    21 => Ok(InvalidAssetUnknownReserve),
                    22 => Ok(InvalidAssetUnsupportedReserve),
                    23 => Ok(TooManyReserves),
                    _ => Err(UnknownModuleStatusCode(status_code)),
                }
            }
        }
        impl From<PopApiError> for Error {
            fn from(error: PopApiError) -> Self {
                match error {
                    PopApiError::Xcm(e) => e,
                    _ => {
                        ::core::panicking::panic_fmt(format_args!("expected xcm error"));
                    }
                }
            }
        }
    }
    pub mod dispatch_error {
        use super::*;
        pub enum TokenError {
            /// Funds are unavailable.
            FundsUnavailable,
            /// Some part of the balance gives the only provider reference to the account and thus cannot
            /// be (re)moved.
            OnlyProvider,
            /// Account cannot exist with the funds that would be given.
            BelowMinimum,
            /// Account cannot be created.
            CannotCreate,
            /// The asset in question is unknown.
            UnknownAsset,
            /// Funds exist but are frozen.
            Frozen,
            /// Operation is not supported by the asset.
            Unsupported,
            /// Account cannot be created for a held balance.
            CannotCreateHold,
            /// Withdrawal would cause unwanted loss of account.
            NotExpendable,
            /// Account cannot receive the assets.
            Blocked,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for TokenError {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(
                            ::scale_info::Path::new_with_replace(
                                "TokenError",
                                "pop_api::v0::dispatch_error",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            ::scale_info::build::Variants::new()
                                .variant(
                                    "FundsUnavailable",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .docs(&["Funds are unavailable."])
                                    },
                                )
                                .variant(
                                    "OnlyProvider",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Some part of the balance gives the only provider reference to the account and thus cannot",
                                                    "be (re)moved.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "BelowMinimum",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Account cannot exist with the funds that would be given.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CannotCreate",
                                    |v| {
                                        v
                                            .index(3usize as ::core::primitive::u8)
                                            .docs(&["Account cannot be created."])
                                    },
                                )
                                .variant(
                                    "UnknownAsset",
                                    |v| {
                                        v
                                            .index(4usize as ::core::primitive::u8)
                                            .docs(&["The asset in question is unknown."])
                                    },
                                )
                                .variant(
                                    "Frozen",
                                    |v| {
                                        v
                                            .index(5usize as ::core::primitive::u8)
                                            .docs(&["Funds exist but are frozen."])
                                    },
                                )
                                .variant(
                                    "Unsupported",
                                    |v| {
                                        v
                                            .index(6usize as ::core::primitive::u8)
                                            .docs(&["Operation is not supported by the asset."])
                                    },
                                )
                                .variant(
                                    "CannotCreateHold",
                                    |v| {
                                        v
                                            .index(7usize as ::core::primitive::u8)
                                            .docs(&["Account cannot be created for a held balance."])
                                    },
                                )
                                .variant(
                                    "NotExpendable",
                                    |v| {
                                        v
                                            .index(8usize as ::core::primitive::u8)
                                            .docs(&["Withdrawal would cause unwanted loss of account."])
                                    },
                                )
                                .variant(
                                    "Blocked",
                                    |v| {
                                        v
                                            .index(9usize as ::core::primitive::u8)
                                            .docs(&["Account cannot receive the assets."])
                                    },
                                ),
                        )
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for TokenError {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        TokenError::FundsUnavailable => "FundsUnavailable",
                        TokenError::OnlyProvider => "OnlyProvider",
                        TokenError::BelowMinimum => "BelowMinimum",
                        TokenError::CannotCreate => "CannotCreate",
                        TokenError::UnknownAsset => "UnknownAsset",
                        TokenError::Frozen => "Frozen",
                        TokenError::Unsupported => "Unsupported",
                        TokenError::CannotCreateHold => "CannotCreateHold",
                        TokenError::NotExpendable => "NotExpendable",
                        TokenError::Blocked => "Blocked",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for TokenError {}
        #[automatically_derived]
        impl ::core::clone::Clone for TokenError {
            #[inline]
            fn clone(&self) -> TokenError {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for TokenError {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for TokenError {
            #[inline]
            fn eq(&self, other: &TokenError) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for TokenError {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for TokenError {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            TokenError::FundsUnavailable => 0_usize,
                            TokenError::OnlyProvider => 0_usize,
                            TokenError::BelowMinimum => 0_usize,
                            TokenError::CannotCreate => 0_usize,
                            TokenError::UnknownAsset => 0_usize,
                            TokenError::Frozen => 0_usize,
                            TokenError::Unsupported => 0_usize,
                            TokenError::CannotCreateHold => 0_usize,
                            TokenError::NotExpendable => 0_usize,
                            TokenError::Blocked => 0_usize,
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        TokenError::FundsUnavailable => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        }
                        TokenError::OnlyProvider => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        }
                        TokenError::BelowMinimum => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        }
                        TokenError::CannotCreate => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        }
                        TokenError::UnknownAsset => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                        }
                        TokenError::Frozen => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                        }
                        TokenError::Unsupported => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                        }
                        TokenError::CannotCreateHold => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                        }
                        TokenError::NotExpendable => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                        }
                        TokenError::Blocked => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for TokenError {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for TokenError {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    match __codec_input_edqy
                        .read_byte()
                        .map_err(|e| {
                            e
                                .chain(
                                    "Could not decode `TokenError`, failed to read variant byte",
                                )
                        })?
                    {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 0usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::FundsUnavailable)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 1usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::OnlyProvider)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 2usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::BelowMinimum)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 3usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::CannotCreate)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 4usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::UnknownAsset)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 5usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::Frozen)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 6usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::Unsupported)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 7usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::CannotCreateHold)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 8usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::NotExpendable)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 9usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(TokenError::Blocked)
                            })();
                        }
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Err(
                                    <_ as ::core::convert::Into<
                                        _,
                                    >>::into(
                                        "Could not decode `TokenError`, variant doesn't exist",
                                    ),
                                )
                            })();
                        }
                    }
                }
            }
        };
        impl TryFrom<u32> for TokenError {
            type Error = crate::PopApiError;
            fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
                use TokenError::*;
                match status_code {
                    0 => Ok(FundsUnavailable),
                    1 => Ok(OnlyProvider),
                    2 => Ok(BelowMinimum),
                    3 => Ok(CannotCreate),
                    4 => Ok(UnknownAsset),
                    5 => Ok(Frozen),
                    6 => Ok(Unsupported),
                    7 => Ok(CannotCreateHold),
                    8 => Ok(NotExpendable),
                    9 => Ok(Blocked),
                    _ => ::core::panicking::panic("not yet implemented"),
                }
            }
        }
        impl From<PopApiError> for TokenError {
            fn from(error: PopApiError) -> Self {
                match error {
                    PopApiError::TokenError(e) => e,
                    _ => ::core::panicking::panic("not yet implemented"),
                }
            }
        }
    }
    pub mod nfts {
        use super::RuntimeCall;
        use crate::{PopApiError::UnknownModuleStatusCode, *};
        use ink::prelude::vec::Vec;
        use primitives::{ApprovalsLimit, BoundedBTreeMap, KeyLimit, MultiAddress};
        pub use primitives::{CollectionId, ItemId};
        use scale::Encode;
        pub use types::*;
        type Result<T> = core::result::Result<T, Error>;
        /// Issue a new collection of non-fungible items
        pub fn create(
            admin: impl Into<MultiAddress<AccountId, ()>>,
            config: CollectionConfig,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::Create {
                        admin: admin.into(),
                        config,
                    }),
                )?,
            )
        }
        /// Destroy a collection of fungible items.
        pub fn destroy(collection: CollectionId) -> Result<()> {
            Ok(dispatch(RuntimeCall::Nfts(NftCalls::Destroy { collection }))?)
        }
        /// Mint an item of a particular collection.
        pub fn mint(
            collection: CollectionId,
            item: ItemId,
            mint_to: impl Into<MultiAddress<AccountId, ()>>,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::Mint {
                        collection,
                        item,
                        mint_to: mint_to.into(),
                        witness_data: None,
                    }),
                )?,
            )
        }
        /// Destroy a single item.
        pub fn burn(collection: CollectionId, item: ItemId) -> Result<()> {
            Ok(dispatch(RuntimeCall::Nfts(NftCalls::Burn { collection, item }))?)
        }
        /// Move an item from the sender account to another.
        pub fn transfer(
            collection: CollectionId,
            item: ItemId,
            dest: impl Into<MultiAddress<AccountId, ()>>,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::Transfer {
                        collection,
                        item,
                        dest: dest.into(),
                    }),
                )?,
            )
        }
        /// Re-evaluate the deposits on some items.
        pub fn redeposit(collection: CollectionId, items: Vec<ItemId>) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::Redeposit {
                        collection,
                        items,
                    }),
                )?,
            )
        }
        /// Change the Owner of a collection.
        pub fn transfer_ownership(
            collection: CollectionId,
            new_owner: impl Into<MultiAddress<AccountId, ()>>,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::TransferOwnership {
                        collection,
                        new_owner: new_owner.into(),
                    }),
                )?,
            )
        }
        /// Set (or reset) the acceptance of ownership for a particular account.
        pub fn set_accept_ownership(
            collection: CollectionId,
            maybe_collection: Option<CollectionId>,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::SetAcceptOwnership {
                        collection,
                        maybe_collection,
                    }),
                )?,
            )
        }
        /// Set the maximum number of items a collection could have.
        pub fn set_collection_max_supply(
            collection: CollectionId,
            max_supply: u32,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::SetCollectionMaxSupply {
                        collection,
                        max_supply,
                    }),
                )?,
            )
        }
        /// Update mint settings.
        pub fn update_mint_settings(
            collection: CollectionId,
            mint_settings: MintSettings,
        ) -> Result<()> {
            Ok(
                dispatch(
                    RuntimeCall::Nfts(NftCalls::UpdateMintSettings {
                        collection,
                        mint_settings,
                    }),
                )?,
            )
        }
        /// Get the owner of the item, if the item exists.
        pub fn owner(
            collection: CollectionId,
            item: ItemId,
        ) -> Result<Option<AccountId>> {
            Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Owner(collection, item)))?)
        }
        /// Get the owner of the collection, if the collection exists.
        pub fn collection_owner(collection: CollectionId) -> Result<Option<AccountId>> {
            Ok(
                state::read(
                    RuntimeStateKeys::Nfts(NftsKeys::CollectionOwner(collection)),
                )?,
            )
        }
        /// Get the details of a collection.
        pub fn collection(
            collection: CollectionId,
        ) -> Result<Option<CollectionDetails>> {
            Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Collection(collection)))?)
        }
        /// Get the details of an item.
        pub fn item(
            collection: CollectionId,
            item: ItemId,
        ) -> Result<Option<ItemDetails>> {
            Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Item(collection, item)))?)
        }
        pub mod approvals {
            use super::*;
            /// Approve an item to be transferred by a delegated third-party account.
            pub fn approve_transfer(
                collection: CollectionId,
                item: ItemId,
                delegate: impl Into<MultiAddress<AccountId, ()>>,
                maybe_deadline: Option<BlockNumber>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::ApproveTransfer {
                            collection,
                            item,
                            delegate: delegate.into(),
                            maybe_deadline,
                        }),
                    )?,
                )
            }
            /// Cancel one of the transfer approvals for a specific item.
            pub fn cancel_approval(
                collection: CollectionId,
                item: ItemId,
                delegate: impl Into<MultiAddress<AccountId, ()>>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::CancelApproval {
                            collection,
                            item,
                            delegate: delegate.into(),
                        }),
                    )?,
                )
            }
            /// Cancel all the approvals of a specific item.
            pub fn clear_all_transfer_approvals(
                collection: CollectionId,
                item: ItemId,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::ClearAllTransferApprovals {
                            collection,
                            item,
                        }),
                    )?,
                )
            }
        }
        pub mod attributes {
            use super::*;
            /// Approve item's attributes to be changed by a delegated third-party account.
            pub fn approve_item_attribute(
                collection: CollectionId,
                item: ItemId,
                delegate: impl Into<MultiAddress<AccountId, ()>>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::ApproveItemAttributes {
                            collection,
                            item,
                            delegate: delegate.into(),
                        }),
                    )?,
                )
            }
            /// Cancel the previously provided approval to change item's attributes.
            pub fn cancel_item_attributes_approval(
                collection: CollectionId,
                item: ItemId,
                delegate: impl Into<MultiAddress<AccountId, ()>>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::CancelItemAttributesApproval {
                            collection,
                            item,
                            delegate: delegate.into(),
                        }),
                    )?,
                )
            }
            /// Set an attribute for a collection or item.
            pub fn set_attribute(
                collection: CollectionId,
                maybe_item: Option<ItemId>,
                namespace: AttributeNamespace<AccountId>,
                key: BoundedVec<u8, KeyLimit>,
                value: BoundedVec<u8, KeyLimit>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::SetAttribute {
                            collection,
                            maybe_item,
                            namespace,
                            key,
                            value,
                        }),
                    )?,
                )
            }
            /// Clear an attribute for a collection or item.
            pub fn clear_attribute(
                collection: CollectionId,
                maybe_item: Option<ItemId>,
                namespace: AttributeNamespace<AccountId>,
                key: BoundedVec<u8, KeyLimit>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::ClearAttribute {
                            collection,
                            maybe_item,
                            namespace,
                            key,
                        }),
                    )?,
                )
            }
            /// Get the attribute value of `item` of `collection` corresponding to `key`.
            pub fn attribute(
                collection: CollectionId,
                item: ItemId,
                key: BoundedVec<u8, KeyLimit>,
            ) -> Result<Option<Vec<u8>>> {
                Ok(
                    state::read(
                        RuntimeStateKeys::Nfts(
                            NftsKeys::Attribute(collection, item, key),
                        ),
                    )?,
                )
            }
            /// Get the system attribute value of `item` of `collection` corresponding to `key` if
            /// `item` is `Some`. Otherwise, returns the system attribute value of `collection`
            /// corresponding to `key`.
            pub fn system_attribute(
                collection: CollectionId,
                item: Option<ItemId>,
                key: BoundedVec<u8, KeyLimit>,
            ) -> Result<Option<Vec<u8>>> {
                Ok(
                    state::read(
                        RuntimeStateKeys::Nfts(
                            NftsKeys::SystemAttribute(collection, item, key),
                        ),
                    )?,
                )
            }
            /// Get the attribute value of `item` of `collection` corresponding to `key`.
            pub fn collection_attribute(
                collection: CollectionId,
                key: BoundedVec<u8, KeyLimit>,
            ) -> Result<Option<Vec<u8>>> {
                Ok(
                    state::read(
                        RuntimeStateKeys::Nfts(
                            NftsKeys::CollectionAttribute(collection, key),
                        ),
                    )?,
                )
            }
        }
        pub mod locking {
            use super::*;
            /// Disallows changing the metadata or attributes of the item.
            pub fn lock_item_properties(
                collection: CollectionId,
                item: ItemId,
                lock_metadata: bool,
                lock_attributes: bool,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::LockItemProperties {
                            collection,
                            item,
                            lock_metadata,
                            lock_attributes,
                        }),
                    )?,
                )
            }
            /// Disallow further unprivileged transfer of an item.
            pub fn lock_item_transfer(
                collection: CollectionId,
                item: ItemId,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::LockItemTransfer {
                            collection,
                            item,
                        }),
                    )?,
                )
            }
            /// Re-allow unprivileged transfer of an item.
            pub fn unlock_item_transfer(
                collection: CollectionId,
                item: ItemId,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::UnlockItemTransfer {
                            collection,
                            item,
                        }),
                    )?,
                )
            }
            /// Disallows specified settings for the whole collection.
            pub fn lock_collection(
                collection: CollectionId,
                lock_settings: CollectionSettings,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::LockCollection {
                            collection,
                            lock_settings,
                        }),
                    )?,
                )
            }
        }
        pub mod metadata {
            use super::*;
            /// Set the metadata for an item.
            pub fn set_metadata(
                collection: CollectionId,
                item: ItemId,
                data: BoundedVec<u8, StringLimit>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::SetMetadata {
                            collection,
                            item,
                            data,
                        }),
                    )?,
                )
            }
            /// Clear the metadata for an item.
            pub fn clear_metadata(collection: CollectionId, item: ItemId) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::ClearMetadata {
                            collection,
                            item,
                        }),
                    )?,
                )
            }
            /// Set the metadata for a collection.
            pub fn set_collection_metadata(
                collection: CollectionId,
                data: BoundedVec<u8, StringLimit>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::SetCollectionMetadata {
                            collection,
                            data,
                        }),
                    )?,
                )
            }
            /// Clear the metadata for a collection.
            pub fn clear_collection_metadata(collection: CollectionId) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::ClearCollectionMetadata {
                            collection,
                        }),
                    )?,
                )
            }
        }
        pub mod roles {
            use super::*;
            /// Change the Issuer, Admin and Freezer of a collection.
            pub fn set_team(
                collection: CollectionId,
                issuer: Option<impl Into<MultiAddress<AccountId, ()>>>,
                admin: Option<impl Into<MultiAddress<AccountId, ()>>>,
                freezer: Option<impl Into<MultiAddress<AccountId, ()>>>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::SetTeam {
                            collection,
                            issuer: issuer.map(|i| i.into()),
                            admin: admin.map(|i| i.into()),
                            freezer: freezer.map(|i| i.into()),
                        }),
                    )?,
                )
            }
        }
        pub mod trading {
            use super::*;
            /// Allows to pay the tips.
            pub fn pay_tips(tips: BoundedVec<ItemTip, MaxTips>) -> Result<()> {
                Ok(dispatch(RuntimeCall::Nfts(NftCalls::PayTips { tips }))?)
            }
            /// Set (or reset) the price for an item.
            pub fn price(
                collection: CollectionId,
                item: ItemId,
                price: Option<Balance>,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::SetPrice {
                            collection,
                            item,
                            price,
                        }),
                    )?,
                )
            }
            /// Allows to buy an item if it's up for sale.
            pub fn buy_item(
                collection: CollectionId,
                item: ItemId,
                bid_price: Balance,
            ) -> Result<()> {
                Ok(
                    dispatch(
                        RuntimeCall::Nfts(NftCalls::BuyItem {
                            collection,
                            item,
                            bid_price,
                        }),
                    )?,
                )
            }
            pub mod swaps {
                use super::*;
                /// Register a new atomic swap, declaring an intention to send an `item` in exchange for
                /// `desired_item` from origin to target on the current chain.
                pub fn create_swap(
                    offered_collection: CollectionId,
                    offered_item: ItemId,
                    desired_collection: CollectionId,
                    maybe_desired_item: Option<ItemId>,
                    maybe_price: Option<PriceWithDirection>,
                    duration: BlockNumber,
                ) -> Result<()> {
                    Ok(
                        dispatch(
                            RuntimeCall::Nfts(NftCalls::CreateSwap {
                                offered_collection,
                                offered_item,
                                desired_collection,
                                maybe_desired_item,
                                maybe_price,
                                duration,
                            }),
                        )?,
                    )
                }
                /// Cancel an atomic swap.
                pub fn cancel_swap(
                    offered_collection: CollectionId,
                    offered_item: ItemId,
                ) -> Result<()> {
                    Ok(
                        dispatch(
                            RuntimeCall::Nfts(NftCalls::CancelSwap {
                                offered_collection,
                                offered_item,
                            }),
                        )?,
                    )
                }
                /// Claim an atomic swap.
                pub fn claim_swap(
                    send_collection: CollectionId,
                    send_item: ItemId,
                    receive_collection: CollectionId,
                    receive_item: ItemId,
                ) -> Result<()> {
                    Ok(
                        dispatch(
                            RuntimeCall::Nfts(NftCalls::ClaimSwap {
                                send_collection,
                                send_item,
                                receive_collection,
                                receive_item,
                            }),
                        )?,
                    )
                }
            }
        }
        pub(crate) enum NftCalls {
            #[codec(index = 0)]
            Create { admin: MultiAddress<AccountId, ()>, config: CollectionConfig },
            #[codec(index = 2)]
            Destroy { collection: CollectionId },
            #[codec(index = 3)]
            Mint {
                collection: CollectionId,
                item: ItemId,
                mint_to: MultiAddress<AccountId, ()>,
                witness_data: Option<()>,
            },
            #[codec(index = 5)]
            Burn { collection: CollectionId, item: ItemId },
            #[codec(index = 6)]
            Transfer {
                collection: CollectionId,
                item: ItemId,
                dest: MultiAddress<AccountId, ()>,
            },
            #[codec(index = 7)]
            Redeposit { collection: CollectionId, items: Vec<ItemId> },
            #[codec(index = 8)]
            LockItemTransfer { collection: CollectionId, item: ItemId },
            #[codec(index = 9)]
            UnlockItemTransfer { collection: CollectionId, item: ItemId },
            #[codec(index = 10)]
            LockCollection {
                collection: CollectionId,
                lock_settings: CollectionSettings,
            },
            #[codec(index = 11)]
            TransferOwnership {
                collection: CollectionId,
                new_owner: MultiAddress<AccountId, ()>,
            },
            #[codec(index = 12)]
            SetTeam {
                collection: CollectionId,
                issuer: Option<MultiAddress<AccountId, ()>>,
                admin: Option<MultiAddress<AccountId, ()>>,
                freezer: Option<MultiAddress<AccountId, ()>>,
            },
            #[codec(index = 15)]
            ApproveTransfer {
                collection: CollectionId,
                item: ItemId,
                delegate: MultiAddress<AccountId, ()>,
                maybe_deadline: Option<BlockNumber>,
            },
            #[codec(index = 16)]
            CancelApproval {
                collection: CollectionId,
                item: ItemId,
                delegate: MultiAddress<AccountId, ()>,
            },
            #[codec(index = 17)]
            ClearAllTransferApprovals { collection: CollectionId, item: ItemId },
            #[codec(index = 18)]
            LockItemProperties {
                collection: CollectionId,
                item: ItemId,
                lock_metadata: bool,
                lock_attributes: bool,
            },
            #[codec(index = 19)]
            SetAttribute {
                collection: CollectionId,
                maybe_item: Option<ItemId>,
                namespace: AttributeNamespace<AccountId>,
                key: BoundedVec<u8, KeyLimit>,
                value: BoundedVec<u8, KeyLimit>,
            },
            #[codec(index = 21)]
            ClearAttribute {
                collection: CollectionId,
                maybe_item: Option<ItemId>,
                namespace: AttributeNamespace<AccountId>,
                key: BoundedVec<u8, KeyLimit>,
            },
            #[codec(index = 22)]
            ApproveItemAttributes {
                collection: CollectionId,
                item: ItemId,
                delegate: MultiAddress<AccountId, ()>,
            },
            #[codec(index = 23)]
            CancelItemAttributesApproval {
                collection: CollectionId,
                item: ItemId,
                delegate: MultiAddress<AccountId, ()>,
            },
            #[codec(index = 24)]
            SetMetadata {
                collection: CollectionId,
                item: ItemId,
                data: BoundedVec<u8, StringLimit>,
            },
            #[codec(index = 25)]
            ClearMetadata { collection: CollectionId, item: ItemId },
            #[codec(index = 26)]
            SetCollectionMetadata {
                collection: CollectionId,
                data: BoundedVec<u8, StringLimit>,
            },
            #[codec(index = 27)]
            ClearCollectionMetadata { collection: CollectionId },
            #[codec(index = 28)]
            SetAcceptOwnership {
                collection: CollectionId,
                maybe_collection: Option<CollectionId>,
            },
            #[codec(index = 29)]
            SetCollectionMaxSupply { collection: CollectionId, max_supply: u32 },
            #[codec(index = 30)]
            UpdateMintSettings { collection: CollectionId, mint_settings: MintSettings },
            #[codec(index = 31)]
            SetPrice { collection: CollectionId, item: ItemId, price: Option<Balance> },
            #[codec(index = 32)]
            BuyItem { collection: CollectionId, item: ItemId, bid_price: Balance },
            #[codec(index = 33)]
            PayTips { tips: BoundedVec<ItemTip, MaxTips> },
            #[codec(index = 34)]
            CreateSwap {
                offered_collection: CollectionId,
                offered_item: ItemId,
                desired_collection: CollectionId,
                maybe_desired_item: Option<ItemId>,
                maybe_price: Option<PriceWithDirection>,
                duration: BlockNumber,
            },
            #[codec(index = 35)]
            CancelSwap { offered_collection: CollectionId, offered_item: ItemId },
            #[codec(index = 36)]
            ClaimSwap {
                send_collection: CollectionId,
                send_item: ItemId,
                receive_collection: CollectionId,
                receive_item: ItemId,
            },
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for NftCalls {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            NftCalls::Create { ref admin, ref config } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(admin))
                                    .saturating_add(::scale::Encode::size_hint(config))
                            }
                            NftCalls::Destroy { ref collection } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                            }
                            NftCalls::Mint {
                                ref collection,
                                ref item,
                                ref mint_to,
                                ref witness_data,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(mint_to))
                                    .saturating_add(::scale::Encode::size_hint(witness_data))
                            }
                            NftCalls::Burn { ref collection, ref item } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                            }
                            NftCalls::Transfer { ref collection, ref item, ref dest } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(dest))
                            }
                            NftCalls::Redeposit { ref collection, ref items } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(items))
                            }
                            NftCalls::LockItemTransfer { ref collection, ref item } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                            }
                            NftCalls::UnlockItemTransfer { ref collection, ref item } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                            }
                            NftCalls::LockCollection {
                                ref collection,
                                ref lock_settings,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(lock_settings))
                            }
                            NftCalls::TransferOwnership {
                                ref collection,
                                ref new_owner,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(new_owner))
                            }
                            NftCalls::SetTeam {
                                ref collection,
                                ref issuer,
                                ref admin,
                                ref freezer,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(issuer))
                                    .saturating_add(::scale::Encode::size_hint(admin))
                                    .saturating_add(::scale::Encode::size_hint(freezer))
                            }
                            NftCalls::ApproveTransfer {
                                ref collection,
                                ref item,
                                ref delegate,
                                ref maybe_deadline,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(delegate))
                                    .saturating_add(::scale::Encode::size_hint(maybe_deadline))
                            }
                            NftCalls::CancelApproval {
                                ref collection,
                                ref item,
                                ref delegate,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(delegate))
                            }
                            NftCalls::ClearAllTransferApprovals {
                                ref collection,
                                ref item,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                            }
                            NftCalls::LockItemProperties {
                                ref collection,
                                ref item,
                                ref lock_metadata,
                                ref lock_attributes,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(lock_metadata))
                                    .saturating_add(::scale::Encode::size_hint(lock_attributes))
                            }
                            NftCalls::SetAttribute {
                                ref collection,
                                ref maybe_item,
                                ref namespace,
                                ref key,
                                ref value,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(maybe_item))
                                    .saturating_add(::scale::Encode::size_hint(namespace))
                                    .saturating_add(::scale::Encode::size_hint(key))
                                    .saturating_add(::scale::Encode::size_hint(value))
                            }
                            NftCalls::ClearAttribute {
                                ref collection,
                                ref maybe_item,
                                ref namespace,
                                ref key,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(maybe_item))
                                    .saturating_add(::scale::Encode::size_hint(namespace))
                                    .saturating_add(::scale::Encode::size_hint(key))
                            }
                            NftCalls::ApproveItemAttributes {
                                ref collection,
                                ref item,
                                ref delegate,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(delegate))
                            }
                            NftCalls::CancelItemAttributesApproval {
                                ref collection,
                                ref item,
                                ref delegate,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(delegate))
                            }
                            NftCalls::SetMetadata {
                                ref collection,
                                ref item,
                                ref data,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(data))
                            }
                            NftCalls::ClearMetadata { ref collection, ref item } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                            }
                            NftCalls::SetCollectionMetadata {
                                ref collection,
                                ref data,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(data))
                            }
                            NftCalls::ClearCollectionMetadata { ref collection } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                            }
                            NftCalls::SetAcceptOwnership {
                                ref collection,
                                ref maybe_collection,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(
                                        ::scale::Encode::size_hint(maybe_collection),
                                    )
                            }
                            NftCalls::SetCollectionMaxSupply {
                                ref collection,
                                ref max_supply,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(max_supply))
                            }
                            NftCalls::UpdateMintSettings {
                                ref collection,
                                ref mint_settings,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(mint_settings))
                            }
                            NftCalls::SetPrice {
                                ref collection,
                                ref item,
                                ref price,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(price))
                            }
                            NftCalls::BuyItem {
                                ref collection,
                                ref item,
                                ref bid_price,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(collection))
                                    .saturating_add(::scale::Encode::size_hint(item))
                                    .saturating_add(::scale::Encode::size_hint(bid_price))
                            }
                            NftCalls::PayTips { ref tips } => {
                                0_usize.saturating_add(::scale::Encode::size_hint(tips))
                            }
                            NftCalls::CreateSwap {
                                ref offered_collection,
                                ref offered_item,
                                ref desired_collection,
                                ref maybe_desired_item,
                                ref maybe_price,
                                ref duration,
                            } => {
                                0_usize
                                    .saturating_add(
                                        ::scale::Encode::size_hint(offered_collection),
                                    )
                                    .saturating_add(::scale::Encode::size_hint(offered_item))
                                    .saturating_add(
                                        ::scale::Encode::size_hint(desired_collection),
                                    )
                                    .saturating_add(
                                        ::scale::Encode::size_hint(maybe_desired_item),
                                    )
                                    .saturating_add(::scale::Encode::size_hint(maybe_price))
                                    .saturating_add(::scale::Encode::size_hint(duration))
                            }
                            NftCalls::CancelSwap {
                                ref offered_collection,
                                ref offered_item,
                            } => {
                                0_usize
                                    .saturating_add(
                                        ::scale::Encode::size_hint(offered_collection),
                                    )
                                    .saturating_add(::scale::Encode::size_hint(offered_item))
                            }
                            NftCalls::ClaimSwap {
                                ref send_collection,
                                ref send_item,
                                ref receive_collection,
                                ref receive_item,
                            } => {
                                0_usize
                                    .saturating_add(::scale::Encode::size_hint(send_collection))
                                    .saturating_add(::scale::Encode::size_hint(send_item))
                                    .saturating_add(
                                        ::scale::Encode::size_hint(receive_collection),
                                    )
                                    .saturating_add(::scale::Encode::size_hint(receive_item))
                            }
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        NftCalls::Create { ref admin, ref config } => {
                            __codec_dest_edqy.push_byte(0u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(admin, __codec_dest_edqy);
                            ::scale::Encode::encode_to(config, __codec_dest_edqy);
                        }
                        NftCalls::Destroy { ref collection } => {
                            __codec_dest_edqy.push_byte(2u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                        }
                        NftCalls::Mint {
                            ref collection,
                            ref item,
                            ref mint_to,
                            ref witness_data,
                        } => {
                            __codec_dest_edqy.push_byte(3u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(mint_to, __codec_dest_edqy);
                            ::scale::Encode::encode_to(witness_data, __codec_dest_edqy);
                        }
                        NftCalls::Burn { ref collection, ref item } => {
                            __codec_dest_edqy.push_byte(5u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                        }
                        NftCalls::Transfer { ref collection, ref item, ref dest } => {
                            __codec_dest_edqy.push_byte(6u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(dest, __codec_dest_edqy);
                        }
                        NftCalls::Redeposit { ref collection, ref items } => {
                            __codec_dest_edqy.push_byte(7u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(items, __codec_dest_edqy);
                        }
                        NftCalls::LockItemTransfer { ref collection, ref item } => {
                            __codec_dest_edqy.push_byte(8u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                        }
                        NftCalls::UnlockItemTransfer { ref collection, ref item } => {
                            __codec_dest_edqy.push_byte(9u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                        }
                        NftCalls::LockCollection {
                            ref collection,
                            ref lock_settings,
                        } => {
                            __codec_dest_edqy.push_byte(10u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(lock_settings, __codec_dest_edqy);
                        }
                        NftCalls::TransferOwnership {
                            ref collection,
                            ref new_owner,
                        } => {
                            __codec_dest_edqy.push_byte(11u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(new_owner, __codec_dest_edqy);
                        }
                        NftCalls::SetTeam {
                            ref collection,
                            ref issuer,
                            ref admin,
                            ref freezer,
                        } => {
                            __codec_dest_edqy.push_byte(12u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(issuer, __codec_dest_edqy);
                            ::scale::Encode::encode_to(admin, __codec_dest_edqy);
                            ::scale::Encode::encode_to(freezer, __codec_dest_edqy);
                        }
                        NftCalls::ApproveTransfer {
                            ref collection,
                            ref item,
                            ref delegate,
                            ref maybe_deadline,
                        } => {
                            __codec_dest_edqy.push_byte(15u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(delegate, __codec_dest_edqy);
                            ::scale::Encode::encode_to(
                                maybe_deadline,
                                __codec_dest_edqy,
                            );
                        }
                        NftCalls::CancelApproval {
                            ref collection,
                            ref item,
                            ref delegate,
                        } => {
                            __codec_dest_edqy.push_byte(16u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(delegate, __codec_dest_edqy);
                        }
                        NftCalls::ClearAllTransferApprovals {
                            ref collection,
                            ref item,
                        } => {
                            __codec_dest_edqy.push_byte(17u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                        }
                        NftCalls::LockItemProperties {
                            ref collection,
                            ref item,
                            ref lock_metadata,
                            ref lock_attributes,
                        } => {
                            __codec_dest_edqy.push_byte(18u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(lock_metadata, __codec_dest_edqy);
                            ::scale::Encode::encode_to(
                                lock_attributes,
                                __codec_dest_edqy,
                            );
                        }
                        NftCalls::SetAttribute {
                            ref collection,
                            ref maybe_item,
                            ref namespace,
                            ref key,
                            ref value,
                        } => {
                            __codec_dest_edqy.push_byte(19u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(maybe_item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(namespace, __codec_dest_edqy);
                            ::scale::Encode::encode_to(key, __codec_dest_edqy);
                            ::scale::Encode::encode_to(value, __codec_dest_edqy);
                        }
                        NftCalls::ClearAttribute {
                            ref collection,
                            ref maybe_item,
                            ref namespace,
                            ref key,
                        } => {
                            __codec_dest_edqy.push_byte(21u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(maybe_item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(namespace, __codec_dest_edqy);
                            ::scale::Encode::encode_to(key, __codec_dest_edqy);
                        }
                        NftCalls::ApproveItemAttributes {
                            ref collection,
                            ref item,
                            ref delegate,
                        } => {
                            __codec_dest_edqy.push_byte(22u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(delegate, __codec_dest_edqy);
                        }
                        NftCalls::CancelItemAttributesApproval {
                            ref collection,
                            ref item,
                            ref delegate,
                        } => {
                            __codec_dest_edqy.push_byte(23u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(delegate, __codec_dest_edqy);
                        }
                        NftCalls::SetMetadata { ref collection, ref item, ref data } => {
                            __codec_dest_edqy.push_byte(24u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(data, __codec_dest_edqy);
                        }
                        NftCalls::ClearMetadata { ref collection, ref item } => {
                            __codec_dest_edqy.push_byte(25u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                        }
                        NftCalls::SetCollectionMetadata { ref collection, ref data } => {
                            __codec_dest_edqy.push_byte(26u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(data, __codec_dest_edqy);
                        }
                        NftCalls::ClearCollectionMetadata { ref collection } => {
                            __codec_dest_edqy.push_byte(27u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                        }
                        NftCalls::SetAcceptOwnership {
                            ref collection,
                            ref maybe_collection,
                        } => {
                            __codec_dest_edqy.push_byte(28u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(
                                maybe_collection,
                                __codec_dest_edqy,
                            );
                        }
                        NftCalls::SetCollectionMaxSupply {
                            ref collection,
                            ref max_supply,
                        } => {
                            __codec_dest_edqy.push_byte(29u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(max_supply, __codec_dest_edqy);
                        }
                        NftCalls::UpdateMintSettings {
                            ref collection,
                            ref mint_settings,
                        } => {
                            __codec_dest_edqy.push_byte(30u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(mint_settings, __codec_dest_edqy);
                        }
                        NftCalls::SetPrice { ref collection, ref item, ref price } => {
                            __codec_dest_edqy.push_byte(31u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(price, __codec_dest_edqy);
                        }
                        NftCalls::BuyItem {
                            ref collection,
                            ref item,
                            ref bid_price,
                        } => {
                            __codec_dest_edqy.push_byte(32u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(collection, __codec_dest_edqy);
                            ::scale::Encode::encode_to(item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(bid_price, __codec_dest_edqy);
                        }
                        NftCalls::PayTips { ref tips } => {
                            __codec_dest_edqy.push_byte(33u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(tips, __codec_dest_edqy);
                        }
                        NftCalls::CreateSwap {
                            ref offered_collection,
                            ref offered_item,
                            ref desired_collection,
                            ref maybe_desired_item,
                            ref maybe_price,
                            ref duration,
                        } => {
                            __codec_dest_edqy.push_byte(34u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(
                                offered_collection,
                                __codec_dest_edqy,
                            );
                            ::scale::Encode::encode_to(offered_item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(
                                desired_collection,
                                __codec_dest_edqy,
                            );
                            ::scale::Encode::encode_to(
                                maybe_desired_item,
                                __codec_dest_edqy,
                            );
                            ::scale::Encode::encode_to(maybe_price, __codec_dest_edqy);
                            ::scale::Encode::encode_to(duration, __codec_dest_edqy);
                        }
                        NftCalls::CancelSwap {
                            ref offered_collection,
                            ref offered_item,
                        } => {
                            __codec_dest_edqy.push_byte(35u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(
                                offered_collection,
                                __codec_dest_edqy,
                            );
                            ::scale::Encode::encode_to(offered_item, __codec_dest_edqy);
                        }
                        NftCalls::ClaimSwap {
                            ref send_collection,
                            ref send_item,
                            ref receive_collection,
                            ref receive_item,
                        } => {
                            __codec_dest_edqy.push_byte(36u8 as ::core::primitive::u8);
                            ::scale::Encode::encode_to(
                                send_collection,
                                __codec_dest_edqy,
                            );
                            ::scale::Encode::encode_to(send_item, __codec_dest_edqy);
                            ::scale::Encode::encode_to(
                                receive_collection,
                                __codec_dest_edqy,
                            );
                            ::scale::Encode::encode_to(receive_item, __codec_dest_edqy);
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for NftCalls {}
        };
        pub enum Error {
            /// The signing account has no permission to do the operation.
            NoPermission,
            /// The given item ID is unknown.
            UnknownCollection,
            /// The item ID has already been used for an item.
            AlreadyExists,
            /// The approval had a deadline that expired, so the approval isn't valid anymore.
            ApprovalExpired,
            /// The owner turned out to be different to what was expected.
            WrongOwner,
            /// The witness data given does not match the current state of the chain.
            BadWitness,
            /// Collection ID is already taken.
            CollectionIdInUse,
            /// Items within that collection are non-transferable.
            ItemsNonTransferable,
            /// The provided account is not a delegate.
            NotDelegate,
            /// The delegate turned out to be different to what was expected.
            WrongDelegate,
            /// No approval exists that would allow the transfer.
            Unapproved,
            /// The named owner has not signed ownership acceptance of the collection.
            Unaccepted,
            /// The item is locked (non-transferable).
            ItemLocked,
            /// Item's attributes are locked.
            LockedItemAttributes,
            /// Collection's attributes are locked.
            LockedCollectionAttributes,
            /// Item's metadata is locked.
            LockedItemMetadata,
            /// Collection's metadata is locked.
            LockedCollectionMetadata,
            /// All items have been minted.
            MaxSupplyReached,
            /// The max supply is locked and can't be changed.
            MaxSupplyLocked,
            /// The provided max supply is less than the number of items a collection already has.
            MaxSupplyTooSmall,
            /// The given item ID is unknown.
            UnknownItem,
            /// Swap doesn't exist.
            UnknownSwap,
            /// The given item has no metadata set.
            MetadataNotFound,
            /// The provided attribute can't be found.
            AttributeNotFound,
            /// Item is not for sale.
            NotForSale,
            /// The provided bid is too low.
            BidTooLow,
            /// The item has reached its approval limit.
            ReachedApprovalLimit,
            /// The deadline has already expired.
            DeadlineExpired,
            /// The duration provided should be less than or equal to `MaxDeadlineDuration`.
            WrongDuration,
            /// The method is disabled by system settings.
            MethodDisabled,
            /// The provided setting can't be set.
            WrongSetting,
            /// Item's config already exists and should be equal to the provided one.
            InconsistentItemConfig,
            /// Config for a collection or an item can't be found.
            NoConfig,
            /// Some roles were not cleared.
            RolesNotCleared,
            /// Mint has not started yet.
            MintNotStarted,
            /// Mint has already ended.
            MintEnded,
            /// The provided Item was already used for claiming.
            AlreadyClaimed,
            /// The provided data is incorrect.
            IncorrectData,
            /// The extrinsic was sent by the wrong origin.
            WrongOrigin,
            /// The provided signature is incorrect.
            WrongSignature,
            /// The provided metadata might be too long.
            IncorrectMetadata,
            /// Can't set more attributes per one call.
            MaxAttributesLimitReached,
            /// The provided namespace isn't supported in this call.
            WrongNamespace,
            /// Can't delete non-empty collections.
            CollectionNotEmpty,
            /// The witness data should be provided.
            WitnessRequired,
        }
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            impl ::scale_info::TypeInfo for Error {
                type Identity = Self;
                fn type_info() -> ::scale_info::Type {
                    ::scale_info::Type::builder()
                        .path(
                            ::scale_info::Path::new_with_replace(
                                "Error",
                                "pop_api::v0::nfts",
                                &[],
                            ),
                        )
                        .type_params(::alloc::vec::Vec::new())
                        .variant(
                            ::scale_info::build::Variants::new()
                                .variant(
                                    "NoPermission",
                                    |v| {
                                        v
                                            .index(0usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The signing account has no permission to do the operation.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "UnknownCollection",
                                    |v| {
                                        v
                                            .index(1usize as ::core::primitive::u8)
                                            .docs(&["The given item ID is unknown."])
                                    },
                                )
                                .variant(
                                    "AlreadyExists",
                                    |v| {
                                        v
                                            .index(2usize as ::core::primitive::u8)
                                            .docs(&["The item ID has already been used for an item."])
                                    },
                                )
                                .variant(
                                    "ApprovalExpired",
                                    |v| {
                                        v
                                            .index(3usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The approval had a deadline that expired, so the approval isn't valid anymore.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "WrongOwner",
                                    |v| {
                                        v
                                            .index(4usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The owner turned out to be different to what was expected.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "BadWitness",
                                    |v| {
                                        v
                                            .index(5usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The witness data given does not match the current state of the chain.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "CollectionIdInUse",
                                    |v| {
                                        v
                                            .index(6usize as ::core::primitive::u8)
                                            .docs(&["Collection ID is already taken."])
                                    },
                                )
                                .variant(
                                    "ItemsNonTransferable",
                                    |v| {
                                        v
                                            .index(7usize as ::core::primitive::u8)
                                            .docs(
                                                &["Items within that collection are non-transferable."],
                                            )
                                    },
                                )
                                .variant(
                                    "NotDelegate",
                                    |v| {
                                        v
                                            .index(8usize as ::core::primitive::u8)
                                            .docs(&["The provided account is not a delegate."])
                                    },
                                )
                                .variant(
                                    "WrongDelegate",
                                    |v| {
                                        v
                                            .index(9usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The delegate turned out to be different to what was expected.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "Unapproved",
                                    |v| {
                                        v
                                            .index(10usize as ::core::primitive::u8)
                                            .docs(
                                                &["No approval exists that would allow the transfer."],
                                            )
                                    },
                                )
                                .variant(
                                    "Unaccepted",
                                    |v| {
                                        v
                                            .index(11usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The named owner has not signed ownership acceptance of the collection.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "ItemLocked",
                                    |v| {
                                        v
                                            .index(12usize as ::core::primitive::u8)
                                            .docs(&["The item is locked (non-transferable)."])
                                    },
                                )
                                .variant(
                                    "LockedItemAttributes",
                                    |v| {
                                        v
                                            .index(13usize as ::core::primitive::u8)
                                            .docs(&["Item's attributes are locked."])
                                    },
                                )
                                .variant(
                                    "LockedCollectionAttributes",
                                    |v| {
                                        v
                                            .index(14usize as ::core::primitive::u8)
                                            .docs(&["Collection's attributes are locked."])
                                    },
                                )
                                .variant(
                                    "LockedItemMetadata",
                                    |v| {
                                        v
                                            .index(15usize as ::core::primitive::u8)
                                            .docs(&["Item's metadata is locked."])
                                    },
                                )
                                .variant(
                                    "LockedCollectionMetadata",
                                    |v| {
                                        v
                                            .index(16usize as ::core::primitive::u8)
                                            .docs(&["Collection's metadata is locked."])
                                    },
                                )
                                .variant(
                                    "MaxSupplyReached",
                                    |v| {
                                        v
                                            .index(17usize as ::core::primitive::u8)
                                            .docs(&["All items have been minted."])
                                    },
                                )
                                .variant(
                                    "MaxSupplyLocked",
                                    |v| {
                                        v
                                            .index(18usize as ::core::primitive::u8)
                                            .docs(&["The max supply is locked and can't be changed."])
                                    },
                                )
                                .variant(
                                    "MaxSupplyTooSmall",
                                    |v| {
                                        v
                                            .index(19usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The provided max supply is less than the number of items a collection already has.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "UnknownItem",
                                    |v| {
                                        v
                                            .index(20usize as ::core::primitive::u8)
                                            .docs(&["The given item ID is unknown."])
                                    },
                                )
                                .variant(
                                    "UnknownSwap",
                                    |v| {
                                        v
                                            .index(21usize as ::core::primitive::u8)
                                            .docs(&["Swap doesn't exist."])
                                    },
                                )
                                .variant(
                                    "MetadataNotFound",
                                    |v| {
                                        v
                                            .index(22usize as ::core::primitive::u8)
                                            .docs(&["The given item has no metadata set."])
                                    },
                                )
                                .variant(
                                    "AttributeNotFound",
                                    |v| {
                                        v
                                            .index(23usize as ::core::primitive::u8)
                                            .docs(&["The provided attribute can't be found."])
                                    },
                                )
                                .variant(
                                    "NotForSale",
                                    |v| {
                                        v
                                            .index(24usize as ::core::primitive::u8)
                                            .docs(&["Item is not for sale."])
                                    },
                                )
                                .variant(
                                    "BidTooLow",
                                    |v| {
                                        v
                                            .index(25usize as ::core::primitive::u8)
                                            .docs(&["The provided bid is too low."])
                                    },
                                )
                                .variant(
                                    "ReachedApprovalLimit",
                                    |v| {
                                        v
                                            .index(26usize as ::core::primitive::u8)
                                            .docs(&["The item has reached its approval limit."])
                                    },
                                )
                                .variant(
                                    "DeadlineExpired",
                                    |v| {
                                        v
                                            .index(27usize as ::core::primitive::u8)
                                            .docs(&["The deadline has already expired."])
                                    },
                                )
                                .variant(
                                    "WrongDuration",
                                    |v| {
                                        v
                                            .index(28usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "The duration provided should be less than or equal to `MaxDeadlineDuration`.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "MethodDisabled",
                                    |v| {
                                        v
                                            .index(29usize as ::core::primitive::u8)
                                            .docs(&["The method is disabled by system settings."])
                                    },
                                )
                                .variant(
                                    "WrongSetting",
                                    |v| {
                                        v
                                            .index(30usize as ::core::primitive::u8)
                                            .docs(&["The provided setting can't be set."])
                                    },
                                )
                                .variant(
                                    "InconsistentItemConfig",
                                    |v| {
                                        v
                                            .index(31usize as ::core::primitive::u8)
                                            .docs(
                                                &[
                                                    "Item's config already exists and should be equal to the provided one.",
                                                ],
                                            )
                                    },
                                )
                                .variant(
                                    "NoConfig",
                                    |v| {
                                        v
                                            .index(32usize as ::core::primitive::u8)
                                            .docs(
                                                &["Config for a collection or an item can't be found."],
                                            )
                                    },
                                )
                                .variant(
                                    "RolesNotCleared",
                                    |v| {
                                        v
                                            .index(33usize as ::core::primitive::u8)
                                            .docs(&["Some roles were not cleared."])
                                    },
                                )
                                .variant(
                                    "MintNotStarted",
                                    |v| {
                                        v
                                            .index(34usize as ::core::primitive::u8)
                                            .docs(&["Mint has not started yet."])
                                    },
                                )
                                .variant(
                                    "MintEnded",
                                    |v| {
                                        v
                                            .index(35usize as ::core::primitive::u8)
                                            .docs(&["Mint has already ended."])
                                    },
                                )
                                .variant(
                                    "AlreadyClaimed",
                                    |v| {
                                        v
                                            .index(36usize as ::core::primitive::u8)
                                            .docs(&["The provided Item was already used for claiming."])
                                    },
                                )
                                .variant(
                                    "IncorrectData",
                                    |v| {
                                        v
                                            .index(37usize as ::core::primitive::u8)
                                            .docs(&["The provided data is incorrect."])
                                    },
                                )
                                .variant(
                                    "WrongOrigin",
                                    |v| {
                                        v
                                            .index(38usize as ::core::primitive::u8)
                                            .docs(&["The extrinsic was sent by the wrong origin."])
                                    },
                                )
                                .variant(
                                    "WrongSignature",
                                    |v| {
                                        v
                                            .index(39usize as ::core::primitive::u8)
                                            .docs(&["The provided signature is incorrect."])
                                    },
                                )
                                .variant(
                                    "IncorrectMetadata",
                                    |v| {
                                        v
                                            .index(40usize as ::core::primitive::u8)
                                            .docs(&["The provided metadata might be too long."])
                                    },
                                )
                                .variant(
                                    "MaxAttributesLimitReached",
                                    |v| {
                                        v
                                            .index(41usize as ::core::primitive::u8)
                                            .docs(&["Can't set more attributes per one call."])
                                    },
                                )
                                .variant(
                                    "WrongNamespace",
                                    |v| {
                                        v
                                            .index(42usize as ::core::primitive::u8)
                                            .docs(
                                                &["The provided namespace isn't supported in this call."],
                                            )
                                    },
                                )
                                .variant(
                                    "CollectionNotEmpty",
                                    |v| {
                                        v
                                            .index(43usize as ::core::primitive::u8)
                                            .docs(&["Can't delete non-empty collections."])
                                    },
                                )
                                .variant(
                                    "WitnessRequired",
                                    |v| {
                                        v
                                            .index(44usize as ::core::primitive::u8)
                                            .docs(&["The witness data should be provided."])
                                    },
                                ),
                        )
                }
            }
        };
        #[automatically_derived]
        impl ::core::fmt::Debug for Error {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        Error::NoPermission => "NoPermission",
                        Error::UnknownCollection => "UnknownCollection",
                        Error::AlreadyExists => "AlreadyExists",
                        Error::ApprovalExpired => "ApprovalExpired",
                        Error::WrongOwner => "WrongOwner",
                        Error::BadWitness => "BadWitness",
                        Error::CollectionIdInUse => "CollectionIdInUse",
                        Error::ItemsNonTransferable => "ItemsNonTransferable",
                        Error::NotDelegate => "NotDelegate",
                        Error::WrongDelegate => "WrongDelegate",
                        Error::Unapproved => "Unapproved",
                        Error::Unaccepted => "Unaccepted",
                        Error::ItemLocked => "ItemLocked",
                        Error::LockedItemAttributes => "LockedItemAttributes",
                        Error::LockedCollectionAttributes => "LockedCollectionAttributes",
                        Error::LockedItemMetadata => "LockedItemMetadata",
                        Error::LockedCollectionMetadata => "LockedCollectionMetadata",
                        Error::MaxSupplyReached => "MaxSupplyReached",
                        Error::MaxSupplyLocked => "MaxSupplyLocked",
                        Error::MaxSupplyTooSmall => "MaxSupplyTooSmall",
                        Error::UnknownItem => "UnknownItem",
                        Error::UnknownSwap => "UnknownSwap",
                        Error::MetadataNotFound => "MetadataNotFound",
                        Error::AttributeNotFound => "AttributeNotFound",
                        Error::NotForSale => "NotForSale",
                        Error::BidTooLow => "BidTooLow",
                        Error::ReachedApprovalLimit => "ReachedApprovalLimit",
                        Error::DeadlineExpired => "DeadlineExpired",
                        Error::WrongDuration => "WrongDuration",
                        Error::MethodDisabled => "MethodDisabled",
                        Error::WrongSetting => "WrongSetting",
                        Error::InconsistentItemConfig => "InconsistentItemConfig",
                        Error::NoConfig => "NoConfig",
                        Error::RolesNotCleared => "RolesNotCleared",
                        Error::MintNotStarted => "MintNotStarted",
                        Error::MintEnded => "MintEnded",
                        Error::AlreadyClaimed => "AlreadyClaimed",
                        Error::IncorrectData => "IncorrectData",
                        Error::WrongOrigin => "WrongOrigin",
                        Error::WrongSignature => "WrongSignature",
                        Error::IncorrectMetadata => "IncorrectMetadata",
                        Error::MaxAttributesLimitReached => "MaxAttributesLimitReached",
                        Error::WrongNamespace => "WrongNamespace",
                        Error::CollectionNotEmpty => "CollectionNotEmpty",
                        Error::WitnessRequired => "WitnessRequired",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Error {}
        #[automatically_derived]
        impl ::core::clone::Clone for Error {
            #[inline]
            fn clone(&self) -> Error {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Error {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Error {
            #[inline]
            fn eq(&self, other: &Error) -> bool {
                let __self_tag = ::core::intrinsics::discriminant_value(self);
                let __arg1_tag = ::core::intrinsics::discriminant_value(other);
                __self_tag == __arg1_tag
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Error {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Encode for Error {
                fn size_hint(&self) -> usize {
                    1_usize
                        + match *self {
                            Error::NoPermission => 0_usize,
                            Error::UnknownCollection => 0_usize,
                            Error::AlreadyExists => 0_usize,
                            Error::ApprovalExpired => 0_usize,
                            Error::WrongOwner => 0_usize,
                            Error::BadWitness => 0_usize,
                            Error::CollectionIdInUse => 0_usize,
                            Error::ItemsNonTransferable => 0_usize,
                            Error::NotDelegate => 0_usize,
                            Error::WrongDelegate => 0_usize,
                            Error::Unapproved => 0_usize,
                            Error::Unaccepted => 0_usize,
                            Error::ItemLocked => 0_usize,
                            Error::LockedItemAttributes => 0_usize,
                            Error::LockedCollectionAttributes => 0_usize,
                            Error::LockedItemMetadata => 0_usize,
                            Error::LockedCollectionMetadata => 0_usize,
                            Error::MaxSupplyReached => 0_usize,
                            Error::MaxSupplyLocked => 0_usize,
                            Error::MaxSupplyTooSmall => 0_usize,
                            Error::UnknownItem => 0_usize,
                            Error::UnknownSwap => 0_usize,
                            Error::MetadataNotFound => 0_usize,
                            Error::AttributeNotFound => 0_usize,
                            Error::NotForSale => 0_usize,
                            Error::BidTooLow => 0_usize,
                            Error::ReachedApprovalLimit => 0_usize,
                            Error::DeadlineExpired => 0_usize,
                            Error::WrongDuration => 0_usize,
                            Error::MethodDisabled => 0_usize,
                            Error::WrongSetting => 0_usize,
                            Error::InconsistentItemConfig => 0_usize,
                            Error::NoConfig => 0_usize,
                            Error::RolesNotCleared => 0_usize,
                            Error::MintNotStarted => 0_usize,
                            Error::MintEnded => 0_usize,
                            Error::AlreadyClaimed => 0_usize,
                            Error::IncorrectData => 0_usize,
                            Error::WrongOrigin => 0_usize,
                            Error::WrongSignature => 0_usize,
                            Error::IncorrectMetadata => 0_usize,
                            Error::MaxAttributesLimitReached => 0_usize,
                            Error::WrongNamespace => 0_usize,
                            Error::CollectionNotEmpty => 0_usize,
                            Error::WitnessRequired => 0_usize,
                            _ => 0_usize,
                        }
                }
                fn encode_to<
                    __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                    match *self {
                        Error::NoPermission => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        }
                        Error::UnknownCollection => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        }
                        Error::AlreadyExists => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        }
                        Error::ApprovalExpired => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                        }
                        Error::WrongOwner => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                        }
                        Error::BadWitness => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                        }
                        Error::CollectionIdInUse => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                        }
                        Error::ItemsNonTransferable => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                        }
                        Error::NotDelegate => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                        }
                        Error::WrongDelegate => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                        }
                        Error::Unapproved => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(10usize as ::core::primitive::u8);
                        }
                        Error::Unaccepted => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(11usize as ::core::primitive::u8);
                        }
                        Error::ItemLocked => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(12usize as ::core::primitive::u8);
                        }
                        Error::LockedItemAttributes => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(13usize as ::core::primitive::u8);
                        }
                        Error::LockedCollectionAttributes => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(14usize as ::core::primitive::u8);
                        }
                        Error::LockedItemMetadata => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(15usize as ::core::primitive::u8);
                        }
                        Error::LockedCollectionMetadata => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(16usize as ::core::primitive::u8);
                        }
                        Error::MaxSupplyReached => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(17usize as ::core::primitive::u8);
                        }
                        Error::MaxSupplyLocked => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(18usize as ::core::primitive::u8);
                        }
                        Error::MaxSupplyTooSmall => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(19usize as ::core::primitive::u8);
                        }
                        Error::UnknownItem => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(20usize as ::core::primitive::u8);
                        }
                        Error::UnknownSwap => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(21usize as ::core::primitive::u8);
                        }
                        Error::MetadataNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(22usize as ::core::primitive::u8);
                        }
                        Error::AttributeNotFound => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(23usize as ::core::primitive::u8);
                        }
                        Error::NotForSale => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(24usize as ::core::primitive::u8);
                        }
                        Error::BidTooLow => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(25usize as ::core::primitive::u8);
                        }
                        Error::ReachedApprovalLimit => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(26usize as ::core::primitive::u8);
                        }
                        Error::DeadlineExpired => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(27usize as ::core::primitive::u8);
                        }
                        Error::WrongDuration => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(28usize as ::core::primitive::u8);
                        }
                        Error::MethodDisabled => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(29usize as ::core::primitive::u8);
                        }
                        Error::WrongSetting => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(30usize as ::core::primitive::u8);
                        }
                        Error::InconsistentItemConfig => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(31usize as ::core::primitive::u8);
                        }
                        Error::NoConfig => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(32usize as ::core::primitive::u8);
                        }
                        Error::RolesNotCleared => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(33usize as ::core::primitive::u8);
                        }
                        Error::MintNotStarted => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(34usize as ::core::primitive::u8);
                        }
                        Error::MintEnded => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(35usize as ::core::primitive::u8);
                        }
                        Error::AlreadyClaimed => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(36usize as ::core::primitive::u8);
                        }
                        Error::IncorrectData => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(37usize as ::core::primitive::u8);
                        }
                        Error::WrongOrigin => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(38usize as ::core::primitive::u8);
                        }
                        Error::WrongSignature => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(39usize as ::core::primitive::u8);
                        }
                        Error::IncorrectMetadata => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(40usize as ::core::primitive::u8);
                        }
                        Error::MaxAttributesLimitReached => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(41usize as ::core::primitive::u8);
                        }
                        Error::WrongNamespace => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(42usize as ::core::primitive::u8);
                        }
                        Error::CollectionNotEmpty => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(43usize as ::core::primitive::u8);
                        }
                        Error::WitnessRequired => {
                            #[allow(clippy::unnecessary_cast)]
                            __codec_dest_edqy
                                .push_byte(44usize as ::core::primitive::u8);
                        }
                        _ => {}
                    }
                }
            }
            #[automatically_derived]
            impl ::scale::EncodeLike for Error {}
        };
        #[allow(deprecated)]
        const _: () = {
            #[automatically_derived]
            impl ::scale::Decode for Error {
                fn decode<__CodecInputEdqy: ::scale::Input>(
                    __codec_input_edqy: &mut __CodecInputEdqy,
                ) -> ::core::result::Result<Self, ::scale::Error> {
                    match __codec_input_edqy
                        .read_byte()
                        .map_err(|e| {
                            e
                                .chain(
                                    "Could not decode `Error`, failed to read variant byte",
                                )
                        })?
                    {
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 0usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NoPermission)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 1usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::UnknownCollection)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 2usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::AlreadyExists)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 3usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ApprovalExpired)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 4usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongOwner)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 5usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::BadWitness)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 6usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CollectionIdInUse)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 7usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ItemsNonTransferable)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 8usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NotDelegate)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 9usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongDelegate)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 10usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::Unapproved)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 11usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::Unaccepted)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 12usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ItemLocked)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 13usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::LockedItemAttributes)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 14usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(
                                    Error::LockedCollectionAttributes,
                                )
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 15usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::LockedItemMetadata)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 16usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::LockedCollectionMetadata)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 17usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MaxSupplyReached)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 18usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MaxSupplyLocked)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 19usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MaxSupplyTooSmall)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 20usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::UnknownItem)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 21usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::UnknownSwap)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 22usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MetadataNotFound)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 23usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::AttributeNotFound)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 24usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NotForSale)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 25usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::BidTooLow)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 26usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::ReachedApprovalLimit)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 27usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::DeadlineExpired)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 28usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongDuration)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 29usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MethodDisabled)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 30usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongSetting)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 31usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::InconsistentItemConfig)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 32usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::NoConfig)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 33usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::RolesNotCleared)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 34usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MintNotStarted)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 35usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MintEnded)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 36usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::AlreadyClaimed)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 37usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::IncorrectData)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 38usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongOrigin)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 39usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongSignature)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 40usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::IncorrectMetadata)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 41usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::MaxAttributesLimitReached)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 42usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WrongNamespace)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 43usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::CollectionNotEmpty)
                            })();
                        }
                        #[allow(clippy::unnecessary_cast)]
                        __codec_x_edqy if __codec_x_edqy
                            == 44usize as ::core::primitive::u8 => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Ok(Error::WitnessRequired)
                            })();
                        }
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            return (move || {
                                ::core::result::Result::Err(
                                    <_ as ::core::convert::Into<
                                        _,
                                    >>::into("Could not decode `Error`, variant doesn't exist"),
                                )
                            })();
                        }
                    }
                }
            }
        };
        impl TryFrom<u32> for Error {
            type Error = PopApiError;
            fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
                use Error::*;
                match status_code {
                    0 => Ok(NoPermission),
                    1 => Ok(UnknownCollection),
                    2 => Ok(AlreadyExists),
                    3 => Ok(ApprovalExpired),
                    4 => Ok(WrongOwner),
                    5 => Ok(BadWitness),
                    6 => Ok(CollectionIdInUse),
                    7 => Ok(ItemsNonTransferable),
                    8 => Ok(NotDelegate),
                    9 => Ok(WrongDelegate),
                    10 => Ok(Unapproved),
                    11 => Ok(Unaccepted),
                    12 => Ok(ItemLocked),
                    13 => Ok(LockedItemAttributes),
                    14 => Ok(LockedCollectionAttributes),
                    15 => Ok(LockedItemMetadata),
                    16 => Ok(LockedCollectionMetadata),
                    17 => Ok(MaxSupplyReached),
                    18 => Ok(MaxSupplyLocked),
                    19 => Ok(MaxSupplyTooSmall),
                    20 => Ok(UnknownItem),
                    21 => Ok(UnknownSwap),
                    22 => Ok(MetadataNotFound),
                    23 => Ok(AttributeNotFound),
                    24 => Ok(NotForSale),
                    25 => Ok(BidTooLow),
                    26 => Ok(ReachedApprovalLimit),
                    27 => Ok(DeadlineExpired),
                    28 => Ok(WrongDuration),
                    29 => Ok(MethodDisabled),
                    30 => Ok(WrongSetting),
                    31 => Ok(InconsistentItemConfig),
                    32 => Ok(NoConfig),
                    33 => Ok(RolesNotCleared),
                    34 => Ok(MintNotStarted),
                    35 => Ok(MintEnded),
                    36 => Ok(AlreadyClaimed),
                    37 => Ok(IncorrectData),
                    38 => Ok(WrongOrigin),
                    39 => Ok(WrongSignature),
                    40 => Ok(IncorrectMetadata),
                    41 => Ok(MaxAttributesLimitReached),
                    42 => Ok(WrongNamespace),
                    43 => Ok(CollectionNotEmpty),
                    44 => Ok(WitnessRequired),
                    _ => Err(UnknownModuleStatusCode(status_code)),
                }
            }
        }
        impl From<PopApiError> for Error {
            fn from(error: PopApiError) -> Self {
                match error {
                    PopApiError::Nfts(e) => e,
                    _ => {
                        ::core::panicking::panic_fmt(
                            format_args!(
                                "unexpected pallet nfts error. This error is unknown to pallet nfts",
                            ),
                        );
                    }
                }
            }
        }
        mod types {
            use super::*;
            use crate::{
                primitives::{CollectionId, ItemId},
                Balance, BlockNumber,
            };
            pub use enumflags2::{bitflags, BitFlags};
            use scale::{Decode, EncodeLike, MaxEncodedLen};
            use scale_info::{
                build::Fields, meta_type, prelude::vec, Path, Type, TypeInfo,
                TypeParameter,
            };
            /// Attribute namespaces for non-fungible tokens.
            pub enum AttributeNamespace<AccountId> {
                /// An attribute was set by the pallet.
                Pallet,
                /// An attribute was set by collection's owner.
                CollectionOwner,
                /// An attribute was set by item's owner.
                ItemOwner,
                /// An attribute was set by pre-approved account.
                Account(AccountId),
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl<AccountId> ::scale::Encode for AttributeNamespace<AccountId>
                where
                    AccountId: ::scale::Encode,
                    AccountId: ::scale::Encode,
                {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                AttributeNamespace::Pallet => 0_usize,
                                AttributeNamespace::CollectionOwner => 0_usize,
                                AttributeNamespace::ItemOwner => 0_usize,
                                AttributeNamespace::Account(ref aa) => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(aa))
                                }
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            AttributeNamespace::Pallet => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(0usize as ::core::primitive::u8);
                            }
                            AttributeNamespace::CollectionOwner => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(1usize as ::core::primitive::u8);
                            }
                            AttributeNamespace::ItemOwner => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(2usize as ::core::primitive::u8);
                            }
                            AttributeNamespace::Account(ref aa) => {
                                __codec_dest_edqy
                                    .push_byte(3usize as ::core::primitive::u8);
                                ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl<AccountId> ::scale::EncodeLike for AttributeNamespace<AccountId>
                where
                    AccountId: ::scale::Encode,
                    AccountId: ::scale::Encode,
                {}
            };
            /// Collection's configuration.
            pub struct CollectionConfig {
                /// Collection's settings.
                pub settings: CollectionSettings,
                /// Collection's max supply.
                pub max_supply: Option<u32>,
                /// Default settings each item will get during the mint.
                pub mint_settings: MintSettings,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for CollectionConfig {
                    fn size_hint(&self) -> usize {
                        0_usize
                            .saturating_add(::scale::Encode::size_hint(&self.settings))
                            .saturating_add(::scale::Encode::size_hint(&self.max_supply))
                            .saturating_add(
                                ::scale::Encode::size_hint(&self.mint_settings),
                            )
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        ::scale::Encode::encode_to(&self.settings, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.max_supply, __codec_dest_edqy);
                        ::scale::Encode::encode_to(
                            &self.mint_settings,
                            __codec_dest_edqy,
                        );
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for CollectionConfig {}
            };
            /// Information about a collection.
            pub struct CollectionDetails {
                /// Collection's owner.
                pub owner: AccountId,
                /// The total balance deposited by the owner for all the storage data associated with this
                /// collection. Used by `destroy`.
                pub owner_deposit: Balance,
                /// The total number of outstanding items of this collection.
                pub items: u32,
                /// The total number of outstanding item metadata of this collection.
                pub item_metadatas: u32,
                /// The total number of outstanding item configs of this collection.
                pub item_configs: u32,
                /// The total number of attributes for this collection.
                pub attributes: u32,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Decode for CollectionDetails {
                    fn decode<__CodecInputEdqy: ::scale::Input>(
                        __codec_input_edqy: &mut __CodecInputEdqy,
                    ) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(CollectionDetails {
                            owner: {
                                let __codec_res_edqy = <AccountId as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `CollectionDetails::owner`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            owner_deposit: {
                                let __codec_res_edqy = <Balance as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e
                                                .chain(
                                                    "Could not decode `CollectionDetails::owner_deposit`",
                                                ),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            items: {
                                let __codec_res_edqy = <u32 as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `CollectionDetails::items`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            item_metadatas: {
                                let __codec_res_edqy = <u32 as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e
                                                .chain(
                                                    "Could not decode `CollectionDetails::item_metadatas`",
                                                ),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            item_configs: {
                                let __codec_res_edqy = <u32 as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e
                                                .chain("Could not decode `CollectionDetails::item_configs`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            attributes: {
                                let __codec_res_edqy = <u32 as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `CollectionDetails::attributes`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                }
            };
            #[automatically_derived]
            impl ::core::fmt::Debug for CollectionDetails {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    let names: &'static _ = &[
                        "owner",
                        "owner_deposit",
                        "items",
                        "item_metadatas",
                        "item_configs",
                        "attributes",
                    ];
                    let values: &[&dyn ::core::fmt::Debug] = &[
                        &self.owner,
                        &self.owner_deposit,
                        &self.items,
                        &self.item_metadatas,
                        &self.item_configs,
                        &&self.attributes,
                    ];
                    ::core::fmt::Formatter::debug_struct_fields_finish(
                        f,
                        "CollectionDetails",
                        names,
                        values,
                    )
                }
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for CollectionDetails {
                    fn size_hint(&self) -> usize {
                        0_usize
                            .saturating_add(::scale::Encode::size_hint(&self.owner))
                            .saturating_add(
                                ::scale::Encode::size_hint(&self.owner_deposit),
                            )
                            .saturating_add(::scale::Encode::size_hint(&self.items))
                            .saturating_add(
                                ::scale::Encode::size_hint(&self.item_metadatas),
                            )
                            .saturating_add(
                                ::scale::Encode::size_hint(&self.item_configs),
                            )
                            .saturating_add(::scale::Encode::size_hint(&self.attributes))
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        ::scale::Encode::encode_to(&self.owner, __codec_dest_edqy);
                        ::scale::Encode::encode_to(
                            &self.owner_deposit,
                            __codec_dest_edqy,
                        );
                        ::scale::Encode::encode_to(&self.items, __codec_dest_edqy);
                        ::scale::Encode::encode_to(
                            &self.item_metadatas,
                            __codec_dest_edqy,
                        );
                        ::scale::Encode::encode_to(
                            &self.item_configs,
                            __codec_dest_edqy,
                        );
                        ::scale::Encode::encode_to(&self.attributes, __codec_dest_edqy);
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for CollectionDetails {}
            };
            #[automatically_derived]
            impl ::core::cmp::Eq for CollectionDetails {
                #[inline]
                #[doc(hidden)]
                #[coverage(off)]
                fn assert_receiver_is_total_eq(&self) -> () {
                    let _: ::core::cmp::AssertParamIsEq<AccountId>;
                    let _: ::core::cmp::AssertParamIsEq<Balance>;
                    let _: ::core::cmp::AssertParamIsEq<u32>;
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for CollectionDetails {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for CollectionDetails {
                #[inline]
                fn eq(&self, other: &CollectionDetails) -> bool {
                    self.owner == other.owner
                        && self.owner_deposit == other.owner_deposit
                        && self.items == other.items
                        && self.item_metadatas == other.item_metadatas
                        && self.item_configs == other.item_configs
                        && self.attributes == other.attributes
                }
            }
            /// Wrapper type for `BitFlags<CollectionSetting>` that implements `Codec`.
            pub struct CollectionSettings(pub BitFlags<CollectionSetting>);
            impl MaxEncodedLen for CollectionSettings {
                fn max_encoded_len() -> usize {
                    <u64>::max_encoded_len()
                }
            }
            impl Encode for CollectionSettings {
                fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
                    self.0.bits().using_encoded(f)
                }
            }
            impl EncodeLike for CollectionSettings {}
            impl Decode for CollectionSettings {
                fn decode<I: scale::Input>(
                    input: &mut I,
                ) -> core::result::Result<Self, scale::Error> {
                    let field = <u64>::decode(input)?;
                    Ok(
                        Self(
                            BitFlags::from_bits(field as u64)
                                .map_err(|_| "invalid value")?,
                        ),
                    )
                }
            }
            impl TypeInfo for CollectionSettings {
                type Identity = Self;
                fn type_info() -> Type {
                    Type::builder()
                        .path(Path::new("BitFlags", "pop_api::v0::nfts::types"))
                        .type_params(
                            <[_]>::into_vec(
                                #[rustc_box]
                                ::alloc::boxed::Box::new([
                                    TypeParameter::new(
                                        "T",
                                        Some(meta_type::<CollectionSetting>()),
                                    ),
                                ]),
                            ),
                        )
                        .composite(
                            Fields::unnamed()
                                .field(|f| f.ty::<u64>().type_name("CollectionSetting")),
                        )
                }
            }
            /// Support for up to 64 user-enabled features on a collection.
            #[repr(u64)]
            pub enum CollectionSetting {
                /// Items in this collection are transferable.
                TransferableItems = 1,
                /// The metadata of this collection can be modified.
                UnlockedMetadata = ::enumflags2::_internal::next_bit(
                    CollectionSetting::TransferableItems as u128,
                ) as u64,
                /// Attributes of this collection can be modified.
                UnlockedAttributes = ::enumflags2::_internal::next_bit(
                    CollectionSetting::TransferableItems as u128
                        | CollectionSetting::UnlockedMetadata as u128,
                ) as u64,
                /// The supply of this collection can be modified.
                UnlockedMaxSupply = ::enumflags2::_internal::next_bit(
                    CollectionSetting::TransferableItems as u128
                        | CollectionSetting::UnlockedMetadata as u128
                        | CollectionSetting::UnlockedAttributes as u128,
                ) as u64,
                /// When this isn't set then the deposit is required to hold the items of this collection.
                DepositRequired = ::enumflags2::_internal::next_bit(
                    CollectionSetting::TransferableItems as u128
                        | CollectionSetting::UnlockedMetadata as u128
                        | CollectionSetting::UnlockedAttributes as u128
                        | CollectionSetting::UnlockedMaxSupply as u128,
                ) as u64,
            }
            #[automatically_derived]
            impl ::core::marker::Copy for CollectionSetting {}
            #[automatically_derived]
            impl ::core::clone::Clone for CollectionSetting {
                #[inline]
                fn clone(&self) -> CollectionSetting {
                    *self
                }
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for CollectionSetting {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                CollectionSetting::TransferableItems => 0_usize,
                                CollectionSetting::UnlockedMetadata => 0_usize,
                                CollectionSetting::UnlockedAttributes => 0_usize,
                                CollectionSetting::UnlockedMaxSupply => 0_usize,
                                CollectionSetting::DepositRequired => 0_usize,
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            CollectionSetting::TransferableItems => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy.push_byte(1 as ::core::primitive::u8);
                            }
                            CollectionSetting::UnlockedMetadata => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(
                                        ::enumflags2::_internal::next_bit(
                                            CollectionSetting::TransferableItems as u128,
                                        ) as u64 as ::core::primitive::u8,
                                    );
                            }
                            CollectionSetting::UnlockedAttributes => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(
                                        ::enumflags2::_internal::next_bit(
                                            CollectionSetting::TransferableItems as u128
                                                | CollectionSetting::UnlockedMetadata as u128,
                                        ) as u64 as ::core::primitive::u8,
                                    );
                            }
                            CollectionSetting::UnlockedMaxSupply => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(
                                        ::enumflags2::_internal::next_bit(
                                            CollectionSetting::TransferableItems as u128
                                                | CollectionSetting::UnlockedMetadata as u128
                                                | CollectionSetting::UnlockedAttributes as u128,
                                        ) as u64 as ::core::primitive::u8,
                                    );
                            }
                            CollectionSetting::DepositRequired => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(
                                        ::enumflags2::_internal::next_bit(
                                            CollectionSetting::TransferableItems as u128
                                                | CollectionSetting::UnlockedMetadata as u128
                                                | CollectionSetting::UnlockedAttributes as u128
                                                | CollectionSetting::UnlockedMaxSupply as u128,
                                        ) as u64 as ::core::primitive::u8,
                                    );
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for CollectionSetting {}
            };
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                impl ::scale_info::TypeInfo for CollectionSetting {
                    type Identity = Self;
                    fn type_info() -> ::scale_info::Type {
                        ::scale_info::Type::builder()
                            .path(
                                ::scale_info::Path::new_with_replace(
                                    "CollectionSetting",
                                    "pop_api::v0::nfts::types",
                                    &[],
                                ),
                            )
                            .type_params(::alloc::vec::Vec::new())
                            .docs(
                                &[
                                    "Support for up to 64 user-enabled features on a collection.",
                                ],
                            )
                            .variant(
                                ::scale_info::build::Variants::new()
                                    .variant(
                                        "TransferableItems",
                                        |v| {
                                            v
                                                .index(1 as ::core::primitive::u8)
                                                .docs(&["Items in this collection are transferable."])
                                        },
                                    )
                                    .variant(
                                        "UnlockedMetadata",
                                        |v| {
                                            v
                                                .index(
                                                    ::enumflags2::_internal::next_bit(
                                                        CollectionSetting::TransferableItems as u128,
                                                    ) as u64 as ::core::primitive::u8,
                                                )
                                                .docs(&["The metadata of this collection can be modified."])
                                        },
                                    )
                                    .variant(
                                        "UnlockedAttributes",
                                        |v| {
                                            v
                                                .index(
                                                    ::enumflags2::_internal::next_bit(
                                                        CollectionSetting::TransferableItems as u128
                                                            | CollectionSetting::UnlockedMetadata as u128,
                                                    ) as u64 as ::core::primitive::u8,
                                                )
                                                .docs(&["Attributes of this collection can be modified."])
                                        },
                                    )
                                    .variant(
                                        "UnlockedMaxSupply",
                                        |v| {
                                            v
                                                .index(
                                                    ::enumflags2::_internal::next_bit(
                                                        CollectionSetting::TransferableItems as u128
                                                            | CollectionSetting::UnlockedMetadata as u128
                                                            | CollectionSetting::UnlockedAttributes as u128,
                                                    ) as u64 as ::core::primitive::u8,
                                                )
                                                .docs(&["The supply of this collection can be modified."])
                                        },
                                    )
                                    .variant(
                                        "DepositRequired",
                                        |v| {
                                            v
                                                .index(
                                                    ::enumflags2::_internal::next_bit(
                                                        CollectionSetting::TransferableItems as u128
                                                            | CollectionSetting::UnlockedMetadata as u128
                                                            | CollectionSetting::UnlockedAttributes as u128
                                                            | CollectionSetting::UnlockedMaxSupply as u128,
                                                    ) as u64 as ::core::primitive::u8,
                                                )
                                                .docs(
                                                    &[
                                                        "When this isn't set then the deposit is required to hold the items of this collection.",
                                                    ],
                                                )
                                        },
                                    ),
                            )
                    }
                }
            };
            impl ::enumflags2::_internal::core::ops::Not for CollectionSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn not(self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self).not()
                }
            }
            impl ::enumflags2::_internal::core::ops::BitOr for CollectionSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn bitor(self, other: Self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self) | other
                }
            }
            impl ::enumflags2::_internal::core::ops::BitAnd for CollectionSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn bitand(self, other: Self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self) & other
                }
            }
            impl ::enumflags2::_internal::core::ops::BitXor for CollectionSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn bitxor(self, other: Self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self) ^ other
                }
            }
            unsafe impl ::enumflags2::_internal::RawBitFlags for CollectionSetting {
                type Numeric = u64;
                const EMPTY: Self::Numeric = 0;
                const DEFAULT: Self::Numeric = 0;
                const ALL_BITS: Self::Numeric = 0 | (Self::TransferableItems as u64)
                    | (Self::UnlockedMetadata as u64) | (Self::UnlockedAttributes as u64)
                    | (Self::UnlockedMaxSupply as u64) | (Self::DepositRequired as u64);
                const BITFLAGS_TYPE_NAME: &'static str = "BitFlags<CollectionSetting>";
                fn bits(self) -> Self::Numeric {
                    self as u64
                }
            }
            impl ::enumflags2::BitFlag for CollectionSetting {}
            /// Information concerning the ownership of a single unique item.
            pub struct ItemDetails {
                /// The owner of this item.
                pub owner: AccountId,
                /// The approved transferrer of this item, if one is set.
                pub approvals: BoundedBTreeMap<
                    AccountId,
                    Option<BlockNumber>,
                    ApprovalsLimit,
                >,
                /// The amount held in the pallet's default account for this item. Free-hold items will
                /// have this as zero.
                pub deposit: Balance,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Decode for ItemDetails {
                    fn decode<__CodecInputEdqy: ::scale::Input>(
                        __codec_input_edqy: &mut __CodecInputEdqy,
                    ) -> ::core::result::Result<Self, ::scale::Error> {
                        ::core::result::Result::Ok(ItemDetails {
                            owner: {
                                let __codec_res_edqy = <AccountId as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `ItemDetails::owner`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            approvals: {
                                let __codec_res_edqy = <BoundedBTreeMap<
                                    AccountId,
                                    Option<BlockNumber>,
                                    ApprovalsLimit,
                                > as ::scale::Decode>::decode(__codec_input_edqy);
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `ItemDetails::approvals`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                            deposit: {
                                let __codec_res_edqy = <Balance as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `ItemDetails::deposit`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            },
                        })
                    }
                }
            };
            #[automatically_derived]
            impl ::core::fmt::Debug for ItemDetails {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "ItemDetails",
                        "owner",
                        &self.owner,
                        "approvals",
                        &self.approvals,
                        "deposit",
                        &&self.deposit,
                    )
                }
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for ItemDetails {
                    fn size_hint(&self) -> usize {
                        0_usize
                            .saturating_add(::scale::Encode::size_hint(&self.owner))
                            .saturating_add(::scale::Encode::size_hint(&self.approvals))
                            .saturating_add(::scale::Encode::size_hint(&self.deposit))
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        ::scale::Encode::encode_to(&self.owner, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.approvals, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.deposit, __codec_dest_edqy);
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for ItemDetails {}
            };
            #[automatically_derived]
            impl ::core::cmp::Eq for ItemDetails {
                #[inline]
                #[doc(hidden)]
                #[coverage(off)]
                fn assert_receiver_is_total_eq(&self) -> () {
                    let _: ::core::cmp::AssertParamIsEq<AccountId>;
                    let _: ::core::cmp::AssertParamIsEq<
                        BoundedBTreeMap<AccountId, Option<BlockNumber>, ApprovalsLimit>,
                    >;
                    let _: ::core::cmp::AssertParamIsEq<Balance>;
                }
            }
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for ItemDetails {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for ItemDetails {
                #[inline]
                fn eq(&self, other: &ItemDetails) -> bool {
                    self.owner == other.owner && self.approvals == other.approvals
                        && self.deposit == other.deposit
                }
            }
            /// Support for up to 64 user-enabled features on an item.
            #[repr(u64)]
            pub enum ItemSetting {
                /// This item is transferable.
                Transferable = 1,
                /// The metadata of this item can be modified.
                UnlockedMetadata = ::enumflags2::_internal::next_bit(
                    ItemSetting::Transferable as u128,
                ) as u64,
                /// Attributes of this item can be modified.
                UnlockedAttributes = ::enumflags2::_internal::next_bit(
                    ItemSetting::Transferable as u128
                        | ItemSetting::UnlockedMetadata as u128,
                ) as u64,
            }
            #[automatically_derived]
            impl ::core::marker::Copy for ItemSetting {}
            #[automatically_derived]
            impl ::core::clone::Clone for ItemSetting {
                #[inline]
                fn clone(&self) -> ItemSetting {
                    *self
                }
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for ItemSetting {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                ItemSetting::Transferable => 0_usize,
                                ItemSetting::UnlockedMetadata => 0_usize,
                                ItemSetting::UnlockedAttributes => 0_usize,
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            ItemSetting::Transferable => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy.push_byte(1 as ::core::primitive::u8);
                            }
                            ItemSetting::UnlockedMetadata => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(
                                        ::enumflags2::_internal::next_bit(
                                            ItemSetting::Transferable as u128,
                                        ) as u64 as ::core::primitive::u8,
                                    );
                            }
                            ItemSetting::UnlockedAttributes => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(
                                        ::enumflags2::_internal::next_bit(
                                            ItemSetting::Transferable as u128
                                                | ItemSetting::UnlockedMetadata as u128,
                                        ) as u64 as ::core::primitive::u8,
                                    );
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for ItemSetting {}
            };
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                impl ::scale_info::TypeInfo for ItemSetting {
                    type Identity = Self;
                    fn type_info() -> ::scale_info::Type {
                        ::scale_info::Type::builder()
                            .path(
                                ::scale_info::Path::new_with_replace(
                                    "ItemSetting",
                                    "pop_api::v0::nfts::types",
                                    &[],
                                ),
                            )
                            .type_params(::alloc::vec::Vec::new())
                            .docs(
                                &["Support for up to 64 user-enabled features on an item."],
                            )
                            .variant(
                                ::scale_info::build::Variants::new()
                                    .variant(
                                        "Transferable",
                                        |v| {
                                            v
                                                .index(1 as ::core::primitive::u8)
                                                .docs(&["This item is transferable."])
                                        },
                                    )
                                    .variant(
                                        "UnlockedMetadata",
                                        |v| {
                                            v
                                                .index(
                                                    ::enumflags2::_internal::next_bit(
                                                        ItemSetting::Transferable as u128,
                                                    ) as u64 as ::core::primitive::u8,
                                                )
                                                .docs(&["The metadata of this item can be modified."])
                                        },
                                    )
                                    .variant(
                                        "UnlockedAttributes",
                                        |v| {
                                            v
                                                .index(
                                                    ::enumflags2::_internal::next_bit(
                                                        ItemSetting::Transferable as u128
                                                            | ItemSetting::UnlockedMetadata as u128,
                                                    ) as u64 as ::core::primitive::u8,
                                                )
                                                .docs(&["Attributes of this item can be modified."])
                                        },
                                    ),
                            )
                    }
                }
            };
            impl ::enumflags2::_internal::core::ops::Not for ItemSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn not(self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self).not()
                }
            }
            impl ::enumflags2::_internal::core::ops::BitOr for ItemSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn bitor(self, other: Self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self) | other
                }
            }
            impl ::enumflags2::_internal::core::ops::BitAnd for ItemSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn bitand(self, other: Self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self) & other
                }
            }
            impl ::enumflags2::_internal::core::ops::BitXor for ItemSetting {
                type Output = ::enumflags2::BitFlags<Self>;
                #[inline(always)]
                fn bitxor(self, other: Self) -> Self::Output {
                    use ::enumflags2::BitFlags;
                    BitFlags::from_flag(self) ^ other
                }
            }
            unsafe impl ::enumflags2::_internal::RawBitFlags for ItemSetting {
                type Numeric = u64;
                const EMPTY: Self::Numeric = 0;
                const DEFAULT: Self::Numeric = 0;
                const ALL_BITS: Self::Numeric = 0 | (Self::Transferable as u64)
                    | (Self::UnlockedMetadata as u64)
                    | (Self::UnlockedAttributes as u64);
                const BITFLAGS_TYPE_NAME: &'static str = "BitFlags<ItemSetting>";
                fn bits(self) -> Self::Numeric {
                    self as u64
                }
            }
            impl ::enumflags2::BitFlag for ItemSetting {}
            /// Wrapper type for `BitFlags<ItemSetting>` that implements `Codec`.
            pub struct ItemSettings(pub BitFlags<ItemSetting>);
            impl MaxEncodedLen for ItemSettings {
                fn max_encoded_len() -> usize {
                    <u64>::max_encoded_len()
                }
            }
            impl Encode for ItemSettings {
                fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
                    self.0.bits().using_encoded(f)
                }
            }
            impl EncodeLike for ItemSettings {}
            impl Decode for ItemSettings {
                fn decode<I: scale::Input>(
                    input: &mut I,
                ) -> core::result::Result<Self, scale::Error> {
                    let field = <u64>::decode(input)?;
                    Ok(
                        Self(
                            BitFlags::from_bits(field as u64)
                                .map_err(|_| "invalid value")?,
                        ),
                    )
                }
            }
            impl TypeInfo for ItemSettings {
                type Identity = Self;
                fn type_info() -> Type {
                    Type::builder()
                        .path(Path::new("BitFlags", "pop_api::v0::nfts::types"))
                        .type_params(
                            <[_]>::into_vec(
                                #[rustc_box]
                                ::alloc::boxed::Box::new([
                                    TypeParameter::new("T", Some(meta_type::<ItemSetting>())),
                                ]),
                            ),
                        )
                        .composite(
                            Fields::unnamed()
                                .field(|f| f.ty::<u64>().type_name("ItemSetting")),
                        )
                }
            }
            /// Information about the tip.
            pub struct ItemTip {
                /// The collection of the item.
                pub(super) collection: CollectionId,
                /// An item of which the tip is sent for.
                pub(super) item: ItemId,
                /// A sender of the tip.
                pub(super) receiver: AccountId,
                /// An amount the sender is willing to tip.
                pub(super) amount: Balance,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for ItemTip {
                    fn size_hint(&self) -> usize {
                        0_usize
                            .saturating_add(::scale::Encode::size_hint(&self.collection))
                            .saturating_add(::scale::Encode::size_hint(&self.item))
                            .saturating_add(::scale::Encode::size_hint(&self.receiver))
                            .saturating_add(::scale::Encode::size_hint(&self.amount))
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        ::scale::Encode::encode_to(&self.collection, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.item, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.receiver, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.amount, __codec_dest_edqy);
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for ItemTip {}
            };
            /// Holds the information about minting.
            pub struct MintSettings {
                /// Whether anyone can mint or if minters are restricted to some subset.
                pub mint_type: MintType,
                /// An optional price per mint.
                pub price: Option<Balance>,
                /// When the mint starts.
                pub start_block: Option<BlockNumber>,
                /// When the mint ends.
                pub end_block: Option<BlockNumber>,
                /// Default settings each item will get during the mint.
                pub default_item_settings: ItemSettings,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for MintSettings {
                    fn size_hint(&self) -> usize {
                        0_usize
                            .saturating_add(::scale::Encode::size_hint(&self.mint_type))
                            .saturating_add(::scale::Encode::size_hint(&self.price))
                            .saturating_add(
                                ::scale::Encode::size_hint(&self.start_block),
                            )
                            .saturating_add(::scale::Encode::size_hint(&self.end_block))
                            .saturating_add(
                                ::scale::Encode::size_hint(&self.default_item_settings),
                            )
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        ::scale::Encode::encode_to(&self.mint_type, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.price, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.start_block, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.end_block, __codec_dest_edqy);
                        ::scale::Encode::encode_to(
                            &self.default_item_settings,
                            __codec_dest_edqy,
                        );
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for MintSettings {}
            };
            /// Mint type. Can the NFT be created by anyone, or only the creator of the collection,
            /// or only by wallets that already hold an NFT from a certain collection?
            /// The ownership of a privately minted NFT is still publicly visible.
            pub enum MintType {
                /// Only an `Issuer` could mint items.
                Issuer,
                /// Anyone could mint items.
                Public,
                /// Only holders of items in specified collection could mint new items.
                HolderOf(CollectionId),
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for MintType {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                MintType::Issuer => 0_usize,
                                MintType::Public => 0_usize,
                                MintType::HolderOf(ref aa) => {
                                    0_usize.saturating_add(::scale::Encode::size_hint(aa))
                                }
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            MintType::Issuer => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(0usize as ::core::primitive::u8);
                            }
                            MintType::Public => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(1usize as ::core::primitive::u8);
                            }
                            MintType::HolderOf(ref aa) => {
                                __codec_dest_edqy
                                    .push_byte(2usize as ::core::primitive::u8);
                                ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for MintType {}
            };
            /// Holds the details about the price.
            pub struct PriceWithDirection {
                /// An amount.
                pub(super) amount: Balance,
                /// A direction (send or receive).
                pub(super) direction: PriceDirection,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for PriceWithDirection {
                    fn size_hint(&self) -> usize {
                        0_usize
                            .saturating_add(::scale::Encode::size_hint(&self.amount))
                            .saturating_add(::scale::Encode::size_hint(&self.direction))
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        ::scale::Encode::encode_to(&self.amount, __codec_dest_edqy);
                        ::scale::Encode::encode_to(&self.direction, __codec_dest_edqy);
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for PriceWithDirection {}
            };
            /// Specifies whether the tokens will be sent or received.
            pub enum PriceDirection {
                /// Tokens will be sent.
                Send,
                /// Tokens will be received.
                Receive,
            }
            #[allow(deprecated)]
            const _: () = {
                #[automatically_derived]
                impl ::scale::Encode for PriceDirection {
                    fn size_hint(&self) -> usize {
                        1_usize
                            + match *self {
                                PriceDirection::Send => 0_usize,
                                PriceDirection::Receive => 0_usize,
                                _ => 0_usize,
                            }
                    }
                    fn encode_to<
                        __CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized,
                    >(&self, __codec_dest_edqy: &mut __CodecOutputEdqy) {
                        match *self {
                            PriceDirection::Send => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(0usize as ::core::primitive::u8);
                            }
                            PriceDirection::Receive => {
                                #[allow(clippy::unnecessary_cast)]
                                __codec_dest_edqy
                                    .push_byte(1usize as ::core::primitive::u8);
                            }
                            _ => {}
                        }
                    }
                }
                #[automatically_derived]
                impl ::scale::EncodeLike for PriceDirection {}
            };
            pub(crate) use impl_codec_bitflags;
        }
    }
    pub mod state {
        use crate::{primitives::storage_keys::RuntimeStateKeys, read_state, PopApiError};
        use scale::Decode;
        pub fn read<T: Decode>(key: RuntimeStateKeys) -> crate::Result<T> {
            read_state(key)
                .and_then(|v| {
                    T::decode(&mut &v[..]).map_err(|_e| PopApiError::DecodingFailed)
                })
        }
    }
    pub fn relay_chain_block_number() -> Result<BlockNumber, PopApiError> {
        state::read(
            RuntimeStateKeys::ParachainSystem(
                ParachainSystemKeys::LastRelayChainBlockNumber,
            ),
        )
    }
    pub(crate) enum RuntimeCall {
        #[codec(index = 10)]
        Balances(balances::BalancesCall),
        #[codec(index = 50)]
        Nfts(nfts::NftCalls),
        #[codec(index = 52)]
        Assets(assets::fungibles::AssetsCall),
    }
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl ::scale::Encode for RuntimeCall {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        RuntimeCall::Balances(ref aa) => {
                            0_usize.saturating_add(::scale::Encode::size_hint(aa))
                        }
                        RuntimeCall::Nfts(ref aa) => {
                            0_usize.saturating_add(::scale::Encode::size_hint(aa))
                        }
                        RuntimeCall::Assets(ref aa) => {
                            0_usize.saturating_add(::scale::Encode::size_hint(aa))
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    RuntimeCall::Balances(ref aa) => {
                        __codec_dest_edqy.push_byte(10u8 as ::core::primitive::u8);
                        ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    RuntimeCall::Nfts(ref aa) => {
                        __codec_dest_edqy.push_byte(50u8 as ::core::primitive::u8);
                        ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    RuntimeCall::Assets(ref aa) => {
                        __codec_dest_edqy.push_byte(52u8 as ::core::primitive::u8);
                        ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl ::scale::EncodeLike for RuntimeCall {}
    };
}
type AccountId = AccountId32;
type Balance = <Environment as ink::env::Environment>::Balance;
type BlockNumber = <Environment as ink::env::Environment>::BlockNumber;
type StringLimit = u32;
type MaxTips = u32;
pub type Result<T> = core::result::Result<T, PopApiError>;
pub enum PopApiError {
    Assets(assets::fungibles::AssetsError),
    Balances(balances::BalancesError),
    Contracts(contracts::Error),
    DecodingFailed,
    Nfts(nfts::Error),
    SystemCallFiltered,
    TokenError(dispatch_error::TokenError),
    UnknownModuleStatusCode(u32),
    UnknownDispatchStatusCode(u32),
    Xcm(cross_chain::Error),
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for PopApiError {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(
                    ::scale_info::Path::new_with_replace("PopApiError", "pop_api", &[]),
                )
                .type_params(::alloc::vec::Vec::new())
                .variant(
                    ::scale_info::build::Variants::new()
                        .variant(
                            "Assets",
                            |v| {
                                v
                                    .index(0usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| {
                                                f
                                                    .ty::<assets::fungibles::AssetsError>()
                                                    .type_name("assets::fungibles::AssetsError")
                                            }),
                                    )
                            },
                        )
                        .variant(
                            "Balances",
                            |v| {
                                v
                                    .index(1usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| {
                                                f
                                                    .ty::<balances::BalancesError>()
                                                    .type_name("balances::BalancesError")
                                            }),
                                    )
                            },
                        )
                        .variant(
                            "Contracts",
                            |v| {
                                v
                                    .index(2usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| {
                                                f.ty::<contracts::Error>().type_name("contracts::Error")
                                            }),
                                    )
                            },
                        )
                        .variant(
                            "DecodingFailed",
                            |v| v.index(3usize as ::core::primitive::u8),
                        )
                        .variant(
                            "Nfts",
                            |v| {
                                v
                                    .index(4usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| f.ty::<nfts::Error>().type_name("nfts::Error")),
                                    )
                            },
                        )
                        .variant(
                            "SystemCallFiltered",
                            |v| v.index(5usize as ::core::primitive::u8),
                        )
                        .variant(
                            "TokenError",
                            |v| {
                                v
                                    .index(6usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| {
                                                f
                                                    .ty::<dispatch_error::TokenError>()
                                                    .type_name("dispatch_error::TokenError")
                                            }),
                                    )
                            },
                        )
                        .variant(
                            "UnknownModuleStatusCode",
                            |v| {
                                v
                                    .index(7usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| f.ty::<u32>().type_name("u32")),
                                    )
                            },
                        )
                        .variant(
                            "UnknownDispatchStatusCode",
                            |v| {
                                v
                                    .index(8usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| f.ty::<u32>().type_name("u32")),
                                    )
                            },
                        )
                        .variant(
                            "Xcm",
                            |v| {
                                v
                                    .index(9usize as ::core::primitive::u8)
                                    .fields(
                                        ::scale_info::build::Fields::unnamed()
                                            .field(|f| {
                                                f.ty::<cross_chain::Error>().type_name("cross_chain::Error")
                                            }),
                                    )
                            },
                        ),
                )
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for PopApiError {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            PopApiError::Assets(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Assets", &__self_0)
            }
            PopApiError::Balances(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Balances",
                    &__self_0,
                )
            }
            PopApiError::Contracts(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Contracts",
                    &__self_0,
                )
            }
            PopApiError::DecodingFailed => {
                ::core::fmt::Formatter::write_str(f, "DecodingFailed")
            }
            PopApiError::Nfts(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Nfts", &__self_0)
            }
            PopApiError::SystemCallFiltered => {
                ::core::fmt::Formatter::write_str(f, "SystemCallFiltered")
            }
            PopApiError::TokenError(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "TokenError",
                    &__self_0,
                )
            }
            PopApiError::UnknownModuleStatusCode(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "UnknownModuleStatusCode",
                    &__self_0,
                )
            }
            PopApiError::UnknownDispatchStatusCode(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "UnknownDispatchStatusCode",
                    &__self_0,
                )
            }
            PopApiError::Xcm(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Xcm", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::Copy for PopApiError {}
#[automatically_derived]
impl ::core::clone::Clone for PopApiError {
    #[inline]
    fn clone(&self) -> PopApiError {
        let _: ::core::clone::AssertParamIsClone<assets::fungibles::AssetsError>;
        let _: ::core::clone::AssertParamIsClone<balances::BalancesError>;
        let _: ::core::clone::AssertParamIsClone<contracts::Error>;
        let _: ::core::clone::AssertParamIsClone<nfts::Error>;
        let _: ::core::clone::AssertParamIsClone<dispatch_error::TokenError>;
        let _: ::core::clone::AssertParamIsClone<u32>;
        let _: ::core::clone::AssertParamIsClone<cross_chain::Error>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PopApiError {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PopApiError {
    #[inline]
    fn eq(&self, other: &PopApiError) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
            && match (self, other) {
                (PopApiError::Assets(__self_0), PopApiError::Assets(__arg1_0)) => {
                    *__self_0 == *__arg1_0
                }
                (PopApiError::Balances(__self_0), PopApiError::Balances(__arg1_0)) => {
                    *__self_0 == *__arg1_0
                }
                (PopApiError::Contracts(__self_0), PopApiError::Contracts(__arg1_0)) => {
                    *__self_0 == *__arg1_0
                }
                (PopApiError::Nfts(__self_0), PopApiError::Nfts(__arg1_0)) => {
                    *__self_0 == *__arg1_0
                }
                (
                    PopApiError::TokenError(__self_0),
                    PopApiError::TokenError(__arg1_0),
                ) => *__self_0 == *__arg1_0,
                (
                    PopApiError::UnknownModuleStatusCode(__self_0),
                    PopApiError::UnknownModuleStatusCode(__arg1_0),
                ) => *__self_0 == *__arg1_0,
                (
                    PopApiError::UnknownDispatchStatusCode(__self_0),
                    PopApiError::UnknownDispatchStatusCode(__arg1_0),
                ) => *__self_0 == *__arg1_0,
                (PopApiError::Xcm(__self_0), PopApiError::Xcm(__arg1_0)) => {
                    *__self_0 == *__arg1_0
                }
                _ => true,
            }
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PopApiError {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<assets::fungibles::AssetsError>;
        let _: ::core::cmp::AssertParamIsEq<balances::BalancesError>;
        let _: ::core::cmp::AssertParamIsEq<contracts::Error>;
        let _: ::core::cmp::AssertParamIsEq<nfts::Error>;
        let _: ::core::cmp::AssertParamIsEq<dispatch_error::TokenError>;
        let _: ::core::cmp::AssertParamIsEq<u32>;
        let _: ::core::cmp::AssertParamIsEq<cross_chain::Error>;
    }
}
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::scale::Encode for PopApiError {
        fn size_hint(&self) -> usize {
            1_usize
                + match *self {
                    PopApiError::Assets(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::Balances(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::Contracts(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::DecodingFailed => 0_usize,
                    PopApiError::Nfts(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::SystemCallFiltered => 0_usize,
                    PopApiError::TokenError(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::UnknownModuleStatusCode(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::UnknownDispatchStatusCode(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    PopApiError::Xcm(ref aa) => {
                        0_usize.saturating_add(::scale::Encode::size_hint(aa))
                    }
                    _ => 0_usize,
                }
        }
        fn encode_to<__CodecOutputEdqy: ::scale::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            match *self {
                PopApiError::Assets(ref aa) => {
                    __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::Balances(ref aa) => {
                    __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::Contracts(ref aa) => {
                    __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::DecodingFailed => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(3usize as ::core::primitive::u8);
                }
                PopApiError::Nfts(ref aa) => {
                    __codec_dest_edqy.push_byte(4usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::SystemCallFiltered => {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_dest_edqy.push_byte(5usize as ::core::primitive::u8);
                }
                PopApiError::TokenError(ref aa) => {
                    __codec_dest_edqy.push_byte(6usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::UnknownModuleStatusCode(ref aa) => {
                    __codec_dest_edqy.push_byte(7usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::UnknownDispatchStatusCode(ref aa) => {
                    __codec_dest_edqy.push_byte(8usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                PopApiError::Xcm(ref aa) => {
                    __codec_dest_edqy.push_byte(9usize as ::core::primitive::u8);
                    ::scale::Encode::encode_to(aa, __codec_dest_edqy);
                }
                _ => {}
            }
        }
    }
    #[automatically_derived]
    impl ::scale::EncodeLike for PopApiError {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl ::scale::Decode for PopApiError {
        fn decode<__CodecInputEdqy: ::scale::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::scale::Error> {
            match __codec_input_edqy
                .read_byte()
                .map_err(|e| {
                    e
                        .chain(
                            "Could not decode `PopApiError`, failed to read variant byte",
                        )
                })?
            {
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 0usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::Assets({
                                let __codec_res_edqy = <assets::fungibles::AssetsError as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `PopApiError::Assets.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 1usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::Balances({
                                let __codec_res_edqy = <balances::BalancesError as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `PopApiError::Balances.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 2usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::Contracts({
                                let __codec_res_edqy = <contracts::Error as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `PopApiError::Contracts.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 3usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(PopApiError::DecodingFailed)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 4usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::Nfts({
                                let __codec_res_edqy = <nfts::Error as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `PopApiError::Nfts.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 5usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(PopApiError::SystemCallFiltered)
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 6usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::TokenError({
                                let __codec_res_edqy = <dispatch_error::TokenError as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `PopApiError::TokenError.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 7usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::UnknownModuleStatusCode({
                                let __codec_res_edqy = <u32 as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e
                                                .chain(
                                                    "Could not decode `PopApiError::UnknownModuleStatusCode.0`",
                                                ),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 8usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::UnknownDispatchStatusCode({
                                let __codec_res_edqy = <u32 as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e
                                                .chain(
                                                    "Could not decode `PopApiError::UnknownDispatchStatusCode.0`",
                                                ),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                #[allow(clippy::unnecessary_cast)]
                __codec_x_edqy if __codec_x_edqy == 9usize as ::core::primitive::u8 => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Ok(
                            PopApiError::Xcm({
                                let __codec_res_edqy = <cross_chain::Error as ::scale::Decode>::decode(
                                    __codec_input_edqy,
                                );
                                match __codec_res_edqy {
                                    ::core::result::Result::Err(e) => {
                                        return ::core::result::Result::Err(
                                            e.chain("Could not decode `PopApiError::Xcm.0`"),
                                        );
                                    }
                                    ::core::result::Result::Ok(__codec_res_edqy) => {
                                        __codec_res_edqy
                                    }
                                }
                            }),
                        )
                    })();
                }
                _ => {
                    #[allow(clippy::redundant_closure_call)]
                    return (move || {
                        ::core::result::Result::Err(
                            <_ as ::core::convert::Into<
                                _,
                            >>::into(
                                "Could not decode `PopApiError`, variant doesn't exist",
                            ),
                        )
                    })();
                }
            }
        }
    }
};
impl ink::env::chain_extension::FromStatusCode for PopApiError {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        use crate::PopApiError::{
            Assets, Balances, Contracts, Nfts, TokenError, UnknownDispatchStatusCode,
            UnknownModuleStatusCode,
        };
        match status_code {
            0 => Ok(()),
            3_000_000..=3_999_999 => {
                let status_code = status_code - 3_000_000;
                match status_code {
                    5 => Err(PopApiError::SystemCallFiltered),
                    10_000..=10_999 => Err(Balances((status_code - 10_000).try_into()?)),
                    40_000..=40_999 => Err(Contracts((status_code - 40_000).try_into()?)),
                    50_000..=50_999 => Err(Nfts((status_code - 50_000).try_into()?)),
                    52_000..=52_999 => Err(Assets((status_code - 52_000).try_into()?)),
                    _ => Err(UnknownModuleStatusCode(status_code)),
                }
            }
            7_000_000..=7_999_999 => {
                Err(TokenError((status_code - 7_000_000).try_into()?))
            }
            _ => Err(UnknownDispatchStatusCode(status_code)),
        }
    }
}
impl From<scale::Error> for PopApiError {
    fn from(_: scale::Error) -> Self {
        {
            ::core::panicking::panic_fmt(
                format_args!("encountered unexpected invalid SCALE encoding"),
            );
        }
    }
}
pub enum Environment {}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::scale_info::TypeInfo for Environment {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(
                    ::scale_info::Path::new_with_replace("Environment", "pop_api", &[]),
                )
                .type_params(::alloc::vec::Vec::new())
                .variant(::scale_info::build::Variants::new())
        }
    }
};
#[automatically_derived]
impl ::core::fmt::Debug for Environment {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {}
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Environment {
    #[inline]
    fn clone(&self) -> Environment {
        match *self {}
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Environment {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Environment {
    #[inline]
    fn eq(&self, other: &Environment) -> bool {
        match *self {}
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for Environment {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
impl ink::env::Environment for Environment {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as ink::env::Environment>::MAX_EVENT_TOPICS;
    type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as ink::env::Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as ink::env::Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as ink::env::Environment>::Timestamp;
    type ChainExtension = PopApi;
}
#[scale_info(crate = ::ink::scale_info)]
pub enum PopApi {}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl ::ink::scale_info::TypeInfo for PopApi {
        type Identity = Self;
        fn type_info() -> ::ink::scale_info::Type {
            ::ink::scale_info::Type::builder()
                .path(
                    ::ink::scale_info::Path::new_with_replace("PopApi", "pop_api", &[]),
                )
                .type_params(::alloc::vec::Vec::new())
                .variant(::ink::scale_info::build::Variants::new())
        }
    }
};
const _: () = {
    #[allow(non_camel_case_types)]
    struct __ink_Private;
    #[allow(non_camel_case_types)]
    pub struct __ink_PopApiInstance {
        __ink_private: __ink_Private,
    }
    impl __ink_PopApiInstance {
        #[allow(private_interfaces)]
        #[inline]
        pub fn dispatch(
            self,
            call: RuntimeCall,
        ) -> <::ink::ValueReturned as ::ink::Output<
            {
                {
                    #[allow(unused_imports)]
                    use ::ink::result_info::IsResultTypeFallback as _;
                    ::ink::result_info::IsResultType::<Result<()>>::VALUE
                }
            },
            true,
            Result<()>,
            PopApiError,
        >>::ReturnType
        where
            <<::ink::ValueReturned as ::ink::Output<
                {
                    {
                        #[allow(unused_imports)]
                        use ::ink::result_info::IsResultTypeFallback as _;
                        ::ink::result_info::IsResultType::<Result<()>>::VALUE
                    }
                },
                true,
                Result<()>,
                PopApiError,
            >>::ReturnType as ::ink::IsResultType>::Err: ::core::convert::From<
                PopApiError,
            >,
        {
            ::ink::env::chain_extension::ChainExtensionMethod::build(59572224u32)
                .input::<RuntimeCall>()
                .output::<
                    Result<()>,
                    {
                        {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<Result<()>>::VALUE
                        }
                    },
                >()
                .handle_error_code::<PopApiError>()
                .call(&call)
        }
        #[allow(private_interfaces)]
        #[inline]
        pub fn read_state(
            self,
            key: RuntimeStateKeys,
        ) -> <::ink::ValueReturned as ::ink::Output<
            {
                {
                    #[allow(unused_imports)]
                    use ::ink::result_info::IsResultTypeFallback as _;
                    ::ink::result_info::IsResultType::<Result<Vec<u8>>>::VALUE
                }
            },
            true,
            Result<Vec<u8>>,
            PopApiError,
        >>::ReturnType
        where
            <<::ink::ValueReturned as ::ink::Output<
                {
                    {
                        #[allow(unused_imports)]
                        use ::ink::result_info::IsResultTypeFallback as _;
                        ::ink::result_info::IsResultType::<Result<Vec<u8>>>::VALUE
                    }
                },
                true,
                Result<Vec<u8>>,
                PopApiError,
            >>::ReturnType as ::ink::IsResultType>::Err: ::core::convert::From<
                PopApiError,
            >,
        {
            ::ink::env::chain_extension::ChainExtensionMethod::build(59572225u32)
                .input::<RuntimeStateKeys>()
                .output::<
                    Result<Vec<u8>>,
                    {
                        {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<Result<Vec<u8>>>::VALUE
                        }
                    },
                >()
                .handle_error_code::<PopApiError>()
                .call(&key)
        }
        #[allow(private_interfaces)]
        #[inline]
        pub fn send_xcm(
            self,
            xcm: CrossChainMessage,
        ) -> <::ink::ValueReturned as ::ink::Output<
            {
                {
                    #[allow(unused_imports)]
                    use ::ink::result_info::IsResultTypeFallback as _;
                    ::ink::result_info::IsResultType::<Result<()>>::VALUE
                }
            },
            true,
            Result<()>,
            PopApiError,
        >>::ReturnType
        where
            <<::ink::ValueReturned as ::ink::Output<
                {
                    {
                        #[allow(unused_imports)]
                        use ::ink::result_info::IsResultTypeFallback as _;
                        ::ink::result_info::IsResultType::<Result<()>>::VALUE
                    }
                },
                true,
                Result<()>,
                PopApiError,
            >>::ReturnType as ::ink::IsResultType>::Err: ::core::convert::From<
                PopApiError,
            >,
        {
            ::ink::env::chain_extension::ChainExtensionMethod::build(59572226u32)
                .input::<CrossChainMessage>()
                .output::<
                    Result<()>,
                    {
                        {
                            #[allow(unused_imports)]
                            use ::ink::result_info::IsResultTypeFallback as _;
                            ::ink::result_info::IsResultType::<Result<()>>::VALUE
                        }
                    },
                >()
                .handle_error_code::<PopApiError>()
                .call(&xcm)
        }
    }
    impl ::ink::ChainExtensionInstance for PopApi {
        type Instance = __ink_PopApiInstance;
        fn instantiate() -> Self::Instance {
            Self::Instance {
                __ink_private: __ink_Private,
            }
        }
    }
};
fn dispatch(call: RuntimeCall) -> Result<()> {
    <<Environment as ink::env::Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
        .dispatch(call)
}
fn read_state(key: RuntimeStateKeys) -> Result<Vec<u8>> {
    <<Environment as ink::env::Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
        .read_state(key)
}
fn send_xcm(xcm: CrossChainMessage) -> Result<()> {
    <<Environment as ink::env::Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
        .send_xcm(xcm)
}
