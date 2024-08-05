//! Types to combine some `fungible::*` and `fungibles::*` implementations into one union
//! `fungibles::*` implementation.

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::tokens::{
	fungible, fungibles, AssetId, DepositConsequence, Fortitude, Precision, Preservation,
	Provenance, Restriction, WithdrawConsequence,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Convert, Zero},
	DispatchError, DispatchResult,
	Either::{self, Left, Right},
	RuntimeDebug,
};
use sp_std::{cmp::Ordering, vec::Vec};

/// The `NativeOrWithId` enum classifies an asset as either `Native` to the current chain or as an
/// asset with a specific ID.
#[derive(Decode, Encode, Default, MaxEncodedLen, TypeInfo, Clone, RuntimeDebug, Eq)]
pub enum NativeOrWithId<AssetId>
where
	AssetId: Ord,
{
	/// Represents the native asset of the current chain.
	///
	/// E.g., DOT for the Polkadot Asset Hub.
	#[default]
	Native,
	/// Represents an asset identified by its underlying `AssetId`.
	WithId(AssetId),
}
impl<AssetId: Ord> From<AssetId> for NativeOrWithId<AssetId> {
	fn from(asset: AssetId) -> Self {
		Self::WithId(asset)
	}
}
impl<AssetId: Ord> Ord for NativeOrWithId<AssetId> {
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			(Self::Native, Self::Native) => Ordering::Equal,
			(Self::Native, Self::WithId(_)) => Ordering::Less,
			(Self::WithId(_), Self::Native) => Ordering::Greater,
			(Self::WithId(id1), Self::WithId(id2)) => <AssetId as Ord>::cmp(id1, id2),
		}
	}
}
impl<AssetId: Ord> PartialOrd for NativeOrWithId<AssetId> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(<Self as Ord>::cmp(self, other))
	}
}
impl<AssetId: Ord> PartialEq for NativeOrWithId<AssetId> {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other) == Ordering::Equal
	}
}

/// Criterion for [`FungibleUnionOf`] where a set for [`NativeOrWithId::Native`] asset located from the left
/// and for [`NativeOrWithId::WithId`] from the right.
pub struct NativeFromLeft;
impl<AssetId: Ord> Convert<NativeOrWithId<AssetId>, Either<(), AssetId>> for NativeFromLeft {
	fn convert(asset: NativeOrWithId<AssetId>) -> Either<(), AssetId> {
		match asset {
			NativeOrWithId::Native => Either::Left(()),
			NativeOrWithId::WithId(id) => Either::Right(id),
		}
	}
}

/// Type to combine some `fungible::*` and `fungibles::*` implementations into one union
/// `fungibles::*` implementation.
///
/// ### Parameters:
/// - `Left` is `fungible::*` implementation that is incorporated into the resulting union.
/// - `Right` is `fungibles::*` implementation that is incorporated into the resulting union.
/// - `Criterion` determines whether the `Fungible` belongs to the `Left` or `Right` set.
/// - `Fungible` is a superset type encompassing asset kinds from `Left` and `Right` sets.
/// - `AccountId` is an account identifier type.
pub struct FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>(
	sp_std::marker::PhantomData<(Left, Right, Criterion, Fungible, AccountId)>,
);

