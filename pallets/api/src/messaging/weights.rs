
use crate::*;

pub trait WeightInfo {
	fn remove(x: u32) -> Weight;
	fn xcm_new_query(x: u32) -> Weight;
	fn xcm_response() -> Weight;
	fn ismp_on_response(x: u32) -> Weight;
	fn ismp_on_timeout(x: u32) -> Weight;
	fn ismp_get(y: u32, x: u32, a: u32) -> Weight;
	fn ismp_post(x: u32, y: u32) -> Weight;
}

#[cfg(test)]
impl WeightInfo for () {
	fn remove(x: u32) -> Weight {
		Default::default()
	}

	fn xcm_new_query(x: u32) -> Weight {
		Default::default()
	}

	fn xcm_response() -> Weight {
		Default::default()
	}

	fn ismp_on_response(x: u32) -> Weight {
		Default::default()
	}

	fn ismp_on_timeout(x: u32) -> Weight {
		Default::default()
	}

	fn ismp_get(y: u32, z: u32, a: u32) -> Weight {
		Default::default()
	}

	fn ismp_post(x: u32, y: u32) -> Weight {
		Default::default()
	}
}
