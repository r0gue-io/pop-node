use super::{Config, Event, Pallet, Payload};
use alloc::{format, string::ToString};
use frame_support::traits::fungible::Mutate;
use ismp::{
    error::Error as IsmpError,
    host::StateMachine,
    module::IsmpModule,
    router::{Post, Request, Response},
};

/// Module callback for the pallet
pub struct IsmpModuleCallback<T: Config>(core::marker::PhantomData<T>);

impl<T: Config> Default for IsmpModuleCallback<T> {
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T: Config> IsmpModule for IsmpModuleCallback<T> {
    fn on_accept(&self, request: Post) -> Result<(), IsmpError> {
        let source_chain = request.source;

        match source_chain {
            StateMachine::Polkadot(_) | StateMachine::Kusama(_) => {
                let payload =
                    <Payload<T::AccountId, <T as Config>::Balance> as codec::Decode>::decode(
                        &mut &*request.data,
                    )
                    .map_err(|_| {
                        IsmpError::ImplementationSpecific(
                            "Failed to decode request data".to_string(),
                        )
                    })?;
                <T::NativeCurrency as Mutate<T::AccountId>>::mint_into(
                    &payload.to,
                    payload.amount.into(),
                )
                .map_err(|_| {
                    IsmpError::ImplementationSpecific("Failed to mint funds".to_string())
                })?;
                Pallet::<T>::deposit_event(Event::<T>::BalanceReceived {
                    from: payload.from,
                    to: payload.to,
                    amount: payload.amount,
                    source_chain,
                });
            }
            source => Err(IsmpError::ImplementationSpecific(format!(
                "Unsupported source {source:?}"
            )))?,
        }

        Ok(())
    }

    fn on_response(&self, response: Response) -> Result<(), IsmpError> {
        match response {
            Response::Post(_) => Err(IsmpError::ImplementationSpecific(
                "Balance transfer protocol does not accept post responses".to_string(),
            ))?,
            Response::Get(res) => Pallet::<T>::deposit_event(Event::<T>::GetResponse(
                res.values.into_values().collect(),
            )),
        };

        Ok(())
    }

    fn on_timeout(&self, request: Request) -> Result<(), IsmpError> {
        let source_chain = request.source_chain();
        let data = match request {
            Request::Post(post) => post.data,
            _ => Err(IsmpError::ImplementationSpecific(
                "Only Post requests allowed, found Get".to_string(),
            ))?,
        };
        let payload =
            <Payload<T::AccountId, <T as Config>::Balance> as codec::Decode>::decode(&mut &*data)
                .map_err(|_| {
                IsmpError::ImplementationSpecific("Failed to decode request data".to_string())
            })?;
        <T::NativeCurrency as Mutate<T::AccountId>>::mint_into(
            &payload.from,
            payload.amount.into(),
        )
        .map_err(|_| IsmpError::ImplementationSpecific("Failed to mint funds".to_string()))?;
        Pallet::<T>::deposit_event(Event::<T>::BalanceReceived {
            from: payload.from,
            to: payload.to,
            amount: payload.amount,
            source_chain,
        });
        Ok(())
    }
}