impl<
		Left: fungible::Inspect<AccountId>,
		Right: fungibles::Inspect<AccountId, Balance = Left::Balance>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::Inspect<AccountId> for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	type AssetId = Fungible;
	type Balance = Left::Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Inspect<AccountId>>::total_issuance(),
			Right(a) => <Right as fungibles::Inspect<AccountId>>::total_issuance(a),
		}
	}
	fn active_issuance(asset: Self::AssetId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Inspect<AccountId>>::active_issuance(),
			Right(a) => <Right as fungibles::Inspect<AccountId>>::active_issuance(a),
		}
	}
	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Inspect<AccountId>>::minimum_balance(),
			Right(a) => <Right as fungibles::Inspect<AccountId>>::minimum_balance(a),
		}
	}
	fn balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Inspect<AccountId>>::balance(who),
			Right(a) => <Right as fungibles::Inspect<AccountId>>::balance(a, who),
		}
	}
	fn total_balance(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Inspect<AccountId>>::total_balance(who),
			Right(a) => <Right as fungibles::Inspect<AccountId>>::total_balance(a, who),
		}
	}
	fn reducible_balance(
		asset: Self::AssetId,
		who: &AccountId,
		preservation: Preservation,
		force: Fortitude,
	) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::Inspect<AccountId>>::reducible_balance(who, preservation, force)
			},
			Right(a) => <Right as fungibles::Inspect<AccountId>>::reducible_balance(
				a,
				who,
				preservation,
				force,
			),
		}
	}
	fn can_deposit(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		provenance: Provenance,
	) -> DepositConsequence {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::Inspect<AccountId>>::can_deposit(who, amount, provenance)
			},
			Right(a) => {
				<Right as fungibles::Inspect<AccountId>>::can_deposit(a, who, amount, provenance)
			},
		}
	}
	fn can_withdraw(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Inspect<AccountId>>::can_withdraw(who, amount),
			Right(a) => <Right as fungibles::Inspect<AccountId>>::can_withdraw(a, who, amount),
		}
	}
	fn asset_exists(asset: Self::AssetId) -> bool {
		match Criterion::convert(asset) {
			Left(()) => true,
			Right(a) => <Right as fungibles::Inspect<AccountId>>::asset_exists(a),
		}
	}
}

impl<
		Left: fungible::InspectHold<AccountId>,
		Right: fungibles::InspectHold<AccountId, Balance = Left::Balance, Reason = Left::Reason>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::InspectHold<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	type Reason = Left::Reason;

	fn reducible_total_balance_on_hold(
		asset: Self::AssetId,
		who: &AccountId,
		force: Fortitude,
	) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::InspectHold<AccountId>>::reducible_total_balance_on_hold(
					who, force,
				)
			},
			Right(a) => {
				<Right as fungibles::InspectHold<AccountId>>::reducible_total_balance_on_hold(
					a, who, force,
				)
			},
		}
	}
	fn hold_available(asset: Self::AssetId, reason: &Self::Reason, who: &AccountId) -> bool {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectHold<AccountId>>::hold_available(reason, who),
			Right(a) => {
				<Right as fungibles::InspectHold<AccountId>>::hold_available(a, reason, who)
			},
		}
	}
	fn total_balance_on_hold(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectHold<AccountId>>::total_balance_on_hold(who),
			Right(a) => <Right as fungibles::InspectHold<AccountId>>::total_balance_on_hold(a, who),
		}
	}
	fn balance_on_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
	) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectHold<AccountId>>::balance_on_hold(reason, who),
			Right(a) => {
				<Right as fungibles::InspectHold<AccountId>>::balance_on_hold(a, reason, who)
			},
		}
	}
	fn can_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
	) -> bool {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectHold<AccountId>>::can_hold(reason, who, amount),
			Right(a) => {
				<Right as fungibles::InspectHold<AccountId>>::can_hold(a, reason, who, amount)
			},
		}
	}
}

impl<
		Left: fungible::InspectFreeze<AccountId>,
		Right: fungibles::InspectFreeze<AccountId, Balance = Left::Balance, Id = Left::Id>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::InspectFreeze<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	type Id = Left::Id;
	fn balance_frozen(asset: Self::AssetId, id: &Self::Id, who: &AccountId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectFreeze<AccountId>>::balance_frozen(id, who),
			Right(a) => <Right as fungibles::InspectFreeze<AccountId>>::balance_frozen(a, id, who),
		}
	}
	fn balance_freezable(asset: Self::AssetId, who: &AccountId) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectFreeze<AccountId>>::balance_freezable(who),
			Right(a) => <Right as fungibles::InspectFreeze<AccountId>>::balance_freezable(a, who),
		}
	}
	fn can_freeze(asset: Self::AssetId, id: &Self::Id, who: &AccountId) -> bool {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::InspectFreeze<AccountId>>::can_freeze(id, who),
			Right(a) => <Right as fungibles::InspectFreeze<AccountId>>::can_freeze(a, id, who),
		}
	}
}

