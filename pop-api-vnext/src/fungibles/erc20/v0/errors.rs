use ink::{sol::SolErrorDecode, sol_error_selector};

use super::*;
use crate::{impl_sol_encoding_for_precompile, sol::PrecompileError};

#[derive(ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum Error {
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
}

impl_sol_encoding_for_precompile!(Error);

impl PrecompileError for Error {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
		if data.len() < 4 {
			return Err(ink::sol::Error);
		}

		match data[..4].try_into().expect("length checked above") {
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
			_ => Err(ink::sol::Error),
		}
	}
}

const INSUFFICIENT_ALLOWANCE: [u8; 4] =
	sol_error_selector!("ERC20InsufficientAllowance", (Address, U256, U256));
const INSUFFICIENT_BALANCE: [u8; 4] =
	sol_error_selector!("ERC20InsufficientBalance", (Address, U256, U256));
const INSUFFICIENT_VALUE: [u8; 4] = sol_error_selector!("ERC20InsufficientValue", ());
const INVALID_RECEIVER: [u8; 4] = sol_error_selector!("ERC20InvalidReceiver", (Address,));
const INVALID_SENDER: [u8; 4] = sol_error_selector!("ERC20InvalidSender", (Address,));
const INVALID_SPENDER: [u8; 4] = sol_error_selector!("ERC20InvalidSpender", (Address,));

#[test]
fn error_encoding_works() {
	use ink::SolEncode;

	for (result, expected) in [
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
		] {
		    assert_eq!(hex::encode(result), expected)
		}
}

#[test]
fn selectors_work() {
	use ink::SolEncode;

	for (encoded, expected) in [
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
	] {
		assert_eq!(encoded[..4], expected);
	}
}
