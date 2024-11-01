/// Represents various errors related to sponsorships.
///
/// The `SponsorshipsError` provides a detailed and specific set of error types that can occur
/// when interacting with sponsorships. Each variant signifies a particular error
/// condition, facilitating precise error handling and debugging.
///
/// The `Other` variant serves as a catch-all for any unexpected errors. For more
/// detailed debugging, the `Other` variant can be converted into the richer `Error` type
/// defined in the primitives crate.
use super::*;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum SponsorshipsError {
	/// An unspecified or unknown error occurred.
	Other(StatusCode),
	/// The account is already being sponsored.
	AlreadySponsored,
	/// This action cannot be sponsored.
	CantSponsor,
	/// The sponsorship doesn't exist.
	UnknownSponsorship,
	/// The cost is higher than the max sponsored.
	SponsorshipOutOfLimits,
}

impl From<StatusCode> for SponsorshipsError {
	/// Converts a `StatusCode` to a `SponsorshipsError`.
	///
	/// This conversion maps a `StatusCode`, returned by the runtime, to a more descriptive
	/// `SponsorshipsError`. This provides better context and understanding of the error, allowing
	/// developers to handle the most important errors effectively.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			[_, SPONSORSHIPS, 0, _] => SponsorshipsError::AlreadySponsored,
			[_, SPONSORSHIPS, 1, _] => SponsorshipsError::CantSponsor,
			[_, SPONSORSHIPS, 2, _] => SponsorshipsError::UnknownSponsorship,
			[_, SPONSORSHIPS, 3, _] => SponsorshipsError::SponsorshipOutOfLimits,
			_ => SponsorshipsError::Other(value),
		}
	}
}

#[cfg(test)]
mod tests {
	use ink::scale::Encode;

	use super::SponsorshipsError;
	use crate::{
		constants::SPONSORSHIPS,
		primitives::{
			ArithmeticError::*,
			Error::{self, *},
			TokenError::*,
			TransactionalError::*,
		},
		StatusCode,
	};

	fn error_into_status_code(error: Error) -> StatusCode {
		let mut encoded_error = error.encode();
		encoded_error.resize(4, 0);
		let value = u32::from_le_bytes(
			encoded_error.try_into().expect("qed, resized to 4 bytes line above"),
		);
		value.into()
	}

	fn into_error<T: From<StatusCode>>(error: Error) -> T {
		error_into_status_code(error).into()
	}

	#[test]
	fn converting_status_code_into_sponsorships_error_works() {
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
			let fungibles_error: SponsorshipsError = status_code.into();
			assert_eq!(fungibles_error, SponsorshipsError::Other(status_code))
		}

		assert_eq!(
			into_error::<SponsorshipsError>(Module { index: SPONSORSHIPS, error: [0, 0] }),
			SponsorshipsError::AlreadySponsored
		);
		assert_eq!(
			into_error::<SponsorshipsError>(Module { index: SPONSORSHIPS, error: [1, 0] }),
			SponsorshipsError::CantSponsor
		);
		assert_eq!(
			into_error::<SponsorshipsError>(Module { index: SPONSORSHIPS, error: [2, 0] }),
			SponsorshipsError::UnknownSponsorship
		);
		assert_eq!(
			into_error::<SponsorshipsError>(Module { index: SPONSORSHIPS, error: [3, 0] }),
			SponsorshipsError::SponsorshipOutOfLimits
		);
	}
}