impl<
		Left: fungible::Unbalanced<AccountId>,
		Right: fungibles::Unbalanced<AccountId, Balance = Left::Balance>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::Unbalanced<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn handle_dust(dust: fungibles::Dust<AccountId, Self>)
	where
		Self: Sized,
	{
		match Criterion::convert(dust.0) {
			Left(()) => {
				<Left as fungible::Unbalanced<AccountId>>::handle_dust(fungible::Dust(dust.1))
			},
			Right(a) => {
				<Right as fungibles::Unbalanced<AccountId>>::handle_dust(fungibles::Dust(a, dust.1))
			},
		}
	}
	fn write_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Result<Option<Self::Balance>, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Unbalanced<AccountId>>::write_balance(who, amount),
			Right(a) => <Right as fungibles::Unbalanced<AccountId>>::write_balance(a, who, amount),
		}
	}
	fn set_total_issuance(asset: Self::AssetId, amount: Self::Balance) -> () {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Unbalanced<AccountId>>::set_total_issuance(amount),
			Right(a) => <Right as fungibles::Unbalanced<AccountId>>::set_total_issuance(a, amount),
		}
	}
	fn decrease_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		preservation: Preservation,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Unbalanced<AccountId>>::decrease_balance(
				who,
				amount,
				precision,
				preservation,
				force,
			),
			Right(a) => <Right as fungibles::Unbalanced<AccountId>>::decrease_balance(
				a,
				who,
				amount,
				precision,
				preservation,
				force,
			),
		}
	}
	fn increase_balance(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::Unbalanced<AccountId>>::increase_balance(who, amount, precision)
			},
			Right(a) => <Right as fungibles::Unbalanced<AccountId>>::increase_balance(
				a, who, amount, precision,
			),
		}
	}
}

impl<
		Left: fungible::UnbalancedHold<AccountId>,
		Right: fungibles::UnbalancedHold<AccountId, Balance = Left::Balance, Reason = Left::Reason>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::UnbalancedHold<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn set_balance_on_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::UnbalancedHold<AccountId>>::set_balance_on_hold(
				reason, who, amount,
			),
			Right(a) => <Right as fungibles::UnbalancedHold<AccountId>>::set_balance_on_hold(
				a, reason, who, amount,
			),
		}
	}
	fn decrease_balance_on_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::UnbalancedHold<AccountId>>::decrease_balance_on_hold(
				reason, who, amount, precision,
			),
			Right(a) => <Right as fungibles::UnbalancedHold<AccountId>>::decrease_balance_on_hold(
				a, reason, who, amount, precision,
			),
		}
	}
	fn increase_balance_on_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::UnbalancedHold<AccountId>>::increase_balance_on_hold(
				reason, who, amount, precision,
			),
			Right(a) => <Right as fungibles::UnbalancedHold<AccountId>>::increase_balance_on_hold(
				a, reason, who, amount, precision,
			),
		}
	}
}

