// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test environment for Nfts pallet.

use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64},
};
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
	traits::{IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

use super::*;
use crate as pallet_nfts;

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Nfts: pallet_nfts,
	}
);

pub type Signature = MultiSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = IdentityLookup<Self::AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

impl Config for Test {
	type ApprovalsLimit = ConstU32<10>;
	type AttributeDepositBase = ConstU64<1>;
	type CollectionApprovalDeposit = ConstU64<1>;
	type CollectionBalanceDeposit = ConstU64<1>;
	type CollectionDeposit = ConstU64<2>;
	type CollectionId = u32;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type Currency = Balances;
	type DepositPerByte = ConstU64<1>;
	type Features = Features;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type ItemDeposit = ConstU64<1>;
	type ItemId = u32;
	type KeyLimit = ConstU32<50>;
	type Locker = ();
	type MaxAttributesPerCall = ConstU32<2>;
	type MaxDeadlineDuration = ConstU64<10000>;
	type MaxTips = ConstU32<10>;
	type MetadataDepositBase = ConstU64<1>;
	/// Using `AccountPublic` here makes it trivial to convert to `AccountId` via `into_account()`.
	type OffchainPublic = AccountPublic;
	/// Off-chain = signature On-chain - therefore no conversion needed.
	/// It needs to be From<MultiSignature> for benchmarking.
	type OffchainSignature = Signature;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type WeightInfo = ();
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
	ext.execute_with(|| System::set_block_number(1));
	ext
}
