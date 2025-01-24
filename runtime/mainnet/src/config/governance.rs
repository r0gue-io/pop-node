use crate::{parameter_types, AccountId, Runtime, RuntimeCall, RuntimeEvent, Ss58Codec};

// Multisig account for sudo, generated from the following signatories:
// - 15VPagCVayS6XvT5RogPYop3BJTJzwqR2mCGR1kVn3w58ygg
// - 142zako1kfvrpQ7pJKYR8iGUD58i4wjb78FUsmJ9WcXmkM5z
// - 15k9niqckMg338cFBoz9vWFGwnCtwPBquKvqJEfHApijZkDz
// - 14G3CUFnZUBnHZUhahexSZ6AgemaW9zMHBnGccy3df7actf4
// - Threshold 2
const SUDO_ADDRESS: &str = "15NMV2JX1NeMwarQiiZvuJ8ixUcvayFDcu1F9Wz1HNpSc8gP";

parameter_types! {
	pub SudoAddress: AccountId = AccountId::from_ss58check(SUDO_ADDRESS).expect("sudo address is valid SS58");
}
impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use super::*;

	#[test]
	fn sudo_account_matches() {
		// Doesn't use SUDO_ADDRESS constant on purpose.
		assert_eq!(
			SudoAddress::get(),
			AccountId::from_ss58check("15NMV2JX1NeMwarQiiZvuJ8ixUcvayFDcu1F9Wz1HNpSc8gP")
				.expect("sudo address is valid SS58")
		);
	}
	#[test]
	fn sudo_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_sudo::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}
}
