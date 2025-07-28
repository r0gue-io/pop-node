use ink::sol_error_selector;

use super::*;
use crate::{impl_sol_encoding_for_precompile, sol::PrecompileError};

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum Error {
	/// The input failed to decode.
	DecodingFailed,
	/// The execution of a XCM message failed.
	ExecutionFailed(Bytes),
	/// Timeouts must be in the future.
	FutureTimeoutMandatory,
	/// Message block limit has been reached for this expiry block. Try a different timeout.
	MaxMessageTimeoutPerBlockReached,
	/// The message was not found.
	MessageNotFound,
	/// Failed to convert origin.
	OriginConversionFailed,
	/// The request is pending.
	RequestPending,
	/// The sending of a XCM message failed.
	SendingFailed(Bytes),
	/// The number of messages exceeds the limit.
	TooManyMessages,
}

impl_sol_encoding_for_precompile!(Error);

impl PrecompileError for Error {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
		if data.len() < 4 {
			return Err(ink::sol::Error);
		}

		match data[..4].try_into().expect("length checked above") {
			DECODING_FAILED => Ok(Self::DecodingFailed),
			FUTURE_TIMEOUT_MANDATORY => Ok(Self::FutureTimeoutMandatory),
			MAX_MESSAGE_TIMEOUT_PER_BLOCK_REACHED => Ok(Self::MaxMessageTimeoutPerBlockReached),
			MESSAGE_NOT_FOUND => Ok(Self::MessageNotFound),
			ORIGIN_CONVERSION_FAILED => Ok(Self::OriginConversionFailed),
			REQUEST_PENDING => Ok(Self::RequestPending),
			TOO_MANY_MESSAGES => Ok(Self::TooManyMessages),
			_ => Err(ink::sol::Error),
		}
	}
}

const DECODING_FAILED: [u8; 4] = sol_error_selector!("DecodingFailed", ());
const FUTURE_TIMEOUT_MANDATORY: [u8; 4] = sol_error_selector!("FutureTimeoutMandatory", ());
const MAX_MESSAGE_TIMEOUT_PER_BLOCK_REACHED: [u8; 4] =
	sol_error_selector!("MaxMessageTimeoutPerBlockReached", ());
const MESSAGE_NOT_FOUND: [u8; 4] = sol_error_selector!("MessageNotFound", ());
const ORIGIN_CONVERSION_FAILED: [u8; 4] = sol_error_selector!("OriginConversionFailed", ());
const REQUEST_PENDING: [u8; 4] = sol_error_selector!("RequestPending", ());
const TOO_MANY_MESSAGES: [u8; 4] = sol_error_selector!("TooManyMessages", ());
