#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::nfts;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	InvalidCollection,
	ItemAlreadyExists,
	NftsError(nfts::Error),
	NotOwner,
}

impl From<nfts::Error> for ContractError {
	fn from(value: nfts::Error) -> Self {
		ContractError::NftsError(value)
	}
}

#[ink::contract(env = pop_api::Environment)]
mod pop_api_spot_order {
	use super::ContractError;

	#[ink(storage)]
	#[derive(Default)]
	pub struct PopApiSpotOrder;

	impl PopApiSpotOrder {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiSpotOrder::new");
			Default::default()
		}

		#[ink(message)]
		pub fn place_spot_order(
			&mut self,
			max_amount: Balance,
			para_id: u32,
		) -> Result<(), ContractError> {
			ink::env::debug_println!(
				"PopApiSpotOrder::place_spot_order: max_amount {:?} para_id: {:?} ",
				max_amount,
				para_id,
			);

			let res = pop_api::cross_chain::coretime::place_spot_order(max_amount, para_id);
			ink::env::debug_println!(
				"PopApiSpotOrder::place_spot_order: res {:?} ",
				res,
			);

			ink::env::debug_println!("PopApiSpotOrder::place_spot_order end");
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiSpotOrder::new();
		}
	}
}
