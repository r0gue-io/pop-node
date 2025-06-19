use super::*;

// NOTE: subject to change based on ink!'s support for solidity custom errors.
pub enum Error {
	/// The token recipient is invalid.
	InvalidRecipient(InvalidRecipient),
	/// The minimum balance should be non-zero.
	MinBalanceZero(MinBalanceZero),
	/// The signing account has no permission to do the operation.
	NoPermission(NoPermission),
	/// The `admin` address cannot be the zero address.
	ZeroAdminAddress(ZeroAdminAddress),
	/// The recipient cannot be the zero address.
	ZeroRecipientAddress(ZeroRecipientAddress),
	/// The sender cannot be the zero address.
	ZeroSenderAddress(ZeroSenderAddress),
	/// The specified `value` cannot be zero.
	ZeroValue(ZeroValue),
}

impl<'a> SolEncode<'a> for Error {
	type SolType = ();

	fn encode(&'a self) -> Vec<u8> {
		use Error::*;
		match self {
			InvalidRecipient(e) => e.abi_encode(),
			MinBalanceZero(e) => e.abi_encode(),
			NoPermission(e) => e.abi_encode(),
			ZeroAdminAddress(e) => e.abi_encode(),
			ZeroRecipientAddress(e) => e.abi_encode(),
			ZeroSenderAddress(e) => e.abi_encode(),
			ZeroValue(e) => e.abi_encode(),
		}
	}

	fn to_sol_type(&'a self) -> Self::SolType {
		()
	}
}

/// The token recipient is invalid.
pub struct InvalidRecipient(pub Address);
impl SolError for InvalidRecipient {
	type Parameters<'a> = (SolAddress,);
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [23, 133, 139, 190];
	const SIGNATURE: &'static str = "InvalidRecipient(address)";

	#[inline]
	fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self(Address::from(*tuple.0 .0))
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		(self.0.to_sol_type().tokenize(),)
	}
}
impl<'a> SolEncode<'a> for InvalidRecipient {
	type SolType = (&'a Address,);

	#[inline]
	fn encode(&'a self) -> Vec<u8> {
		self.abi_encode()
	}

	#[inline]
	fn to_sol_type(&'a self) -> Self::SolType {
		(&self.0,)
	}
}
impl From<InvalidRecipient> for Error {
	fn from(value: InvalidRecipient) -> Self {
		Self::InvalidRecipient(value)
	}
}

/// The minimum balance should be non-zero.
pub struct MinBalanceZero;
impl SolError for MinBalanceZero {
	type Parameters<'a> = ();
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [95, 21, 97, 139];
	const SIGNATURE: &'static str = "MinBalanceZero()";

	#[inline]
	fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		()
	}
}
impl From<MinBalanceZero> for Error {
	fn from(value: MinBalanceZero) -> Self {
		Self::MinBalanceZero(value)
	}
}

/// The signing account has no permission to do the operation.
pub struct NoPermission;
impl SolError for NoPermission {
	type Parameters<'a> = ();
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [157, 123, 54, 157];
	const SIGNATURE: &'static str = "NoPermission()";

	#[inline]
	fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		()
	}
}
impl<'a> SolEncode<'a> for NoPermission {
	type SolType = ();

	fn encode(&'a self) -> Vec<u8> {
		self.abi_encode()
	}

	fn to_sol_type(&'a self) -> Self::SolType {
		()
	}
}
impl From<NoPermission> for Error {
	fn from(value: NoPermission) -> Self {
		Self::NoPermission(value)
	}
}

/// The `admin` address cannot be the zero address.
pub struct ZeroAdminAddress;
impl SolError for ZeroAdminAddress {
	type Parameters<'a> = ();
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [62, 243, 155, 129];
	const SIGNATURE: &'static str = "ZeroAdminAddress()";

	#[inline]
	fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		()
	}
}
impl From<ZeroAdminAddress> for Error {
	fn from(value: ZeroAdminAddress) -> Self {
		Self::ZeroAdminAddress(value)
	}
}

/// The recipient cannot be the zero address.
pub struct ZeroRecipientAddress;
impl SolError for ZeroRecipientAddress {
	type Parameters<'a> = ();
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [206, 239, 152, 87];
	const SIGNATURE: &'static str = "ZeroRecipientAddress()";

	#[inline]
	fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		()
	}
}
impl From<ZeroRecipientAddress> for Error {
	fn from(value: ZeroRecipientAddress) -> Self {
		Self::ZeroRecipientAddress(value)
	}
}

/// The sender cannot be the zero address.
pub struct ZeroSenderAddress;
impl SolError for ZeroSenderAddress {
	type Parameters<'a> = ();
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [255, 54, 43, 196];
	const SIGNATURE: &'static str = "ZeroSenderAddress()";

	#[inline]
	fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		()
	}
}
impl From<ZeroSenderAddress> for Error {
	fn from(value: ZeroSenderAddress) -> Self {
		Self::ZeroSenderAddress(value)
	}
}

/// The `value` should be non-zero.
pub struct ZeroValue;
impl SolError for ZeroValue {
	type Parameters<'a> = ();
	type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

	const SELECTOR: [u8; 4] = [124, 148, 110, 215];
	const SIGNATURE: &'static str = "ZeroValue()";

	#[inline]
	fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
		Self
	}

	#[inline]
	fn tokenize(&self) -> Self::Token<'_> {
		()
	}
}
impl From<ZeroValue> for Error {
	fn from(value: ZeroValue) -> Self {
		Self::ZeroValue(value)
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
