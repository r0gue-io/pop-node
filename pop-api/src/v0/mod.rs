use crate::{primitives::error::Error, StatusCode};

#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "balances")]
pub mod balances;
#[cfg(feature = "cross-chain")]
pub mod cross_chain;
#[cfg(feature = "nfts")]
pub mod nfts;

pub(crate) const V0: u8 = 0;

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}
