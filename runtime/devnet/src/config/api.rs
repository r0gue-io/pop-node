use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall, RuntimeEvent,
};
use codec::Decode;
use cumulus_primitives_core::Weight;
use filtering::*;
use frame_support::traits::Contains;
use pallet_api::extension::*;
pub(crate) use pallet_api::Extension;
use sp_core::ConstU8;
use versioning::*;

/// A query of runtime state.
#[derive(Decode, Debug)]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

impl Readable for RuntimeRead {
	/// Determines the weight of the read, used to charge the appropriate weight before the read is performed.
	fn weight(&self) -> Weight {
		// TODO: defer to relevant pallet - e.g. RuntimeRead::Fungibles(key) => fungibles::Pallet::read_weight(key),
		<Runtime as frame_system::Config>::DbWeight::get().reads(1_u64)
	}

	/// Performs the read and returns the result.
	fn read(self) -> Vec<u8> {
		match self {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
		}
	}
}

impl fungibles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

#[derive(Default)]
pub struct Config;
impl pallet_api::extension::Config for Config {
	/// Functions used by the Pop API.
	///
	/// Each function corresponds to specific functionality provided by the API, facilitating the
	/// interaction between smart contracts and the runtime.
	type Functions = (
		// Dispatching calls
		DispatchCall<
			Runtime,
			DecodesAs<VersionedRuntimeCall, DecodingFailedError, DispatchCallLogTarget>,
			IdentifiedByFirstByteOfFunctionId<ConstU8<0>>,
			Filter,
			DispatchCallLogTarget,
		>,
		// Reading state
		ReadState<
			Runtime,
			RuntimeRead,
			DecodesAs<VersionedRuntimeRead, DecodingFailedError, ReadStateLogTarget>,
			IdentifiedByFirstByteOfFunctionId<ConstU8<1>>,
			Filter,
			ReadStateLogTarget,
		>,
	);
	/// Ensure errors are versioned.
	type Error = VersionedErrorConverter;

	const LOG_TARGET: &'static str = LOG_TARGET;
}

mod filtering {
	use super::*;

	pub struct Filter;

	impl Contains<RuntimeCall> for Filter {
		fn contains(c: &RuntimeCall) -> bool {
			use fungibles::Call::*;
			matches!(
				c,
				RuntimeCall::Fungibles(
					transfer { .. }
						| transfer_from { .. } | approve { .. }
						| increase_allowance { .. }
						| decrease_allowance { .. }
						| create { .. } | set_metadata { .. }
						| start_destroy { .. } | clear_metadata { .. }
						| mint { .. } | burn { .. }
				)
			)
		}
	}

	impl Contains<RuntimeRead> for Filter {
		fn contains(r: &RuntimeRead) -> bool {
			use fungibles::Read::*;
			matches!(
				r,
				RuntimeRead::Fungibles(
					TotalSupply(..)
						| BalanceOf { .. } | Allowance { .. }
						| TokenName(..) | TokenSymbol(..)
						| TokenDecimals(..) | AssetExists(..)
				)
			)
		}
	}
}

mod versioning {
	use super::*;
	use codec::Encode;
	use pallet_contracts::chain_extension::{
		Environment, Ext, Result, RetVal, RetVal::Converging, State,
	};
	use pop_chain_extension::ErrorProvider;
	use sp_runtime::DispatchError;

	/// Versioned runtime calls.
	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeCall {
		/// Version zero of runtime calls.
		#[codec(index = 0)]
		V0(RuntimeCall),
	}

	impl From<VersionedRuntimeCall> for RuntimeCall {
		fn from(value: VersionedRuntimeCall) -> Self {
			// Allows mapping from some previous runtime call shape to a current valid runtime call
			match value {
				VersionedRuntimeCall::V0(call) => call,
			}
		}
	}

	/// Versioned runtime state reads.
	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeRead {
		/// Version zero of runtime state reads.
		#[codec(index = 0)]
		V0(RuntimeRead),
	}

