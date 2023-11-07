// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use std::{default, marker::PhantomData};

// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// const N: usize = 5;

// // #[derive(Default)]

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig {
//     q_enable: Selector,

//     value_l: Column<Advice>,

//     ordered_value_l: Column<Advice>,
//     // // constant: Column<Fixed>,
//     // pub instance: Column<Instance>,
//     // pub instance1: Column<Instance>,
// }

// #[derive(Debug, Clone)]
// pub struct TestChip<F> {
//     config: TestCircuitConfig,
//     _marker: PhantomData<F>,
// }

// impl<F: Field> TestChip<F> {
//     pub fn construct(config: TestCircuitConfig) -> Self {
//         Self {
//             config,
//             _marker: PhantomData,
//         }
//     }

//     pub fn configure(meta: &mut ConstraintSystem<F>) -> TestCircuitConfig {
//         let q_enable = meta.complex_selector();

//         let value_l = meta.advice_column();

//         let ordered_value_l = meta.advice_column();

//         // let constant = meta.fixed_column();
//         // let instance = meta.instance_column();
//         // let instance1 = meta.instance_column();

//         // meta.enable_constant(constant);
//         // meta.enable_equality(col_out);
//         meta.enable_equality(value_l);
//         meta.enable_equality(ordered_value_l);

//         // meta.enable_equality(instance);
//         // meta.enable_equality(instance1);

//         // Sort the arrays

//         meta.create_gate("permutation check", |meta| {
//             let q_enable = meta.query_selector(q_enable);
//             let sorted_value_l = meta.query_advice(value_l, Rotation::cur());
//             let sorted_ordered_value_l = meta.query_advice(ordered_value_l, Rotation::cur());

//             // vec![q_enable * (a - b)]
//             vec![q_enable * (sorted_value_l - sorted_ordered_value_l)]
//         });

//         TestCircuitConfig {
//             q_enable,
//             value_l,
//             ordered_value_l,
//             // instance,
//             // instance1,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         a: [u64; N],
//         b: [u64; N],
//         // c: F,
//     ) -> Result<(), Error> {
//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 for i in 0..N {
//                     self.config.q_enable.enable(&mut region, i)?;

//                     region.assign_advice(
//                         || "value left",
//                         self.config.value_l,
//                         i,
//                         || Value::known(F::from(a[i])),
//                         // Value::known(F::from(v_value_l)),
//                     )?;

//                     region.assign_advice(
//                         || "ordered value",
//                         self.config.ordered_value_l,
//                         i,
//                         || Value::known(F::from(b[i])),
//                         // || Value::known(F::from(b)),
//                     )?;
//                 }
//                 Ok(())
//             },
//         )
//     }

//     // pub fn expose_public(
//     //     &self,
//     //     layouter: &mut impl Layouter<F>,
//     //     cell: AssignedCell<F, F>,
//     //     row: usize,
//     // ) -> Result<(), Error> {
//     //     layouter.constrain_instance(cell.cell(), self.config.instance, row)
//     // }

//     // pub fn expose_public1(
//     //     &self,
//     //     layouter: &mut impl Layouter<F>,
//     //     cell: AssignedCell<F, F>,
//     //     row: usize,
//     // ) -> Result<(), Error> {
//     //     layouter.constrain_instance(cell.cell(), self.config.instance1, row)
//     // }
// }

// struct MyCircuit<F> {
//     pub value_l: [u64; N],

//     pub ordered_value_l: [u64; N],

//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             value_l: [0; N],         // Initialize the array with default values if necessary
//             ordered_value_l: [0; N], // You can use the default value for u64

//             _marker: PhantomData,
//         }
//     }
// }

// impl<F: Field> Circuit<F> for MyCircuit<F> {
//     type Config = TestCircuitConfig;
//     type FloorPlanner = SimpleFloorPlanner;

//     fn without_witnesses(&self) -> Self {
//         Self::default()
//     }

//     fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//         TestChip::configure(meta)
//     }

//     fn synthesize(
//         &self,
//         config: Self::Config,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(), Error> {
//         let test_chip = TestChip::construct(config);

//         test_chip.assign(&mut layouter, self.value_l, self.ordered_value_l)?;

//         // test_chip.expose_public(&mut layouter, out_b_cell.clone(), 0)?;
//         // test_chip.expose_public1(&mut layouter, out_b_cell, 0)?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {

//     use super::MyCircuit;
//     use super::N;
//     // use halo2_proofs::poly::commitment::Params
//     use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

//     use std::marker::PhantomData;

//     #[test]
//     fn test_range() {
//         let k = 12;

//         // initate usernames and balances array
//         let mut value_l: [u64; N] = [0; N];

//         value_l[0] = 4;
//         value_l[1] = 3;
//         value_l[2] = 1;
//         value_l[3] = 2;
//         value_l[4] = 0;

//         let mut ordered_value_l: [u64; N] = [0; N];

//         for i in 0..N {
//             ordered_value_l[i] = i as u64;
//         }

//         let circuit = MyCircuit::<Fp> {
//             value_l,
//             ordered_value_l,
//             _marker: PhantomData,
//         };

//         // let z = [vec![Fp::from(4)], vec![Fp::from(4)]];

//         let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//         prover.assert_satisfied();
//     }
// }
