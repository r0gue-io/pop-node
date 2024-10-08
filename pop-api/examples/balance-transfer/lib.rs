#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod transfer_contract {
	use ink::xcm::prelude::*;

	/// No storage is needed for this simple contract.
	#[ink(storage)]
	pub struct TransferContract {}

	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		TransferFailed,
		AHTransferFailed,
	}

	impl TransferContract {
		/// Creates a new instance of this contract.
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Self {}
		}

		/// Transfers `value` amount of tokens to the specified `account`.
		///
		/// # Arguments
		///
		/// * `account` - The account to which the funds should be transferred.
		/// * `value` - The amount of tokens to transfer.
		///
		/// # Errors
		///
		/// - Panics if the contract does not have sufficient balance.
		/// - Panics if the transfer fails.
		#[ink(message)]
		pub fn transfer(&mut self, account: AccountId, value: Balance) -> Result<(), Error> {
			self.env().transfer(account, value).map_err(|_| Error::TransferFailed)
		}

		#[ink(message)]
		pub fn ah_transfer(
			&mut self,
			value: Balance,
			to: AccountId,
			fee: Balance,
		) -> Result<(), Error> {
			// let ah = Junctions::from([Parachain(1000)]);
			// let destination: Location = Location { parents: 1, interior: ah };
			// let asset: Asset = (Location::parent(), value).into();
			// let fee_asset: Asset = (Location::parent(), fee).into();
			// let beneficiary = AccountId32 { network: None, id: to.0 };
			//
			// // XCM instructions to be executed on AssetHub
			// let dest_xcm: Xcm<()> = Xcm::builder()
			// 	.withdraw_asset(asset.clone().into())
			// 	.buy_execution(fee_asset.into(), WeightLimit::Unlimited)
			// 	.deposit_asset(asset.into(), beneficiary.into())
			// 	.clear_origin()
			// 	.build();
			//
			// // Construct the full XCM message
			// let local_xcm: Xcm<()> = Xcm::builder()
			// 	.withdraw_asset(asset.clone().into())
			// 	.burn_asset(asset.clone().into())
			// 	.build();
			//
			// let _hash = self
			// 	.env()
			// 	.xcm_send(&VersionedLocation::V4(destination), &VersionedXcm::V4(message))
			// 	.map_err(|_| Error::AHTransferFailed)?;
			//
			// Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn transfer_works() {
			// given
			let contract_balance = 100;
			let accounts = default_accounts();
			let mut contract = create_contract(contract_balance);

			// when
			set_balance(accounts.eve, 0);
			contract.transfer(accounts.eve, 80);

			// then
			assert_eq!(get_balance(accounts.eve), 80);
		}

		#[ink::test]
		#[should_panic(expected = "insufficient funds!")]
		fn transfer_fails_insufficient_funds() {
			// given
			let contract_balance = 100;
			let accounts = default_accounts();
			let mut contract = create_contract(contract_balance);

			// when
			contract.transfer(accounts.eve, 120);

			// then
			// Must panic due to insufficient funds
		}

		/// Helper functions to create contract, manage balances and accounts for testing.
		fn create_contract(initial_balance: Balance) -> TransferContract {
			let accounts = default_accounts();
			set_sender(accounts.alice);
			set_balance(contract_id(), initial_balance);
			TransferContract::new()
		}

		fn contract_id() -> AccountId {
			ink::env::test::callee::<ink::env::DefaultEnvironment>()
		}

		fn set_sender(sender: AccountId) {
			ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
		}

		fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
			ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
		}

		fn set_balance(account_id: AccountId, balance: Balance) {
			ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
				account_id, balance,
			);
		}

		fn get_balance(account_id: AccountId) -> Balance {
			ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
				.expect("Cannot get account balance")
		}
	}
}
