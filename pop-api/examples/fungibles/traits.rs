use super::*;

#[ink::trait_definition]
pub trait MinterRole {
	/// Check if the caller is the minter of the contract.
	#[ink(message)]
	fn ensure_minter(&self, account: AccountId) -> Result<(), Psp22Error>;

	/// Add a new minter by existing minters.
	#[ink(message)]
	fn add_minter(&mut self, minter: AccountId) -> Result<(), Psp22Error>;

	/// Add a new minter by existing minters.
	#[ink(message)]
	fn remove_minter(&mut self, minter: AccountId) -> Result<(), Psp22Error>;
}
