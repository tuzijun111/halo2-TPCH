// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use std::{default, marker::PhantomData};

// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// const N: usize = 5;

// // #[derive(Default)]
// // define circuit struct using array of usernames and balances

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: Selector,
//     q_accu: Selector,
//     value_l: Column<Advice>,
//     value_r: Column<Advice>,
//     check: Column<Advice>,
//     // col_out: Column<Advice>,
//     // l: Column<Advice>,
//     r: Column<Advice>,
//     // constant: Column<Fixed>,
//     pub instance: Column<Instance>,

//     lt: LtConfig<F, 8>,
// }

// #[derive(Debug, Clone)]
// pub struct TestChip<F: Field> {
//     config: TestCircuitConfig<F>,
// }

// impl<F: Field> TestChip<F> {
//     pub fn construct(config: TestCircuitConfig<F>) -> Self {
//         Self { config }
//     }

//     pub fn configure(meta: &mut ConstraintSystem<F>) -> TestCircuitConfig<F> {
//         let q_enable = meta.complex_selector();
//         let q_accu = meta.complex_selector();

//         let value_l = meta.advice_column();
//         let value_r = meta.advice_column();
//         let check = meta.advice_column();
//         // let col_out = meta.advice_column();
//         // let l = meta.advice_column();
//         let r = meta.advice_column();
//         let constant = meta.fixed_column();
//         let instance = meta.instance_column();

//         meta.enable_constant(constant);
//         // meta.enable_equality(col_out);
//         meta.enable_equality(value_l);
//         meta.enable_equality(r);
//         meta.enable_equality(instance);

//         let lt = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(value_l, Rotation::cur()),
//             |meta| meta.query_advice(value_r, Rotation::cur()),
//         );

//         meta.create_gate(
//             "verifies that `check` current confif = is_lt from LtChip ",
//             |meta| {
//                 let q_enable = meta.query_selector(q_accu);

//                 // This verifies lt(value_l::cur, value_r::cur) is calculated correctly
//                 let check = meta.query_advice(check, Rotation::cur());

//                 // value_l  |  value_r  | check   |  col_out | q_enable  |
//                 // ---------------------------------------------------------
//                 // 1        | 10        | true    |  1+11*0  |    1      |
//                 // 11       | 10        | false   |  1       |    1      |
//                 // 5        | 10        | true    |   1+11*0     |  0     |

//                 vec![q_enable * (lt.is_lt(meta, None) - check)]
//             },
//         );

//         meta.create_gate("accumulate constraint", |meta| {
//             let q_accu = meta.query_selector(q_accu);
//             let prev_b = meta.query_advice(r, Rotation::cur());
//             // let prev_c = meta.query_advice(col_c, Rotation::prev());
//             let a = meta.query_advice(value_l, Rotation::cur());
//             let b = meta.query_advice(r, Rotation::next());
//             let check = meta.query_advice(check, Rotation::cur());
//             // let c = meta.query_advice(col_c, Rotation::cur());

//             // Previous accumulator amount + new value from a_cell
//             vec![q_accu * (a * check + prev_b - b)]
//             // vec![s * (a - b)]
//         });

//         TestCircuitConfig {
//             q_enable,
//             q_accu,
//             value_l,
//             value_r,
//             check,
//             lt,
//             // col_out,
//             // l,
//             r,
//             // constant,
//             instance,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         a: [u64; N],
//         b: u64,
//         check: [bool; N],
//         // c: F,
//     ) -> Result<AssignedCell<F, F>, Error> {
//         let chip = LtChip::construct(self.config.lt);
//         // let mut prev_b;
//         chip.load(layouter)?;

//         layouter.assign_region(
//             || "witness1",
//             |mut region| {
//                 for i in 0..N {
//                     region.assign_advice(
//                         || "value left",
//                         self.config.value_l,
//                         i,
//                         || Value::known(F::from(a[i])),
//                         // Value::known(F::from(v_value_l)),
//                     )?;

//                     region.assign_advice(
//                         || "value right",
//                         self.config.value_r,
//                         i,
//                         || Value::known(F::from(b)),
//                     )?;

//                     region.assign_advice(
//                         || "check",
//                         self.config.check,
//                         i,
//                         || Value::known(F::from(check[i] as u64)),
//                     )?;

//                     // if i != 0 {
//                     //     config.q_enable.enable(&mut region, i)?;
//                     // }
//                     self.config.q_enable.enable(&mut region, i)?;

//                     // let scalar_value = v.into_bits();
//                     chip.assign(&mut region, i, F::from(a[i]), F::from(b))?;
//                 }

//                 let mut prev_b = region.assign_advice_from_constant(
//                     || "first accu",
//                     self.config.r,
//                     0,
//                     F::zero(),
//                 )?;

