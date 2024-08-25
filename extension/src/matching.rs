use super::*;

/// Trait for matching a function.
pub trait Matches {
	/// Determines whether a function is a match.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn matches<E: Ext, S: State>(env: &Environment<E, S>) -> bool;
}

/// Matches on an extension and function identifier.
pub struct Equals<E, F>(PhantomData<(E, F)>);
impl<ExtId: Get<u16>, FuncId: Get<u16>> Matches for Equals<ExtId, FuncId> {
	fn matches<E: Ext, S: State>(env: &Environment<E, S>) -> bool {
		env.ext_id() == ExtId::get() && env.func_id() == FuncId::get()
	}
}

/// Matches on a function identifier only.
pub struct FunctionId<T>(PhantomData<T>);
impl<T: Get<u16>> Matches for FunctionId<T> {
	fn matches<E: Ext, S: State>(env: &Environment<E, S>) -> bool {
		env.func_id() == T::get()
	}
}

/// Matches on a function identifier only.
pub struct WithFuncId<T>(PhantomData<T>);
impl<T: Get<u32>> Matches for WithFuncId<T> {
	fn matches<E: Ext, S: State>(env: &Environment<E, S>) -> bool {
		let ext_id: [u8; 2] = env.ext_id().to_le_bytes();
		let func_id: [u8; 2] = env.func_id().to_le_bytes();
		u32::from_le_bytes([func_id[0], func_id[1], ext_id[0], ext_id[1]]) == T::get()
	}
}
