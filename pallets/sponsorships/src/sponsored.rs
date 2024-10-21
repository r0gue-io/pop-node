use codec::{Decode, Encode, HasCompact};
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	traits::IsSubType,
	weights::Weight,
};
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension, StaticLookup},
	transaction_validity::{TransactionValidity, TransactionValidityError},
};

use super::*;

/// A [`SignedExtension`] that handles fees sponsorship.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct Sponsored<T, S>(pub S, core::marker::PhantomData<T>);

// Make this extension "invisible" from the outside (ie metadata type information)
impl<T, S: StaticTypeInfo> TypeInfo for Sponsored<T, S> {
	type Identity = S;

	fn type_info() -> scale_info::Type {
		S::type_info()
	}
}

impl<T, S: Encode> core::fmt::Debug for Sponsored<T, S> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "Sponsored<{:?}>", self.0.encode())
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}
#[cfg(test)]
impl<T, S> From<S> for Sponsored<T, S> {
	fn from(s: S) -> Self {
		Self(s, core::marker::PhantomData)
	}
}

impl<T: Config, S> Sponsored<T, S>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
		+ IsSubType<Call<T>>
		+ IsSubType<pallet_contracts::Call<T>>,
	<<<T as pallet_contracts::Config>::Currency as frame_support::traits::fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance as HasCompact>::Type: Clone + Eq + PartialEq + core::fmt::Debug + TypeInfo + Encode,
{
	fn is_contracts_call(call: &<T as frame_system::Config>::RuntimeCall) -> Option<T::AccountId> {
		match call.is_sub_type() {
			Some(pallet_contracts::Call::<T>::call { dest, .. }) =>
				T::Lookup::lookup(dest.clone()).ok(),
			_ => None,
		}
	}

	fn is_sponsored(who: &T::AccountId, contract: &T::AccountId) -> Option<Weight> {
		Pallet::<T>::is_sponsored_by(who, contract)
	}
}

impl<
		T: Config + Send + Sync,
		S: SignedExtension<AccountId = T::AccountId, Call = <T as frame_system::Config>::RuntimeCall>,
	> SignedExtension for Sponsored<T, S>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
		+ IsSubType<Call<T>>
		+ IsSubType<pallet_contracts::Call<T>>,
	// S::Call: IsSubType<pallet_contracts::Call<T>>,
	<T as frame_system::Config>::RuntimeCall: IsSubType<pallet_contracts::Call<T>>,
	<<<T as pallet_contracts::Config>::Currency as frame_support::traits::fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance as HasCompact>::Type: Clone + Eq + PartialEq + core::fmt::Debug + TypeInfo + Encode,
{
	type AccountId = T::AccountId;
	type AdditionalSigned = S::AdditionalSigned;
	// type Call = S::Call;
	type Call = <T as frame_system::Config>::RuntimeCall;
	type Pre = (Option<Self::AccountId>, <S as SignedExtension>::Pre);

	// From the outside this extension should be "invisible", because it just extends the wrapped
	// extension with an extra check in `pre_dispatch` and `post_dispatch`. Thus, we should forward
	// the identifier of the wrapped extension to let wallets see this extension as it would only be
	// the wrapped extension itself.
	const IDENTIFIER: &'static str = S::IDENTIFIER;

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
		self.0.additional_signed()
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {

		// Assure that whoever has to pay for fees, has enough balance.

		let who = Self::is_contracts_call(call)
			.and_then(|contract| {
				if Self::is_sponsored(&who, &contract).is_some() {
					Some(contract.clone())
				} else {
					None
				}
			})
			.unwrap_or_else(|| who.clone());

		self.0.validate(&who, call, info, len)
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {

		let sponsor = Self::is_contracts_call(call)
			.and_then(|contract| {
				if Self::is_sponsored(&who, &contract).is_some() {
					Some(contract.clone())
				} else {
					None
				}
			});
		let who = sponsor.clone().unwrap_or(who.clone());

		Ok((sponsor, self.0.pre_dispatch(&who, call, info, len)?))
	}

	fn post_dispatch(
		pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some(pre) = pre {
			if let Some(sponsor) = pre.0 {
				Pallet::<T>::deposit_event(Event::<T>::CallSponsored { by: sponsor });
			}
			S::post_dispatch(Some(pre.1), info, post_info, len, result)?;
		}
		Ok(())
	}
}
