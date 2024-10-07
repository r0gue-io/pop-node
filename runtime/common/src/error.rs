use core::fmt::Debug;

use codec::{Decode, Encode};
pub use frame_support::sp_runtime::{
	ArithmeticError, DispatchError, ModuleError, TokenError, TransactionalError,
};
use frame_support::traits::PalletInfoAccess;

// Error type aliases for the pop runtime modules.
type AssetsInstance<T> = <T as pallet_api::fungibles::pallet::Config>::AssetsInstance;
pub type AssetsErrorOf<T> = pallet_assets::Error<T, AssetsInstance<T>>;
pub type BalancesErrorOf<T> = pallet_balances::Error<T>;
pub type ContractsErrorOf<T> = pallet_contracts::Error<T>;
// Pallet type aliases for the pop runtime modules.
type AssetsOf<T> = pallet_assets::Pallet<T, AssetsInstance<T>>;
type BalancesOf<T> = pallet_balances::Pallet<T>;
type ContractsOf<T> = pallet_contracts::Pallet<T>;

const DECODING_FAILED_ERROR: [u8; 4] = [11, 0, 0, 0];

#[derive(Encode, Decode, Debug)]
pub enum RuntimeError<T>
where
	T: pallet_api::fungibles::Config + pallet_balances::Config + pallet_contracts::Config,
{
	Raw(DispatchError),
	Assets(AssetsErrorOf<T>),
	Balances(BalancesErrorOf<T>),
	Contracts(ContractsErrorOf<T>),
}

impl<T> From<RuntimeError<T>> for u32
where
	T: pallet_api::fungibles::Config + pallet_balances::Config + pallet_contracts::Config,
{
	fn from(value: RuntimeError<T>) -> Self {
		use pop_primitives::Error;
		let dispatch_error = match value {
			RuntimeError::Raw(dispatch_error) => dispatch_error,
			RuntimeError::Assets(error) => error.into(),
			RuntimeError::Balances(error) => error.into(),
			RuntimeError::Contracts(error) => error.into(),
		};
		let primitive_error = match dispatch_error {
			DispatchError::Module(error) => {
				// Note: message not used
				let ModuleError { index, error, message: _message } = error;
				// Map `pallet-contracts::Error::DecodingFailed` to `Error::DecodingFailed`
				if index as usize == ContractsOf::<T>::index() && error == DECODING_FAILED_ERROR {
					Error::DecodingFailed
				} else {
					// Note: lossy conversion of error value due to returned contract status code
					// size limitation
					Error::Module { index, error: [error[0], error[1]] }
				}
			},
			_ => dispatch_error.into(),
		};
		Error::from(primitive_error).into()
	}
}

impl<T> From<u32> for RuntimeError<T>
where
	T: pallet_api::fungibles::Config + pallet_balances::Config + pallet_contracts::Config,
{
	fn from(value: u32) -> Self {
		fn decode<T: Decode>(data: &[u8]) -> T {
			T::decode(&mut &data[..]).expect("Decoding failed")
		}
		let encoded = value.to_le_bytes();
		match encoded {
			[3, index, module_error @ ..] => {
				let index = index as usize;
				match index {
					_ if index == AssetsOf::<T>::index() =>
						RuntimeError::Assets(decode(&module_error)),
					_ if index == BalancesOf::<T>::index() =>
						RuntimeError::Balances(decode(&module_error)),
					_ if index == ContractsOf::<T>::index() =>
						RuntimeError::Contracts(decode(&module_error)),
					_ => panic!("Decoding failed"),
				}
			},
			_ => RuntimeError::Raw(decode(&encoded)),
		}
	}
}

#[track_caller]
pub fn assert_runtime_err_inner<T, R, E: Into<u32>>(
	result: Result<R, E>,
	expected_error: RuntimeError<T>,
) where
	T: pallet_api::fungibles::Config + pallet_balances::Config + pallet_contracts::Config + Debug,
{
	let expected_code: u32 = expected_error.into();
	if let Err(error) = result {
		let error_code: u32 = error.into();
		if error_code != expected_code {
			panic!(
				r#"assertion `left == right` failed
  left: {:?}
 right: {:?}"#,
				RuntimeError::<T>::from(error_code),
				RuntimeError::<T>::from(expected_code),
			);
		}
	} else {
		panic!(
			r#"assertion `left == right` failed
  left: Ok()
 right: {:?}"#,
			RuntimeError::<T>::from(expected_code),
		);
	}
}

#[macro_export]
macro_rules! assert_runtime_err {
	($result:expr, $error:expr $(,)?) => {
		$crate::error::assert_runtime_err_inner($result, $error);
	};
}

#[cfg(test)]
mod test {
	use frame_support::{sp_runtime::ArithmeticError, traits::PalletInfoAccess};
	use pop_api::primitives::{ArithmeticError::Overflow, Error as PopApiError};

	use super::{AssetsErrorOf, BalancesErrorOf, RuntimeError};
	use crate::mock::{Assets, Balances, Test};

	#[test]
	fn runtime_error_to_primitives_error_conversion_works() {
		vec![
			(
				RuntimeError::<Test>::Raw(ArithmeticError::Overflow.into()),
				PopApiError::Arithmetic(Overflow),
			),
			(
				RuntimeError::<Test>::Assets(AssetsErrorOf::<Test>::BalanceLow),
				PopApiError::Module { index: Assets::index() as u8, error: [0, 0] },
			),
			(
				RuntimeError::<Test>::Assets(AssetsErrorOf::<Test>::NoAccount),
				PopApiError::Module { index: Assets::index() as u8, error: [1, 0] },
			),
			(
				RuntimeError::<Test>::Assets(AssetsErrorOf::<Test>::NoPermission),
				PopApiError::Module { index: Assets::index() as u8, error: [2, 0] },
			),
			(
				RuntimeError::<Test>::Balances(BalancesErrorOf::<Test>::VestingBalance),
				PopApiError::Module { index: Balances::index() as u8, error: [0, 0] },
			),
		]
		.into_iter()
		.for_each(|t| {
			let runtime_error: u32 = t.0.into();
			let pop_api_error: u32 = t.1.into();
			// `u32` assertion.
			assert_eq!(runtime_error, pop_api_error);
		});
	}
}
