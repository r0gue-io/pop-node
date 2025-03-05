//! Benchmarking setup for pallet-motion

use frame_benchmarking::v2::*;
use frame_support::traits::EnsureOrigin;

use super::*;
#[allow(unused)]
use crate::Pallet as Motion;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks(
where
	<T as Config>::RuntimeCall: From<frame_system::Call<T>>,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn simple_majority() -> Result<(), BenchmarkError> {
		let call: <T as Config>::RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		let origin = <T as Config>::SimpleMajorityOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Origin should be present"))?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, Box::new(call));

		assert_last_event::<T>(Event::DispatchSimpleMajority { motion_result: Ok(()) }.into());
		Ok(())
	}

	#[benchmark]
	fn super_majority() -> Result<(), BenchmarkError> {
		let call: <T as Config>::RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		let origin = <T as Config>::SuperMajorityOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Origin should be present"))?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, Box::new(call));

		assert_last_event::<T>(Event::DispatchSuperMajority { motion_result: Ok(()) }.into());
		Ok(())
	}

	#[benchmark]
	fn unanimous() -> Result<(), BenchmarkError> {
		let call: <T as Config>::RuntimeCall = frame_system::Call::remark { remark: vec![] }.into();
		let origin = <T as Config>::UnanimousOrigin::try_successful_origin()
			.map_err(|_| BenchmarkError::Stop("Origin should be present"))?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, Box::new(call));

		assert_last_event::<T>(Event::DispatchUnanimous { motion_result: Ok(()) }.into());
		Ok(())
	}

	impl_benchmark_test_suite!(Motion, crate::mock::new_test_ext(), crate::mock::Test);
}
