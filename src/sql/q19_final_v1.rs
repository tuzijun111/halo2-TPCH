// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::lessthan_or_equal::{LtEqChip, LtEqConfig, LtEqInstruction};
// use gadgets::lessthan_or_equal_generic::{
//     LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
// };

// use std::{default, marker::PhantomData};

// // use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
// use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
// use itertools::iproduct;
// use std::cmp::Ordering;
// use std::cmp::Reverse;
// use std::collections::HashSet;

// use std::mem;

// const N: usize = 1;
// const NUM_BYTES: usize = 3;

// // #[derive(Default)]
// // We should use the selector to skip the row which does not satisfy shipdate values

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: [Selector; 5],

//     lineitem: [Column<Advice>; 6], // l_partkey, l_shipmode, l_shipinstruct, l_extendedprice, l_discount, l_quantity
//     part: [Column<Advice>; 4],     // p_partkey, p_brand, p_container, p_size,

//     condition_equal: [Column<Advice>; 24], // p_brand = ':1' ...
//     condition_compare: [Column<Advice>; 12], // p_brand = ':1' ...

//     // check: [Column<Advice>; 1],
//     revenue: [Column<Advice>; 2], // for select l_orderkey from lineitem group by l_orderkey having sum(l_quantity) > :1

//     // groupby: [Column<Advice>; 2],
//     equal_condition: Vec<IsZeroConfig<F>>, // p_brand = ':1', p_container in ('SM CASE', 'SM BOX', 'SM PACK', 'SM PKG')   multiply 3
//     compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>, // p_size between 1 and 5
// }

// #[derive(Debug, Clone)]
// pub struct TestChip<F: Field> {
//     config: TestCircuitConfig<F>,
// }

// // conditions for filtering in tables: customer, orders,lineitem
// //   c_mktsegment = ':1', o_orderdate < date ':2', and l_shipdate > date ':2'

// // Circuits illustration
// // | l_orderkey |  l_extendedprice | l_discount | l_shipdate | ...
// // ------+-------+------------+------------------------+-------------------------------
// //    |     |       |         0              |  0

// impl<F: Field> TestChip<F> {
//     pub fn construct(config: TestCircuitConfig<F>) -> Self {
//         Self { config }
//     }

//     pub fn configure(meta: &mut ConstraintSystem<F>) -> TestCircuitConfig<F> {
//         let q_enable = [meta.complex_selector(); 5];

//         let lineitem = [meta.advice_column(); 6];

//         let part = [meta.advice_column(); 4];

//         let condition_equal = [meta.advice_column(); 24];
//         let condition_compare = [meta.advice_column(); 12];
//         let check = [meta.advice_column(); 3];

//         let revenue = [meta.advice_column(), meta.advice_column()];
//         let is_zero_advice_column = [meta.advice_column(); 24];

//         // let groupby = [meta.advice_column(), meta.advice_column()];

//         for i in 0..6 {
//             meta.enable_equality(lineitem[i]);
//         }
//         for i in 0..4 {
//             meta.enable_equality(part[i]);
//         }
//         for i in 0..24 {
//             meta.enable_equality(condition_equal[i]);
//         }
//         for i in 0..12 {
//             meta.enable_equality(condition_compare[i]);
//         }

//         let mut equal_condition = Vec::with_capacity(24);
//         for i in 0..3 {
//             let config = IsZeroChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| {
//                     meta.query_advice(part[1], Rotation::cur())
//                         - meta.query_advice(condition_equal[i], Rotation::cur())
//                 },
//                 is_zero_advice_column[i],
//             );

//             equal_condition.push(config);
//         }

//         for i in 3..15 {
//             let config = IsZeroChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| {
//                     meta.query_advice(part[2], Rotation::cur())
//                         - meta.query_advice(condition_equal[i], Rotation::cur())
//                 },
//                 is_zero_advice_column[i],
//             );

//             equal_condition.push(config);
//         }

//         for i in 15..21 {
//             let config = IsZeroChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| {
//                     meta.query_advice(lineitem[1], Rotation::cur())
//                         - meta.query_advice(condition_equal[i], Rotation::cur())
//                 },
//                 is_zero_advice_column[i],
//             );

//             equal_condition.push(config);
//         }

//         for i in 21..24 {
//             let config = IsZeroChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| {
//                     meta.query_advice(lineitem[2], Rotation::cur())
//                         - meta.query_advice(condition_equal[i], Rotation::cur())
//                 },
//                 is_zero_advice_column[i],
//             );

//             equal_condition.push(config);
//         }

//         let mut compare_condition = Vec::with_capacity(12);

