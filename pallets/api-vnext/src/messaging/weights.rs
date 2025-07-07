use frame_support::pallet_prelude::Weight;

pub trait WeightInfo {
	fn block_number() -> Weight;
	fn get_response() -> Weight;
	fn poll_status() -> Weight;
	fn remove(x: u32) -> Weight;
	fn xcm_new_query(x: u32) -> Weight;
	fn xcm_response() -> Weight;
	fn ismp_on_response(x: u32) -> Weight;
	fn ismp_on_timeout(x: u32) -> Weight;
	fn ismp_get(x: u32, y: u32, a: u32) -> Weight;
	fn ismp_post(x: u32, y: u32) -> Weight;
	fn top_up_callback_weight() -> Weight;
}

impl WeightInfo for () {
	fn block_number() -> Weight {
		Default::default()
	}

	fn get_response() -> Weight {
		Default::default()
	}

	fn poll_status() -> Weight {
		Default::default()
	}

	fn remove(_x: u32) -> Weight {
		Default::default()
	}

	fn xcm_new_query(_x: u32) -> Weight {
		Default::default()
	}

	fn xcm_response() -> Weight {
		Default::default()
	}

	fn ismp_on_response(_x: u32) -> Weight {
		Default::default()
	}

	fn ismp_on_timeout(_x: u32) -> Weight {
		Default::default()
	}

	fn ismp_get(_x: u32, _y: u32, _a: u32) -> Weight {
		Default::default()
	}

	fn ismp_post(_x: u32, _y: u32) -> Weight {
		Default::default()
	}

	fn top_up_callback_weight() -> Weight {
		Default::default()
	}
}
