use alloc::vec;

use frame_support::{
	sp_runtime::{ArithmeticError, DispatchError, ModuleError},
	traits::PalletInfo,
};
pub(super) use pallet_revive::precompiles::{
	alloy::{primitives::U256, sol_types::SolCall},
	AddressMatcher::Fixed,
	Error, RuntimeCosts,
};

use super::*;

/// APIs for cross-chain messaging using the Interoperable State Machine Protocol (ISMP)..
pub mod ismp;
/// APIs for cross-chain messaging using Polkadot's Cross-Consensus Messaging (XCM).
pub mod xcm;

/// The first version of the Messaging API.
pub mod v0;
