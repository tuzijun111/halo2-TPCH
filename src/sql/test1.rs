// use halo2_gadgets::utilities::lookup_range_check::LookupRangeCheckConfig;

// use halo2_proofs::{
//     arithmetic::Field,
//     circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
//     plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Instance, Selector},
//     poly::Rotation,
// };

// #[derive(Debug, Clone)]
// // We add the is_zero_config to the FunctionConfig as this is the gadget that we'll be using
// // The is_zero_config is the configuration for the IsZeroChip and is composed of an advice column and an expression
// struct FunctionConfig<F: Field> {
//     selector: Selector,
//     a: Column<Advice>,
//     b: Column<Advice>,
//     a_equals_b: LookupRangeCheckConfig<F, K>;
//     output: Column<Advice>,
//     instance: Column<Instance>,
// }

// #[derive(Debug, Clone)]
// struct FunctionChip<F: Field> {
//     config: FunctionConfig<F>,
// }

// impl<F: Field> FunctionChip<F> {
//     pub fn construct(config: FunctionConfig<F>) -> Self {
//         Self { config }
//     }

//     // Chip configuration. This is where we define the gates
//     pub fn configure(meta: &mut ConstraintSystem<F>) -> FunctionConfig<F> {
//         let selector = meta.selector();
//         let a = meta.advice_column();
//         let b = meta.advice_column();
//         let output = meta.advice_column();
//         let is_zero_advice_column = meta.advice_column();
//         let instance = meta.instance_column();

//         meta.enable_equality(instance);
//         meta.enable_equality(a);
//         meta.enable_equality(b);
//         meta.enable_equality(output);

//         // We set the configuration for our gadget chip here!
//         let a_equals_b = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(selector), // this is the q_enable
//             |meta| meta.query_advice(a, Rotation::cur()) - meta.query_advice(b, Rotation::cur()), // this is the value
//             is_zero_advice_column, // this is the advice column that stores value_inv
//         );

//         // We now need to set up our custom gate!
//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(selector);
//             let a = meta.query_advice(a, Rotation::cur());
//             let b = meta.query_advice(b, Rotation::cur());

//             // a  |  b  | s      |a == b | output  |  s * (a == b) * (output - 1) | s * (1 - a == b) * (output - 0)
//             // --------------------------------
//             // 10 | 10  | 1      | 1     | 1       | 1 * 1 * 0                    | 1 * 0 * 1 = 0
//             // 10 | 12  | 1      | 0     | 0       | 1 * 0 * (-1)                 | 1 * 1 * 0 = 0
//             let output = meta.query_advice(output, Rotation::cur());

//             vec![
//                 s.clone() * (a_equals_b.expr() * (output.clone() - (a.clone() - b.clone()))), // in this case output == c
//                 s * (Expression::Constant(F::ONE) - a_equals_b.expr()) * (output - (a + b)), // in this case output == a - b
//             ]
//             // vec![
//             //     s.clone() * (a_equals_b.expr() * (output.clone() - Expression::Constant(F::one()))), // in this case output == c
//             //     s * (Expression::Constant(F::one()) - a_equals_b.expr()) * (output - Expression::Constant(F::zero())), // in this case output == a - b
//             // ]
//         });

//         FunctionConfig {
//             selector,
//             a,
//             b,
//             a_equals_b,
//             output,
//             instance,
//         }
//     }

//     // execute assignment on a, b, c, output column + is_zero advice column
//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         a: F,
//         b: F,
//     ) -> Result<AssignedCell<F, F>, Error> {
//         let is_zero_chip = IsZeroChip::construct(self.config.a_equals_b.clone());

//         layouter.assign_region(
//             || "f(a, b) = if a == b {1} else {0}",
//             |mut region| {
//                 // let a1 = a;
//                 // let b1 = b;
//                 self.config.selector.enable(&mut region, 0)?;
//                 region.assign_advice(|| "a", self.config.a, 0, || Value::known(a))?;
//                 region.assign_advice(|| "b", self.config.b, 0, || Value::known(b))?;
//                 // a.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;

//                 // remember that the is_zero assign will assign the inverse of the value provided to the advice column
//                 is_zero_chip.assign(&mut region, 0, Value::known(a - b))?;
//                 // let output = if a1 == b1 {F::from(1) } else { F::from(0)};
//                 let output = if a == b { a - b } else { a + b };
//                 // region.assign_advice(|| "output", self.config.output, 0, || Value::known(output))

//                 let out_cell = region.assign_advice(
//                     || "output",
//                     self.config.output,
//                     0,
//                     || Value::known(output),
//                 );
//                 out_cell
//                 // Ok(())
//             },
//         )
//     }

//     pub fn expose_public(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         cell: AssignedCell<F, F>,
//         row: usize,
//     ) -> Result<(), Error> {
//         layouter.constrain_instance(cell.cell(), self.config.instance, row)
//     }
// }

// #[derive(Default)]
// struct FunctionCircuit<F> {
//     a: F,
//     b: F,
// }

// impl<F: Field> Circuit<F> for FunctionCircuit<F> {
//     type Config = FunctionConfig<F>;
//     type FloorPlanner = SimpleFloorPlanner;

//     fn without_witnesses(&self) -> Self {
//         Self::default()
//     }

//     fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//         FunctionChip::configure(meta)
//     }

//     fn synthesize(
//         &self,
//         config: Self::Config,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(), Error> {
//         let chip = FunctionChip::construct(config);
//         let out_cell = chip.assign(&mut layouter, self.a, self.b)?;
//         chip.expose_public(&mut layouter, out_cell, 0)?;
//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::circuits::utils::full_prover;
//     use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

//     #[test]
//     fn test_example3() {
//         let k = 4;
//         let circuit = FunctionCircuit {
//             a: Fp::from(3),
//             b: Fp::from(2),
//         };
//         let public_input = vec![Fp::from(5)];

//         // let prover = MockProver::run(4, &circuit, vec![public_input]).unwrap();
//         // prover.assert_satisfied();
//         full_prover(circuit, k, &public_input);
//     }
// }