impl<
		Left: fungible::Mutate<AccountId>,
		Right: fungibles::Mutate<AccountId, Balance = Left::Balance>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId: Eq,
	> fungibles::Mutate<AccountId> for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn mint_into(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Mutate<AccountId>>::mint_into(who, amount),
			Right(a) => <Right as fungibles::Mutate<AccountId>>::mint_into(a, who, amount),
		}
	}
	fn burn_from(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::Mutate<AccountId>>::burn_from(who, amount, precision, force)
			},
			Right(a) => {
				<Right as fungibles::Mutate<AccountId>>::burn_from(a, who, amount, precision, force)
			},
		}
	}
	fn shelve(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Mutate<AccountId>>::shelve(who, amount),
			Right(a) => <Right as fungibles::Mutate<AccountId>>::shelve(a, who, amount),
		}
	}
	fn restore(
		asset: Self::AssetId,
		who: &AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Mutate<AccountId>>::restore(who, amount),
			Right(a) => <Right as fungibles::Mutate<AccountId>>::restore(a, who, amount),
		}
	}
	fn transfer(
		asset: Self::AssetId,
		source: &AccountId,
		dest: &AccountId,
		amount: Self::Balance,
		preservation: Preservation,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::Mutate<AccountId>>::transfer(source, dest, amount, preservation)
			},
			Right(a) => <Right as fungibles::Mutate<AccountId>>::transfer(
				a,
				source,
				dest,
				amount,
				preservation,
			),
		}
	}

	fn set_balance(asset: Self::AssetId, who: &AccountId, amount: Self::Balance) -> Self::Balance {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::Mutate<AccountId>>::set_balance(who, amount),
			Right(a) => <Right as fungibles::Mutate<AccountId>>::set_balance(a, who, amount),
		}
	}
}

impl<
		Left: fungible::MutateHold<AccountId>,
		Right: fungibles::MutateHold<AccountId, Balance = Left::Balance, Reason = Left::Reason>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::MutateHold<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateHold<AccountId>>::hold(reason, who, amount),
			Right(a) => <Right as fungibles::MutateHold<AccountId>>::hold(a, reason, who, amount),
		}
	}
	fn release(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => {
				<Left as fungible::MutateHold<AccountId>>::release(reason, who, amount, precision)
			},
			Right(a) => <Right as fungibles::MutateHold<AccountId>>::release(
				a, reason, who, amount, precision,
			),
		}
	}
	fn burn_held(
		asset: Self::AssetId,
		reason: &Self::Reason,
		who: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateHold<AccountId>>::burn_held(
				reason, who, amount, precision, force,
			),
			Right(a) => <Right as fungibles::MutateHold<AccountId>>::burn_held(
				a, reason, who, amount, precision, force,
			),
		}
	}
	fn transfer_on_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		source: &AccountId,
		dest: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		mode: Restriction,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateHold<AccountId>>::transfer_on_hold(
				reason, source, dest, amount, precision, mode, force,
			),
			Right(a) => <Right as fungibles::MutateHold<AccountId>>::transfer_on_hold(
				a, reason, source, dest, amount, precision, mode, force,
			),
		}
	}
	fn transfer_and_hold(
		asset: Self::AssetId,
		reason: &Self::Reason,
		source: &AccountId,
		dest: &AccountId,
		amount: Self::Balance,
		precision: Precision,
		preservation: Preservation,
		force: Fortitude,
	) -> Result<Self::Balance, DispatchError> {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateHold<AccountId>>::transfer_and_hold(
				reason,
				source,
				dest,
				amount,
				precision,
				preservation,
				force,
			),
			Right(a) => <Right as fungibles::MutateHold<AccountId>>::transfer_and_hold(
				a,
				reason,
				source,
				dest,
				amount,
				precision,
				preservation,
				force,
			),
		}
	}
}

impl<
		Left: fungible::MutateFreeze<AccountId>,
		Right: fungibles::MutateFreeze<AccountId, Balance = Left::Balance, Id = Left::Id>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::MutateFreeze<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn set_freeze(
		asset: Self::AssetId,
		id: &Self::Id,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateFreeze<AccountId>>::set_freeze(id, who, amount),
			Right(a) => {
				<Right as fungibles::MutateFreeze<AccountId>>::set_freeze(a, id, who, amount)
			},
		}
	}
	fn extend_freeze(
		asset: Self::AssetId,
		id: &Self::Id,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateFreeze<AccountId>>::extend_freeze(id, who, amount),
			Right(a) => {
				<Right as fungibles::MutateFreeze<AccountId>>::extend_freeze(a, id, who, amount)
			},
		}
	}
	fn thaw(asset: Self::AssetId, id: &Self::Id, who: &AccountId) -> DispatchResult {
		match Criterion::convert(asset) {
			Left(()) => <Left as fungible::MutateFreeze<AccountId>>::thaw(id, who),
			Right(a) => <Right as fungibles::MutateFreeze<AccountId>>::thaw(a, id, who),
		}
	}
}

