// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use std::{default, marker::PhantomData};

// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// const N: usize = 1000;

// // #[derive(Default)]
// // define circuit struct using array of usernames and balances

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: Selector,
//     q_accu: Selector,
//     l_extendedprice: Column<Advice>,
//     l_discount: Column<Advice>,
//     left_discount: Column<Advice>,
//     right_discount: Column<Advice>,
//     check: Column<Advice>,
//     sum_v: Column<Advice>, // for storing accumulated sum

//     l_shipdate: Column<Advice>,
//     l_quantity: Column<Advice>,

//     pub instance: Column<Instance>,

//     lt_left_discount: LtConfig<F, 8>,
//     lt_right_discount: LtConfig<F, 8>,
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

//         let l_extendedprice = meta.advice_column();
//         let l_discount = meta.advice_column();
//         let left_discount = meta.advice_column();
//         let right_discount = meta.advice_column();

//         let check = meta.advice_column();
//         let sum_v = meta.advice_column();

//         let l_shipdate = meta.advice_column();

//         let l_quantity = meta.advice_column();

//         let constant = meta.fixed_column();
//         let instance = meta.instance_column();

//         meta.enable_equality(l_extendedprice);
//         meta.enable_equality(l_discount);
//         meta.enable_equality(l_shipdate);
//         meta.enable_equality(l_quantity);
//         // meta.enable_equality(left_discount);
//         // meta.enable_equality(right_discount);
//         meta.enable_equality(instance);
//         meta.enable_equality(sum_v);
//         meta.enable_constant(constant);

//         let lt_left_discount = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(left_discount, Rotation::cur()),
//             |meta| meta.query_advice(l_discount, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         let lt_right_discount = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(l_discount, Rotation::cur()),
//             |meta| meta.query_advice(right_discount, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         let lt_quantity = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(l_quantity, Rotation::cur()),
//             |meta| meta.query_advice(right_quantity, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         meta.create_gate(
//             "verifies that `check` current confif = is_lt from LChip ",
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);

//                 // This verifies lt(value_l::cur, value_r::cur) is calculated correctly
//                 let check = meta.query_advice(check, Rotation::cur());

//                 vec![
//                     q_enable
//                         * ((lt_left_discount.is_lt(meta, None)
//                             * lt_right_discount.is_lt(meta, None)
//                             * lt_left_shipdate.is_lt(meta, None)
//                             * lt_right_shipdate.is_lt(meta, None)
//                             * lt_quantity.is_lt(meta, None))
//                             - check),
//                 ]
//             },
//         );

//         meta.create_gate("accumulate constraint", |meta| {
//             let q_accu = meta.query_selector(q_accu);
//             let prev_b = meta.query_advice(sum_v, Rotation::cur());
//             // let prev_c = meta.query_advice(col_c, Rotation::prev());
//             let extendedprice = meta.query_advice(l_extendedprice, Rotation::cur());
//             let discount = meta.query_advice(l_discount, Rotation::cur());
//             let sum_v = meta.query_advice(sum_v, Rotation::next());
//             let check = meta.query_advice(check, Rotation::cur());
//             // let c = meta.query_advice(col_c, Rotation::cur());

//             // Previous accumulator amount + new value (a) from a_cell
//             vec![q_accu * ((extendedprice * discount) * check + prev_b - sum_v)]
//         });

//         let num = meta.num_advice_columns();
//         println!("Number of columns: {}", num);

//         TestCircuitConfig {
//             q_enable,
//             q_accu,
//             l_quantity,
//             right_quantity,
//             l_shipdate,
//             left_shipdate,
//             right_shipdate,
//             l_extendedprice,
//             l_discount,
//             left_discount,
//             right_discount,
//             check,
//             instance,
//             sum_v,
//             lt_left_discount,
//             lt_right_discount,
//             lt_left_shipdate,
//             lt_right_shipdate,
//             lt_quantity,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         l_extendedprice: [u64; N],
//         l_discount: [u64; N], // l_discount
//         left_discount: u64,   // left_discount
//         right_discount: u64,  // right_discount
//         l_shipdate: [u64; N],
//         left_shipdate: u64,
//         right_shipdate: u64,
//         l_quantity: [u64; N],
//         right_quantity: u64,

//         check: [bool; N],
//     ) -> Result<AssignedCell<F, F>, Error> {
//         let chip1 = LtChip::construct(self.config.lt_left_discount);
//         let chip2 = LtChip::construct(self.config.lt_right_discount);
//         let chip3 = LtChip::construct(self.config.lt_left_shipdate);
//         let chip4 = LtChip::construct(self.config.lt_right_shipdate);
//         let chip5 = LtChip::construct(self.config.lt_quantity);

