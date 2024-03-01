use frame_support::{
	ensure,
	traits::{tokens::nonfungibles_v2, Get, Incrementable},
};
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, prelude::*};
use xcm::latest::prelude::*;
use xcm::opaque::v3::MultiLocation;
use xcm_builder::{AssetChecking, MintLocation};
use xcm_executor::{traits::{
	ConvertLocation, Error as MatchError, MatchesNonFungibles, TransactAsset,
}, AssetsInHolding};

#[derive(
	Copy, Clone, Decode, Encode, Eq, PartialEq, Ord, PartialOrd, Debug, TypeInfo, MaxEncodedLen,
)]
/// Represents a collection ID based on a MultiLocation.
///
/// This structure provides a way to map a MultiLocation to a collection ID,
/// which is useful for describing collections that do not follow an incremental pattern.
pub struct MultiLocationCollectionId(pub MultiLocation);

impl MultiLocationCollectionId {
	/// Consume `self` and return the inner MultiLocation.
	pub (crate) fn into_inner(self) -> MultiLocation {
		self.0
	}

	/// Return a reference to the inner MultiLocation.
	pub (crate) fn inner(&self) -> &MultiLocation {
		&self.0
	}
}

impl Incrementable for MultiLocationCollectionId {
    fn increment(&self) -> Option<Self>{
        None
    }

    fn initial_value() -> Option<Self> {
       None 
    }

}

impl From<MultiLocation> for MultiLocationCollectionId {
	fn from(value: MultiLocation) -> Self {
		MultiLocationCollectionId(value)
	}
}

impl From<MultiLocationCollectionId> for MultiLocation {
	fn from(value: MultiLocationCollectionId) -> MultiLocation {
		value.into_inner()
	}
}

const LOG_TARGET: &str = "xcm::nonfungibles_adapter_pop";
/// Adapter for transferring non-fungible tokens (NFTs) using [`nonfungibles_v2`].
///
/// This adapter facilitates the transfer of NFTs between different locations.
pub struct NonFungiblesTransferAdapterPop<Assets, Matcher, AccountIdConverter, AccountId>(
	PhantomData<(Assets, Matcher, AccountIdConverter, AccountId)>,
);
impl<
		Assets: nonfungibles_v2::Transfer<AccountId>,
		Matcher: MatchesNonFungibles<Assets::CollectionId, Assets::ItemId>,
		AccountIdConverter: ConvertLocation<AccountId>,
		AccountId: Clone, // can't get away without it since Currency is generic over it.
	> TransactAsset for NonFungiblesTransferAdapterPop<Assets, Matcher, AccountIdConverter, AccountId>
{
	fn transfer_asset(
		what: &Asset,
		from: &Location,
		to: &Location,
		context: &XcmContext,
	) -> Result<AssetsInHolding, XcmError> {
		log::trace!(
			target: LOG_TARGET,
			"transfer_asset what: {:?}, from: {:?}, to: {:?}, context: {:?}",
			what,
			from,
			to,
			context,
		);
		// Check we handle this asset.
		let (class, instance) = Matcher::matches_nonfungibles(what)?;
		let destination = AccountIdConverter::convert_location(to)
			.ok_or(MatchError::AccountIdConversionFailed)?;
		Assets::transfer(&class, &instance, &destination)
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
		Ok(what.clone().into())
	}
}

/// Adapter for mutating non-fungible tokens (NFTs) using [`nonfungibles_v2`].
///
/// This adapter provides functions to withdraw, deposit, check in and check out non fungibles.
pub struct NonFungiblesMutateAdapterPop<
	Assets,
	Matcher,
	AccountIdConverter,
	AccountId,
	CheckAsset,
	CheckingAccount,
	ItemConfig,
>(
	PhantomData<(
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		ItemConfig,
	)>,
)
where
	ItemConfig: Default;

impl<
		Assets: nonfungibles_v2::Mutate<AccountId, ItemConfig>,
		Matcher: MatchesNonFungibles<Assets::CollectionId, Assets::ItemId>,
		AccountIdConverter: ConvertLocation<AccountId>,
		AccountId: Clone + Eq, // can't get away without it since Currency is generic over it.
		CheckAsset: AssetChecking<Assets::CollectionId>,
		CheckingAccount: Get<Option<AccountId>>,
		ItemConfig: Default,
	>
	NonFungiblesMutateAdapterPop<
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		ItemConfig,
	>
{
	fn can_accrue_checked(class: Assets::CollectionId, instance: Assets::ItemId) -> XcmResult {
		ensure!(Assets::owner(&class, &instance).is_none(), XcmError::NotDepositable);
		Ok(())
	}
	fn can_reduce_checked(class: Assets::CollectionId, instance: Assets::ItemId) -> XcmResult {
		if let Some(checking_account) = CheckingAccount::get() {
			// This is an asset whose teleports we track.
			let owner = Assets::owner(&class, &instance);
			ensure!(owner == Some(checking_account), XcmError::NotWithdrawable);
			ensure!(Assets::can_transfer(&class, &instance), XcmError::NotWithdrawable);
		}
		Ok(())
	}
	fn accrue_checked(class: Assets::CollectionId, instance: Assets::ItemId) {
		if let Some(checking_account) = CheckingAccount::get() {
			let ok = Assets::mint_into(
				&class,
				&instance,
				&checking_account,
				&ItemConfig::default(),
				true,
			)
			.is_ok();
			debug_assert!(ok, "`mint_into` cannot generally fail; qed");
		}
	}
	fn reduce_checked(class: Assets::CollectionId, instance: Assets::ItemId) {
		let ok = Assets::burn(&class, &instance, None).is_ok();
		debug_assert!(ok, "`can_check_in` must have returned `true` immediately prior; qed");
	}
}

