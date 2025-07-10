use ink::sol::SolErrorEncode;

use super::*;

#[derive(ink::SolErrorDecode, ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum Error {
	/// Indicates a failure with the `spender`’s `allowance`.
	ERC20InsufficientAllowance(Address, U256, U256),
	/// Indicates an error related to the current `balance` of a `sender`.
	ERC20InsufficientBalance(Address, U256, U256),
	/// Indicates an error related to a specified `value`.
	ERC20InsufficientValue,
	/// Indicates a failure with the `approver` of a token to be approved.
	ERC20InvalidApprover(Address),
	/// Indicates a failure with the token `receiver`.
	ERC20InvalidReceiver(Address),
	/// Indicates a failure with the token `sender`.
	ERC20InvalidSender(Address),
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

#[test]
fn error_encoding_works() {
	for (result, expected) in [
		    (
				ERC20InsufficientAllowance([255u8; 20].into(), U256::MAX, U256::MAX).abi_encode(),
				"fb8f41b2000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InsufficientBalance([255u8; 20].into(), U256::MAX, U256::MAX).abi_encode(),
				"e450d38c000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			),
			(ERC20InsufficientValue.abi_encode(),"bffe98ad"),
			(
			    ERC20InvalidApprover([255u8; 20].into()).abi_encode(),
				"e602df05000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InvalidReceiver([255u8; 20].into()).abi_encode(),
				"ec442f05000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InvalidSender([255u8; 20].into()).abi_encode(),
				"96c6fd1e000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
		] {
		    assert_eq!(hex::encode(result), expected)
		}
}
