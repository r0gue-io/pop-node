#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = pop_api::Environment)]
mod spot_order {

	#[ink(storage)]
	#[derive(Default)]
	pub struct SpotOrder;

	impl SpotOrder {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("SpotOrder::new");
			Default::default()
		}

		#[ink(message)]
		pub fn place_spot_order(
			&mut self,
			max_amount: Balance,
			para_id: u32,
		) {
			ink::env::debug_println!(
				"SpotOrder::place_spot_order: max_amount {:?} para_id: {:?} ",
				max_amount,
				para_id,
			);

			#[allow(unused_variables)]
			let res = pop_api::cross_chain::coretime::place_spot_order(max_amount, para_id);
			ink::env::debug_println!(
				"SpotOrder::place_spot_order: res {:?} ",
				res,
			);

			ink::env::debug_println!("SpotOrder::place_spot_order end");
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			SpotOrder::new();
		}
	}
}