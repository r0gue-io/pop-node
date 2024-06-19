use crate::PopApiError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	/// Invalid schedule supplied, e.g. with zero weight of a basic operation.
	InvalidSchedule,
	/// Invalid combination of flags supplied to `seal_call` or `seal_delegate_call`.
	InvalidCallFlags,
	/// The executed contract exhausted its gas limit.
	OutOfGas,
	/// The output buffer supplied to a contract API call was too small.
	OutputBufferTooSmall,
	/// Performing the requested transfer failed. Probably because there isn't enough
	/// free balance in the sender's account.
	TransferFailed,
	/// Performing a call was denied because the calling depth reached the limit
	/// of what is specified in the schedule.
	MaxCallDepthReached,
	/// No contract was found at the specified address.
	ContractNotFound,
	/// The code supplied to `instantiate_with_code` exceeds the limit specified in the
	/// current schedule.
	CodeTooLarge,
	/// No code could be found at the supplied code hash.
	CodeNotFound,
	/// No code info could be found at the supplied code hash.
	CodeInfoNotFound,
	/// A buffer outside of sandbox memory was passed to a contract API function.
	OutOfBounds,
	/// Input passed to a contract API function failed to decode as expected type.
	//  TODO: Something is wrong with how the call is encoded, check parameters.
	DecodingFailed,
	/// Contract trapped during execution.
	ContractTrapped,
	/// The size defined in `T::MaxValueSize` was exceeded.
	ValueTooLarge,
	/// Termination of a contract is not allowed while the contract is already
	/// on the call stack. Can be triggered by `seal_terminate`.
	TerminatedWhileReentrant,
	/// `seal_call` forwarded this contracts input. It therefore is no longer available.
	InputForwarded,
	/// The subject passed to `seal_random` exceeds the limit.
	RandomSubjectTooLong,
	/// The amount of topics passed to `seal_deposit_events` exceeds the limit.
	TooManyTopics,
	/// The chain does not provide a chain extension. Calling the chain extension results
	/// in this error. Note that this usually  shouldn't happen as deploying such contracts
	/// is rejected.
	NoChainExtension,
	/// Failed to decode the XCM program.
	XCMDecodeFailed,
	/// A contract with the same AccountId already exists.
	DuplicateContract,
	/// A contract self destructed in its constructor.
	///
	/// This can be triggered by a call to `seal_terminate`.
	TerminatedInConstructor,
	/// A call tried to invoke a contract that is flagged as non-reentrant.
	/// The only other cause is that a call from a contract into the runtime tried to call back
	/// into `pallet-contracts`. This would make the whole pallet reentrant with regard to
	/// contract code execution which is not supported.
	ReentranceDenied,
	/// Origin doesn't have enough balance to pay the required storage deposits.
	StorageDepositNotEnoughFunds,
	/// More storage was created than allowed by the storage deposit limit.
	StorageDepositLimitExhausted,
	/// Code removal was denied because the code is still in use by at least one contract.
	CodeInUse,
	/// The contract ran to completion but decided to revert its storage changes.
	/// Please note that this error is only returned from extrinsics. When called directly
	/// or via RPC an `Ok` will be returned. In this case the caller needs to inspect the flags
	/// to determine whether a reversion has taken place.
	ContractReverted,
	/// The contract's code was found to be invalid during validation.
	///
	/// The most likely cause of this is that an API was used which is not supported by the
	/// node. This happens if an older node is used with a new version of ink!. Try updating
	/// your node to the newest available version.
	///
	/// A more detailed error can be found on the node console if debug messages are enabled
	/// by supplying `-lruntime::contracts=debug`.
	CodeRejected,
	/// An indeterministic code was used in a context where this is not permitted.
	Indeterministic,
	/// A pending migration needs to complete before the extrinsic can be called.
	MigrationInProgress,
	/// Migrate dispatch call was attempted but no migration was performed.
	NoMigrationPerformed,
	/// The contract has reached its maximum number of delegate dependencies.
	MaxDelegateDependenciesReached,
	/// The dependency was not found in the contract's delegate dependencies.
	DelegateDependencyNotFound,
	/// The contract already depends on the given delegate dependency.
	DelegateDependencyAlreadyExists,
	/// Can not add a delegate dependency to the code hash of the contract itself.
	CannotAddSelfAsDelegateDependency,
}

impl TryFrom<u32> for Error {
	type Error = PopApiError;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use Error::*;
		match status_code {
			0 => Ok(InvalidSchedule),
			1 => Ok(InvalidCallFlags),
			2 => Ok(OutOfGas),
			3 => Ok(OutputBufferTooSmall),
			4 => Ok(TransferFailed),
			5 => Ok(MaxCallDepthReached),
			6 => Ok(ContractNotFound),
			7 => Ok(CodeTooLarge),
			8 => Ok(CodeNotFound),
			9 => Ok(CodeInfoNotFound),
			10 => Ok(OutOfBounds),
			11 => Ok(DecodingFailed),
			12 => Ok(ContractTrapped),
			13 => Ok(ValueTooLarge),
			14 => Ok(TerminatedWhileReentrant),
			15 => Ok(InputForwarded),
			16 => Ok(RandomSubjectTooLong),
			17 => Ok(TooManyTopics),
			18 => Ok(NoChainExtension),
			19 => Ok(XCMDecodeFailed),
			20 => Ok(DuplicateContract),
			21 => Ok(TerminatedInConstructor),
			22 => Ok(ReentranceDenied),
			23 => Ok(StorageDepositNotEnoughFunds),
			24 => Ok(StorageDepositLimitExhausted),
			25 => Ok(CodeInUse),
			26 => Ok(ContractReverted),
			27 => Ok(CodeRejected),
			28 => Ok(Indeterministic),
			29 => Ok(MigrationInProgress),
			30 => Ok(NoMigrationPerformed),
			31 => Ok(MaxDelegateDependenciesReached),
			32 => Ok(DelegateDependencyNotFound),
			33 => Ok(DelegateDependencyAlreadyExists),
			34 => Ok(CannotAddSelfAsDelegateDependency),
			_ => todo!(),
		}
	}
}

// impl From<PopApiError> for Error {
// 	fn from(error: PopApiError) -> Self {
// 		match error {
// 			PopApiError::Contracts(e) => e,
// 			_ => panic!("expected balances error"),
// 		}
// 	}
// }
