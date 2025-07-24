//! The `pop-api` crate provides an API for smart contracts to interact with the Pop Network
//! runtime.
//!
//! This crate abstracts away complexities to deliver a streamlined developer experience while
//! supporting multiple API versions to ensure backward compatibility. It is designed with a focus
//! on stability, future-proofing, and storage efficiency, allowing developers to easily integrate
//! powerful runtime features into their contracts without unnecessary overhead.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::Address;
pub use sol::{revert, SolErrorDecode};

/// An index to a block.
pub type BlockNumber = u32;
/// The onchain environment provided by Pop.
pub type Pop = ink::env::DefaultEnvironment;

/// The various general errors which may occur.
pub mod errors;
/// APIs for fungible tokens.
#[cfg(feature = "fungibles")]
pub mod fungibles;
/// APIs for cross-chain messaging.
#[cfg(feature = "messaging")]
pub mod messaging;
/// Types and utilities for working with Solidity ABI encoding.
pub mod sol;

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
const fn fixed_address(n: u16) -> Address {
	let shifted = (n as u32) << 16;

	let suffix = shifted.to_be_bytes();
	let mut address = [0u8; 20];
	let mut i = 16;
	while i < address.len() {
		address[i] = suffix[i - 16];
		i = i + 1;
	}
	ink::H160(address)
}

/// Calculates the address of a precompile at index `n` and with some additional prefix.
#[cfg(feature = "fungibles")]
#[inline]
fn prefixed_address(n: u16, prefix: u32) -> Address {
	let mut address = fixed_address(n);
	address.0[..4].copy_from_slice(&prefix.to_be_bytes());
	address
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
