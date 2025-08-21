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
	/// Reason why a dispatch call failed.
	Dispatch(DispatchError),
	/// Indicates a failure with the `spender`’s `allowance`.
	ERC20InsufficientAllowance(Address, U256, U256),
	/// Indicates an error related to the current `balance` of a `sender`.
	ERC20InsufficientBalance(Address, U256, U256),
	/// Indicates an error related to a specified `value`.
	ERC20InsufficientValue,
	/// Indicates a failure with the token `receiver`.
	ERC20InvalidReceiver(Address),
	/// Indicates a failure with the token `sender`.
	ERC20InvalidSender(Address),
	/// Indicates a failure with the `spender` to be approved.
	ERC20InvalidSpender(Address),
	/// Reason why a pallet call failed.
	Module {
		/// Module index, matching the metadata module index.
		index: u8,
		/// Module specific error value.
		error: FixedBytes<4>,
	},
	/// An error to do with tokens.
	Token(TokenError),
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
			INSUFFICIENT_ALLOWANCE => {
				#[derive(ink::SolErrorDecode)]
				struct ERC20InsufficientAllowance(Address, U256, U256);

				let decoded = ERC20InsufficientAllowance::decode(data)?;
				Ok(Self::ERC20InsufficientAllowance(decoded.0, decoded.1, decoded.2))
			},
			INSUFFICIENT_BALANCE => {
				#[derive(ink::SolErrorDecode)]
				struct ERC20InsufficientBalance(Address, U256, U256);

				let decoded = ERC20InsufficientBalance::decode(data)?;
				Ok(Self::ERC20InsufficientBalance(decoded.0, decoded.1, decoded.2))
			},
			INSUFFICIENT_VALUE => Ok(Self::ERC20InsufficientValue),
			INVALID_RECEIVER => {
				#[derive(ink::SolErrorDecode)]
				struct ERC20InvalidReceiver(Address);

				let decoded = ERC20InvalidReceiver::decode(data)?;
				Ok(Self::ERC20InvalidReceiver(decoded.0))
			},
			INVALID_SENDER => {
				#[derive(ink::SolErrorDecode)]
				struct ERC20InvalidSender(Address);

				let decoded = ERC20InvalidSender::decode(data)?;
				Ok(Self::ERC20InvalidSender(decoded.0))
			},
			INVALID_SPENDER => {
				#[derive(ink::SolErrorDecode)]
				struct ERC20InvalidSpender(Address);

				let decoded = ERC20InvalidSpender::decode(data)?;
				Ok(Self::ERC20InvalidSpender(decoded.0))
			},
			MODULE => {
				let ModuleError { index, error } = ModuleError::decode(&data[4..])?;
				Ok(Self::Module { index, error })
			},
			TOKEN => Ok(Self::Token(TokenError::decode(&data[4..])?)),
			TRANSACTIONAL => Ok(Self::Transactional(TransactionalError::decode(&data[4..])?)),
			TRIE => Ok(Self::Trie(TrieError::decode(&data[4..])?)),
			_ => Err(ink::sol::Error),
		}
	}
}

const ARITHMETIC: [u8; 4] = sol_error_selector!("Arithmetic", (u8,));
const DISPATCH: [u8; 4] = sol_error_selector!("Dispatch", (u8,));
const INSUFFICIENT_ALLOWANCE: [u8; 4] =
	sol_error_selector!("ERC20InsufficientAllowance", (Address, U256, U256));
const INSUFFICIENT_BALANCE: [u8; 4] =
	sol_error_selector!("ERC20InsufficientBalance", (Address, U256, U256));
const INSUFFICIENT_VALUE: [u8; 4] = sol_error_selector!("ERC20InsufficientValue", ());
const INVALID_RECEIVER: [u8; 4] = sol_error_selector!("ERC20InvalidReceiver", (Address,));
const INVALID_SENDER: [u8; 4] = sol_error_selector!("ERC20InvalidSender", (Address,));
const INVALID_SPENDER: [u8; 4] = sol_error_selector!("ERC20InvalidSpender", (Address,));
const MODULE: [u8; 4] = sol_error_selector!("Module", (u8, FixedBytes<4>));
const TOKEN: [u8; 4] = sol_error_selector!("Token", (u8,));
const TRANSACTIONAL: [u8; 4] = sol_error_selector!("Transactional", (u8,));
const TRIE: [u8; 4] = sol_error_selector!("Trie", (u8,));

