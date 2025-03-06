
use super::*;
use sp_runtime::SaturatedConversion;


#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum ProtocolStorageDeposit {
    XcmQueries,
    IsmpRequests,
}

fn calculate_type_deposit<T: Config, U: MaxEncodedLen>() -> BalanceOf<T> {
    T::ByteFee::get() * U::max_encoded_len().saturated_into() 
}

fn calculate_protocol_deposit<T: Config>(p: ProtocolStorageDeposit) -> BalanceOf<T> {
    let base: usize = match p {
        ProtocolStorageDeposit::XcmQueries => {
            KeyLenOf::<XcmQueries<T>>::get() as usize +
					AccountIdOf::<T>::max_encoded_len() +
					MessageId::max_encoded_len() +
					Option::<Callback<T::AccountId>>::max_encoded_len()
        },
        ProtocolStorageDeposit::IsmpRequests => {
            KeyLenOf::<IsmpRequests<T>>::get() as usize +
						AccountIdOf::<T>::max_encoded_len() +
						MessageId::max_encoded_len()
        },
    };
    T::ByteFee::get() * base.saturated_into()
}

fn calculate_message_deposit<T: Config>() -> BalanceOf<T> {
    T::ByteFee::get() * (KeyLenOf::<Messages<T>>::get() as usize + Message::<T>::max_encoded_len()).saturated_into()
}


pub fn calculate_deposit<T: Config, U: MaxEncodedLen>(p: ProtocolStorageDeposit) -> BalanceOf<T> {
    calculate_type_deposit::<T, U>()
    .saturating_add(calculate_protocol_deposit::<T>(p))
    .saturating_add(calculate_message_deposit::<T>())
}

