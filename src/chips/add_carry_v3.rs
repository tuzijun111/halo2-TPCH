// // use super::utils::f_to_nbits;
// use eth_types::Field;
// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
// use std::marker::PhantomData;

// #[derive(Debug, Clone)]
// pub struct AddCarryConfig<F: Field> {
//     pub advice: [Column<Advice>; 2],
//     pub constant: Column<Fixed>,
//     pub instance: Column<Instance>,
//     pub selector: Selector,
//     pub _marker: PhantomData<F>,
// }

// #[derive(Debug, Clone)]
// pub struct AddCarryChip<F: Field> {
//     config: AddCarryConfig<F>,
// }

// impl<F: Field> AddCarryChip<F> {
//     pub fn construct(config: AddCarryConfig<F>) -> Self {
//         Self { config }
//     }

//     pub fn configure(
//         meta: &mut ConstraintSystem<F>,
//         advice: [Column<Advice>; 2],
//         constant: Column<Fixed>,
//         selector: Selector,
//         instance: Column<Instance>,
//     ) -> AddCarryConfig<F> {
//         let col_a = advice[0];
//         let col_b = advice[1];
//         // let col_c = advice[2];
//         let add_carry_selector = selector;

//         // Enable equality on the advice and instance column to enable permutation check
//         meta.enable_equality(col_a);
//         meta.enable_equality(col_b);
//         // meta.enable_equality(col_c);
//         meta.enable_equality(instance);

//         // Enable constant column
//         meta.enable_constant(constant);

//         // enforce dummy hash function by creating a custom gate
//         meta.create_gate("accumulate constraint", |meta| {
//             let s = meta.query_selector(add_carry_selector);
//             let prev_b = meta.query_advice(col_b, Rotation::prev());
//             // let prev_c = meta.query_advice(col_c, Rotation::prev());
//             let a = meta.query_advice(col_a, Rotation::cur());
//             let b = meta.query_advice(col_b, Rotation::cur());
//             // let c = meta.query_advice(col_c, Rotation::cur());

//             // Previous accumulator amount + new value from a_cell
//             vec![s * (a + prev_b - b)]
//             // vec![s * (a - b)]
//         });

//         AddCarryConfig {
//             advice: [col_a, col_b],
//             constant,
//             instance,
//             selector: add_carry_selector,
//             _marker: PhantomData,
//         }
//     }

//     // Initial accumulator values from instance for expriment
//     pub fn assign_first_row(
//         &self,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), Error> {
//         layouter.assign_region(
//             || "Initialize first row as zero",
//             |mut region| {
//                 // self.config.selector.enable(&mut region, 0)?;

//                 let a_cell = region.assign_advice_from_constant(
//                     || "first input",
//                     self.config.advice[0],
//                     0,
//                     F::zero(),
//                 )?;

//                 let b_cell = region.assign_advice_from_constant(
//                     || "first accu",
//                     self.config.advice[1],
//                     0,
//                     F::zero(),
//                 )?;

//                 Ok((a_cell, b_cell))
//             },
//         )
//     }

//     pub fn assign_advice_row(
//         &self,
//         mut layouter: impl Layouter<F>,
//         a: Value<F>,
//         prev_b: AssignedCell<F, F>,
//         nrows: usize,
//     ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>), Error> {
//         layouter.assign_region(
//             || "adivce row for accumulating",
//             |mut region| {
//                 let a_cell =
//                     region.assign_advice(|| "a", self.config.advice[0], 0, || a.clone())?;

//                 // assigning two columns of accumulating value
//                 let b_cell = region.assign_advice(
//                     || "sum_hi",
//                     self.config.advice[1],
//                     0,
//                     || prev_b.value().copied() + a,
//                 )?;

//                 for row in 2..nrows {
//                     // enable hash selector
//                     // self.config.selector.enable(&mut region, row)?;

//                     // Assign new amount to the cell inside the region
//                     let a_cell = region.assign_advice(
//                         || "a",
//                         self.config.advice[0],
//                         row - 1,
//                         || a.clone(),
//                     )?;

//                     // assigning two columns of accumulating value
//                     let b_cell = region.assign_advice(
//                         || "sum_hi",
//                         self.config.advice[1],
//                         row - 1,
//                         || prev_b.value().copied() + a,
//                     )?;
//                 }

//                 Ok((a_cell, b_cell))
//             },
//         )
//     }

//     // Enforce permutation check between b & cell and instance column
//     pub fn expose_public(
//         &self,
//         mut layouter: impl Layouter<F>,
//         cell: &AssignedCell<F, F>,
//         row: usize,
//     ) -> Result<(), Error> {
//         layouter.constrain_instance(cell.cell(), self.config.instance, row)
//     }
// }