//         for i in 0..3 {
//             let config = LtEqGenericChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| vec![meta.query_advice(condition_compare[i], Rotation::cur())],
//                 |meta| vec![meta.query_advice(part[3], Rotation::cur())],
//             );
//             compare_condition.push(config);
//         }
//         for i in 3..6 {
//             let config = LtEqGenericChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| vec![meta.query_advice(part[3], Rotation::cur())],
//                 |meta| vec![meta.query_advice(condition_compare[i], Rotation::cur())],
//             );
//             compare_condition.push(config);
//         }
//         for i in 6..9 {
//             let config = LtEqGenericChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| vec![meta.query_advice(condition_compare[i], Rotation::cur())],
//                 |meta| vec![meta.query_advice(lineitem[5], Rotation::cur())],
//             );
//             compare_condition.push(config);
//         }
//         for i in 9..12 {
//             let config = LtEqGenericChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_enable[0]),
//                 |meta| vec![meta.query_advice(lineitem[5], Rotation::cur())],
//                 |meta| vec![meta.query_advice(condition_compare[i], Rotation::cur())],
//             );
//             compare_condition.push(config);
//         }

//         TestCircuitConfig {
//             q_enable,
//             lineitem,
//             part,
//             condition_equal,
//             condition_compare,
//             revenue,
//             equal_condition,
//             compare_condition,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         lineitem: [[F; N]; 6],
//         part: [[F; N]; 4],
//         condition_equal: [F; 24],
//         condition_compare: [F; 12],
//     ) -> Result<(), Error> {
//         let mut compare_chip = Vec::with_capacity(12);
//         let mut equal_chip = Vec::with_capacity(24);
//         for i in 0..12 {
//             let chip = LtEqGenericChip::construct(self.config.compare_condition[i].clone());
//             chip.load(layouter)?;
//             compare_chip.push(chip);
//         }
//         for i in 0..24 {
//             let chip = IsZeroChip::construct(self.config.equal_condition[i].clone());
//             equal_chip.push(chip);
//         }

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 //assign input values
//                 for i in 0..lineitem.len() {
//                     for j in 0..lineitem[0].len() {
//                         region.assign_advice(
//                             || "l",
//                             self.config.lineitem[i],
//                             i,
//                             || Value::known(lineitem[i][j]),
//                         )?;
//                     }
//                 }

//                 for i in 0..part.len() {
//                     for j in 0..part[0].len() {
//                         region.assign_advice(
//                             || "p",
//                             self.config.part[i],
//                             i,
//                             || Value::known(part[i][j]),
//                         )?;
//                     }
//                 }
//                 // join prepare
//                 let p_combined: Vec<Vec<_>> = (0..part[0].len())
//                     .map(|i| part.iter().map(|row| row[i]).collect())
//                     .collect();
//                 let p_combined_1: Vec<Vec<F>> = p_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|v| {
//                         v[1] == condition_equal[0]
//                             && (v[2] == condition_equal[3]
//                                 || v[2] == condition_equal[6]
//                                 || v[2] == condition_equal[9]
//                                 || v[2] == condition_equal[12])
//                             && (condition_compare[0] <= v[3] && v[3] <= condition_compare[3])
//                     })
//                     .collect();
//                 let p_combined_2: Vec<Vec<F>> = p_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|v| {
//                         v[1] == condition_equal[1]
//                             && (v[2] == condition_equal[4]
//                                 || v[2] == condition_equal[7]
//                                 || v[2] == condition_equal[10]
//                                 || v[2] == condition_equal[13])
//                             && (condition_compare[1] <= v[3] && v[3] <= condition_compare[4])
//                     })
//                     .collect();
//                 let p_combined_3: Vec<Vec<F>> = p_combined
//                     .into_iter()
//                     .filter(|v| {
//                         v[1] == condition_equal[2]
//                             && (v[2] == condition_equal[5]
//                                 || v[2] == condition_equal[8]
//                                 || v[2] == condition_equal[11]
//                                 || v[2] == condition_equal[14])
//                             && (condition_compare[2] <= v[3] && v[3] <= condition_compare[5])
//                     })
//                     .collect();

//                 let l_combined: Vec<Vec<_>> = (0..lineitem[0].len())
//                     .map(|i| lineitem.iter().map(|row| row[i]).collect())
//                     .collect();

//                 let l_combined_1: Vec<Vec<F>> = l_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|v| {
//                         (v[1] == condition_equal[15] || v[1] == condition_equal[18])
//                             && (v[2] == condition_equal[21])
//                             && (condition_compare[6] <= v[5] && v[5] <= condition_compare[9])
//                     })
//                     .collect();

//                 let l_combined_2: Vec<Vec<F>> = l_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|v| {
//                         (v[1] == condition_equal[16] || v[1] == condition_equal[19])
//                             && (v[2] == condition_equal[22])
//                             && (condition_compare[7] <= v[5] && v[5] <= condition_compare[10])
//                     })
//                     .collect();

//                 let l_combined_3: Vec<Vec<F>> = l_combined
//                     .into_iter()
//                     .filter(|v| {
//                         (v[1] == condition_equal[17] || v[1] == condition_equal[20])
//                             && (v[2] == condition_equal[23])
//                             && (condition_compare[8] <= v[5] && v[5] <= condition_compare[11])
//                     })
//                     .collect();

//                 //create the values for join and disjoin
//                 let mut join_value: Vec<Vec<_>> = vec![Default::default(); 6];
//                 let mut disjoin_value: Vec<Vec<_>> = vec![Default::default(); 6];
//                 // c_custkey = o_custkey
//                 // and o_orderkey = l_orderkey

