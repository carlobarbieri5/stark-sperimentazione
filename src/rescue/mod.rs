// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.
//
// Adapted from winterfell v0.13.1 examples/src/rescue: the AIR (air.rs),
// prover (prover.rs), and Rescue permutation (rescue.rs) are copied VERBATIM.
// Only the example CLI / `Example`-trait glue has been stripped, and the native
// hash-chain baseline (`compute_hash_chain`) kept for the T_C measurement.

use core::marker::PhantomData;

use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::{fields::f128::BaseElement, FieldElement},
    ProofOptions, Prover,
};

#[allow(clippy::module_inception)]
pub(crate) mod rescue;

mod air;
pub use air::{PublicInputs, RescueAir};

mod prover;
pub use prover::RescueProver;

// CONSTANTS
// ================================================================================================

const CYCLE_LENGTH: usize = 16;
const NUM_HASH_ROUNDS: usize = 14;
const TRACE_WIDTH: usize = 4;

// NATIVE HASH CHAIN (T_C baseline)
// ================================================================================================

/// Runs the Rescue hash chain natively, with no proving. Used as the T_C naive baseline.
pub fn compute_hash_chain(seed: [BaseElement; 2], length: usize) -> [BaseElement; 2] {
    let mut values = seed;
    let mut result = [BaseElement::ZERO; 2];
    for _ in 0..length {
        rescue::hash(values, &mut result);
        values.copy_from_slice(&result);
    }
    result
}
