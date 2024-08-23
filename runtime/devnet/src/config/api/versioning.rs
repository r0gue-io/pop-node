use super::*;
use sp_runtime::ModuleError;

type Version = u8;

/// Versioned runtime calls.
#[derive(Decode, Debug)]
pub enum VersionedRuntimeCall {
	/// Version zero of runtime calls.
	#[codec(index = 0)]
	V0(RuntimeCall),
}

impl From<VersionedRuntimeCall> for RuntimeCall {
	fn from(value: VersionedRuntimeCall) -> Self {
		// Allows mapping from some previous runtime call shape to a current valid runtime call
		match value {
			VersionedRuntimeCall::V0(call) => call,
		}
	}
}

/// Versioned runtime state reads.
#[derive(Decode, Debug)]
pub enum VersionedRuntimeRead {
	/// Version zero of runtime state reads.
	#[codec(index = 0)]
	V0(RuntimeRead),
}

impl From<VersionedRuntimeRead> for RuntimeRead {
	fn from(value: VersionedRuntimeRead) -> Self {
		// Allows mapping from some previous runtime read shape to a current valid runtime read
		match value {
			VersionedRuntimeRead::V0(read) => read,
		}
	}
}

/// Versioned runtime state read results.
#[derive(Debug)]
pub enum VersionedRuntimeResult {
	/// Version zero of runtime read results.
	V0(RuntimeResult),
}

impl From<(RuntimeResult, Version)> for VersionedRuntimeResult {
	fn from(value: (RuntimeResult, Version)) -> Self {
		let (result, version) = value;
		match version {
			// Allows mapping from current `RuntimeResult` to a specific/prior version
			0 => VersionedRuntimeResult::V0(result),
			// TODO: should never occur due to version processing/validation when request received
			_ => unimplemented!(),
		}
	}
}

impl From<VersionedRuntimeResult> for Vec<u8> {
	fn from(result: VersionedRuntimeResult) -> Self {
		match result {
			// Simply unwrap and return the encoded result
			VersionedRuntimeResult::V0(result) => result.encode(),
		}
	}
}

/// Versioned errors.
#[derive(Debug)]
pub enum VersionedError {
	/// Version zero of errors.
	V0(pop_primitives::v0::Error),
}

impl From<(DispatchError, Version)> for VersionedError {
	fn from(value: (DispatchError, Version)) -> Self {
		let (error, version) = value;
		match version {
			// Allows mapping from current `DispatchError` to a specific/prior version of `Error`
			0 => VersionedError::V0(V0Error::from(error).0),
			// TODO: should never occur due to version processing/validation when request received
			_ => unimplemented!(),
		}
	}
}

impl From<VersionedError> for u32 {
	fn from(value: VersionedError) -> Self {
		match value {
			VersionedError::V0(error) => error.into(),
		}
	}
}

