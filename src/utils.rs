// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.
//
// Constraint-evaluation helpers used by the Rescue AIR. Copied verbatim from
// winterfell v0.13.1 examples/src/utils (only the trace-printing helpers and the
// unused `is_binary` / `rescue` submodule were dropped).

use winterfell::math::FieldElement;

// CONSTRAINT EVALUATION HELPERS
// ================================================================================================

/// Returns zero only when a == b.
pub fn are_equal<E: FieldElement>(a: E, b: E) -> E {
    a - b
}

/// Returns zero only when a == zero.
pub fn is_zero<E: FieldElement>(a: E) -> E {
    a
}

/// Return zero when a == one, and one when a == zero;
/// assumes that a is a binary value.
pub fn not<E: FieldElement>(a: E) -> E {
    E::ONE - a
}

// TRAIT TO SIMPLIFY CONSTRAINT AGGREGATION
// ================================================================================================

pub trait EvaluationResult<E> {
    fn agg_constraint(&mut self, index: usize, flag: E, value: E);
}

impl<E: FieldElement> EvaluationResult<E> for [E] {
    fn agg_constraint(&mut self, index: usize, flag: E, value: E) {
        self[index] += flag * value;
    }
}

impl<E: FieldElement> EvaluationResult<E> for Vec<E> {
    fn agg_constraint(&mut self, index: usize, flag: E, value: E) {
        self[index] += flag * value;
    }
}
