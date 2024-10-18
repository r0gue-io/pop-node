use codec::HasCompact;
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	pallet_prelude::{Decode, Encode, TypeInfo},
	traits::{IsSubType, IsType},
};
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::StaticTypeInfo;
use sp_runtime::{
	traits::{
		DispatchInfoOf, Dispatchable, PostDispatchInfoOf, Saturating, SignedExtension, StaticLookup,
	},
	transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
	FixedPointOperand,
};

use crate::{types::*, Call, Config, Pallet};

/// A [`SignedExtension`] that handles fees sponsorship.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ContractFeeHandler<T: Config, S>(
	#[codec(compact)] OnChargeTransactionBalanceOf<T>,
	core::marker::PhantomData<S>,
);

// Make this extension "invisible" from the outside (ie metadata type information)
impl<T: Config, S: StaticTypeInfo> TypeInfo for ContractFeeHandler<T, S> {
	type Identity = S;

	fn type_info() -> scale_info::Type {
		S::type_info()
	}
}

impl<T: Config, S: Encode> core::fmt::Debug for ContractFeeHandler<T, S> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "ContractFeeHandler<{:?}>", self.0)
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}

impl<T: Config, S: SignedExtension<AccountId = T::AccountId>> ContractFeeHandler<T, S>
where
	<T as frame_system::Config>::RuntimeCall:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
	OnChargeTransactionBalanceOf<T>: Send + Sync + From<u64>,
{
	/// Utility constructor. Used only in client/factory code.
	pub fn from(fee: OnChargeTransactionBalanceOf<T>) -> Self {
		Self(fee, core::marker::PhantomData)
	}

	/// Returns the tip as being chosen by the transaction sender.
	pub fn tip(&self) -> OnChargeTransactionBalanceOf<T> {
		self.0
	}

	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
	) -> Result<(OnChargeTransactionBalanceOf<T>, LiquidityInfoOf<T>), TransactionValidityError> {
		let tip = self.0;
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, tip);
		<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
			who, call, info, fee, tip,
		)
		.map(|i| (fee, i))
	}
}

impl<T: Config + Send + Sync, S: SignedExtension<AccountId = T::AccountId>> SignedExtension
	for ContractFeeHandler<T, S>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
		+ IsSubType<Call<T>>
		+ IsSubType<pallet_contracts::Call<T>>,
	OnChargeTransactionBalanceOf<T>:
		Send + Sync + FixedPointOperand + From<u64> + IsType<BalanceOf<T>>,
	<ContractsBalanceOf<T> as HasCompact>::Type:
		Clone + Eq + PartialEq + core::fmt::Debug + TypeInfo + Encode,
{
	type AccountId = T::AccountId;
	type AdditionalSigned = ();
	// type Call = S::Call;
	type Call = <T as frame_system::Config>::RuntimeCall;
	type Pre = (
		// tip
		OnChargeTransactionBalanceOf<T>,
		// who
		Self::AccountId,
		// imbalance resulting from withdrawing the fee
		LiquidityInfoOf<T>,
		// contract address.
		Option<Self::AccountId>,
	);

	// From the outside this extension should be "invisible", because it just extends the wrapped
	// extension with an extra check in `pre_dispatch` and `post_dispatch`. Thus, we should forward
	// the identifier of the wrapped extension to let wallets see this extension as it would only be
	// the wrapped extension itself.
	const IDENTIFIER: &'static str = S::IDENTIFIER;

	fn additional_signed(&self) -> Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		let (fee, ..) = self.withdraw_fee(who, call, info, len)?;
		let priority = pallet_transaction_payment::ChargeTransactionPayment::<T>::get_priority(
			info, len, self.0, fee,
		);
		Ok(ValidTransaction { priority, ..Default::default() })
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let contract = if let Some(pallet_contracts::Call::call { dest, .. }) = call.is_sub_type() {
			T::Lookup::lookup(dest.clone()).ok()
		} else {
			None
		};
		let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;
		Ok((self.tip(), who.clone(), initial_payment, contract))
	}

	fn post_dispatch(
		pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some((tip, who, initial_payment, contract)) = pre {
			pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch(
				Some((tip, who, initial_payment)),
				info,
				post_info,
				len,
				result,
			)?;
			if let Some(contract) = contract {
				// Check if contract is registered to track usage.
				if crate::RegisteredContracts::<T>::contains_key(&contract) {
					let actual_fee = pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
						len as u32, info, post_info, tip,
					);
					let era = crate::CurrentEra::<T>::get();
					crate::ContractUsagePerEra::<T>::mutate(contract.clone(), era, |fees| {
						*fees = fees.saturating_add(actual_fee.into())
					});
					crate::EraInformation::<T>::mutate(era, |era_info| {
						era_info.add_contract_fee(actual_fee.into());
					});
					Pallet::<T>::deposit_event(crate::Event::<T>::ContractCalled {
						contract: contract.clone(),
					});
				}
			}
		}
		Ok(())
	}
}
