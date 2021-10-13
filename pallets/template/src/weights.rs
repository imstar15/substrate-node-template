// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

//! Autogenerated weights for pallet_template
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-10-13, STEPS: `[20, ]`, REPEAT: 50, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/node-template
// benchmark
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --extrinsic=*
// --pallet
// pallet-template
// --steps
// 20
// --repeat
// 50
// --template=./.maintain/frame-weight-template.hbs
// --output=pallets/template/src/weights.rs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_template.
pub trait WeightInfo {
	fn do_something(s: u32, ) -> Weight;
	fn calculate(s: u32, ) -> Weight;
}

/// Weights for pallet_template using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn do_something(_s: u32, ) -> Weight {
		(12_328_000 as Weight)
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn calculate(s: u32, ) -> Weight {
		(12_000_000 as Weight)
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn do_something(_s: u32, ) -> Weight {
		(12_328_000 as Weight)
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
	fn calculate(s: u32, ) -> Weight {
		(12_000_000 as Weight)
			// Standard Error: 0
			.saturating_add((1_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(RocksDbWeight::get().writes(1 as Weight))
	}
}