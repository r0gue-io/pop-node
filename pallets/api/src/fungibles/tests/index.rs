use super::*;

mod ensure_read_variant_index {
	use super::*;

	#[test]
	fn total_supply() {
		let total_supply = TotalSupply::<Test>(Default::default());
		assert_eq!(total_supply.encode()[0], 0);
	}

	#[test]
	fn balance_of() {
		let balance_of = BalanceOf::<Test> { token: Default::default(), owner: Default::default() };
		assert_eq!(balance_of.encode()[0], 1);
	}

	#[test]
	fn allowance() {
		let allowance = Allowance::<Test> {
			token: Default::default(),
			owner: Default::default(),
			spender: Default::default(),
		};
		assert_eq!(allowance.encode()[0], 2);
	}

	#[test]
	fn token_name() {
		let token_name = TokenName::<Test>(Default::default());
		assert_eq!(token_name.encode()[0], 8);
	}

	#[test]
	fn token_symbol() {
		let token_symbol = TokenSymbol::<Test>(Default::default());
		assert_eq!(token_symbol.encode()[0], 9);
	}

	#[test]
	fn token_decimals() {
		let token_decimals = TokenDecimals::<Test>(Default::default());
		assert_eq!(token_decimals.encode()[0], 10);
	}

	#[test]
	fn token_exists() {
		let token_exists = TokenExists::<Test>(Default::default());
		assert_eq!(token_exists.encode()[0], 18);
	}
}

mod ensure_dispatchable_index {
	use super::{new_test_ext, Encode, RuntimeCall};
	use crate::{fungibles::Call::*, mock::RuntimeCall::Fungibles};

	#[test]
	fn transfer() {
		new_test_ext().execute_with(|| {
			let transfer: RuntimeCall = Fungibles(transfer {
				token: Default::default(),
				to: Default::default(),
				value: Default::default(),
			});
			assert_eq!(transfer.encode()[1], 3);
		});
	}

	#[test]
	fn transfer_from() {
		new_test_ext().execute_with(|| {
			let transfer_from: RuntimeCall = Fungibles(transfer_from {
				token: Default::default(),
				from: Default::default(),
				to: Default::default(),
				value: Default::default(),
			});
			assert_eq!(transfer_from.encode()[1], 4);
		});
	}

	#[test]
	fn approve() {
		new_test_ext().execute_with(|| {
			let approve: RuntimeCall = Fungibles(approve {
				token: Default::default(),
				spender: Default::default(),
				value: Default::default(),
			});
			assert_eq!(approve.encode()[1], 5);
		});
	}

	#[test]
	fn increase_allowance() {
		new_test_ext().execute_with(|| {
			let increase_allowance: RuntimeCall = Fungibles(increase_allowance {
				token: Default::default(),
				spender: Default::default(),
				value: Default::default(),
			});
			assert_eq!(increase_allowance.encode()[1], 6);
		});
	}

	#[test]
	fn decrease_allowance() {
		new_test_ext().execute_with(|| {
			let decrease_allowance: RuntimeCall = Fungibles(decrease_allowance {
				token: Default::default(),
				spender: Default::default(),
				value: Default::default(),
			});
			assert_eq!(decrease_allowance.encode()[1], 7);
		});
	}

	#[test]
	fn create() {
		new_test_ext().execute_with(|| {
			let create: RuntimeCall = Fungibles(create {
				id: Default::default(),
				admin: Default::default(),
				min_balance: Default::default(),
			});
			assert_eq!(create.encode()[1], 11);
		});
	}

	#[test]
	fn start_destroy() {
		new_test_ext().execute_with(|| {
			let start_destroy: RuntimeCall = Fungibles(start_destroy { token: Default::default() });
			assert_eq!(start_destroy.encode()[1], 12);
		});
	}

	#[test]
	fn set_metadata() {
		new_test_ext().execute_with(|| {
			let set_metadata: RuntimeCall = Fungibles(set_metadata {
				token: Default::default(),
				name: Default::default(),
				symbol: Default::default(),
				decimals: Default::default(),
			});
			assert_eq!(set_metadata.encode()[1], 16);
		});
	}

	#[test]
	fn clear_metadata() {
		new_test_ext().execute_with(|| {
			let clear_metadata: RuntimeCall =
				Fungibles(clear_metadata { token: Default::default() });
			assert_eq!(clear_metadata.encode()[1], 17);
		});
	}

	#[test]
	fn mint() {
		new_test_ext().execute_with(|| {
			let mint: RuntimeCall = Fungibles(mint {
				token: Default::default(),
				account: Default::default(),
				value: Default::default(),
			});
			assert_eq!(mint.encode()[1], 19);
		});
	}

	#[test]
	fn burn() {
		new_test_ext().execute_with(|| {
			let burn: RuntimeCall = Fungibles(burn {
				token: Default::default(),
				account: Default::default(),
				value: Default::default(),
			});
			assert_eq!(burn.encode()[1], 20);
		});
	}
}