pub struct ConvertImbalanceDropHandler<
	Left,
	Right,
	Criterion,
	Fungible,
	Balance,
	AssetId,
	AccountId,
>(sp_std::marker::PhantomData<(Left, Right, Criterion, Fungible, Balance, AssetId, AccountId)>);

impl<
		Left: fungible::HandleImbalanceDrop<Balance>,
		Right: fungibles::HandleImbalanceDrop<AssetId, Balance>,
		Criterion: Convert<Fungible, Either<(), AssetId>>,
		Fungible,
		Balance,
		AssetId,
		AccountId,
	> fungibles::HandleImbalanceDrop<Fungible, Balance>
	for ConvertImbalanceDropHandler<Left, Right, Criterion, Fungible, Balance, AssetId, AccountId>
{
	fn handle(asset: Fungible, amount: Balance) {
		match Criterion::convert(asset) {
			Left(()) => Left::handle(amount),
			Right(a) => Right::handle(a, amount),
		}
	}
}

impl<
		Left: fungible::Inspect<AccountId>,
		Right: fungibles::Inspect<AccountId, Balance = Left::Balance> + fungibles::Create<AccountId>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::Create<AccountId> for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn create(
		asset: Fungible,
		admin: AccountId,
		is_sufficient: bool,
		min_balance: Self::Balance,
	) -> DispatchResult {
		match Criterion::convert(asset) {
			// no-op for `Left` since `Create` trait is not defined within `fungible::*`.
			Left(()) => Ok(()),
			Right(a) => <Right as fungibles::Create<AccountId>>::create(
				a,
				admin,
				is_sufficient,
				min_balance,
			),
		}
	}
}

impl<
		Left: fungible::Inspect<AccountId>,
		Right: fungibles::approvals::Inspect<AccountId, Balance = Left::Balance>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::approvals::Inspect<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn allowance(asset: Self::AssetId, owner: &AccountId, delegate: &AccountId) -> Self::Balance {
		match Criterion::convert(asset) {
			Either::Left(()) => Zero::zero(),
			Either::Right(id) => <Right as fungibles::approvals::Inspect<AccountId>>::allowance(
				id, &owner, &delegate,
			),
		}
	}
}

impl<
		Left: fungible::Inspect<AccountId>,
		Right: fungibles::metadata::Inspect<AccountId, Balance = Left::Balance>,
		Criterion: Convert<Fungible, Either<(), Right::AssetId>>,
		Fungible: AssetId,
		AccountId,
	> fungibles::metadata::Inspect<AccountId>
	for FungibleUnionOf<Left, Right, Criterion, Fungible, AccountId>
{
	fn name(asset: Self::AssetId) -> Vec<u8> {
		match Criterion::convert(asset) {
			Either::Left(()) => todo!("Retrieve the native token metadata from chain spec"),
			Either::Right(id) => <Right as fungibles::metadata::Inspect<AccountId>>::name(id),
		}
	}

	fn symbol(asset: Self::AssetId) -> Vec<u8> {
		match Criterion::convert(asset) {
			Either::Left(()) => todo!("Retrieve the native token metadata from chain spec"),
			Either::Right(id) => <Right as fungibles::metadata::Inspect<AccountId>>::symbol(id),
		}
	}

	fn decimals(asset: Self::AssetId) -> u8 {
		match Criterion::convert(asset) {
			Either::Left(()) => todo!("Retrieve the native token metadata from chain spec"),
			Either::Right(id) => <Right as fungibles::metadata::Inspect<AccountId>>::decimals(id),
		}
	}
}