	impl From<VersionedRuntimeRead> for RuntimeRead {
		fn from(value: VersionedRuntimeRead) -> Self {
			// Allows mapping from some previous runtime call shape to a current valid runtime read
			match value {
				VersionedRuntimeRead::V0(read) => read,
			}
		}
	}

	pub struct VersionedErrorConverter;
	impl ErrorConverter for VersionedErrorConverter {
		fn convert<E: Ext, S: State>(
			error: DispatchError,
			env: Environment<E, S>,
		) -> Result<RetVal> {
			Ok(Converging(error::convert_to_status_code(error, env.func_id().to_le_bytes()[0])))
		}
	}

	pub struct DecodingFailedError;
	impl ErrorProvider for DecodingFailedError {
		const ERROR: DispatchError = error::DECODING_FAILED_ERROR;
	}

	// TODO: refactor to move components to pallet-api
	mod error {
		use super::*;

		pub(crate) const DECODING_FAILED_ERROR: DispatchError =
			DispatchError::Other("DecodingFailed");
		// TODO: issue #93, we can also encode the `pop_primitives::Error::UnknownCall` which means we do use
		//  `Error` in the runtime and it should stay in primitives. Perhaps issue #91 will also influence
		//  here. Should be looked at together.
		const DECODING_FAILED_ERROR_ENCODED: [u8; 4] = [255u8, 0, 0, 0];
		pub(crate) const UNKNOWN_CALL_ERROR: DispatchError = DispatchError::Other("UnknownCall");
		// TODO: see above.
		const UNKNOWN_CALL_ERROR_ENCODED: [u8; 4] = [254u8, 0, 0, 0];

		/// Converts a `DispatchError` to a `u32` status code based on the version of the API the contract uses.
		/// The contract calling the chain extension can optionally convert the status code to the descriptive `Error`.
		///
		/// For `Error` see `pop_primitives::<version>::error::Error`.
		///
		/// The error encoding can vary per version, allowing for flexible and backward-compatible error handling.
		/// As a result, contracts maintain compatibility across different versions of the runtime.
		///
		/// # Parameters
		///
		/// - `error`: The `DispatchError` encountered during contract execution.
		/// - `version`: The version of the chain extension, used to determine the known errors.
		pub(crate) fn convert_to_status_code(error: DispatchError, version: u8) -> u32 {
			let mut encoded_error: [u8; 4] = match error {
				// "UnknownCall" and "DecodingFailed" are mapped to specific errors in the API and will
				// never change.
				UNKNOWN_CALL_ERROR => UNKNOWN_CALL_ERROR_ENCODED,
				DECODING_FAILED_ERROR => DECODING_FAILED_ERROR_ENCODED,
				_ => {
					let mut encoded_error = error.encode();
					// Resize the encoded value to 4 bytes in order to decode the value in a u32 (4 bytes).
					encoded_error.resize(4, 0);
					encoded_error.try_into().expect("qed, resized to 4 bytes line above")
				},
			};
			match version {
				// If an unknown variant of the `DispatchError` is detected the error needs to be converted
				// into the encoded value of `Error::Other`. This conversion is performed by shifting the bytes one
				// position forward (discarding the last byte as it is not used) and setting the first byte to the
				// encoded value of `Other` (0u8). This ensures the error is correctly categorized as an `Other`
				// variant which provides all the necessary information to debug which error occurred in the runtime.
				//
				// Byte layout explanation:
				// - Byte 0: index of the variant within `Error`
				// - Byte 1:
				//   - Must be zero for `UNIT_ERRORS`.
				//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
				//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
				// - Byte 2:
				//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
				// - Byte 3:
				//   - Unused or represents further nested information.
				0 => v0::handle_unknown_error(&mut encoded_error),
				_ => encoded_error = UNKNOWN_CALL_ERROR_ENCODED,
			}
			u32::from_le_bytes(encoded_error)
		}

		mod v0 {
			#[cfg(test)]
			use super::convert_to_status_code;

