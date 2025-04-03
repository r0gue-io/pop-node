use sp_runtime::SaturatedConversion;
use sp_std::ops::Mul;

use super::*;

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum ProtocolStorageDeposit {
	XcmQueries,
	IsmpRequests,
}

/// Calculate the deposit required for the space used for a specific protocol.
pub fn calculate_protocol_deposit<T: Config, ByteFee: Get<BalanceOf<T>>>(
	p: ProtocolStorageDeposit,
) -> BalanceOf<T> {
	let base: usize = match p {
		ProtocolStorageDeposit::XcmQueries => (KeyLenOf::<XcmQueries<T>>::get() as usize)
			.saturating_add(AccountIdOf::<T>::max_encoded_len())
			.saturating_add(MessageId::max_encoded_len())
			.saturating_add(Option::<Callback>::max_encoded_len()),

		ProtocolStorageDeposit::IsmpRequests => (KeyLenOf::<IsmpRequests<T>>::get() as usize)
			.saturating_add(AccountIdOf::<T>::max_encoded_len())
			.saturating_add(MessageId::max_encoded_len()),
	};
	ByteFee::get().saturating_mul(base.saturated_into())
}

/// Calculate the deposit for the storage used for the Message enum.
pub fn calculate_message_deposit<T: Config, ByteFee: Get<BalanceOf<T>>>() -> BalanceOf<T> {
	ByteFee::get().saturating_mul(
		(KeyLenOf::<Messages<T>>::get() as usize + Message::<T>::max_encoded_len())
			.saturated_into(),
	)
}

/// Blanket implementation of generating the deposit for a type that implements MaxEncodedLen.
pub fn calculate_deposit_of<T: Config, ByteFee: Get<BalanceOf<T>>, U: MaxEncodedLen>(
) -> BalanceOf<T> {
	ByteFee::get().saturating_mul(U::max_encoded_len().saturated_into())
}

#[cfg(test)]
mod tests {
	use frame_support::pallet_prelude::Get;

	use super::*;
	use crate::mock::*;

	struct Two;
	impl Get<u128> for Two {
		fn get() -> u128 {
			2
		}
	}

	#[test]
	fn calculate_deposit_of_works() {
		new_test_ext().execute_with(|| {
			// 4 + 4 bytes.
			#[derive(
				Copy, Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo,
			)]
			struct Data {
				pub a: u32,
				pub b: u32,
			}

			// 8 * 2 = 16 units
			assert_eq!(calculate_deposit_of::<Test, Two, Data>(), 16);
		})
	}
}
