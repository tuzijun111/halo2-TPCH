pub mod add_carry_v1;
pub mod add_carry_v2;
pub mod add_carry_v3;
pub mod hash_v1;
pub mod hash_v2;
pub mod inclusion_check;
pub mod inclusion_check_v2;
pub mod is_zero;
pub mod is_zero_v1;
pub mod is_zero_v2;

pub mod less_than;
pub mod less_than_vector;
pub mod lessthan_or_equal;
pub mod lessthan_or_equal_generic;
pub mod lessthan_or_equal_v1;
pub mod lessthan_or_equal_vector;

pub mod less_than_v1_test;
pub mod merkle_sum_tree;
pub mod merkle_v1;
pub mod merkle_v2;
pub mod merkle_v3;
pub mod overflow_check;
pub mod overflow_check_v2;
pub mod permutation_any;
pub mod poseidon;
pub mod safe_accumulator;
pub mod util;
pub mod utils;

// use halo2_proofs::arithmetic::Field;
// use num_traits::cast::FromPrimitive;

use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};

pub fn bool_check<F: PrimeField>(value: Expression<F>) -> Expression<F> {
    range_check(value, 2)
}

// pub fn range_check<F: Field>(word: Expression<F>, range: usize) -> Expression<F> {
//     (1..range).fold(word.clone(), |acc, i| {
//         acc * (Expression::Constant(F::from(i as u64)) - word.clone())
//     })
// }

pub fn range_check<F: PrimeField>(word: Expression<F>, range: usize) -> Expression<F> {
    (1..range).fold(word.clone(), |acc, i| {
        let i_as_field_element = F::from(i as u64); // Hypothetical method; replace with your field's equivalent
        acc * (Expression::Constant(i_as_field_element) - word.clone())
    })
}
