use codec::HasCompact;
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	pallet_prelude::{Decode, Encode, TypeInfo},
	traits::{fungible::Inspect, IsSubType, IsType},
};
use pallet_revive::AddressMapper;
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::StaticTypeInfo;
use sp_core::{crypto::AccountId32, U256};
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, Saturating, SignedExtension},
	transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
	FixedPointOperand, Permill,
};

use crate::{types::*, Call, Config, Pallet};

/// A [`SignedExtension`] that handles fees sponsorship.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ContractFeeHandler<T: Config>(core::marker::PhantomData<T>);

impl<T: Config> core::fmt::Debug for ContractFeeHandler<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "ContractFeeHandler<{:?}>", self.0)
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> SignedExtension
    for ContractFeeHandler<T>
where
    <T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>
        + IsSubType<Call<T>>
        + IsSubType<pallet_revive::Call<T>>,
    OnChargeTransactionBalanceOf<T>:
        Send + Sync + FixedPointOperand + From<u64> + IsType<BalanceOf<T>>,
    <<<T as pallet_revive::Config>::Currency as Inspect<
    <T as frame_system::Config>::AccountId,
>>::Balance as HasCompact>::Type:
        Clone + Eq + PartialEq + core::fmt::Debug + TypeInfo + Encode,
    <T as frame_system::Config>::RuntimeCall: IsSubType<pallet_revive::Call<T>>,
    <T as frame_system::Config>::AccountId: From<AccountId32>,
    <<T as pallet_revive::Config>::Currency as frame_support::traits::fungible::Inspect<
        <T as frame_system::Config>::AccountId,
    >>::Balance: Into<U256>,
    <<T as pallet_revive::Config>::Time as frame_support::traits::Time>::Moment: Into<U256>,
    <<T as pallet_revive::Config>::Currency as frame_support::traits::fungible::Inspect<
        <T as frame_system::Config>::AccountId,
    >>::Balance: TryFrom<U256>,
{
    type AccountId = T::AccountId;
    type AdditionalSigned = ();
    type Call = <T as frame_system::Config>::RuntimeCall;
    type Pre = (
        // contract address.
        Option<Self::AccountId>,
    );

    const IDENTIFIER: &'static str = "Incentives";

    fn additional_signed(&self) -> Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let contract = if let Some(pallet_revive::Call::call { dest, .. }) = call.is_sub_type() {
            Some(<T as pallet_revive::Config>::AddressMapper::to_account_id(dest))
        } else {
            None
        };
        Ok((contract,))
    }

    fn post_dispatch(
        pre: Option<Self::Pre>,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        if let Some(( contract,)) = pre {
            // pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch(
            //  Some((tip, who, initial_payment)),
            //  info,
            //  post_info,
            //  len,
            //  result,
            // )?;
            if let Some(contract) = contract {
                // Check if contract is registered to track usage.
                if crate::RegisteredContracts::<T>::contains_key(&contract) {
                    let actual_fee = pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
                        len as u32, info, post_info, OnChargeTransactionBalanceOf::<T>::from(0),
                    );
                    // TODO: This should not be hardcoded here. The 50% is specified in the runtime
                    // for DealWithFees.
                    let incentives_fee = Permill::from_percent(50) * actual_fee;
                    let era = crate::CurrentEra::<T>::get();
                    crate::ContractUsagePerEra::<T>::mutate(contract.clone(), era, |fees| {
                        *fees = fees.saturating_add(incentives_fee.into())
                    });
                    crate::EraInformation::<T>::mutate(era, |era_info| {
                        era_info.add_contract_fee(incentives_fee.into());
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
