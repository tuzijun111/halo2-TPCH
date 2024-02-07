// use super::super::chips::less_than_v1_test::{LessThanChip, LessThanConfig};

// // use ff::Field;
// use eth_types::Field;
// // use halo2_proofs::{circuit::Value, halo2curves::bn256::Fr as Fp};
// use halo2_proofs::{circuit::*, plonk::*};

// #[derive(Default)]

// // define circuit struct using array of usernames and balances
// struct MyCircuit<F: Field> {
//     pub input: Vec<Value<F>>,
//     pub input2: Vec<Value<F>>,
//     pub table: Vec<Value<F>>,
//     pub table2: Vec<Value<F>>,
//     pub table4: Vec<Value<F>>,
//     pub table6: Vec<Value<F>>,
// }

// impl<F: Field> Circuit<F> for MyCircuit<F> {
//     type Config = LessThanConfig;
//     type FloorPlanner = SimpleFloorPlanner;

//     fn without_witnesses(&self) -> Self {
//         Self::default()
//     }

//     fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//         let input = meta.advice_column();
//         let input2 = meta.advice_column();
//         let table: Column<Advice> = meta.advice_column();
//         let table2: Column<Advice> = meta.advice_column();

//         LessThanChip::configure(meta, input, input2, table, table2)
//     }

//     fn synthesize(
//         &self,
//         config: Self::Config,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(), Error> {
//         // We create a new instance of chip using the config passed as input
//         let chip = LessThanChip::<F>::construct(config);

//         // assign value to the chip
//         let input = self.input.clone();
//         let input2 = self.input2.clone();
//         let table = self.table.clone();
//         let table2 = self.table2.clone();
//         let table4 = self.table4.clone();
//         let table6 = self.table6.clone();

//         // // only select 12 values from self.input
//         // let input = [Value::known(F::from(0)); 20];
//         // let mut input_vec: Vec<Value<F>> = input.to_vec();
//         // input_vec[0] = self.input[0].clone();
//         // input_vec[1] = self.input[2].clone();
//         // input_vec[2] = self.input[4].clone();
//         // input_vec[3] = self.input[6].clone();

//         let _ = chip.assign(
//             layouter.namespace(|| "init table"),
//             input,
//             input2,
//             table,
//             table2,
//             table4,
//             table6,
//         );

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::MyCircuit;
//     use eth_types::Field;
//     use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::bn256::Fr as Fp};
//     #[test]
//     fn test_less_than_2() {
//         let k = 10;

//         // initate value
//         let mut value = [Value::known(Fp::from(8)); 10];
//         let mut value2 = [Value::known(Fp::from(8)); 10];
//         value[0] = Value::known(Fp::from(0 as u64));
//         value[2] = Value::known(Fp::from(4 as u64));
//         value[4] = Value::known(Fp::from(2 as u64));
//         value[6] = Value::known(Fp::from(6 as u64));

//         value2[0] = Value::known(Fp::from(0 as u64));
//         value2[2] = Value::known(Fp::from(6 as u64));
//         value2[4] = Value::known(Fp::from(3 as u64));
//         value2[6] = Value::known(Fp::from(9 as u64));

//         let mut table = [Value::known(Fp::from(0)), Value::known(Fp::from(2))];
//         let mut table2 = [Value::known(Fp::from(0)), Value::known(Fp::from(3))];
//         let mut table4 = [Value::known(Fp::from(4)), Value::known(Fp::from(6))];
//         let mut table6 = [Value::known(Fp::from(6)), Value::known(Fp::from(9))];
//         let circuit = MyCircuit::<Fp> {
//             input: value.to_vec(),
//             input2: value2.to_vec(),
//             table: table.to_vec(),
//             table2: table2.to_vec(),
//             table4: table4.to_vec(),
//             table6: table6.to_vec(),
//         };

//         // let target = 800;

//         // define public inputs looping from target to 0 and adding each value to pub_inputs vector
//         // let mut pub_inputs = vec![];
//         // for i in 700..target {
//         //     pub_inputs.push(Fp::from(i));
//         // }

//         // should verify as value is less than target
//         let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//         prover.assert_satisfied();

//         // // shouldn't verify as value is greater than target
//         // let target_2 = 754;

//         // let mut pub_inputs_2 = vec![];
//         // for i in 0..target_2 {
//         //     pub_inputs_2.push(Fp::from(i));
//         // }

//         // let invalid_prover = MockProver::run(k, &circuit, vec![pub_inputs_2]).unwrap();

//         // assert!(invalid_prover.verify().is_err());
//     }
// }