struct V0Error(pop_primitives::v0::Error);
impl From<DispatchError> for V0Error {
	fn from(error: DispatchError) -> Self {
		use pop_primitives::v0::*;
		use sp_runtime::{ArithmeticError::*, TokenError::*, TransactionalError::*};
		use DispatchError::*;
		// Mappings exist here to avoid taking a dependency of sp_runtime on pop-primitives
		Self(match error {
			Other(_message) => {
				// Note: lossy conversion: message not used due to returned contract status code size limitation
				Error::Other
			},
			CannotLookup => Error::CannotLookup,
			BadOrigin => Error::BadOrigin,
			Module(error) => {
				// Note: message not used
				let ModuleError { index, error, message: _message } = error;
				// Map `pallet-contracts::Error::DecodingFailed` to `Error::DecodingFailed`
				if index as usize
					== <crate::Contracts as frame_support::traits::PalletInfoAccess>::index()
					&& error == DECODING_FAILED_ERROR
				{
					Error::DecodingFailed
				} else {
					// Note: lossy conversion of error value due to returned contract status code size limitation
					Error::Module { index, error: [error[0], error[1]] }
				}
			},
			ConsumerRemaining => Error::ConsumerRemaining,
			NoProviders => Error::NoProviders,
			TooManyConsumers => Error::TooManyConsumers,
			Token(error) => Error::Token(match error {
				FundsUnavailable => TokenError::FundsUnavailable,
				OnlyProvider => TokenError::OnlyProvider,
				BelowMinimum => TokenError::BelowMinimum,
				CannotCreate => TokenError::CannotCreate,
				UnknownAsset => TokenError::UnknownAsset,
				Frozen => TokenError::Frozen,
				Unsupported => TokenError::Unsupported,
				CannotCreateHold => TokenError::CannotCreateHold,
				NotExpendable => TokenError::NotExpendable,
				Blocked => TokenError::Blocked,
			}),
			Arithmetic(error) => Error::Arithmetic(match error {
				Underflow => ArithmeticError::Underflow,
				Overflow => ArithmeticError::Overflow,
				DivisionByZero => ArithmeticError::DivisionByZero,
			}),
			Transactional(error) => Error::Transactional(match error {
				LimitReached => TransactionalError::LimitReached,
				NoLayer => TransactionalError::NoLayer,
			}),
			Exhausted => Error::Exhausted,
			Corruption => Error::Corruption,
			Unavailable => Error::Unavailable,
			RootNotAllowed => Error::RootNotAllowed,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use pop_primitives::{ArithmeticError::*, Error, TokenError::*, TransactionalError::*};
	use sp_runtime::ModuleError;
	use DispatchError::*;

	// Compare all the different `DispatchError` variants with the expected `Error`.
	#[test]
	fn dispatch_error_to_error() {
		let test_cases = vec![
			(Other(""), (Error::Other)),
			(Other("UnknownCall"), Error::Other),
			(Other("DecodingFailed"), Error::Other),
			(Other("Random"), (Error::Other)),
			(CannotLookup, Error::CannotLookup),
			(BadOrigin, Error::BadOrigin),
			(
				Module(ModuleError { index: 1, error: [2, 0, 0, 0], message: Some("hallo") }),
				Error::Module { index: 1, error: [2, 0] },
			),
			(
				Module(ModuleError { index: 1, error: [2, 2, 0, 0], message: Some("hallo") }),
				Error::Module { index: 1, error: [2, 2] },
			),
			(
				Module(ModuleError { index: 1, error: [2, 2, 2, 0], message: Some("hallo") }),
				Error::Module { index: 1, error: [2, 2] },
			),
			(
				Module(ModuleError { index: 1, error: [2, 2, 2, 4], message: Some("hallo") }),
				Error::Module { index: 1, error: [2, 2] },
			),
			(pallet_contracts::Error::<Runtime>::DecodingFailed.into(), Error::DecodingFailed),
			(ConsumerRemaining, Error::ConsumerRemaining),
			(NoProviders, Error::NoProviders),
			(TooManyConsumers, Error::TooManyConsumers),
			(Token(sp_runtime::TokenError::BelowMinimum), Error::Token(BelowMinimum)),
			(Arithmetic(sp_runtime::ArithmeticError::Overflow), Error::Arithmetic(Overflow)),
			(
				Transactional(sp_runtime::TransactionalError::LimitReached),
				Error::Transactional(LimitReached),
			),
			(Exhausted, Error::Exhausted),
			(Corruption, Error::Corruption),
			(Unavailable, Error::Unavailable),
			(RootNotAllowed, Error::RootNotAllowed),
		];
		for (dispatch_error, expected) in test_cases {
			let error = V0Error::from(dispatch_error).0;
			assert_eq!(error, expected);
		}
	}

	#[test]
	fn decoding_failed_error_encoding_works() {
		let Module(error) = pallet_contracts::Error::<Runtime>::DecodingFailed.into() else {
			unreachable!()
		};
		assert_eq!(error.error, DECODING_FAILED_ERROR)
	}
}
