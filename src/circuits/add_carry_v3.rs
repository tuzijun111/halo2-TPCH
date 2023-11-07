// use eth_types::Field;
// use halo2_proofs::{circuit::*, plonk::*};

// use super::super::chips::add_carry_v3::{AddCarryChip, AddCarryConfig};

// #[derive(Default)]
// struct AddCarryCircuit<F: Field> {
//     pub a: Vec<Value<F>>,
// }

// impl<F: Field> Circuit<F> for AddCarryCircuit<F> {
//     type Config = AddCarryConfig<F>;
//     type FloorPlanner = SimpleFloorPlanner;

//     fn without_witnesses(&self) -> Self {
//         Self::default()
//     }

//     fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//         let col_a = meta.advice_column();
//         let col_b = meta.advice_column();
//         // let col_c = meta.advice_column();
//         let constant = meta.fixed_column();
//         let carry_selector = meta.complex_selector();
//         let instance = meta.instance_column();
//         meta.enable_equality(instance);
//         meta.enable_equality(col_b);

//         AddCarryChip::configure(meta, [col_a, col_b], constant, carry_selector, instance)
//     }

//     fn synthesize(
//         &self,
//         config: Self::Config,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(), Error> {
//         let chip = AddCarryChip::construct(config);

//         let (_, mut prev_b) = chip.assign_first_row(layouter.namespace(|| "load first row"))?;

//         for (i, a) in self.a.iter().enumerate() {
//             let (_, v_b) = chip.assign_advice_row(
//                 layouter.namespace(|| format!("load row {}", i)),
//                 *a,
//                 prev_b,
//                 5,
//             )?;
//             prev_b = v_b;
//         }

//         // // check computation result
//         // chip.expose_public(layouter.namespace(|| "carry check"), &prev_b, 0)?;

//         // let (_, b_cell) = chip.assign_advice_row(
//         //     layouter.namespace(|| format!("load row ")),
//         //     self.a[0],
//         //     prev_b,
//         // )?;

//         // check computation result
//         // chip.expose_public(layouter.namespace(|| "carry check"), &prev_b, 0)?;
//         chip.expose_public(layouter.namespace(|| "carry check"), &prev_b, 0)?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::AddCarryCircuit;
//     use halo2_proofs::{
//         circuit::Value,
//         dev::{FailureLocation, MockProver, VerifyFailure},
//         halo2curves::bn256::Fr as Fp,
//         plonk::Any,
//     };

//     #[test]
//     fn test_carry_1() {
//         let k = 6;

//         // a: new value
//         let a = vec![
//             // Value::known(Fp::from((1 << 16) - 1)),
//             Value::known(Fp::from(1)),
//             Value::known(Fp::from(2)),
//             Value::known(Fp::from(3)),
//             Value::known(Fp::from(4)),
//         ];
//         let public_inputs = vec![Fp::from(10)]; // initial accumulated values

//         let circuit = AddCarryCircuit::<Fp> { a };
//         let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
//         prover.assert_satisfied();
//         assert_eq!(prover.verify(), Ok(()));
//     }
// }