impl<
		Assets: nonfungibles_v2::Mutate<AccountId, ItemConfig>,
		Matcher: MatchesNonFungibles<Assets::CollectionId, Assets::ItemId>,
		AccountIdConverter: ConvertLocation<AccountId>,
		AccountId: Clone + Eq, // can't get away without it since Currency is generic over it.
		CheckAsset: AssetChecking<Assets::CollectionId>,
		CheckingAccount: Get<Option<AccountId>>,
		ItemConfig: Default,
	> TransactAsset
	for NonFungiblesMutateAdapterPop<
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		ItemConfig,
	>
{
	fn can_check_in(_origin: &Location, what: &Asset, context: &XcmContext) -> XcmResult {
		log::trace!(
			target: LOG_TARGET,
			"can_check_in origin: {:?}, what: {:?}, context: {:?}",
			_origin,
			what,
			context,
		);
		// Check we handle this asset.
		let (class, instance) = Matcher::matches_nonfungibles(what)?;
		match CheckAsset::asset_checking(&class) {
			// We track this asset's teleports to ensure no more come in than have gone out.
			Some(MintLocation::Local) => Self::can_reduce_checked(class, instance),
			// We track this asset's teleports to ensure no more go out than have come in.
			Some(MintLocation::NonLocal) => Self::can_accrue_checked(class, instance),
			_ => Ok(()),
		}
	}

	fn check_in(_origin: &Location, what: &Asset, context: &XcmContext) {
		log::trace!(
			target: LOG_TARGET,
			"check_in origin: {:?}, what: {:?}, context: {:?}",
			_origin,
			what,
			context,
		);
		if let Ok((class, instance)) = Matcher::matches_nonfungibles(what) {
			match CheckAsset::asset_checking(&class) {
				// We track this asset's teleports to ensure no more come in than have gone out.
				Some(MintLocation::Local) => Self::reduce_checked(class, instance),
				// We track this asset's teleports to ensure no more go out than have come in.
				Some(MintLocation::NonLocal) => Self::accrue_checked(class, instance),
				_ => (),
			}
		}
	}

	fn can_check_out(_dest: &Location, what: &Asset, context: &XcmContext) -> XcmResult {
		log::trace!(
			target: LOG_TARGET,
			"can_check_out dest: {:?}, what: {:?}, context: {:?}",
			_dest,
			what,
			context,
		);
		// Check we handle this asset.
		let (class, instance) = Matcher::matches_nonfungibles(what)?;
		match CheckAsset::asset_checking(&class) {
			// We track this asset's teleports to ensure no more come in than have gone out.
			Some(MintLocation::Local) => Self::can_accrue_checked(class, instance),
			// We track this asset's teleports to ensure no more go out than have come in.
			Some(MintLocation::NonLocal) => Self::can_reduce_checked(class, instance),
			_ => Ok(()),
		}
	}

	fn check_out(_dest: &Location, what: &Asset, context: &XcmContext) {
		log::trace!(
			target: LOG_TARGET,
			"check_out dest: {:?}, what: {:?}, context: {:?}",
			_dest,
			what,
			context,
		);
		if let Ok((class, instance)) = Matcher::matches_nonfungibles(what) {
			match CheckAsset::asset_checking(&class) {
				// We track this asset's teleports to ensure no more come in than have gone out.
				Some(MintLocation::Local) => Self::accrue_checked(class, instance),
				// We track this asset's teleports to ensure no more go out than have come in.
				Some(MintLocation::NonLocal) => Self::reduce_checked(class, instance),
				_ => (),
			}
		}
	}

	fn deposit_asset(what: &Asset, who: &Location, context: Option<&XcmContext>) -> XcmResult {
		log::trace!(
			target: LOG_TARGET,
			"deposit_asset what: {:?}, who: {:?}, context: {:?}",
			what,
			who,
			context,
		);
		// Check we handle this asset.
		let (class, instance) = Matcher::matches_nonfungibles(what)?;
		let who = AccountIdConverter::convert_location(who)
			.ok_or(MatchError::AccountIdConversionFailed)?;

		Assets::mint_into(&class, &instance, &who, &ItemConfig::default(), true)
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))
	}

	fn withdraw_asset(
		what: &Asset,
		who: &Location,
		maybe_context: Option<&XcmContext>,
	) -> Result<AssetsInHolding, XcmError> {
		log::trace!(
			target: LOG_TARGET,
			"withdraw_asset what: {:?}, who: {:?}, maybe_context: {:?}",
			what,
			who,
			maybe_context,
		);
		// Check we handle this asset.
		let who = AccountIdConverter::convert_location(who)
			.ok_or(MatchError::AccountIdConversionFailed)?;
		let (class, instance) = Matcher::matches_nonfungibles(what)?;
		Assets::burn(&class, &instance, Some(&who))
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
		Ok(what.clone().into())
	}
}

