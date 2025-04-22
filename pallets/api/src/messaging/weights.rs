use crate::*;

pub trait WeightInfo {
	fn remove(x: u32) -> Weight;
	fn xcm_new_query(x: u32) -> Weight;
	fn xcm_response() -> Weight;
	fn ismp_on_response(x: u32) -> Weight;
	fn ismp_on_timeout(x: u32) -> Weight;
	fn ismp_get(x: u32, y: u32, a: u32) -> Weight;
	fn ismp_post(x: u32, y: u32) -> Weight;
	fn top_up_callback_weight() -> Weight;
}

#[cfg(test)]
impl WeightInfo for () {
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
