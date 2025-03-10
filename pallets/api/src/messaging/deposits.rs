use sp_runtime::SaturatedConversion;

use super::*;
use sp_std::ops::Mul;

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum ProtocolStorageDeposit {
	XcmQueries,
	IsmpRequests,
}

/// Calculate the deposit required for the space used for a specific protocol.
pub fn calculate_protocol_deposit<T: Config, ByteFee: Get<BalanceOf<T>>>(p: ProtocolStorageDeposit) -> BalanceOf<T> {
	let base: usize = match p {
		ProtocolStorageDeposit::XcmQueries =>
			KeyLenOf::<XcmQueries<T>>::get() as usize +
				AccountIdOf::<T>::max_encoded_len() +
				MessageId::max_encoded_len() +
				Option::<Callback<T::AccountId>>::max_encoded_len(),
		ProtocolStorageDeposit::IsmpRequests =>
			KeyLenOf::<IsmpRequests<T>>::get() as usize +
				AccountIdOf::<T>::max_encoded_len() +
				MessageId::max_encoded_len(),
	};
	ByteFee::get() * base.saturated_into()
}

/// Calculate the deposit for the storage used for the Message enum.
pub fn calculate_message_deposit<T: Config, ByteFee: Get<BalanceOf<T>>>() -> BalanceOf<T> {
	ByteFee::get() *
		(KeyLenOf::<Messages<T>>::get() as usize + Message::<T>::max_encoded_len())
			.saturated_into()
}

/// Blanket implementation of generating the deposit for a type that implements MaxEncodedLen.
pub fn calculate_deposit_of<T: Config, ByteFee: Get<BalanceOf<T>>, U: MaxEncodedLen>() -> BalanceOf<T> {
	ByteFee::get() * U::max_encoded_len().saturated_into()
}

