pub use ink::{abi::Sol, primitives::sol::SolErrorDecode, SolEncode};

/// Reverts the current contract execution, rolling back any changes and returning the specified
/// `error`.
// Helper until Solidity support added for Rust errors for automatic reversion based on returning an
// error.
pub fn revert(error: &impl for<'a> SolEncode<'a>) -> ! {
	use ink::env::{return_value_solidity, ReturnFlags};
	return_value_solidity(ReturnFlags::REVERT, error)
}

// Decoding of an error from a precompile, where custom errors have been base64 encoded.
pub(crate) trait PrecompileError: Sized {
	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error>;
}

#[macro_export]
macro_rules! impl_sol_encoding_for_precompile {
    ($($type:ty),*) => {
        $(
            impl ink::sol::SolErrorDecode for  $type {
               	fn decode(data: &[u8]) -> Result<Self, ink::sol::Error> {
                    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
                    use ink::{prelude::string::String, sol_error_selector};
                    use crate::sol::PrecompileError;

                    // Check if `Error(string)`
                    pub(crate) const ERROR: [u8; 4] = sol_error_selector!("Error", (String,));
              		if data.len() < 4 || data[..4] != ERROR {
             			return <Self as PrecompileError>::decode(data);
              		}

                    // Decode as `Error(string)`, then via `base64::decode` and finally decode into `Error`
                    #[derive(ink::SolErrorDecode)]
                    pub(crate) struct Error(pub(crate) String);
              		let error = Error::decode(data)?;
              		let data = BASE64.decode(error.0).map_err(|_| ink::sol::Error)?;
              		return <Self as PrecompileError>::decode(data.as_slice());
               	}
            }

            impl<'a> ink::SolEncode<'a> for $type {
               	type SolType = ();

               	fn encode(&'a self) -> Vec<u8> {
              		ink::primitives::sol::SolErrorEncode::encode(self)
               	}

               	fn to_sol_type(&'a self) -> Self::SolType {
                    ()
                }
            }
        )*
    };
}
