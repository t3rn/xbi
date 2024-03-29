//! Autogenerated weights for pallet_circuit_circuit_portal
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-09-24, STEPS: `[50, ]`, REPEAT: 100, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/release/circuit
// benchmark
// --chain=dev
// --steps=50
// --repeat=100
// --pallet=pallet_circuit
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./src/circuit/src/weights.rs
// --template=../benchmarking/frame-weight-template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, ClassifyDispatch, DispatchClass, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_circuit_circuit_portal.
pub trait WeightInfo {
    fn batch_enter_xbi() -> Weight;
}

/// Weights for pallet_circuit_circuit_portal using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn batch_enter_xbi() -> Weight {
        6_984_000_u64
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn batch_enter_xbi() -> Weight {
        6_984_000_u64
    }
}

// impl<T: frame_system::Config> ClassifyDispatch<T> for SubstrateWeight<T> {
//     fn batch_enter_xbi() -> Weight {
//         6_984_000_u64
//     }
//
// }
//
