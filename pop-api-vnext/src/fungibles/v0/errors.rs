use ink::{
	sol::{SolDecode, SolErrorDecode},
	sol_error_selector,
};

use super::*;
use crate::{
	errors::{
		ArithmeticError, DispatchError, FixedBytes, ModuleError, TokenError, TransactionalError,
		TrieError,
	},
	impl_sol_encoding_for_precompile,
	sol::PrecompileError,
};

#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
#[derive(ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum Error {
	/// An arithmetic error occurred.
	Arithmetic(ArithmeticError),
	/// The metadata provided is invalid.
	BadMetadata,
	/// Reason why a dispatch call failed.
	Dispatch(DispatchError),
	/// The account balance is insufficient.
	InsufficientBalance,
	/// The token recipient is invalid.
	InvalidRecipient(Address),
	/// The minimum balance should be non-zero.
	MinBalanceZero,
	/// Reason why a pallet call failed.
	Module {
		/// Module index, matching the metadata module index.
		index: u8,
		/// Module specific error value.
		error: FixedBytes<4>,
	},
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The token is not live, and likely being destroyed.
	NotLive,
	/// An error to do with tokens.
	Token(TokenError),
	/// The number of transactional layers has been reached, or we are not in a transactional
	/// layer.
	Transactional(TransactionalError),
	/// An error with tries.
	Trie(TrieError),
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

impl_sol_encoding_for_precompile!(Error);

impl PrecompileError for Error {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
		if data.len() < 4 {
			return Err(ink::sol::Error);
		}

