use super::*;

/// Trait for matching a function.
pub trait Matches {
	/// Determines whether a function is a match.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn matches(env: &impl Environment) -> bool;
}

/// Matches an extension and function identifier.
pub struct Equals<E, F>(PhantomData<(E, F)>);
impl<ExtId: Get<u16>, FuncId: Get<u16>> Matches for Equals<ExtId, FuncId> {
	fn matches(env: &impl Environment) -> bool {
		env.ext_id() == ExtId::get() && env.func_id() == FuncId::get()
	}
}

/// Matches a function identifier only.
pub struct FunctionId<T>(PhantomData<T>);
impl<T: Get<u16>> Matches for FunctionId<T> {
	fn matches(env: &impl Environment) -> bool {
		env.func_id() == T::get()
	}
}

/// Matches a `u32` function identifier.
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
	use sp_core::{ConstU16, ConstU32};

	use super::*;
	use crate::mock::MockEnvironment;

	#[test]
	fn equals_matches() {
		let env = MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 2]), vec![]);
		assert!(Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));
	}

	#[test]
	fn equals_does_not_match() {
		// Fails due to the invalid function id.
		let env = MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 3]), vec![]);
		assert!(!Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));

		// Fails due to the invalid extension id.
		let env = MockEnvironment::new(u32::from_be_bytes([0u8, 2, 0, 2]), vec![]);
		assert!(!Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));
	}

	#[test]
	fn function_id_matches() {
		let env = MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 2]), vec![]);
		assert!(FunctionId::<ConstU16<2>>::matches(&env));
	}

	#[test]
	fn function_id_does_not_match() {
		let env = MockEnvironment::new(u32::from_be_bytes([0u8, 1, 0, 3]), vec![]);
		assert!(!FunctionId::<ConstU16<2>>::matches(&env));
	}

	#[test]
	fn func_id_matches() {
		let env = MockEnvironment::default();
		assert!(WithFuncId::<ConstU32<0>>::matches(&env));
	}

	#[test]
	fn func_id_does_not_match() {
		let env = MockEnvironment::new(1, vec![]);
		assert!(!WithFuncId::<ConstU32<0>>::matches(&env));
	}
}
