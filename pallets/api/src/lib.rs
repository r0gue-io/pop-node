#![cfg_attr(not(feature = "std"), no_std)]

pub use extension::Extension;

pub mod extension;
pub mod fungibles;
#[cfg(test)]
mod mock;
