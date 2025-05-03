// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::vec::Vec;
use core::{marker::PhantomData, num::NonZero};

use alloy_core::sol;

use crate::{
	precompiles::{BuiltinAddressMatcher, BuiltinPrecompile, Error, Ext, ExtWithInfo},
	Config,
};

sol! {
	interface IBenchmarking {
		function bench(bytes calldata input) external;
	}
}

pub struct WithInfo<T>(PhantomData<T>);

impl<T: Config> BuiltinPrecompile for WithInfo<T> {
	type Interface = IBenchmarking::IBenchmarkingCalls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = true;
	const MATCHER: BuiltinAddressMatcher =
		BuiltinAddressMatcher::Fixed(NonZero::new(0xFF_FF).unwrap());

	fn call_with_info(
		_address: &[u8; 20],
		_input: &Self::Interface,
		_env: &mut impl ExtWithInfo<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		Ok(Vec::new())
	}
}

pub struct NoInfo<T>(PhantomData<T>);

impl<T: Config> BuiltinPrecompile for NoInfo<T> {
	type Interface = IBenchmarking::IBenchmarkingCalls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: BuiltinAddressMatcher =
		BuiltinAddressMatcher::Fixed(NonZero::new(0xEF_FF).unwrap());

	fn call(
		_address: &[u8; 20],
		_input: &Self::Interface,
		_env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		Ok(Vec::new())
	}
}