/// Adapter for handling non-fungible tokens (NFTs) using [`nonfungibles_v2`].
///
/// This adapter combines the functionalities of both the [`NonFungiblesTransferAdapterPop`] and [`NonFungiblesMutateAdapterPop`] adapters,
/// allowing handling NFTs in various scenarios.
/// For detailed information on the functions, refer to [`TransactAsset`].
pub struct NonFungiblesAdapterPop<
	Assets,
	Matcher,
	AccountIdConverter,
	AccountId,
	CheckAsset,
	CheckingAccount,
	ItemConfig,
>(
	PhantomData<(
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		ItemConfig,
	)>,
)
where
	ItemConfig: Default;
impl<
		Assets: nonfungibles_v2::Mutate<AccountId, ItemConfig> + nonfungibles_v2::Transfer<AccountId>,
		Matcher: MatchesNonFungibles<Assets::CollectionId, Assets::ItemId>,
		AccountIdConverter: ConvertLocation<AccountId>,
		AccountId: Clone + Eq, // can't get away without it since Currency is generic over it.
		CheckAsset: AssetChecking<Assets::CollectionId>,
		CheckingAccount: Get<Option<AccountId>>,
		ItemConfig: Default,
	> TransactAsset
	for NonFungiblesAdapterPop<
		Assets,
		Matcher,
		AccountIdConverter,
		AccountId,
		CheckAsset,
		CheckingAccount,
		ItemConfig,
	>
{
	fn can_check_in(origin: &Location, what: &Asset, context: &XcmContext) -> XcmResult {
		NonFungiblesMutateAdapterPop::<
			Assets,
			Matcher,
			AccountIdConverter,
			AccountId,
			CheckAsset,
			CheckingAccount,
			ItemConfig,
		>::can_check_in(origin, what, context)
	}

	fn check_in(origin: &Location, what: &Asset, context: &XcmContext) {
		NonFungiblesMutateAdapterPop::<
			Assets,
			Matcher,
			AccountIdConverter,
			AccountId,
			CheckAsset,
			CheckingAccount,
			ItemConfig,
		>::check_in(origin, what, context)
	}

	fn can_check_out(dest: &Location, what: &Asset, context: &XcmContext) -> XcmResult {
		NonFungiblesMutateAdapterPop::<
			Assets,
			Matcher,
			AccountIdConverter,
			AccountId,
			CheckAsset,
			CheckingAccount,
			ItemConfig,
		>::can_check_out(dest, what, context)
	}

	fn check_out(dest: &Location, what: &Asset, context: &XcmContext) {
		NonFungiblesMutateAdapterPop::<
			Assets,
			Matcher,
			AccountIdConverter,
			AccountId,
			CheckAsset,
			CheckingAccount,
			ItemConfig,
		>::check_out(dest, what, context)
	}

	fn deposit_asset(what: &Asset, who: &Location, context: Option<&XcmContext>) -> XcmResult {
		NonFungiblesMutateAdapterPop::<
			Assets,
			Matcher,
			AccountIdConverter,
			AccountId,
			CheckAsset,
			CheckingAccount,
			ItemConfig,
		>::deposit_asset(what, who, context)
	}

	fn withdraw_asset(
		what: &Asset,
		who: &Location,
		maybe_context: Option<&XcmContext>,
	) -> Result<AssetsInHolding, XcmError> {
		NonFungiblesMutateAdapterPop::<
			Assets,
			Matcher,
			AccountIdConverter,
			AccountId,
			CheckAsset,
			CheckingAccount,
			ItemConfig,
		>::withdraw_asset(what, who, maybe_context)
	}

	fn transfer_asset(
		what: &Asset,
		from: &Location,
		to: &Location,
		context: &XcmContext,
	) -> Result<AssetsInHolding, XcmError> {
		NonFungiblesTransferAdapterPop::<Assets, Matcher, AccountIdConverter, AccountId>::transfer_asset(
			what, from, to, context,
		)
	}
}