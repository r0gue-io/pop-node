use crate::StatusCode;
use pop_primitives::error::Error;

#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "balances")]
pub mod balances;
#[cfg(feature = "cross-chain")]
pub mod cross_chain;
#[cfg(feature = "nfts")]
pub mod nfts;

pub(crate) const VERSION: u8 = 0;

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}

impl From<Error> for StatusCode {
	fn from(value: Error) -> Self {
		StatusCode::from(u32::from(value))
	}
}
