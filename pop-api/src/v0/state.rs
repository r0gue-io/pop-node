use crate::{primitives::storage_keys::RuntimeStateKeys, read_state, PopApiError};
use scale::Decode;

pub fn read<T: Decode>(key: RuntimeStateKeys) -> crate::Result<T> {
	read_state(key).and_then(|v| T::decode(&mut &v[..]).map_err(|_e| PopApiError::DecodingFailed))
}
