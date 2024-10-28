use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	traits::IsSubType,
};
use pallet_revive::AddressMapper;
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::{StaticTypeInfo, TypeInfo};
use sp_core::U256;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
	transaction_validity::{TransactionValidity, TransactionValidityError},
};
use types::*;

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
		+ IsSubType<pallet_revive::Call<T>>,
	<<T as pallet_revive::Config>::Currency as frame_support::traits::fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance: Into<U256>,
	<<T as pallet_revive::Config>::Currency as frame_support::traits::fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance: TryFrom<U256>,
	<<T as pallet_revive::Config>::Time as frame_support::traits::Time>::Moment: Into<U256>,
{
	fn is_contracts_call(
		call: &<T as frame_system::Config>::RuntimeCall,
	) -> Option<AccountIdOf<T>> {
		match call.is_sub_type() {
			Some(pallet_revive::Call::<T>::call { dest, .. }) => {
				let account_id = <T as pallet_revive::Config>::AddressMapper::to_account_id(dest);
				Some(account_id)
			},
			_ => None,
		}
	}

	fn is_sponsored(who: &AccountIdOf<T>, contract: &AccountIdOf<T>) -> Option<BalanceOf<T>> {
		Pallet::<T>::is_sponsored_by(who, contract)
	}
}

impl<
		T: Config + Send + Sync,
		S: SignedExtension<
			AccountId = AccountIdOf<T>,
			Call = <T as frame_system::Config>::RuntimeCall,
		>,
	> SignedExtension for Sponsored<T, S>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
		+ IsSubType<Call<T>>
		+ IsSubType<pallet_revive::Call<T>>,
	<<T as pallet_revive::Config>::Currency as frame_support::traits::fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance: Into<U256>,
	<<T as pallet_revive::Config>::Currency as frame_support::traits::fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance: TryFrom<U256>,
	<<T as pallet_revive::Config>::Time as frame_support::traits::Time>::Moment: Into<U256>,
	BalanceOf<T>: Send + Sync + From<u64>,
{
	type AccountId = AccountIdOf<T>;
	type AdditionalSigned = S::AdditionalSigned;
	type Call = <T as frame_system::Config>::RuntimeCall;
	type Pre = (
		(Option<Self::AccountId>, Self::AccountId),
		(BalanceOf<T>, Self::AccountId, LiquidityInfoOf<T>),
	);

	// From the outside this extension should be "invisible", because it just extends the wrapped
	// extension with an extra check in `pre_dispatch` and `post_dispatch`. Thus, we should forward
	// the identifier of the wrapped extension to let wallets see this extension as it would only be
	// the wrapped extension itself.
	const IDENTIFIER: &'static str = S::IDENTIFIER;

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		let who = Self::is_contracts_call(call)
			.and_then(|contract| {
				Self::is_sponsored(&who, &contract).and_then(|_amount| {
					let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(
						len as u32,
						info,
						Default::default(), // TODO: include tip
					);
					if Pallet::<T>::can_decrease(&who, &contract, fee) {
						Some(contract.clone())
					} else {
						// Fees cannot be withdrawn from sponsorship
						None
					}
				})
			})
			.unwrap_or_else(|| who.clone());

		self.0.validate(&who, call, info, len)
	}

	fn additional_signed(&self) -> Result<Self::AdditionalSigned, TransactionValidityError> {
		self.0.additional_signed()
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let caller = who.clone();
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(
			len as u32,
			info,
			Default::default(), // TODO: include tip
		);

		let sponsor = Self::is_contracts_call(call).and_then(|contract| {
			if Self::is_sponsored(&who, &contract).is_some() {
				let _ = Pallet::<T>::withdraw_from_sponsorship(&who, &contract, fee);
				Some(contract.clone())
			} else {
				None
			}
		});
		let who = sponsor.clone().unwrap_or(who.clone());
		<T as pallet_transaction_payment::Config>::OnChargeTransaction::withdraw_fee(
			&who,
			call,
			info,
			fee,
			Default::default(), // TODO: include tip
		)
		.map(|liquidity_info| ((sponsor, caller), (Default::default(), who, liquidity_info)))
	}

	fn post_dispatch(
		pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some(((maybe_sponsor, caller), (tip, who, imbalance))) = pre {
			if let Some(sponsor) = maybe_sponsor {
				let actual_fee = pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
					len as u32, info, post_info, tip,
				);
				let _ = Pallet::<T>::withdraw_from_sponsorship(&caller, &sponsor, actual_fee);
				Pallet::<T>::deposit_event(Event::<T>::CallSponsored { by: sponsor });
			}
			pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch(
				Some((tip, who, imbalance)),
				info,
				post_info,
				len,
				result,
			)?;
		}
		Ok(())
	}
}