		match data[..4].try_into().expect("length checked above") {
			ARITHMETIC => Ok(Self::Arithmetic(ArithmeticError::decode(&data[4..])?)),
			BAD_METADATA => Ok(Self::BadMetadata),
			DISPATCH => Ok(Self::Dispatch(DispatchError::decode(&data[4..])?)),
			INSUFFICIENT_BALANCE => Ok(Self::InsufficientBalance),
			INVALID_RECIPIENT => {
				#[derive(ink::SolErrorDecode)]
				struct InvalidRecipient(Address);

				let decoded = InvalidRecipient::decode(&data)?;
				Ok(Self::InvalidRecipient(decoded.0))
			},
			MIN_BALANCE_ZERO => Ok(Self::MinBalanceZero),
			MODULE => {
				let ModuleError { index, error } = ModuleError::decode(&data[4..])?;
				Ok(Self::Module { index, error })
			},
			NO_PERMISSION => Ok(Self::NoPermission),
			NOT_LIVE => Ok(Self::NotLive),
			TOKEN => Ok(Self::Token(TokenError::decode(&data[4..])?)),
			TRANSACTIONAL => Ok(Self::Transactional(TransactionalError::decode(&data[4..])?)),
			TRIE => Ok(Self::Trie(TrieError::decode(&data[4..])?)),
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

pub(crate) const ARITHMETIC: [u8; 4] = sol_error_selector!("Arithmetic", (u8,));
const BAD_METADATA: [u8; 4] = sol_error_selector!("BadMetadata", ());
pub(crate) const DISPATCH: [u8; 4] = sol_error_selector!("Dispatch", (u8,));
const INSUFFICIENT_BALANCE: [u8; 4] = sol_error_selector!("InsufficientBalance", ());
const INVALID_RECIPIENT: [u8; 4] = sol_error_selector!("InvalidRecipient", (Address,));
const MIN_BALANCE_ZERO: [u8; 4] = sol_error_selector!("MinBalanceZero", ());
pub(crate) const MODULE: [u8; 4] = sol_error_selector!("Module", (u8, FixedBytes<4>));
const NO_PERMISSION: [u8; 4] = sol_error_selector!("NoPermission", ());
const NOT_LIVE: [u8; 4] = sol_error_selector!("NotLive", ());
pub(crate) const TOKEN: [u8; 4] = sol_error_selector!("Token", (u8,));
pub(crate) const TRANSACTIONAL: [u8; 4] = sol_error_selector!("Transactional", (u8,));
pub(crate) const TRIE: [u8; 4] = sol_error_selector!("Trie", (u8,));
const UNAPPROVED: [u8; 4] = sol_error_selector!("Unapproved", ());
const UNKNOWN: [u8; 4] = sol_error_selector!("Unknown", ());
const ZERO_ADMIN_ADDRESS: [u8; 4] = sol_error_selector!("ZeroAdminAddress", ());
const ZERO_RECIPIENT_ADDRESS: [u8; 4] = sol_error_selector!("ZeroRecipientAddress", ());
const ZERO_SENDER_ADDRESS: [u8; 4] = sol_error_selector!("ZeroSenderAddress", ());
const ZERO_VALUE: [u8; 4] = sol_error_selector!("ZeroValue", ());

#[test]
fn error_decoding_works() {
	use ink::SolBytes;

	for (encoded, expected) in [
		(
			"7fdb06c50000000000000000000000000000000000000000000000000000000000000001",
			Arithmetic(ArithmeticError::Overflow),
		),
		("1ab2b983", BadMetadata),
		(
			"20c5a2a9000000000000000000000000000000000000000000000000000000000000000d",
			Dispatch(DispatchError::RootNotAllowed),
		),
		("f4d678b8", InsufficientBalance),
		(
			"17858bbe000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
			InvalidRecipient([255; 20].into()),
		),
		("5f15618b", MinBalanceZero),
		(
			"3323f3c100000000000000000000000000000000000000000000000000000000000000ffffffffff00000000000000000000000000000000000000000000000000000000",
			Module { index: 255, error: SolBytes([255; 4]) },
		),
		("9d7b369d", NoPermission),
		("baf13b3f", NotLive),
		(
			"57fdc3d80000000000000000000000000000000000000000000000000000000000000009",
			Token(TokenError::Blocked),
		),
		(
			"3008a37e0000000000000000000000000000000000000000000000000000000000000001",
			Transactional(TransactionalError::NoLayer),
		),
		(
			"3ea87b59000000000000000000000000000000000000000000000000000000000000000d",
			Trie(TrieError::DecodeError),
		),
		("91a7df1a", Unapproved),
		("0cf64598", Unknown),
		("3ef39b81", ZeroAdminAddress),
		("ceef9857", ZeroRecipientAddress),
		("ff362bc4", ZeroSenderAddress),
		("7c946ed7", ZeroValue),
	] {
	    let data = hex::decode(encoded).unwrap();
		let decoded = <Error as SolErrorDecode>::decode(data.as_slice()).expect(&format!("unable to decode {encoded}"));
		assert_eq!(decoded, expected)
	}
}

#[test]
fn error_encoding_works() {
	use ink::{SolBytes, SolEncode};

	for (result, expected) in [
		(
			Arithmetic(ArithmeticError::Overflow).encode(),
			"7fdb06c50000000000000000000000000000000000000000000000000000000000000001",
		),
		(
			Dispatch(DispatchError::BadOrigin).encode(),
			"20c5a2a90000000000000000000000000000000000000000000000000000000000000002",
		),
		(
			InvalidRecipient([255u8; 20].into()).encode(),
			"17858bbe000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
		),
		(MinBalanceZero.encode(), "5f15618b"),
		(
			Module{ index: 255, error: SolBytes([255; 4]) }.encode(),
			"3323f3c100000000000000000000000000000000000000000000000000000000000000ffffffffff00000000000000000000000000000000000000000000000000000000",
		),
		(NoPermission.encode(), "9d7b369d"),
		(
			Token(TokenError::Unknown).encode(),
			"57fdc3d80000000000000000000000000000000000000000000000000000000000000004",
		),
		(
			Token(TokenError::BelowMinimum).encode(),
			"57fdc3d80000000000000000000000000000000000000000000000000000000000000002",
		),
		(
			Transactional(TransactionalError::NoLayer).encode(),
			"3008a37e0000000000000000000000000000000000000000000000000000000000000001",
		),
		(
			Trie(TrieError::DecodeError).encode(),
			"3ea87b59000000000000000000000000000000000000000000000000000000000000000d",
		),
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
	use ink::{SolBytes, SolEncode};

	for (encoded, expected) in [
		(Error::Arithmetic(ArithmeticError::Overflow).encode()[..4].to_vec(), ARITHMETIC),
		(Error::BadMetadata.encode(), BAD_METADATA),
		(Error::Dispatch(DispatchError::BadOrigin).encode()[..4].to_vec(), DISPATCH),
		(Error::InsufficientBalance.encode(), INSUFFICIENT_BALANCE),
		(Error::InvalidRecipient(Address::default()).encode()[..4].to_vec(), INVALID_RECIPIENT),
		(Error::MinBalanceZero.encode(), MIN_BALANCE_ZERO),
		(Error::Module { index: 255, error: SolBytes([255; 4]) }.encode()[..4].to_vec(), MODULE),
		(Error::NoPermission.encode(), NO_PERMISSION),
		(Error::NotLive.encode(), NOT_LIVE),
		(Error::Token(TokenError::Unknown).encode()[..4].to_vec(), TOKEN),
		(
			Error::Transactional(TransactionalError::LimitReached).encode()[..4].to_vec(),
			TRANSACTIONAL,
		),
		(Error::Trie(TrieError::DecodeError).encode()[..4].to_vec(), TRIE),
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
