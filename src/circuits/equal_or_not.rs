// use super::super::is_zero::{IsZeroChip, IsZeroConfig};
// use halo2_proofs::{
//     arithmetic::FieldExt,
//     circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
//     plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Instance, Selector},
//     poly::Rotation,
// };
// use std::marker::PhantomData;

// #[derive(Debug, Clone)]
// // We add the is_zero_config to the FunctionConfig as this is the gadget that we'll be using
// // The is_zero_config is the configuration for the IsZeroChip and is composed of an advice column and an expression
// struct FunctionConfig<F: FieldExt> {
//     selector: Selector,
//     a: Column<Advice>,
//     b: Column<Advice>,

//     a_equals_b: IsZeroConfig<F>,
//     output: Column<Advice>,
//     instance: Column<Instance>,
// }

// #[derive(Debug, Clone)]
// struct FunctionChip<F: FieldExt> {
//     config: FunctionConfig<F>,
// }

// impl<F: FieldExt> FunctionChip<F> {
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
//         meta.enable_equality(a);
//         meta.enable_equality(b);
//         meta.enable_equality(output);
//         meta.enable_equality(instance);

//         // meta.enable_equality(instance);

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
//             // let _a = meta.query_advice(a, Rotation::cur());
//             // let _b = meta.query_advice(b, Rotation::cur());

//             // a  |  b  | s      |a == b | output  |  s * (a == b) * (output - 1) | s * (1 - a == b) * (output - 0)
//             // --------------------------------
//             // 10 | 12  | 1      | 0     |  0      | 1 * 0 * -1                   | 1 * 1 * 0 = 0
//             // 10 | 10  | 1      | 1     |  1      | 1 * 1 * 0 (output == 1)      | 1 * 0 * 1 = 0
//             let output = meta.query_advice(output, Rotation::cur());
//             vec![
//                 s.clone() * (a_equals_b.expr() * (output.clone() - Expression::Constant(F::one()))), // in this case output == 1
//                 s * (Expression::Constant(F::one()) - a_equals_b.expr()) * (output), // in this case output == 0
//             ]
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
//         offset: usize,
//         // c: F,
//     ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
//         let is_zero_chip = IsZeroChip::construct(self.config.a_equals_b.clone());

//         layouter.assign_region(
//             || "f(a, b) = if a == b {1} else {0}",
//             |mut region| {
//                 self.config.selector.enable(&mut region, offset)?;
//                 let a_cell =
//                     region.assign_advice(|| "a", self.config.a, offset, || Value::known(a))?;
//                 let b_cell =
//                     region.assign_advice(|| "b", self.config.b, offset, || Value::known(b))?;
//                 // region.assign_advice(|| "c", self.config.c, 0, || Value::known(c))?;
//                 // remember that the is_zero assign will assign the inverse of the value provided to the advice column
//                 is_zero_chip.assign(&mut region, 0, Value::known(a - b))?;
//                 let output = if a == b { F::from(1) } else { F::from(0) };
//                 let out_cell = region.assign_advice(
//                     || "output",
//                     self.config.output,
//                     offset,
//                     || Value::known(output),
//                 )?;

//                 Ok((a_cell, b_cell, out_cell))
//                 // Ok([out_cell, out_cell])
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
//     a: [F; 10],
//     b: F,
//     _marker: PhantomData<F>,
//     // c: F,
// }

// impl<F: FieldExt> Circuit<F> for FunctionCircuit<F> {
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
//         // chip.assign(layouter, self.a, self.b, self.c)?;
//         // chip.assign(layouter, self.a, self.b)?;
//         for i in 0..self.a.len() {
//             let (_, _, out_cell) = chip.assign(&mut layouter, self.a[i], self.b, 0)?;
//             chip.expose_public(&mut layouter, out_cell, i)?;
//         }
//         // let (_, _, out_cell) = chip.assign(&mut layouter, self.a[0], self.b, 0)?;
//         // chip.expose_public(&mut layouter, out_cell.clone(), 0)?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

//     #[test]
//     fn test_example1() {
//         let a: [Fp; 10] = [
//             Fp::from(1),
//             Fp::from(2),
//             Fp::from(1),
//             Fp::from(9),
//             Fp::from(9),
//             Fp::from(1000),
//             Fp::from(65),
//             Fp::from(21),
//             Fp::from(0),
//             Fp::from(100),
//         ];
//         let b = Fp::from(1);
//         // let constant = Fp::from(16);
//         // let z = Fp::from(0);
//         let z: [Fp; 10] = [
//             Fp::from(1),
//             Fp::from(0),
//             Fp::from(1),
//             Fp::from(0),
//             Fp::from(0),
//             Fp::from(0),
//             Fp::from(0),
//             Fp::from(0),
//             Fp::from(0),
//             Fp::from(0),
//         ];
//         let circuit = FunctionCircuit {
//             a,
//             b,
//             _marker: PhantomData,
//         };
//         // let c = vec![z];

//         let prover: MockProver<Fp> = MockProver::run(8, &circuit, vec![z.to_vec()]).unwrap();
//         prover.assert_satisfied();
//     }
// }
