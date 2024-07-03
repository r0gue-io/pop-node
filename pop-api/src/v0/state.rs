use crate::{error::StatusCode, primitives::storage_keys::RuntimeStateKeys, read_state};
use ink::scale::Decode;

#[inline]
pub fn read<T: Decode>(key: RuntimeStateKeys) -> crate::Result<T> {
	read_state(key).and_then(|v| T::decode(&mut &v[..]).map_err(|_e| StatusCode(255u32)))
}
