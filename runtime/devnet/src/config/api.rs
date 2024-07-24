use crate::{config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Contains;

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeStateKeys<T: fungibles::Config> {
	#[codec(index = 150)]
	Fungibles(fungibles::Read<T>),
}

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
}

/// A type to identify allowed calls to the Runtime from contracts. Used by Pop API
pub struct AllowedApiCalls;

impl Contains<RuntimeCall> for AllowedApiCalls {
	fn contains(c: &RuntimeCall) -> bool {
		use fungibles::Call as FungiblesCall;
		matches!(
			c,
			RuntimeCall::Fungibles(
				FungiblesCall::transfer { .. }
					| FungiblesCall::approve { .. }
					| FungiblesCall::increase_allowance { .. }
			)
		)
	}
}
