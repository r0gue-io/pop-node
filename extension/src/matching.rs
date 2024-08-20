use super::*;

/// Trait for matching a function.
pub trait Matches {
	/// Determines whether a function is a match.
	///
	/// # Parameters
	/// - `ext_id` - The specified chain extension identifier.
	/// - `func_id` - The specified function identifier.
	fn matches(ext_id: u16, func_id: u16) -> bool;
}

/// Matches on an extension and function identifier.
pub struct Equals<E, F>(PhantomData<(E, F)>);
impl<E: Get<u16>, F: Get<u16>> Matches for Equals<E, F> {
	fn matches(ext_id: u16, func_id: u16) -> bool {
		ext_id == E::get() && func_id == F::get()
	}
}

/// Matches on a function identifier only.
pub struct FunctionId<T>(PhantomData<T>);
impl<T: Get<u16>> Matches for FunctionId<T> {
	fn matches(_ext_id: u16, func_id: u16) -> bool {
		func_id == T::get()
	}
}