			pub(crate) fn handle_unknown_error(encoded_error: &mut [u8; 4]) {
				let unknown = match encoded_error[0] {
					code if UNIT_ERRORS.contains(&code) => nested_errors(&encoded_error[1..], None),
					// Single nested errors with a limit in their nesting.
					//
					// `TokenError`: has ten variants - translated to a limit of nine.
					7 => nested_errors(&encoded_error[1..], Some(9)),
					// `ArithmeticError`: has 3 variants - translated to a limit of two.
					8 => nested_errors(&encoded_error[1..], Some(2)),
					// `TransactionalError`: has 2 variants - translated to a limit of one.
					9 => nested_errors(&encoded_error[1..], Some(1)),
					code if DOUBLE_NESTED_ERRORS.contains(&code) => {
						nested_errors(&encoded_error[3..], None)
					},
					_ => true,
				};
				if unknown {
					encoded_error[..].rotate_right(1);
					encoded_error[0] = 0u8;
				}
			}

			// Unit `Error` variants.
			// (variant: index):
			// - CannotLookup: 1,
			// - BadOrigin: 2,
			// - ConsumerRemaining: 4,
			// - NoProviders: 5,
			// - TooManyConsumers: 6,
			// - Exhausted: 10,
			// - Corruption: 11,
			// - Unavailable: 12,
			// - RootNotAllowed: 13,
			// - UnknownFunctionId: 254,
			// - DecodingFailed: 255,
			const UNIT_ERRORS: [u8; 11] = [1, 2, 4, 5, 6, 10, 11, 12, 13, 254, 255];

			#[cfg(test)]
			const SINGLE_NESTED_ERRORS: [u8; 3] = [7, 8, 9];

			// Double nested `Error` variants
			// (variant: index):
			// - Module: 3,
			const DOUBLE_NESTED_ERRORS: [u8; 1] = [3];

			// Checks for unknown nested errors within the `DispatchError`.
			// - For single nested errors with a limit, it verifies if the nested value exceeds the limit.
			// - For other nested errors, it checks if any subsequent bytes are non-zero.
			//
			// `nested_error` - The slice of bytes representing the nested error.
			// `limit` - An optional limit for single nested errors.
			fn nested_errors(nested_error: &[u8], limit: Option<u8>) -> bool {
				match limit {
					Some(l) => nested_error[0] > l || nested_error[1..].iter().any(|&x| x != 0u8),
					None => nested_error.iter().any(|&x| x != 0u8),
				}
			}

			#[cfg(test)]
			mod tests {
				use super::*;
				use pop_primitives::error::{
					ArithmeticError::*,
					Error::{self, *},
					TokenError::*,
					TransactionalError::*,
				};
				use sp_runtime::DispatchError;

				// Compare all the different `DispatchError` variants with the expected `Error`.
				#[test]
				fn dispatch_error_to_error() {
					let test_cases = vec![
						(
							DispatchError::Other(""),
							(Other { dispatch_error_index: 0, error_index: 0, error: 0 }),
						),
						(DispatchError::Other("UnknownCall"), UnknownCall),
						(DispatchError::Other("DecodingFailed"), DecodingFailed),
						(DispatchError::CannotLookup, CannotLookup),
						(DispatchError::BadOrigin, BadOrigin),
						(
							DispatchError::Module(sp_runtime::ModuleError {
								index: 1,
								error: [2, 0, 0, 0],
								message: Some("hallo"),
							}),
							Module { index: 1, error: 2 },
						),
						(DispatchError::ConsumerRemaining, ConsumerRemaining),
						(DispatchError::NoProviders, NoProviders),
						(DispatchError::TooManyConsumers, TooManyConsumers),
						(
							DispatchError::Token(sp_runtime::TokenError::BelowMinimum),
							Token(BelowMinimum),
						),
						(
							DispatchError::Arithmetic(sp_runtime::ArithmeticError::Overflow),
							Arithmetic(Overflow),
						),
						(
							DispatchError::Transactional(
								sp_runtime::TransactionalError::LimitReached,
							),
							Transactional(LimitReached),
						),
						(DispatchError::Exhausted, Exhausted),
						(DispatchError::Corruption, Corruption),
						(DispatchError::Unavailable, Unavailable),
						(DispatchError::RootNotAllowed, RootNotAllowed),
					];
					for (dispatch_error, expected) in test_cases {
						let status_code = convert_to_status_code(dispatch_error, 0);
						let error: Error = status_code.into();
						assert_eq!(error, expected);
					}
				}

