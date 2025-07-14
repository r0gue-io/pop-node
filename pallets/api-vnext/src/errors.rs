/// Evaluate an expression, assert it returns an expected `Error::Revert` value and that
/// runtime storage has not been mutated (i.e. expression is a no-operation).
///
/// Used as `assert_revert!(expression_to_assert, expected_error_expression)`.
#[cfg(test)]
macro_rules! assert_revert {
	($x:expr, $e:expr $(,)?) => {{
		use pallet_revive::precompiles::{alloy::sol_types::Revert, Error};

		// Use function to resolve type inference from expected error
		fn decode_revert<T: SolError>(revert: Revert, _expected: &T) -> T {
			use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
			let decoded = BASE64.decode(revert.reason).expect("base64 decoding error");
			T::abi_decode(&decoded).expect("sol decoding error")
		}

		let Err(Error::Revert(revert)) = $x else { panic!("expected revert, got {:?}", $x) };
		assert_eq!(decode_revert(revert, &$e), $e);
	}};
}

/// Evaluate `$x:expr` and if not true return `Err(Error::Revert)` with the reason containing a
/// base64-encoding of the specified Solidity error.
///
/// Used as `ensure!(expression_to_ensure, error_to_return_on_false)`.
macro_rules! ensure {
	($x:expr, $e:expr $(,)?) => {{
		use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
		use pallet_revive::precompiles::{
			alloy::sol_types::{Revert, SolError},
			Error,
		};

		if !$x {
			let encoded = BASE64.encode(<_ as SolError>::abi_encode(&$e));
			frame_support::fail!(Error::Revert(Revert { reason: encoded }))
		}
	}};
}

/// Implement [`From`] for Solidity errors, which converts the error into a [`Revert`] error with
/// the reason containing a base64-encoding of the specified Solidity error.
macro_rules! impl_from_sol_error {
    ($($error_type:path),+ $(,)?) => {
        $(
            impl From<$error_type> for Error {
                fn from(error: $error_type) -> Self {
                    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
                    use pallet_revive::precompiles::{
             			alloy::sol_types::{Revert, SolError},
             			Error,
              		};

                    let reason = BASE64.encode(error.abi_encode().as_slice());
                    Error::Revert(Revert { reason })
                }
            }
        )+
    };
}
