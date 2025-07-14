use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ink::{
	sol::{SolErrorDecode, SolErrorEncode},
	sol_error_selector,
};

use super::*;
use crate::sol::PrecompileError;

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)] // TODO: check removal
pub enum Error {
	/// The metadata provided is invalid.
	BadMetadata,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The account balance is insufficient.
	InsufficientBalance,
	/// The token recipient is invalid.
	InvalidRecipient(Address),
	/// The minimum balance should be non-zero.
	MinBalanceZero,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The token is not live, and likely being destroyed.
	NotLive,
	/// The token balance overflowed.
	Overflow,
	/// No approval exists that would allow the transfer.
	Unapproved,
	/// The given token identifier is unknown.
	Unknown,
	/// The `admin` address cannot be the zero address.
	ZeroAdminAddress,
	/// The recipient cannot be the zero address.
	ZeroRecipientAddress,
	/// The sender cannot be the zero address.
	ZeroSenderAddress,
	/// The specified `value` cannot be zero.
	ZeroValue,
}

impl SolErrorDecode for Error {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
		use crate::sol::{Error, ERROR};

		if data.len() < 4 || data[..4] != ERROR {
			return <Self as PrecompileError>::decode(data);
		}

		// Decode as `Error(string)`, then via `base64::decode` and finally decode into `Error`
		let error = Error::decode(data)?;
		let data = BASE64.decode(error.0).map_err(|_| ink::sol::Error)?;
		return <Self as PrecompileError>::decode(data.as_slice());
	}
}

impl crate::sol::PrecompileError for Error {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
		if data.len() < 4 {
			return Err(ink::sol::Error);
		}

		match data[..4].try_into().expect("length checked above") {
			BAD_METADATA => Ok(Self::BadMetadata),
			BELOW_MINIMUM => Ok(Self::BelowMinimum),
			CANNOT_CREATE => Ok(Self::CannotCreate),
			INSUFFICIENT_BALANCE => Ok(Self::InsufficientBalance),
			INVALID_RECIPIENT => {
				#[derive(ink::SolErrorDecode)]
				struct InvalidRecipient(Address);

				let decoded = InvalidRecipient::decode(data)?;
				Ok(Self::InvalidRecipient(decoded.0))
			},
			MIN_BALANCE_ZERO => Ok(Self::MinBalanceZero),
			NO_PERMISSION => Ok(Self::NoPermission),
			NOT_LIVE => Ok(Self::NotLive),
			OVERFLOW => Ok(Self::Overflow),
			UNAPPROVED => Ok(Self::Unapproved),
			UNKNOWN => Ok(Self::Unknown),
			ZERO_ADMIN_ADDRESS => Ok(Self::ZeroAdminAddress),
			ZERO_RECIPIENT_ADDRESS => Ok(Self::ZeroRecipientAddress),
			ZERO_SENDER_ADDRESS => Ok(Self::ZeroSenderAddress),
			ZERO_VALUE => Ok(Self::ZeroValue),
			_ => Err(ink::sol::Error),
		}
	}
}

impl<'a> ink::SolEncode<'a> for Error {
	type SolType = ();

	fn encode(&'a self) -> Vec<u8> {
		SolErrorEncode::encode(self)
	}

	fn to_sol_type(&'a self) -> Self::SolType {
		()
	}
}

const BAD_METADATA: [u8; 4] = sol_error_selector!("BadMetadata", ());
const BELOW_MINIMUM: [u8; 4] = sol_error_selector!("BelowMinimum", ());
const CANNOT_CREATE: [u8; 4] = sol_error_selector!("CannotCreate", ());
const INSUFFICIENT_BALANCE: [u8; 4] = sol_error_selector!("InsufficientBalance", ());
const INVALID_RECIPIENT: [u8; 4] = sol_error_selector!("InvalidRecipient", (Address,));
const MIN_BALANCE_ZERO: [u8; 4] = sol_error_selector!("MinBalanceZero", ());
const NO_PERMISSION: [u8; 4] = sol_error_selector!("NoPermission", ());
const NOT_LIVE: [u8; 4] = sol_error_selector!("NotLive", ());
const OVERFLOW: [u8; 4] = sol_error_selector!("Overflow", ());
const UNAPPROVED: [u8; 4] = sol_error_selector!("Unapproved", ());
const UNKNOWN: [u8; 4] = sol_error_selector!("Unknown", ());
const ZERO_ADMIN_ADDRESS: [u8; 4] = sol_error_selector!("ZeroAdminAddress", ());
const ZERO_RECIPIENT_ADDRESS: [u8; 4] = sol_error_selector!("ZeroRecipientAddress", ());
const ZERO_SENDER_ADDRESS: [u8; 4] = sol_error_selector!("ZeroSenderAddress", ());
const ZERO_VALUE: [u8; 4] = sol_error_selector!("ZeroValue", ());

#[test]
fn error_encoding_works() {
	for (result, expected) in [
		(
			InvalidRecipient([255u8; 20].into()).encode(),
			"17858bbe000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
		),
		(MinBalanceZero.encode(), "5f15618b"),
		(NoPermission.encode(), "9d7b369d"),
		(ZeroAdminAddress.encode(), "3ef39b81"),
		(ZeroRecipientAddress.encode(), "ceef9857"),
		(ZeroSenderAddress.encode(), "ff362bc4"),
		(ZeroValue.encode(), "7c946ed7"),
	] {
		assert_eq!(hex::encode(result), expected)
	}
}

#[test]
fn selectors_work() {
	for (encoded, expected) in [
		(Error::BadMetadata.encode(), BAD_METADATA),
		(Error::BelowMinimum.encode(), BELOW_MINIMUM),
		(Error::CannotCreate.encode(), CANNOT_CREATE),
		(Error::InsufficientBalance.encode(), INSUFFICIENT_BALANCE),
		(Error::InvalidRecipient(Address::default()).encode()[..4].to_vec(), INVALID_RECIPIENT),
		(Error::MinBalanceZero.encode(), MIN_BALANCE_ZERO),
		(Error::NoPermission.encode(), NO_PERMISSION),
		(Error::NotLive.encode(), NOT_LIVE),
		(Error::Overflow.encode(), OVERFLOW),
		(Error::Unapproved.encode(), UNAPPROVED),
		(Error::Unknown.encode(), UNKNOWN),
		(Error::ZeroAdminAddress.encode(), ZERO_ADMIN_ADDRESS),
		(Error::ZeroRecipientAddress.encode(), ZERO_RECIPIENT_ADDRESS),
		(Error::ZeroSenderAddress.encode(), ZERO_SENDER_ADDRESS),
		(Error::ZeroValue.encode(), ZERO_VALUE),
	] {
		assert_eq!(encoded, expected);
	}
}