				// Compare all the different `DispatchError::Other` possibilities with the expected `Error`.
				#[test]
				fn other_error() {
					let test_cases = vec![
						(
							DispatchError::Other("Random"),
							(Other { dispatch_error_index: 0, error_index: 0, error: 0 }),
						),
						(DispatchError::Other("UnknownCall"), UnknownCall),
						(DispatchError::Other("DecodingFailed"), DecodingFailed),
					];
					for (dispatch_error, expected) in test_cases {
						let status_code = convert_to_status_code(dispatch_error, 0);
						let error: Error = status_code.into();
						assert_eq!(error, expected);
					}
				}

				// Compare all the different `DispatchError::Module` nesting possibilities, which can not be
				// handled, with the expected `Error`. See `double_nested_error_variants` fourth byte nesting.
				#[test]
				fn test_module_error() {
					let test_cases = vec![
						DispatchError::Module(sp_runtime::ModuleError {
							index: 1,
							error: [2, 2, 0, 0],
							message: Some("Random"),
						}),
						DispatchError::Module(sp_runtime::ModuleError {
							index: 1,
							error: [2, 2, 2, 0],
							message: Some("Random"),
						}),
						DispatchError::Module(sp_runtime::ModuleError {
							index: 1,
							error: [2, 2, 2, 2],
							message: Some("Random"),
						}),
					];
					for dispatch_error in test_cases {
						let status_code = convert_to_status_code(dispatch_error, 0);
						let error: Error = status_code.into();
						assert_eq!(
							error,
							Other { dispatch_error_index: 3, error_index: 1, error: 2 }
						);
					}
				}

				// Converts 4 bytes into `Error` and handles unknown errors (used in `convert_to_status_code`).
				fn into_error(mut error_bytes: [u8; 4]) -> Error {
					handle_unknown_error(&mut error_bytes);
					u32::from_le_bytes(error_bytes).into()
				}

				// Tests the `handle_unknown_error` for `Error`, version 0.
				//
				// Unit variants:
				// If the encoded value indicates a nested `Error` which is known by V0 as a
				// unit variant, the encoded value is converted into `Error::Other`.
				//
				// Example: the error `BadOrigin` (encoded: `[2, 0, 0, 0]`) with a non-zero value for one
				// of the bytes [1..4]: `[2, 0, 1, 0]` is converted into `[0, 2, 0, 1]` (shifting the bits
				// one forward). This is decoded to `Error::Other { dispatch_error: 2, index: 0, error: 1 }`.
				#[test]
				fn unit_error_variants() {
					let errors = vec![
						CannotLookup,
						BadOrigin,
						ConsumerRemaining,
						NoProviders,
						TooManyConsumers,
						Exhausted,
						Corruption,
						Unavailable,
						RootNotAllowed,
						UnknownCall,
						DecodingFailed,
					];
					// Compare an `Error`, which is converted from an encoded value, with the expected `Error`.
					for (i, &error_code) in UNIT_ERRORS.iter().enumerate() {
						// No nesting and unit variant correctly returned.
						assert_eq!(into_error([error_code, 0, 0, 0]), errors[i]);
						// Unexpected second byte nested.
						assert_eq!(
							into_error([error_code, 1, 0, 0]),
							(Other { dispatch_error_index: error_code, error_index: 1, error: 0 }),
						);
						// Unexpected third byte nested.
						assert_eq!(
							into_error([error_code, 1, 1, 0]),
							(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }),
						);
						// Unexpected fourth byte nested.
						assert_eq!(
							into_error([error_code, 1, 1, 1]),
							(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }),
						);
					}
				}

