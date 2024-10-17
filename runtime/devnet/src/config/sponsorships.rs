use crate::{Runtime, RuntimeEvent};

impl pallet_sponsorships::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_sponsorships::weights::SubstrateWeight<Self>;
}
