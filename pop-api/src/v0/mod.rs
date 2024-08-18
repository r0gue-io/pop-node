use crate::{
	build_extension_method,
	constants::{DISPATCH, READ_STATE},
	StatusCode,
};
use ink::{env::chain_extension::ChainExtensionMethod, scale::Decode};

/// APIs for asset-related use cases.
#[cfg(feature = "assets")]
pub mod assets;

pub(crate) const V0: u8 = 0;

/// Reason why a Pop API call failed.
#[derive(Debug, Eq, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[repr(u8)]
#[allow(clippy::unnecessary_cast)]
pub enum Error {
	/// An unknown error occurred. This variant captures any unexpected errors that the
	/// contract cannot specifically handle. It is useful for cases where there are breaking
	/// changes in the runtime or when an error falls outside the predefined categories. The
	/// variant includes:
	Other {
		/// The index within the `DispatchError`.
		dispatch_error_index: u8,
		/// The index within the `DispatchError` variant (e.g. a `TokenError`).
		error_index: u8,
		/// The specific error code or sub-index, providing additional context (e.g.
		/// error` in `ModuleError`).
		error: u8,
	} = 0,
	/// Failed to lookup some data.
	CannotLookup = 1,
	/// A bad origin.
	BadOrigin = 2,
	/// A custom error in a module.
	///
	Module {
		/// The pallet index.
		index: u8,
		/// The error within the pallet.
		error: u8,
	} = 3,
	/// At least one consumer is remaining so the account cannot be destroyed.
	ConsumerRemaining = 4,
	/// There are no providers so the account cannot be created.
	NoProviders = 5,
	/// There are too many consumers so the account cannot be created.
	TooManyConsumers = 6,
	/// An error to do with tokens.
	Token(TokenError) = 7,
	/// An arithmetic error.
	Arithmetic(ArithmeticError) = 8,
	/// The number of transactional layers has been reached, or we are not in a transactional
	/// layer.
	Transactional(TransactionalError) = 9,
	/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
	Exhausted = 10,
	/// The state is corrupt; this is generally not going to fix itself.
	Corruption = 11,
	/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
	Unavailable = 12,
	/// Root origin is not allowed.
	RootNotAllowed = 13,
	/// Unknown call.
	UnknownCall = 254,
	/// Decoding failed.
	DecodingFailed = 255,
}

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}

impl From<u32> for Error {
	/// Converts a `u32` status code into an `Error`.
	///
	/// This conversion maps a raw status code returned by the runtime into the more
	/// descriptive `Error` enum variant, providing better context and understanding of the
	/// error.
	fn from(value: u32) -> Self {
		let encoded = value.to_le_bytes();
		Error::decode(&mut &encoded[..]).unwrap_or(Error::DecodingFailed)
	}
}

/// Description of what went wrong when trying to complete an operation on a token.
#[derive(Debug, Eq, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum TokenError {
	/// Funds are unavailable.
	FundsUnavailable,
	/// Some part of the balance gives the only provider reference to the account and thus cannot
	/// be (re)moved.
	OnlyProvider,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The asset in question is unknown.
	UnknownAsset,
	/// Funds exist but are frozen.
	Frozen,
	/// Operation is not supported by the asset.
	Unsupported,
	/// Account cannot be created for a held balance.
	CannotCreateHold,
	/// Withdrawal would cause unwanted loss of account.
	NotExpendable,
	/// Account cannot receive the assets.
	Blocked,
}

/// Arithmetic errors.
#[derive(Debug, Eq, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum ArithmeticError {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
}

/// Errors related to transactional storage layers.
#[derive(Debug, Eq, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum TransactionalError {
	/// Too many transactional layers have been spawned.
	LimitReached,
	/// A transactional layer was expected, but does not exist.
	NoLayer,
}

// Helper method to build a dispatch call `ChainExtensionMethod`
//
// Parameters:
// - 'module': The index of the runtime module
// - 'dispatchable': The index of the module dispatchable functions
fn build_dispatch(module: u8, dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method(V0, DISPATCH, module, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'module': The index of the runtime module.
// - 'state_query': The index of the runtime state query.
fn build_read_state(module: u8, state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method(V0, READ_STATE, module, state_query)
}