//                 let mut combined = Vec::new();
//                 combined.push(p_combined_1.clone()); // its length is 4
//                 combined.push(l_combined_1.clone()); // 6
//                 combined.push(p_combined_2.clone()); // its length is 4
//                 combined.push(l_combined_2.clone()); // 6
//                 combined.push(p_combined_3.clone()); // its length is 4
//                 combined.push(l_combined_3.clone()); // 6

//                 // (input1 index, input2 index, join attribute index of input1, join attribute of input2)
//                 let index = [
//                     (0, 1, 0, 0), //   p_partkey = l_partkey
//                     (2, 3, 0, 0), //   p_partkey = l_partkey
//                     (4, 5, 0, 0), //   p_partkey = l_partkey
//                 ];

//                 for i in 0..index.len() {
//                     for val in combined[index[i].0].iter() {
//                         if let Some(_) = combined[index[i].1]
//                             .iter()
//                             .find(|v| v[index[i].3] == val[index[i].2])
//                         {
//                             join_value[i * 2].push(val); // join values
//                         } else {
//                             disjoin_value[i * 2].push(val); // disjoin values
//                         }
//                     }
//                     for val in combined[index[i].1].iter() {
//                         if let Some(_) = combined[index[i].0]
//                             .iter()
//                             .find(|v| v[index[i].2] == val[index[i].3])
//                         {
//                             join_value[i * 2 + 1].push(val);
//                         } else {
//                             disjoin_value[i * 2 + 1].push(val);
//                         }
//                     }
//                 }

//                 fn join_vectors<F>(
//                     vectors: &[Vec<Vec<F>>],
//                     join_index: &[(usize, usize, usize, usize)],
//                 ) -> Vec<Vec<F>>
//                 where
//                     F: Clone + PartialEq,
//                 {
//                     let mut join_result = vectors[0].to_vec();
//                     for i in 1..vectors.len() {
//                         let mut next_join = Vec::new();
//                         for ab in join_result.iter() {
//                             for c in vectors[i].iter() {
//                                 if ab[join_index[i - 1].2] == c[join_index[i - 1].3] {
//                                     let mut joined = ab.to_vec();
//                                     joined.extend_from_slice(c);
//                                     next_join.push(joined);
//                                 }
//                             }
//                         }
//                         join_result = next_join;
//                     }

//                     join_result
//                 }

//                 let mut combined_vector_1 = Vec::new();
//                 let mut combined_vector_2 = Vec::new();
//                 let mut combined_vector_3 = Vec::new();
//                 combined_vector_1.extend_from_slice(&p_combined_1);
//                 combined_vector_1.extend_from_slice(&l_combined_1);
//                 combined_vector_2.extend_from_slice(&l_combined_2);
//                 combined_vector_2.extend_from_slice(&l_combined_2);
//                 combined_vector_3.extend_from_slice(&l_combined_3);
//                 combined_vector_3.extend_from_slice(&l_combined_3);
//                 let mut cartesian_product1 = join_vectors(&[combined_vector_1], &[index[0]]);
//                 let mut cartesian_product2 = join_vectors(&[combined_vector_2], &[index[1]]);
//                 let mut cartesian_product3 = join_vectors(&[combined_vector_3], &[index[2]]);

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
// }

// struct MyCircuit<F: Copy> {
//     lineitem: [[F; N]; 6],
//     part: [[F; N]; 4],
//     condition_equal: [F; 24],
//     condition_compare: [F; 12],

//     _marker: PhantomData<F>,
// }

// impl<F: Copy + Default> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             lineitem: [Default::default(); 6],
//             part: [Default::default(); 4],

//             condition_equal: [Default::default(); 24],
//             condition_compare: [Default::default(); 12],
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

//         let out_b_cells = test_chip.assign(
//             &mut layouter,
//             self.lineitem,
//             self.part,
//             self.condition_equal,
//             self.condition_compare,
//         )?;

//         // for (i, cell) in out_b_cells.iter().enumerate() {
//         //     test_chip.expose_public(&mut layouter, cell.clone(), i)?;
//         // }

//         // test_chip.expose_public(&mut layouter, out_b_cell, 0)?;

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::MyCircuit;
//     use super::N;
//     // use rand::Rng;
//     // use halo2_proofs::poly::commitment::Params
//     use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

//     use std::marker::PhantomData;

//     #[test]
//     fn test_1() {
//         let k = 10;
//         // let mut rng = rand::thread_rng();

//         let lineitem = [[Fp::from(2); N]; 6];
//         let part = [[Fp::from(2); N]; 4];
//         let condition_equal = [Fp::from(2); 24];
//         let condition_compare = [Fp::from(2); 12];

//         let circuit = MyCircuit::<Fp> {
//             lineitem,
//             part,
//             condition_equal,
//             condition_compare,

//             _marker: PhantomData,
//         };

//         // let prover = MockProver::run(k, &circuit, vec![z.to_vec()]).unwrap();
//         let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//         prover.assert_satisfied();
//     }
// }
// // time cargo test --package halo2-experiments --lib -- sql::q3_final_v1::tests::test_1 --exact --nocapture
