use super::FungiblesError;
use crate::{
	constants::{ASSETS, BALANCES},
	primitives::{
		ArithmeticError::*,
		Error::{self, *},
		TokenError::*,
		TransactionalError::*,
	},
	StatusCode,
};
use ink::scale::{Decode, Encode};

fn error_into_status_code(error: Error) -> StatusCode {
	let mut encoded_error = error.encode();
	encoded_error.resize(4, 0);
	let value =
		u32::from_le_bytes(encoded_error.try_into().expect("qed, resized to 4 bytes line above"));
	value.into()
}

fn into_fungibles_error(error: Error) -> FungiblesError {
	let status_code: StatusCode = error_into_status_code(error);
	status_code.into()
}

// If we ever want to change the conversion from bytes to `u32`.
#[test]
fn status_code_vs_encoded() {
	assert_eq!(u32::decode(&mut &[3u8, 10, 2, 0][..]).unwrap(), 133635u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 0, 0][..]).unwrap(), 13315u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 1, 0][..]).unwrap(), 78851u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 2, 0][..]).unwrap(), 144387u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 3, 0][..]).unwrap(), 209923u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 5, 0][..]).unwrap(), 340995u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 7, 0][..]).unwrap(), 472067u32);
	assert_eq!(u32::decode(&mut &[3u8, 52, 10, 0][..]).unwrap(), 668675u32);
}

#[test]
fn converting_status_code_into_fungibles_error_works() {
	let other_errors = vec![
		Other,
		CannotLookup,
		BadOrigin,
		// `ModuleError` other than assets module.
		Module { index: 2, error: [5, 0] },
		ConsumerRemaining,
		NoProviders,
		TooManyConsumers,
		Token(OnlyProvider),
		Arithmetic(Overflow),
		Transactional(NoLayer),
		Exhausted,
		Corruption,
		Unavailable,
		RootNotAllowed,
		Unknown { dispatch_error_index: 5, error_index: 5, error: 1 },
		DecodingFailed,
	];
	for error in other_errors {
		let status_code: StatusCode = error_into_status_code(error);
		let fungibles_error: FungiblesError = status_code.into();
		assert_eq!(fungibles_error, FungiblesError::Other(status_code))
	}

	assert_eq!(
		into_fungibles_error(Module { index: BALANCES, error: [2, 0] }),
		FungiblesError::NoBalance
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [0, 0] }),
		FungiblesError::NoAccount
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [1, 0] }),
		FungiblesError::NoPermission
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [2, 0] }),
		FungiblesError::Unknown
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [3, 0] }),
		FungiblesError::InUse
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [5, 0] }),
		FungiblesError::MinBalanceZero
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [7, 0] }),
		FungiblesError::InsufficientAllowance
	);
	assert_eq!(
		into_fungibles_error(Module { index: ASSETS, error: [10, 0] }),
		FungiblesError::NotLive
	);
}
