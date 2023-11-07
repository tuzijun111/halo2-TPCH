// use eth_types::Field;
// // use gadgets::less_than::{LChip, LConfig, LInstruction}; // not the less_than.rs in the chips folder
// use gadgets::less_than_copy::{LChip, LConfig, LInstruction};
// use std::marker::PhantomData;

// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// const N: usize = 5;

// // #[derive(Default)]
// // define circuit struct using array of usernames and balances
// struct MyCircuit<F> {
//     pub value: [u64; N],
//     pub l: u64,
//     pub r: u64,
//     pub check1: [bool; N],
//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             value: [0; N], // Initialize the array with default values if necessary
//             l: Default::default(),
//             r: Default::default(),
//             check1: [false; N],
//             _marker: PhantomData,
//         }
//     }
// }

// #[derive(Clone, Debug)]
// struct TestCircuitConfig<F> {
//     q_enable: Selector,
//     value: Column<Advice>,
//     l: Column<Advice>,
//     r: Column<Advice>,
//     check1: Column<Advice>,
//     // check2: Column<Advice>,
//     lt_1: LConfig<F, 8>,
//     // lt_2: LConfig<F, 8>,
// }

// impl<F: Field> Circuit<F> for MyCircuit<F> {
//     type Config = TestCircuitConfig<F>;
//     type FloorPlanner = SimpleFloorPlanner;

//     fn without_witnesses(&self) -> Self {
//         Self::default()
//     }

//     fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
//         let q_enable = meta.complex_selector();

//         let value = meta.advice_column();
//         let l = meta.advice_column();
//         let r = meta.advice_column();
//         let check1 = meta.advice_column();
//         // let check2 = meta.advice_column();

//         let lt_1 = LChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(l, Rotation::cur()),
//             |meta| meta.query_advice(value, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         // let lt_2 = LChip::configure(
//         //     meta,
//         //     |meta| meta.query_selector(q_enable),
//         //     |meta| meta.query_advice(value, Rotation::cur()),
//         //     |meta| meta.query_advice(r, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         // );

//         let config = Self::Config {
//             q_enable,
//             value,
//             check1,
//             // check2,
//             lt_1,
//             // lt_2,
//             l,
//             r,
//         };

//         meta.create_gate(
//             "verifies that `check` current confif = is_lt from LChip ",
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);

//                 // This verifies lt(value_l::cur, value_r::cur) is calculated correctly
//                 let check1 = meta.query_advice(config.check1, Rotation::cur());

//                 vec![
//                     q_enable
//                         * ((config.lt_1.is_lt(meta, None) * config.lt_2.is_lt(meta, None))
//                             - check1),
//                 ]

//                 // vec![
//                 //     q_enable.clone() * (config.lt_1.is_lt(meta, None) - check1.clone()),
//                 //     q_enable * (config.lt_2.is_lt(meta, None) - check1),
//                 // ]
//             },
//         );

//         let num = meta.num_advice_columns();
//         println!("Number of columns: {}", num);

//         config
//     }

//     fn synthesize(
//         &self,
//         config: Self::Config,
//         mut layouter: impl Layouter<F>,
//     ) -> Result<(), Error> {
//         let chip1 = LChip::construct(config.lt_1);
//         let chip2 = LChip::construct(config.lt_2);

//         chip1.load(&mut layouter)?;
//         chip2.load(&mut layouter)?;

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 for i in 0..self.value.len() {
//                     region.assign_advice(
//                         || "value",
//                         config.value,
//                         i,
//                         || Value::known(F::from(self.value[i])),
//                         // Value::known(F::from(v_value_l)),
//                     )?;

//                     region.assign_advice(
//                         || "l",
//                         config.l,
//                         i,
//                         || Value::known(F::from(self.l)),
//                         // Value::known(F::from(v_value_l)),
//                     )?;

//                     region.assign_advice(
//                         || "r",
//                         config.r,
//                         i,
//                         || Value::known(F::from(self.r)),
//                         // Value::known(F::from(v_value_l)),
//                     )?;

//                     region.assign_advice(
//                         || "check1",
//                         config.check1,
//                         i,
//                         // || Value::known(F::from(self.check[i] as u64) + F::from(1)),
//                         || Value::known(F::from(self.check1[i] as u64)),
//                     )?;

//                     // if i != 0 {
//                     //     config.q_enable.enable(&mut region, i)?;
//                     // }
//                     config.q_enable.enable(&mut region, i)?;

//                     // let scalar_value = v.into_bits();
//                     chip1.assign(&mut region, i, F::from(self.l), F::from(self.value[i]))?;

//                     chip2.assign(&mut region, i, F::from(self.value[i]), F::from(self.r))?;
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
//         let mut value: [u64; N] = [1; N];
//         value[0] = 1000;
//         let l: u64 = 0;
//         let r: u64 = 256;
//         // let check: [bool; 4] = [true, true, false, false];
//         let mut check1: [bool; N] = [true; N];
//         check1[0] = false;

//         let circuit = MyCircuit::<Fp> {
//             value,
//             l,
//             r,
//             check1,
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
