use super::*;

/// Trait for matching a function.
pub trait Matches {
	/// Determines whether a function is a match.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn matches(env: &impl Environment) -> bool;
}

/// Matches on an extension and function identifier.
pub struct Equals<E, F>(PhantomData<(E, F)>);
impl<ExtId: Get<u16>, FuncId: Get<u16>> Matches for Equals<ExtId, FuncId> {
	fn matches(env: &impl Environment) -> bool {
		env.ext_id() == ExtId::get() && env.func_id() == FuncId::get()
	}
}

/// Matches on a function identifier only.
pub struct FunctionId<T>(PhantomData<T>);
impl<T: Get<u16>> Matches for FunctionId<T> {
	fn matches(env: &impl Environment) -> bool {
		env.func_id() == T::get()
	}
}

/// Matches on an extension and function identifier together.
pub struct WithFuncId<T>(PhantomData<T>);
impl<T: Get<u32>> Matches for WithFuncId<T> {
	fn matches(env: &impl Environment) -> bool {
		let ext_id: [u8; 2] = env.ext_id().to_le_bytes();
		let func_id: [u8; 2] = env.func_id().to_le_bytes();
		u32::from_le_bytes([func_id[0], func_id[1], ext_id[0], ext_id[1]]) == T::get()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{MockEnvironment, MockExt};
	use sp_core::{ConstU16, ConstU32};

	#[test]
	fn matching_equals_works() {
		let env =
			MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 2]), vec![], MockExt::default());
		assert!(Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));
	}

	#[test]
	fn matching_equals_invalid() {
		let env =
			MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 3]), vec![], MockExt::default());
		assert!(!Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));
	}

	#[test]
	fn matching_function_id_works() {
		let env =
			MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 2]), vec![], MockExt::default());
		assert!(FunctionId::<ConstU16<2>>::matches(&env));
	}

	#[test]
	fn matching_function_id_invalid() {
		let env =
			MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 3]), vec![], MockExt::default());
		assert!(!FunctionId::<ConstU16<2>>::matches(&env));
	}

	#[test]
	fn matching_with_func_id_works() {
		let env = MockEnvironment::default();
		assert!(WithFuncId::<ConstU32<0>>::matches(&env));
	}

	#[test]
	fn matching_with_func_id_invalid() {
		let env = MockEnvironment::new(1, vec![], MockExt::default());
		assert!(!WithFuncId::<ConstU32<0>>::matches(&env));
	}
}
