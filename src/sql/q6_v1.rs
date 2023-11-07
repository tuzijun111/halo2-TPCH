// use eth_types::Field;
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction}; // not the less_than.rs in the chips folder
// use std::marker::PhantomData;

// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// const N: usize = 100;

// // #[derive(Default)]
// // define circuit struct using array of usernames and balances
// struct MyCircuit<F> {
//     pub value_l: [u64; N],
//     pub value_r: u64,
//     pub check: [bool; N],
//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             value_l: [0; N],             // Initialize the array with default values if necessary
//             value_r: Default::default(), // You can use the default value for u64
//             check: [true; N],            // You can use the default value for [bool; 4]
//             _marker: PhantomData,
//         }
//     }
// }

// #[derive(Clone, Debug)]
// struct TestCircuitConfig<F> {
//     q_enable: Selector,
//     value_l: Column<Advice>,
//     value_r: Column<Advice>,
//     check: Column<Advice>,
//     col_out: Column<Advice>,

//     lt: LtConfig<F, 8>,
// }

// impl<F: Field> Circuit<F> for MyCircuit<F> {
//     type Config = TestCircuitConfig<F>;
//     type FloorPlanner = SimpleFloorPlanner;

//     fn without_witnesses(&self) -> Self {
//         Self::default()
//     }

//     fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//         let q_enable = meta.complex_selector();

//         let value_l = meta.advice_column();
//         let value_r = meta.advice_column();
//         let check = meta.advice_column();
//         let col_out = meta.advice_column();

//         let lt = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(value_l, Rotation::cur()),
//             |meta| meta.query_advice(value_r, Rotation::cur()),
//         );

//         let config = Self::Config {
//             q_enable,
//             value_l,
//             value_r,
//             check,
//             lt,
//             col_out,
//         };

//         meta.create_gate(
//             "verifies that `check` current confif = is_lt from LtChip ",
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);

//                 // This verifies lt(value_l::cur, value_r::cur) is calculated correctly
//                 let check = meta.query_advice(config.check, Rotation::cur());

//                 // value_l  |  value_r  | check   |  col_out | q_enable  |
//                 // ---------------------------------------------------------
//                 // 1        | 10        | true    |  1+11*0  |    1      |
//                 // 11       | 10        | false   |  1       |    1      |
//                 // 5        | 10        | true    |   1+11*0     |  0     |

//                 vec![q_enable * (config.lt.is_lt(meta, None) - check)]
//             },
//         );

//         config
//     }

//     fn synthesize(
//         &self,
//         config: Self::Config,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(), Error> {
//         let chip = LtChip::construct(config.lt);

//         chip.load(&mut layouter)?;

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 for (i, v) in self.value_l.iter().enumerate() {
//                     region.assign_advice(
//                         || "value left",
//                         config.value_l,
//                         i,
//                         || Value::known(F::from(self.value_l[i])),
//                         // Value::known(F::from(v_value_l)),
//                     )?;

//                     region.assign_advice(
//                         || "value right",
//                         config.value_r,
//                         i,
//                         || Value::known(F::from(self.value_r)),
//                     )?;

//                     region.assign_advice(
//                         || "check",
//                         config.check,
//                         i,
//                         // || Value::known(F::from(self.check[i] as u64) + F::from(1)),
//                         || Value::known(F::from(self.check[i] as u64)),
//                     )?;

//                     // if i != 0 {
//                     //     config.q_enable.enable(&mut region, i)?;
//                     // }
//                     config.q_enable.enable(&mut region, i)?;

//                     // let scalar_value = v.into_bits();
//                     chip.assign(
//                         &mut region,
//                         i,
//                         F::from(self.value_l[i]),
//                         F::from(self.value_r),
//                     )?;
//                 }
//                 Ok(())
//             },
//         )
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
//     fn test_less_than_2() {
//         let k = 9;

//         // initate usernames and balances array
//         let value_l: [u64; N] = [1; N];
//         let value_r: u64 = 256;
//         // let check: [bool; 4] = [true, true, false, false];
//         let check: [bool; N] = [true; N];

//         let circuit = MyCircuit::<Fp> {
//             value_l,
//             value_r,
//             check,
//             _marker: PhantomData,
//         };

//         let prover = MockProver::run(k, &circuit, vec![]).unwrap();
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
