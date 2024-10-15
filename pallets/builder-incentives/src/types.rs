use frame_support::pallet_prelude::*;
use sp_core::H160;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// Era number type
pub type EraNumber = u32;

/// Trait defining the interface for dApp staking `smart contract types` handler.
///
/// It can be used to create a representation of the specified smart contract instance type.
pub trait SmartContractHandle<AccountId> {
	/// Create a new smart contract representation for the specified EVM address.
	fn evm(address: H160) -> Self;
	/// Create a new smart contract representation for the specified Wasm address.
	fn wasm(address: AccountId) -> Self;
}

/// Multi-VM pointer to smart contract instance.
#[derive(
	PartialEq,
	Eq,
	Copy,
	Clone,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	Hash,
	scale_info::TypeInfo,
)]
pub enum SmartContract<AccountId> {
	/// EVM smart contract instance.
	Evm(H160),
	/// Wasm smart contract instance.
	Wasm(AccountId),
}

impl<AccountId> SmartContractHandle<AccountId> for SmartContract<AccountId> {
	fn evm(address: H160) -> Self {
		Self::Evm(address)
	}

	fn wasm(address: AccountId) -> Self {
		Self::Wasm(address)
	}
}