//                 // let mut prev_b = b0_cell.clone();
//                 for row in 1..N + 1 {
//                     // enable hash selector
//                     // if row != N {
//                     //     config.q_accu.enable(&mut region, row)?;
//                     // }
//                     self.config.q_accu.enable(&mut region, row - 1)?;

//                     // assigning two columns of accumulating value
//                     // }

//                     let b_cell: AssignedCell<F, F> = region.assign_advice(
//                         || "sum_hi",
//                         self.config.r,
//                         row,
//                         || {
//                             prev_b.value().copied()
//                                 + Value::known(F::from(a[row - 1] * (check[row - 1] as u64)))
//                         },
//                     )?;
//                     prev_b = b_cell;
//                     // println!(
//                     //     "show: {:?}, {:?}",
//                     //     prev_b.value().copied(),
//                     //     Value::known(F::from(a[row - 1]))
//                     // );
//                 }
//                 Ok(prev_b)
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

// struct MyCircuit<F> {
//     pub value_l: [u64; N],
//     // pub value_l: Vec<Value<F>>,
//     pub value_r: u64,
//     pub check: [bool; N],
//     // pub l: u64,
//     // pub r: u64,
//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             value_l: [0; N],             // Initialize the array with default values if necessary
//             value_r: Default::default(), // You can use the default value for u64
//             check: [false; N],           // You can use the default value for [bool; 4]
//             // l: Default::default(),
//             // r: Default::default(),
//             _marker: PhantomData,
//         }
//     }
// }

// impl<F: Field> Circuit<F> for MyCircuit<F> {
//     type Config = TestCircuitConfig<F>;
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

//         let out_b_cell = test_chip.assign(&mut layouter, self.value_l, self.value_r, self.check)?;

//         test_chip.expose_public(&mut layouter, out_b_cell, 0)?;

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
//         let k = 18;

//         // initate usernames and balances array
//         let mut value_l: [u64; N] = [1; N];

//         value_l[0] = 1000;

//         let value_r: u64 = 256;
//         // let check: [bool; 4] = [true, true, false, false];
//         let mut check: [bool; N] = [true; N];
//         check[0] = false;

//         let circuit = MyCircuit::<Fp> {
//             value_l,
//             value_r,
//             check,
//             _marker: PhantomData,
//         };

//         let z = [Fp::from(4)];

//         let prover = MockProver::run(k, &circuit, vec![z.to_vec()]).unwrap();
//         prover.assert_satisfied();

//         // let params: Params<EqAffine> = halo2_proofs::poly::commitment::Params::new(k);

//         // // Generate verification key.
//         // println!("Generating Verification Key");
//         // let vk = keygen_vk(&params, &circuit).unwrap();
//         // // println!("vk: {:?}", vk);

//         // // // Generate proving key.
//         // println!("Generating Proving Key from Verification Key");
//         // let pk = keygen_pk(&params, vk, &circuit).unwrap();

//         // let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);

//         // // println!("{:?}", &[&[&[z]]]);

//         // println!("Generating Proof!");
//         // create_proof(
//         //     &params,
//         //     &pk,
//         //     &[circuit],
//         //     &[&[&[z]]],
//         //     &mut OsRng,
//         //     &mut transcript,
//         // )
//         // .expect("Failed to create proof!");

//         // // ANCHOR_END: create-proof
//         // // ANCHOR: write-proof

//         // let proof_path = "./proof";
//         // let proof = transcript.finalize();
//         // File::create(Path::new(proof_path))
//         //     .expect("Failed to create proof file")
//         //     .write_all(&proof[..])
//         //     .expect("Failed to write proof");
//         // println!("Proof written to: {}", proof_path);

//         // // ANCHOR_END: write-proof
//         // // ANCHOR: verify-proof

//         // let mut transcript_proof: Blake2bRead<
//         //     &[u8],
//         //     EqAffine,
//         //     halo2_proofs::transcript::Challenge255<EqAffine>,
//         // > = Blake2bRead::init(&proof[..]);
//         // // println!("{:?}", transcript_proof);

//         // // println!("Instances: {:?}", &[&[&[z]]]);

//         // // Verify the proof
//         // println!("Verifying Proof");
//         // let verified_proof_result = verify_proof(
//         //     &params,
//         //     pk.get_vk(),
//         //     SingleVerifier::new(&params),
//         //     &[&[&[z]]],
//         //     &mut transcript_proof,
//         // );

//         // // println!("{:#?}", transcript_proof);    //seems the transcript_proof becomes empty

//         // // Print "OK(())" if the proof is valid or an error message otherwise.
//         // if verified_proof_result.is_ok() {
//         //     println!("Proof verified!");
//         // } else {
//         //     println!(
//         //         "Proof verification failed! {}",
//         //         verified_proof_result.err().unwrap()
//         //     );
//         // }
//     }
// }
