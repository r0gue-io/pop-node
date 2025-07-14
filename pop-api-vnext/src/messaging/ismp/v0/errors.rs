use ink::sol_error_selector;

use super::*;
use crate::{impl_sol_encoding_for_precompile, sol::PrecompileError};

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)] // TODO: check removal
pub enum Error {
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
	/// The request is pending.
	RequestPending,
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
			MAX_CONTEXT_EXCEEDED => Ok(Self::MaxContextExceeded),
			MAX_DATA_EXCEEDED => Ok(Self::MaxDataExceeded),
			MAX_KEY_EXCEEDED => Ok(Self::MaxKeyExceeded),
			MAX_KEYS_EXCEEDED => Ok(Self::MaxKeysExceeded),
			MESSAGE_NOT_FOUND => Ok(Self::MessageNotFound),
			REQUEST_PENDING => Ok(Self::RequestPending),
			TOO_MANY_MESSAGES => Ok(Self::TooManyMessages),
			_ => Err(ink::sol::Error),
		}
	}
}

const MAX_CONTEXT_EXCEEDED: [u8; 4] = sol_error_selector!("MaxContextExceeded", ());
const MAX_DATA_EXCEEDED: [u8; 4] = sol_error_selector!("MaxDataExceeded", ());
const MAX_KEY_EXCEEDED: [u8; 4] = sol_error_selector!("MaxKeyExceeded", ());
const MAX_KEYS_EXCEEDED: [u8; 4] = sol_error_selector!("MaxKeysExceeded", ());
const MESSAGE_NOT_FOUND: [u8; 4] = sol_error_selector!("MessageNotFound", ());
const REQUEST_PENDING: [u8; 4] = sol_error_selector!("RequestPending", ());
const TOO_MANY_MESSAGES: [u8; 4] = sol_error_selector!("TooManyMessages", ());
