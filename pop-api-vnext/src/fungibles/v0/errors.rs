use ink::sol::SolErrorEncode;

use super::*;

#[derive(ink::SolErrorDecode, ink::SolErrorEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub enum Error {
	/// The token recipient is invalid.
	InvalidRecipient(Address),
	/// The minimum balance should be non-zero.
	MinBalanceZero,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The `admin` address cannot be the zero address.
	ZeroAdminAddress,
	/// The recipient cannot be the zero address.
	ZeroRecipientAddress,
	/// The sender cannot be the zero address.
	ZeroSenderAddress,
	/// The specified `value` cannot be zero.
	ZeroValue,
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
			InvalidRecipient([255u8; 20].into()).abi_encode(),
			"17858bbe000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
		),
		(MinBalanceZero.abi_encode(), "5f15618b"),
		(NoPermission.abi_encode(), "9d7b369d"),
		(ZeroAdminAddress.abi_encode(), "3ef39b81"),
		(ZeroRecipientAddress.abi_encode(), "ceef9857"),
		(ZeroSenderAddress.abi_encode(), "ff362bc4"),
		(ZeroValue.abi_encode(), "7c946ed7"),
	] {
		assert_eq!(hex::encode(result), expected)
	}
}
