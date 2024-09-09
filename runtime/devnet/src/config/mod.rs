// Public due to contract end-to-end testing with pop-drink crate.
pub mod api;
// Public due to pop api integration tests crate.
pub mod assets;
mod contracts;
mod proxy;
// Public due to integration tests crate.
pub mod xcm;
