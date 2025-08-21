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
	/// The input failed to decode.
	DecodingFailed,
	/// Reason why a dispatch call failed.
	Dispatch(DispatchError),
	/// The execution of a XCM message failed.
	ExecutionFailed(DynBytes),
	/// Timeouts must be in the future.
	FutureTimeoutMandatory,
	/// Message block limit has been reached for this expiry block. Try a different timeout.
	MaxMessageTimeoutPerBlockReached,
	/// The message was not found.
	MessageNotFound,
	/// Reason why a pallet call failed.
	Module {
		/// Module index, matching the metadata module index.
		index: u8,
		/// Module specific error value.
		error: FixedBytes<4>,
	},
	/// Failed to convert origin.
	OriginConversionFailed,
	/// The request is pending.
	RequestPending,
	/// The sending of a XCM message failed.
	SendingFailed(DynBytes),
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
			ARITHMETIC => Ok(Self::Arithmetic(<ArithmeticError as SolDecode>::decode(&data[4..])?)),
			DECODING_FAILED => Ok(Self::DecodingFailed),
			DISPATCH => Ok(Self::Dispatch(<DispatchError as SolDecode>::decode(&data[4..])?)),
			EXECUTION_FAILED =>
				Ok(Self::ExecutionFailed(<DynBytes as SolDecode>::decode(&data[4..])?)),
			FUTURE_TIMEOUT_MANDATORY => Ok(Self::FutureTimeoutMandatory),
			MAX_MESSAGE_TIMEOUT_PER_BLOCK_REACHED => Ok(Self::MaxMessageTimeoutPerBlockReached),
			MESSAGE_NOT_FOUND => Ok(Self::MessageNotFound),
			MODULE => {
				let ModuleError { index, error } = <ModuleError as SolDecode>::decode(&data[4..])?;
				Ok(Self::Module { index, error })
			},
			ORIGIN_CONVERSION_FAILED => Ok(Self::OriginConversionFailed),
			REQUEST_PENDING => Ok(Self::RequestPending),
			SENDING_FAILED => Ok(Self::SendingFailed(<DynBytes as SolDecode>::decode(&data[4..])?)),
			TOKEN => Ok(Self::Token(<TokenError as SolDecode>::decode(&data[4..])?)),
			TOO_MANY_MESSAGES => Ok(Self::TooManyMessages),
			TRANSACTIONAL =>
				Ok(Self::Transactional(<TransactionalError as SolDecode>::decode(&data[4..])?)),
			TRIE => Ok(Self::Trie(<TrieError as SolDecode>::decode(&data[4..])?)),
			_ => Err(ink::sol::Error),
		}
	}
}

const DECODING_FAILED: [u8; 4] = sol_error_selector!("DecodingFailed", ());
const EXECUTION_FAILED: [u8; 4] = sol_error_selector!("ExecutionFailed", (DynBytes,));
const FUTURE_TIMEOUT_MANDATORY: [u8; 4] = sol_error_selector!("FutureTimeoutMandatory", ());
const MAX_MESSAGE_TIMEOUT_PER_BLOCK_REACHED: [u8; 4] =
	sol_error_selector!("MaxMessageTimeoutPerBlockReached", ());
const ORIGIN_CONVERSION_FAILED: [u8; 4] = sol_error_selector!("OriginConversionFailed", ());
const SENDING_FAILED: [u8; 4] = sol_error_selector!("SendingFailed", (DynBytes,));

#[test]
fn error_decoding_works() {
	use ink::sol::SolErrorDecode;

	for (encoded, expected) in [
		(
			"7fdb06c50000000000000000000000000000000000000000000000000000000000000001",
			Arithmetic(ArithmeticError::Overflow),
		),
		("72065cff", DecodingFailed),
		(
			"20c5a2a9000000000000000000000000000000000000000000000000000000000000000d",
			Dispatch(DispatchError::RootNotAllowed),
		),
		(
			"15fcd67500000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000",
			ExecutionFailed(DynBytes(Vec::default()))
		),
		("885a28b2", FutureTimeoutMandatory),
		("fc952e0b", MaxMessageTimeoutPerBlockReached),
		("28915ac7", MessageNotFound),
		(
			"3323f3c100000000000000000000000000000000000000000000000000000000000000ffffffffff00000000000000000000000000000000000000000000000000000000",
			Module { index: 255, error: FixedBytes([255; 4]) },
		),
		("8926fba8", OriginConversionFailed),
		("806d0f74", RequestPending),
		(
		    "0ff105a200000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000",
			SendingFailed(DynBytes(Vec::default()))
		),
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
