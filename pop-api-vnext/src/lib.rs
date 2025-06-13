#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{Address, SolEncode};

type Pop = ink::env::DefaultEnvironment;

/// APIs for fungible tokens.
#[cfg(feature = "fungibles")]
pub mod fungibles;

#[macro_export]
macro_rules! ensure {
	( $x:expr, $y:expr $(,)? ) => {{
		if !$x {
			return Err($y.into());
		}
	}};
}

/// Calculates the address of a precompile at index `n`.
#[inline]
fn fixed_address(n: u16) -> Address {
	let shifted = (n as u32) << 16;

	let suffix = shifted.to_be_bytes();
	let mut address = [0u8; 20];
	let mut i = 16;
	while i < address.len() {
		address[i] = suffix[i - 16];
		i = i + 1;
	}
	address.into()
}

/// Calculates the address of a precompile at index `n` and with some additional prefix.
#[inline]
fn prefixed_address(n: u16, prefix: u32) -> Address {
	let mut address = fixed_address(n);
	address.0[..4].copy_from_slice(&prefix.to_be_bytes());
	address
}

/// Reverts the current contract execution, rolling back any changes and returning the specified
/// `error`.
// Helper until Solidity support added for Rust errors for automatic reversion based on returning an
// error.
pub fn revert(error: &impl for<'a> SolEncode<'a>) -> ! {
	use ink::env::{return_value_solidity, ReturnFlags};
	return_value_solidity(ReturnFlags::REVERT, error)
}

#[test]
fn fixed_address_works() {
	assert_eq!(hex::encode(fixed_address(100)), "0000000000000000000000000000000000640000")
}

#[test]
fn prefixed_address_works() {
	assert_eq!(
		hex::encode(prefixed_address(101, u32::MAX)),
		"ffffffff00000000000000000000000000650000"
	);
}
