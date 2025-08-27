use ink::{sol::SolDecode, sol_error_selector};

use super::*;
use crate::{
	errors::{
		ArithmeticError, DispatchError, FixedBytes, ModuleError, TokenError, TransactionalError,
		TrieError,
	},
	impl_sol_encoding_for_precompile,
	messaging::v0::errors::{
		ARITHMETIC, DISPATCH, MESSAGE_NOT_FOUND, MODULE, REQUEST_PENDING, TOKEN, TOO_MANY_MESSAGES,
		TRANSACTIONAL, TRIE,
	},
	sol::PrecompileError,
};

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum Error {
	/// An arithmetic error occurred.
	Arithmetic(ArithmeticError),
	/// Reason why a dispatch call failed.
	Dispatch(DispatchError),
	/// The context exceeds the maximum allowed size.
	MaxContextExceeded,
	/// The data exceeds the maximum allowed size.
	MaxDataExceeded,
	/// A key exceeds the maximum allowed size.
	MaxKeyExceeded,
	/// The number of keys exceeds the maximum allowed size.
	MaxKeysExceeded,
	/// The message was not found.
	MessageNotFound,
	/// Reason why a pallet call failed.
	Module {
		/// Module index, matching the metadata module index.
		index: u8,
		/// Module specific error value.
		error: FixedBytes<4>,
	},
	/// The request is pending.
	RequestPending,
	/// An error to do with tokens.
	Token(TokenError),
	/// The number of messages exceeds the limit.
	TooManyMessages,
	/// The number of transactional layers has been reached, or we are not in a transactional
	/// layer.
	Transactional(TransactionalError),
	/// An error with tries.
	Trie(TrieError),
}

impl_sol_encoding_for_precompile!(Error);

impl PrecompileError for Error {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
		if data.len() < 4 {
			return Err(ink::sol::Error);
		}

		match data[..4].try_into().expect("length checked above") {
			ARITHMETIC => Ok(Self::Arithmetic(ArithmeticError::decode(&data[4..])?)),
			DISPATCH => Ok(Self::Dispatch(DispatchError::decode(&data[4..])?)),
			MAX_CONTEXT_EXCEEDED => Ok(Self::MaxContextExceeded),
			MAX_DATA_EXCEEDED => Ok(Self::MaxDataExceeded),
			MAX_KEY_EXCEEDED => Ok(Self::MaxKeyExceeded),
			MAX_KEYS_EXCEEDED => Ok(Self::MaxKeysExceeded),
			MESSAGE_NOT_FOUND => Ok(Self::MessageNotFound),
			MODULE => {
				let ModuleError { index, error } = ModuleError::decode(&data[4..])?;
				Ok(Self::Module { index, error })
			},
			REQUEST_PENDING => Ok(Self::RequestPending),
			TOKEN => Ok(Self::Token(TokenError::decode(&data[4..])?)),
			TOO_MANY_MESSAGES => Ok(Self::TooManyMessages),
			TRANSACTIONAL => Ok(Self::Transactional(TransactionalError::decode(&data[4..])?)),
			TRIE => Ok(Self::Trie(TrieError::decode(&data[4..])?)),
			_ => Err(ink::sol::Error),
		}
	}
}

const MAX_CONTEXT_EXCEEDED: [u8; 4] = sol_error_selector!("MaxContextExceeded", ());
const MAX_DATA_EXCEEDED: [u8; 4] = sol_error_selector!("MaxDataExceeded", ());
const MAX_KEY_EXCEEDED: [u8; 4] = sol_error_selector!("MaxKeyExceeded", ());
const MAX_KEYS_EXCEEDED: [u8; 4] = sol_error_selector!("MaxKeysExceeded", ());

#[test]
fn error_decoding_works() {
	use ink::{sol::SolErrorDecode, SolBytes};

	for (encoded, expected) in [
		(
			"7fdb06c50000000000000000000000000000000000000000000000000000000000000001",
			Arithmetic(ArithmeticError::Overflow),
		),
		(
			"20c5a2a9000000000000000000000000000000000000000000000000000000000000000d",
			Dispatch(DispatchError::RootNotAllowed),
		),
		("8ad49075", MaxContextExceeded),
		("deadaa39", MaxDataExceeded),
		("3d903c2e", MaxKeyExceeded),
		("81463867", MaxKeysExceeded),
		("28915ac7", MessageNotFound),
		(
			"3323f3c100000000000000000000000000000000000000000000000000000000000000ffffffffff00000000000000000000000000000000000000000000000000000000",
			Module { index: 255, error: SolBytes([255; 4]) },
		),
		("806d0f74", RequestPending),
		(
			"57fdc3d80000000000000000000000000000000000000000000000000000000000000009",
			Token(TokenError::Blocked),
		),
		("1ec0b2f7", TooManyMessages),
		(
			"3008a37e0000000000000000000000000000000000000000000000000000000000000001",
			Transactional(TransactionalError::NoLayer),
		),
		(
			"3ea87b59000000000000000000000000000000000000000000000000000000000000000d",
			Trie(TrieError::DecodeError),
		),
	] {
	    let data = hex::decode(encoded).unwrap();
		let decoded = <Error as SolErrorDecode>::decode(data.as_slice()).expect(&format!("unable to decode {encoded}"));
		assert_eq!(decoded, expected)
	}
}