//         chip1.load(layouter)?;
//         chip2.load(layouter)?;
//         chip3.load(layouter)?;
//         chip4.load(layouter)?;
//         chip5.load(layouter)?;

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 for i in 0..N {
//                     region.assign_advice(
//                         || "l_extendedprice value",
//                         self.config.l_extendedprice,
//                         i,
//                         || Value::known(F::from(l_extendedprice[i])),
//                     )?;

//                     region.assign_advice(
//                         || "l_discount value",
//                         self.config.l_discount,
//                         i,
//                         || Value::known(F::from(l_discount[i])),
//                     )?;

//                     region.assign_advice(
//                         || "left_discount value",
//                         self.config.left_discount,
//                         i,
//                         || Value::known(F::from(left_discount)),
//                     )?;

//                     region.assign_advice(
//                         || "right_discount value",
//                         self.config.right_discount,
//                         i,
//                         || Value::known(F::from(right_discount)),
//                     )?;

//                     region.assign_advice(
//                         || "l_shipdate value",
//                         self.config.l_shipdate,
//                         i,
//                         || Value::known(F::from(l_shipdate[i])),
//                     )?;

//                     region.assign_advice(
//                         || "left_shipdate value",
//                         self.config.left_shipdate,
//                         i,
//                         || Value::known(F::from(left_shipdate)),
//                     )?;

//                     region.assign_advice(
//                         || "right_shipdate value",
//                         self.config.right_shipdate,
//                         i,
//                         || Value::known(F::from(right_shipdate)),
//                     )?;

//                     region.assign_advice(
//                         || "l_quantity value",
//                         self.config.l_quantity,
//                         i,
//                         || Value::known(F::from(l_quantity[i])),
//                     )?;

//                     region.assign_advice(
//                         || "right_quantity value",
//                         self.config.right_quantity,
//                         i,
//                         || Value::known(F::from(right_quantity)),
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
//                     chip1.assign(
//                         &mut region,
//                         i,
//                         F::from(left_discount),
//                         F::from(l_discount[i]),
//                     )?;

//                     chip2.assign(
//                         &mut region,
//                         i,
//                         F::from(l_discount[i]),
//                         F::from(right_discount),
//                     )?;

//                     chip3.assign(
//                         &mut region,
//                         i,
//                         F::from(left_shipdate),
//                         F::from(l_shipdate[i]),
//                     )?;

//                     chip4.assign(
//                         &mut region,
//                         i,
//                         F::from(l_shipdate[i]),
//                         F::from(right_shipdate),
//                     )?;

//                     chip5.assign(
//                         &mut region,
//                         i,
//                         F::from(l_quantity[i]),
//                         F::from(right_quantity),
//                     )?;
//                 }

//                 let mut prev_b = region.assign_advice_from_constant(
//                     || "first accu",
//                     self.config.sum_v,
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

//                     let b_cell: AssignedCell<F, F> = region.assign_advice(
//                         || "sum_hi",
//                         self.config.sum_v,
//                         row,
//                         || {
//                             prev_b.value().copied()
//                                 + Value::known(F::from(
//                                     l_extendedprice[row - 1]
//                                         * l_discount[row - 1]
//                                         * (check[row - 1] as u64),
//                                 ))
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
//     pub l_extendedprice: [u64; N],
//     pub l_discount: [u64; N],
//     pub left_discount: u64,
//     pub right_discount: u64,
//     pub l_shipdate: [u64; N],
//     pub left_shipdate: u64,
//     pub right_shipdate: u64,
//     pub l_quantity: [u64; N],
//     pub right_quantity: u64,
//     pub check: [bool; N],
//     // pub l: u64,
//     // pub r: u64,
//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             l_extendedprice: [0; N],
//             l_discount: [0; N], // Initialize the array with default values if necessary
//             left_discount: Default::default(), // You can use the default value for u64
//             right_discount: Default::default(),
//             l_shipdate: [0; N],
//             left_shipdate: Default::default(),
//             right_shipdate: Default::default(),
//             l_quantity: [0; N],
//             right_quantity: Default::default(),

//             check: [false; N], // You can use the default value for [bool; 4]
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

//         let out_b_cell = test_chip.assign(
//             &mut layouter,
//             self.l_extendedprice,
//             self.l_discount,
//             self.left_discount,
//             self.right_discount,
//             self.l_shipdate,
//             self.left_shipdate,
//             self.right_shipdate,
//             self.l_quantity,
//             self.right_quantity,
//             self.check,
//         )?;

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
//     fn test_1() {
//         let k = 18;

//         // initate inputes
//         let mut l_extendedprice: [u64; N] = [1; N];
//         let mut l_discount: [u64; N] = [2; N];
//         let left_discount = 1;
//         let right_discount = 5;
//         let mut l_shipdate: [u64; N] = [2; N];
//         let left_shipdate = 1;
//         let right_shipdate = 5;
//         let mut l_quantity: [u64; N] = [2; N];
//         let right_quantity = 5;

//         l_discount[0] = 1000;

//         // let mut l_discount: Vec<u64> = Vec::new();
//         // for i in 0..N {
//         //     if i == 0 {
//         //         l_discount.push(1000);
//         //     } else {
//         //         l_discount.push(1);
//         //     }
//         // }

//         let mut check: [bool; N] = [true; N];
//         check[0] = false;

//         let circuit = MyCircuit::<Fp> {
//             l_extendedprice,
//             l_discount,
//             left_discount,
//             right_discount,
//             l_shipdate,
//             left_shipdate,
//             right_shipdate,
//             l_quantity,
//             right_quantity,
//             check,
//             _marker: PhantomData,
//         };

//         let z = [Fp::from(2 * (N as u64 - 1))];

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
