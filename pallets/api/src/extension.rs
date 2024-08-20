use core::marker::PhantomData;
use frame_support::traits::Get;
pub use pop_chain_extension::{Config, DispatchCall, ReadState, Readable};
use pop_chain_extension::{Decodes, Environment, Ext, Matches, Processor, State};

pub type Extension<Functions> = pop_chain_extension::Extension<Functions>;

/// Decodes output by prepending bytes from ext_id() + func_id()
pub type DecodesAs<Output> = Decodes<Output, Prepender>;

/// Prepends bytes from ext_id() + func_id() to prefix the encoded input bytes to determine the versioned output
pub struct Prepender;
impl Processor for Prepender {
	fn process<E: Ext, S: State>(value: &mut Vec<u8>, env: &mut Environment<E, S>) {
		// TODO: revisit the ordering based on specced standard
		// Resolve version, pallet and call index from environment
		let version = env.func_id().to_le_bytes()[0];
		let (pallet_index, call_index) = {
			let bytes = env.ext_id().to_le_bytes();
			(bytes[0], bytes[1])
		};
		// Prepend bytes
		value.insert(0, version);
		value.insert(1, pallet_index);
		value.insert(2, call_index);
	}
}

/// Matches on the first byte of a function identifier only.
pub struct IdentifiedByFirstByteOfFunctionId<T>(PhantomData<T>);
impl<T: Get<u8>> Matches for IdentifiedByFirstByteOfFunctionId<T> {
	fn matches(_ext_id: u16, func_id: u16) -> bool {
		let bytes = func_id.to_le_bytes();
		bytes[1] == T::get()
	}
}
