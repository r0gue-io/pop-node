use crate::{Runtime, RuntimeCall, RuntimeEvent};

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
	fn sudo_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_sudo::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}
}
