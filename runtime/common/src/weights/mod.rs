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

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

//! Expose the auto generated weight files.

pub mod block_weights;
pub mod cumulus_pallet_parachain_system;
pub mod cumulus_pallet_weight_reclaim;
pub mod cumulus_pallet_xcmp_queue;
pub mod extrinsic_weights;
pub mod frame_system;
pub mod frame_system_extensions;
pub mod pallet_assets;
pub mod pallet_balances;
pub mod pallet_collator_selection;
pub mod pallet_collective;
pub mod pallet_message_queue;
pub mod pallet_migrations;
pub mod pallet_motion;
pub mod pallet_multisig;
pub mod pallet_nfts;
pub mod pallet_preimage;
pub mod pallet_proxy;
pub mod pallet_revive;
pub mod pallet_scheduler;
pub mod pallet_session;
pub mod pallet_sudo;
pub mod pallet_timestamp;
pub mod pallet_transaction_payment;
pub mod pallet_treasury;
pub mod pallet_utility;
pub mod pallet_xcm;
pub mod paritydb_weights;
pub mod rocksdb_weights;
pub mod xcm;

pub use block_weights::constants::BlockExecutionWeight;
pub use extrinsic_weights::constants::ExtrinsicBaseWeight;
pub use rocksdb_weights::constants::RocksDbWeight;
