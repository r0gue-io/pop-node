use crate::AccountIdOf;
use core::fmt::Debug;
use frame_support::pallet_prelude::Weight;
use pallet_contracts::chain_extension::{BufInBufOutState, ChargedAmount, Result, State};
use sp_std::vec::Vec;

/// Provides access to the parameters passed to a chain extension and its execution environment.
///
/// A wrapper trait for `pallet_contracts::chain_extension::Environment`. All comments have been
/// copied solely for consistent developer experience in line with the wrapped type.
pub trait Environment {
	/// The account identifier type for the runtime.
	type AccountId;
	/// The charged weight type.
	type ChargedAmount: Debug;

	/// The function id within the `id` passed by a contract.
	///
	/// It returns the two least significant bytes of the `id` passed by a contract as the other
	/// two bytes represent the chain extension itself (the code which is calling this function).
	fn func_id(&self) -> u16;

	/// The chain extension id within the `id` passed by a contract.
	///
	/// It returns the two most significant bytes of the `id` passed by a contract which represent
	/// the chain extension itself (the code which is calling this function).
	fn ext_id(&self) -> u16;

	/// Charge the passed `amount` of weight from the overall limit.
	///
	/// It returns `Ok` when there the remaining weight budget is larger than the passed
	/// `weight`. It returns `Err` otherwise. In this case the chain extension should
	/// abort the execution and pass through the error.
	///
	/// The returned value can be used to with [`Self::adjust_weight`]. Other than that
	/// it has no purpose.
	///
	/// # Note
	///
	/// Weight is synonymous with gas in substrate.
	fn charge_weight(&mut self, amount: Weight) -> Result<Self::ChargedAmount>;

	/// Adjust a previously charged amount down to its actual amount.
	///
	/// This is when a maximum a priori amount was charged and then should be partially
	/// refunded to match the actual amount.
	fn adjust_weight(&mut self, charged: Self::ChargedAmount, actual_weight: Weight);

	/// Grants access to the execution environment of the current contract call.
	///
	/// Consult the functions on the returned type before re-implementing those functions.
	// TODO: improve the return type to &mut
	fn ext(&mut self) -> impl Ext<AccountId = Self::AccountId>;
}

/// A wrapper type for `pallet_contracts::chain_extension::Environment`.
pub(crate) struct Env<'a, 'b, E: pallet_contracts::chain_extension::Ext, S: State>(
	pub(crate) pallet_contracts::chain_extension::Environment<'a, 'b, E, S>,
);

impl<'a, 'b, E: pallet_contracts::chain_extension::Ext, S: State> Environment
	for Env<'a, 'b, E, S>
{
	type AccountId = AccountIdOf<E::T>;
	type ChargedAmount = ChargedAmount;

	fn func_id(&self) -> u16 {
		self.0.func_id()
	}

	fn ext_id(&self) -> u16 {
		self.0.ext_id()
	}

	fn charge_weight(&mut self, amount: Weight) -> Result<Self::ChargedAmount> {
		self.0.charge_weight(amount)
	}

	fn adjust_weight(&mut self, charged: Self::ChargedAmount, actual_weight: Weight) {
		self.0.adjust_weight(charged, actual_weight)
	}

	fn ext(&mut self) -> impl Ext<AccountId = Self::AccountId> {
		ExternalEnvironment(self.0.ext())
	}
}

/// A state that uses a buffer as input.
///
/// A wrapper trait for `pallet_contracts::chain_extension::BufIn` related function available on
/// `pallet_contracts::chain_extension::Environment`. All comments have been copied solely for
/// consistent developer experience in line with the wrapped type.
pub trait BufIn {
	/// The length of the input as passed in as `input_len`.
	///
	/// A chain extension would use this value to calculate the dynamic part of its
	/// weight. For example a chain extension that calculates the hash of some passed in
	/// bytes would use `in_len` to charge the costs of hashing that amount of bytes.
	/// This also subsumes the act of copying those bytes as a benchmarks measures both.
	fn in_len(&self) -> u32;
	/// Reads `min(max_len, in_len)` from contract memory.
	///
	/// This does **not** charge any weight. The caller must make sure that the an
	/// appropriate amount of weight is charged **before** reading from contract memory.
	/// The reason for that is that usually the costs for reading data and processing
	/// said data cannot be separated in a benchmark. Therefore a chain extension would
	/// charge the overall costs either using `max_len` (worst case approximation) or using
	/// [`in_len()`](Self::in_len).
	fn read(&self, max_len: u32) -> Result<Vec<u8>>;
}

impl<'a, 'b, E: pallet_contracts::chain_extension::Ext> BufIn for Env<'a, 'b, E, BufInBufOutState> {
	fn in_len(&self) -> u32 {
		self.0.in_len()
	}

	fn read(&self, max_len: u32) -> Result<Vec<u8>> {
		self.0.read(max_len)
	}
}

/// A state that uses a buffer as output.
///
/// A wrapper trait for `pallet_contracts::chain_extension::BufOut` related function available on
/// `pallet_contracts::chain_extension::Environment`. All comments have been copied solely for
/// consistent developer experience in line with the wrapped type.
pub trait BufOut {
	/// Write the supplied buffer to contract memory.
	///
	/// If the contract supplied buffer is smaller than the passed `buffer` an `Err` is returned.
	/// If `allow_skip` is set to true the contract is allowed to skip the copying of the buffer
	/// by supplying the guard value of `pallet-contracts::SENTINEL` as `out_ptr`. The
	/// `weight_per_byte` is only charged when the write actually happens and is not skipped or
	/// failed due to a too small output buffer.
	fn write(
		&mut self,
		buffer: &[u8],
		allow_skip: bool,
		weight_per_byte: Option<Weight>,
	) -> Result<()>;
}

impl<'a, 'b, E: pallet_contracts::chain_extension::Ext> BufOut
	for Env<'a, 'b, E, BufInBufOutState>
{
	fn write(
		&mut self,
		buffer: &[u8],
		allow_skip: bool,
		weight_per_byte: Option<Weight>,
	) -> Result<()> {
		self.0.write(buffer, allow_skip, weight_per_byte)
	}
}

/// An interface that provides access to the external environment in which the smart-contract is
/// executed.
///
/// This interface is specialized to an account of the executing code, so all operations are
/// implicitly performed on that account.
///
/// A wrapper trait for `pallet_contracts::chain_extension::Ext`. All comments have been copied
/// solely for consistent developer experience in line with the wrapped type.
pub trait Ext {
	/// The account identifier type for the runtime.
	type AccountId;

	/// Returns a reference to the account id of the current contract.
	fn address(&self) -> &Self::AccountId;
}

impl Ext for () {
	type AccountId = ();

	fn address(&self) -> &Self::AccountId {
		&()
	}
}

/// A wrapper type for a type implementing `pallet_contracts::chain_extension::Ext`.
pub(crate) struct ExternalEnvironment<'a, T: pallet_contracts::chain_extension::Ext>(&'a mut T);

impl<'a, E: pallet_contracts::chain_extension::Ext> Ext for ExternalEnvironment<'a, E> {
	type AccountId = AccountIdOf<E::T>;
	fn address(&self) -> &Self::AccountId {
		self.0.address()
	}
}

#[test]
fn default_ext_works() {
	assert_eq!(().address(), &())
}