				// Single nested variants:
				// If the encoded value indicates a double nested `Error` which is known by V0
				// as a single nested variant, the encoded value is converted into `Error::Other`.
				//
				// Example: the error `Arithmetic(Overflow)` (encoded: `[8, 1, 0, 0]`) with a non-zero
				// value for one of the bytes [2..4]: `[8, 1, 1, 0]` is converted into `[0, 8, 1, 1]`. This is
				// decoded to `Error::Other { dispatch_error: 8, index: 1,  error: 1 }`.
				#[test]
				fn single_nested_error_variants() {
					let errors = vec![
						[Token(FundsUnavailable), Token(OnlyProvider)],
						[Arithmetic(Underflow), Arithmetic(Overflow)],
						[Transactional(LimitReached), Transactional(NoLayer)],
					];
					// Compare an `Error`, which is converted from an encoded value, with the expected `Error`.
					for (i, &error_code) in SINGLE_NESTED_ERRORS.iter().enumerate() {
						// No nested and single nested variant correctly returned.
						assert_eq!(into_error([error_code, 0, 0, 0]), errors[i][0]);
						assert_eq!(into_error([error_code, 1, 0, 0]), errors[i][1]);
						// Unexpected third byte nested.
						assert_eq!(
							into_error([error_code, 1, 1, 0]),
							(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }),
						);
						// Unexpected fourth byte nested.
						assert_eq!(
							into_error([error_code, 1, 1, 1]),
							Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
						);
					}
				}

				#[test]
				fn single_nested_unknown_variants() {
					// Unknown `TokenError` variant.
					assert_eq!(
						into_error([7, 10, 0, 0]),
						Other { dispatch_error_index: 7, error_index: 10, error: 0 }
					);
					// Unknown `Arithmetic` variant.
					assert_eq!(
						into_error([8, 3, 0, 0]),
						Other { dispatch_error_index: 8, error_index: 3, error: 0 }
					);
					// Unknown `Transactional` variant.
					assert_eq!(
						into_error([9, 2, 0, 0]),
						Other { dispatch_error_index: 9, error_index: 2, error: 0 }
					);
				}

				// Double nested variants:
				// If the encoded value indicates a triple nested `Error` which is known by V0
				// as a double nested variant, the encoded value is converted into `Error::Other`.
				//
				// Example: the error `Module { index: 10, error 5 }` (encoded: `[3, 10, 5, 0]`) with a non-zero
				// value for the last byte: `[3, 10, 5, 3]` is converted into `[0, 3, 10, 5]`. This is
				// decoded to `Error::Other { dispatch_error: 3, index: 10,  error: 5 }`.
				#[test]
				fn double_nested_error_variants() {
					// Compare an `Error`, which is converted from an encoded value, with the expected `Error`.
					// No nesting and unit variant correctly returned.
					assert_eq!(into_error([3, 0, 0, 0]), Module { index: 0, error: 0 });
					// Allowed single nesting and variant correctly returned.
					assert_eq!(into_error([3, 1, 0, 0]), Module { index: 1, error: 0 });
					// Allowed double nesting and variant correctly returned.
					assert_eq!(into_error([3, 1, 1, 0]), Module { index: 1, error: 1 });
					// Unexpected fourth byte nested.
					assert_eq!(
						into_error([3, 1, 1, 1]),
						Other { dispatch_error_index: 3, error_index: 1, error: 1 },
					);
				}

				#[test]
				fn test_random_encoded_values() {
					assert_eq!(
						into_error([100, 100, 100, 100]),
						Other { dispatch_error_index: 100, error_index: 100, error: 100 }
					);
					assert_eq!(
						into_error([200, 200, 200, 200]),
						Other { dispatch_error_index: 200, error_index: 200, error: 200 }
					);
				}
			}
		}
	}
}
