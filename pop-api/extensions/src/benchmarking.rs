#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::v2::*;

// We need the Config trait from one of the pallets to setup our dummy pallet
trait Config: pallet_balances::Config {}
impl<T: pallet_balances::Config> Config for T {}

pub struct Pallet<T> {
    _phantom: sp_std::marker::PhantomData<T>,
}

pub type PopApiExtensionBenchmarking<T> = Pallet<T>;

#[benchmarks]
pub mod benchmarks {
    use super::{Config, PopApiExtensionBenchmarking, *};
    use codec::{Encode, Decode};
    #[benchmark]
    fn decode(n: Linear<0, 1_000_000>) {
        #[derive(Encode, Decode)]
        struct TestStruct {
            pub data: String
        }
        let mut test_struct = TestStruct {
            data: (0..n).map(|_| 'a').collect()
        };
        let test_struct_encoded = test_struct.encode();

        #[block]
        {
            let _: TestStruct = TestStruct::decode(&mut &test_struct_encoded[..]).unwrap();
        }
    }

    impl_benchmark_test_suite!(PopApiExtensionBenchmarking, crate::tests::new_test_ext(), crate::tests::Test);
}