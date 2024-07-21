use crate::{primitives::error::Error, StatusCode};

pub mod assets;

pub(crate) const V0: u8 = 0;

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}
