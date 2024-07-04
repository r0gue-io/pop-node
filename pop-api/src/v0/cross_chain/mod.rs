use crate::{BlockNumber, ParachainSystemKeys, Result, RuntimeStateKeys};

pub mod coretime;

pub fn relay_chain_block_number() -> Result<BlockNumber> {
	crate::v0::state::read(RuntimeStateKeys::ParachainSystem(
		ParachainSystemKeys::LastRelayChainBlockNumber,
	))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CrossChainError {
	/// The desired destination was unreachable, generally because there is a no way of routing
	/// to it.
	Unreachable,
	/// There was some other issue (i.e. not to do with routing) in sending the message.
	/// Perhaps a lack of space for buffering the message.
	SendFailure,
	/// The message execution fails the filter.
	Filtered,
	/// The message's weight could not be determined.
	UnweighableMessage,
	/// The destination `Location` provided cannot be inverted.
	DestinationNotInvertible,
	/// The assets to be sent are empty.
	Empty,
	/// Could not re-anchor the assets to declare the fees for the destination chain.
	CannotReanchor,
	/// Too many assets have been attempted for transfer.
	TooManyAssets,
	/// Origin is invalid for sending.
	InvalidOrigin,
	/// The version of the `Versioned` value used is not able to be interpreted.
	BadVersion,
	/// The given location could not be used (e.g. because it cannot be expressed in the
	/// desired version of XCM).
	BadLocation,
	/// The referenced subscription could not be found.
	NoSubscription,
	/// The location is invalid since it already has a subscription from us.
	AlreadySubscribed,
	/// Could not check-out the assets for teleportation to the destination chain.
	CannotCheckOutTeleport,
	/// The owner does not own (all) of the asset that they wish to do the operation on.
	LowBalance,
	/// The asset owner has too many locks on the asset.
	TooManyLocks,
	/// The given account is not an identifiable sovereign account for any location.
	AccountNotSovereign,
	/// The operation required fees to be paid which the initiator could not meet.
	FeesNotMet,
	/// A remote lock with the corresponding data could not be found.
	LockNotFound,
	/// The unlock operation cannot succeed because there are still consumers of the lock.
	InUse,
	/// Invalid non-concrete asset.
	InvalidAssetNotConcrete,
	/// Invalid asset, reserve chain could not be determined for it.
	InvalidAssetUnknownReserve,
	/// Invalid asset, do not support remote asset reserves with different fees reserves.
	InvalidAssetUnsupportedReserve,
	/// Too many assets with different reserve locations have been attempted for transfer.
	TooManyReserves,
	/// Local XCM execution incomplete.
	LocalExecutionIncomplete,
}

impl TryFrom<u32> for CrossChainError {
	type Error = crate::error::Error;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use CrossChainError::*;
		match status_code {
			0 => Ok(Unreachable),
			1 => Ok(SendFailure),
			2 => Ok(Filtered),
			3 => Ok(UnweighableMessage),
			4 => Ok(DestinationNotInvertible),
			5 => Ok(Empty),
			6 => Ok(CannotReanchor),
			7 => Ok(TooManyAssets),
			8 => Ok(InvalidOrigin),
			9 => Ok(BadVersion),
			10 => Ok(BadLocation),
			11 => Ok(NoSubscription),
			12 => Ok(AlreadySubscribed),
			13 => Ok(CannotCheckOutTeleport),
			14 => Ok(LowBalance),
			15 => Ok(TooManyLocks),
			16 => Ok(AccountNotSovereign),
			17 => Ok(FeesNotMet),
			18 => Ok(LockNotFound),
			19 => Ok(InUse),
			20 => Ok(InvalidAssetNotConcrete),
			21 => Ok(InvalidAssetUnknownReserve),
			22 => Ok(InvalidAssetUnsupportedReserve),
			23 => Ok(TooManyReserves),
			_ => todo!(),
		}
	}
}
