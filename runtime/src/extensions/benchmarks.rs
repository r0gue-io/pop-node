#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use frame_benchmarking::v2::*;
trait Config: pallet_balances::Config {}
impl<T: pallet_balances::Config> Config for T {}

pub struct Pallet<T> {
    _phantom: sp_std::marker::PhantomData<T>,
}

pub type PopApiExtensionBenchmarking<T> = Pallet<T>;

#[benchmarks]
mod benchmarks {
    use super::*;
    use codec::{Encode, Decode};
    #[benchmark]
    fn decode(n: Linear<0, 1000>) {
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
}