#[test]
fn error_decoding_works() {
	for (encoded, expected) in [
		(
			"7fdb06c50000000000000000000000000000000000000000000000000000000000000001",
			Arithmetic(ArithmeticError::Overflow),
		),
		(
			"20c5a2a9000000000000000000000000000000000000000000000000000000000000000d",
			Dispatch(DispatchError::RootNotAllowed),
		),
		(
			"fb8f41b2000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			ERC20InsufficientAllowance(
				[255; 20].into(),
				U256::MAX,
				U256::MAX,
			),
		),
		(
			"e450d38c000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			ERC20InsufficientBalance(
				[255; 20].into(),
				U256::MAX,
				U256::MAX,
			),
		),
		(
			"bffe98ad",
			ERC20InsufficientValue,
		),
		(
			"ec442f05000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
			ERC20InvalidReceiver([255; 20].into()),
		),
		(
			"96c6fd1e000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
			ERC20InvalidSender([255; 20].into()),
		),
		(
			"94280d62000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
			ERC20InvalidSpender([255; 20].into()),
		),
		(
			"3323f3c100000000000000000000000000000000000000000000000000000000000000ffffffffff00000000000000000000000000000000000000000000000000000000",
			Module { index: 255, error: FixedBytes([255; 4]) },
		),
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
	] {
        let data = hex::decode(encoded).unwrap();
        let decoded = <Error as SolErrorDecode>::decode(data.as_slice()).expect(&format!("unable to decode {encoded}"));
        assert_eq!(decoded, expected)
	}
}

#[test]
fn error_encoding_works() {
	use ink::SolEncode;

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
				ERC20InsufficientAllowance([255u8; 20].into(), U256::MAX, U256::MAX).encode(),
				"fb8f41b2000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InsufficientBalance([255u8; 20].into(), U256::MAX, U256::MAX).encode(),
				"e450d38c000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			),
			(ERC20InsufficientValue.encode(),"bffe98ad"),
			(
			    ERC20InvalidReceiver([255u8; 20].into()).encode(),
				"ec442f05000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InvalidSender([255u8; 20].into()).encode(),
				"96c6fd1e000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InvalidSpender([255u8; 20].into()).encode(),
				"94280d62000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
				Module{ index: 255, error: FixedBytes([255; 4]) }.encode(),
				"3323f3c100000000000000000000000000000000000000000000000000000000000000ffffffffff00000000000000000000000000000000000000000000000000000000",
			),
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
		] {
		    assert_eq!(hex::encode(result), expected)
		}
}

#[test]
fn selectors_work() {
	use ink::SolEncode;

	for (encoded, expected) in [
		(Error::Arithmetic(ArithmeticError::Overflow).encode()[..4].to_vec(), ARITHMETIC),
		(Error::Dispatch(DispatchError::BadOrigin).encode()[..4].to_vec(), DISPATCH),
		(
			Error::ERC20InsufficientAllowance(Address::default(), U256::default(), U256::default())
				.encode(),
			INSUFFICIENT_ALLOWANCE,
		),
		(
			Error::ERC20InsufficientBalance(Address::default(), U256::default(), U256::default())
				.encode(),
			INSUFFICIENT_BALANCE,
		),
		(Error::ERC20InsufficientValue.encode(), INSUFFICIENT_VALUE),
		(Error::ERC20InvalidReceiver(Address::default()).encode(), INVALID_RECEIVER),
		(Error::ERC20InvalidSender(Address::default()).encode(), INVALID_SENDER),
		(Error::ERC20InvalidSpender(Address::default()).encode(), INVALID_SPENDER),
		(Error::Module { index: 255, error: FixedBytes([255; 4]) }.encode()[..4].to_vec(), MODULE),
		(Error::Token(TokenError::Unknown).encode()[..4].to_vec(), TOKEN),
		(
			Error::Transactional(TransactionalError::LimitReached).encode()[..4].to_vec(),
			TRANSACTIONAL,
		),
		(Error::Trie(TrieError::DecodeError).encode()[..4].to_vec(), TRIE),
	] {
		assert_eq!(encoded[..4], expected);
	}
}
