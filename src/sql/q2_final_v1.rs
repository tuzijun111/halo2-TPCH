// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::lessthan_or_equal::{LtEqChip, LtEqConfig, LtEqInstruction};
// use gadgets::lessthan_or_equal_generic::{
//     LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
// };

// use std::{default, marker::PhantomData};

// use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};
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
//     q_enable: [Selector; 5], // 0: iszero;
//     // q_perm: [Selector; 8],
//     q_cond: [Selector; 3],
//     q_sort: [Selector; 4], // s_acctbal desc, n_name, s_name, p_partkey;

//     part: [Column<Advice>; 4],     //  p_partkey, p_size, p_type, p_mfgr
//     partsupp: [Column<Advice>; 3], // ps_partkey, ps_suppkey, ps_supplycost
//     supplier: [Column<Advice>; 7], // s_name, s_acctbal, s_address, s_phone, s_comment, s_suppkey, s_nationkey
//     nation: [Column<Advice>; 3],   // n_name, n_nationkey, n_regionkey,
//     region: [Column<Advice>; 2],   // r_name, r_regionkey

//     condition: [Column<Advice>; 4], // sum(l_quantity) > :1
//     equal_check: [Column<Advice>; 3],

//     // let index = [
//     //     (0, 1, 0, 0), //   p_partkey = ps_partkey
//     //     (1, 2, 1, 5), // s_suppkey = ps_suppkey
//     //     (2, 3, 6, 1), //  s_nationkey = n_nationkey
//     //     (3, 4, 2, 1), // n_regionkey = r_regionkey
//     // ];
//     join_and_disjoin_part: [[Column<Advice>; 4]; 2],
//     join_and_disjoin_partsupp: [[Column<Advice>; 3]; 4],
//     join_and_disjoin_supplier: [[Column<Advice>; 7]; 4],
//     join_and_disjoin_nation: [[Column<Advice>; 3]; 4],
//     join_and_disjoin_region: [[Column<Advice>; 2]; 2],
//     //    the second join and only update p_ and ps_
//     //     (0, 1, 0, 0), //   p_partkey = ps_partkey
//     //     (1, 2, 1, 5), // s_suppkey = ps_suppkey
//     join_and_disjoin_part1: [[Column<Advice>; 4]; 2],
//     join_and_disjoin_partsupp1: [[Column<Advice>; 3]; 4],
//     // order by
//     // s_acctbal desc,
//     // n_name,
//     // s_name,
//     // p_partkey;
//     orderby: [Column<Advice>; 4],

//     equal_condition: Vec<IsZeroConfig<F>>, // p_size = :1, p_type like '%:2', r_name = ':3'
//     compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>, //
//     perm: Vec<PermAnyConfig>,
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
//         // let q_perm = [meta.complex_selector(); 8];
//         let q_cond = [meta.complex_selector(); 3];
//         let q_sort = [meta.complex_selector(); 4];

//         let part = [meta.advice_column(); 4];
//         let supplier = [meta.advice_column(); 7];
//         let partsupp = [meta.advice_column(); 3];
//         let nation = [meta.advice_column(); 3];
//         let region = [meta.advice_column(); 2];
//         let condition = [meta.advice_column(); 4];
//         let equal_check = [meta.advice_column(); 3];

//         let join_and_disjoin_part = [[meta.advice_column(); 4]; 2];
//         let join_and_disjoin_supplier = [[meta.advice_column(); 7]; 4];
//         let join_and_disjoin_partsupp = [[meta.advice_column(); 3]; 4];
//         let join_and_disjoin_nation = [[meta.advice_column(); 3]; 4];
//         let join_and_disjoin_region = [[meta.advice_column(); 2]; 2];

//         let join_and_disjoin_part1 = [[meta.advice_column(); 4]; 2];
//         let join_and_disjoin_partsupp1 = [[meta.advice_column(); 3]; 4];

//         let orderby = [meta.advice_column(); 4];

//         let is_zero_advice_column = [meta.advice_column(); 6];

//         for i in 0..2 {
//             meta.enable_equality(region[i]);
//         }

//         for i in 0..3 {
//             meta.enable_equality(partsupp[i]);
//             meta.enable_equality(nation[i]);
//         }

//         for i in 0..4 {
//             meta.enable_equality(part[i]);
//             meta.enable_equality(condition[i]);
//         }

//         for i in 0..7 {
//             meta.enable_equality(supplier[i]);
//         }

//         for i in 0..join_and_disjoin_part.len() {
//             for j in 0..join_and_disjoin_part[0].len() {
//                 meta.enable_equality(join_and_disjoin_part[i][j]);
//             }

//             for j in 0..join_and_disjoin_part1[0].len() {
//                 meta.enable_equality(join_and_disjoin_part[i][j]);
//             }

//             for j in 0..join_and_disjoin_region[0].len() {
//                 meta.enable_equality(join_and_disjoin_region[i][j]);
//             }
//         }

//         for i in 0..join_and_disjoin_supplier.len() {
//             for j in 0..join_and_disjoin_supplier[0].len() {
//                 meta.enable_equality(join_and_disjoin_supplier[i][j]);
//             }
//             for j in 0..join_and_disjoin_partsupp[0].len() {
//                 meta.enable_equality(join_and_disjoin_partsupp[i][j]);
//             }
//             for j in 0..join_and_disjoin_partsupp1[0].len() {
//                 meta.enable_equality(join_and_disjoin_partsupp[i][j]);
//             }
//             for j in 0..join_and_disjoin_nation[0].len() {
//                 meta.enable_equality(join_and_disjoin_nation[i][j]);
//             }
//         }

//         let mut equal_condition = Vec::with_capacity(3);
//         // and p_size = :1; condition[0]
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]),
//             |meta| {
//                 meta.query_advice(part[1], Rotation::cur())  // p_size
//                     - meta.query_advice(condition[0], Rotation::cur())
//             },
//             is_zero_advice_column[0],
//         );
//         equal_condition.push(config);
//         //p_type like '%:2'; condition[1]
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]),
//             |meta| {
//                 meta.query_advice(part[2], Rotation::cur())  // p_type
//                     - meta.query_advice(condition[1], Rotation::cur())
//             },
//             is_zero_advice_column[1],
//         );
//         equal_condition.push(config);
//         // r_name = ':3'; condition[2]
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]),
//             |meta| {
//                 meta.query_advice(region[0], Rotation::cur())  // r_name
//                     - meta.query_advice(condition[2], Rotation::cur())
//             },
//             is_zero_advice_column[2],
//         );
//         equal_condition.push(config);

//         // call PermAnyChips:configure
//         let mut perm = Vec::new();
//         let join_len = [4, 7, 3, 3, 2, 4, 3];
//         for v in join_len {
//             let mut input_columns = Vec::new();
//             let mut table_columns = Vec::new();
//             for _ in 0..v {
//                 input_columns.push(meta.advice_column());
//                 table_columns.push(meta.advice_column());
//             }
//             let config = PermAnyChip::configure(meta, input_columns, table_columns);
//             perm.push(config);
//         }

//         // constraints for s_acctbal desc, n_name, s_name, p_partkey;
//         let mut compare_condition = Vec::new();

//         // gate s_acctbal[i-1] >= s_acctbal[i-1]
//         let config = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| vec![meta.query_advice(orderby[0], Rotation::cur())],
//             |meta| vec![meta.query_advice(orderby[0], Rotation::prev())],
//         );
//         compare_condition.push(config);

//         // gate for s_acctbal[i-1] = s_acctbal[i]
//         let config1 = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| {
//                 meta.query_advice(orderby[0], Rotation::prev())  // p_size
//                     - meta.query_advice(orderby[0], Rotation::cur())
//             },
//             is_zero_advice_column[3], // s_acctbal[i] = s_acctbal[i-1]
//         );
//         equal_condition.push(config1.clone());
//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let q_sort = meta.query_selector(q_sort[0]);
//             let output = meta.query_advice(equal_check[0], Rotation::cur());
//             vec![
//                 q_sort.clone() * (config1.clone().expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 q_sort * (Expression::Constant(F::ONE) - config1.clone().expr()) * (output), // in this case output == 0
//             ]
//         });
//         // n_name[i-1] = n_name[i]
//         let config2 = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| {
//                 meta.query_advice(orderby[2], Rotation::prev())  // p_size
//                     - meta.query_advice(orderby[2], Rotation::cur())
//             },
//             is_zero_advice_column[4], // s_acctbal[i] = s_acctbal[i-1]
//         );
//         equal_condition.push(config2.clone());
//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let q_sort = meta.query_selector(q_sort[0]);
//             let output = meta.query_advice(equal_check[1], Rotation::cur());
//             vec![
//                 q_sort.clone() * (config2.clone().expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 q_sort * (Expression::Constant(F::ONE) - config2.clone().expr()) * (output), // in this case output == 0
//             ]
//         });

//         // s_name[i-1] = s_name[i]
//         let config3 = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| {
//                 meta.query_advice(orderby[1], Rotation::prev())  // p_size
//                     - meta.query_advice(orderby[1], Rotation::cur())
//             },
//             is_zero_advice_column[5], // s_acctbal[i] = s_acctbal[i-1]
//         );
//         equal_condition.push(config3.clone());
//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let q_sort = meta.query_selector(q_sort[0]);
//             let output = meta.query_advice(equal_check[2], Rotation::cur());
//             vec![
//                 q_sort.clone() * (config3.clone().expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 q_sort * (Expression::Constant(F::ONE) - config3.clone().expr()) * (output), // in this case output == 0
//             ]
//         });

//         // gate s_acctbal[i] = s_acctbal[i-1] and n_name[i] = n_name[i-1]
//         let config_lt_1 = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| vec![meta.query_advice(orderby[2], Rotation::prev())],
//             |meta| vec![meta.query_advice(orderby[2], Rotation::cur())],
//         );
//         compare_condition.push(config_lt_1.clone());
//         let config_lt_2 = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| vec![meta.query_advice(orderby[1], Rotation::prev())],
//             |meta| vec![meta.query_advice(orderby[1], Rotation::cur())],
//         );
//         compare_condition.push(config_lt_2.clone());
//         let config_lt_3 = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[0]),
//             |meta| vec![meta.query_advice(orderby[3], Rotation::prev())],
//             |meta| vec![meta.query_advice(orderby[3], Rotation::cur())],
//         );
//         compare_condition.push(config_lt_3.clone());

//         meta.create_gate(
//             "verifies three orderby scenarios",
//             |meta| {
//                 let q_sort = meta.query_selector(q_sort[0]);
//                 // let output = meta.query_advice(equal_check[0], Rotation::cur());
//                 vec![
//                     q_sort.clone()
//                         * (config1.expr() * config_lt_1.is_lt(meta, None)
//                             - Expression::Constant(F::ONE))   // or
//                         * (config1.expr() * config2.expr() * config_lt_2.is_lt(meta, None)
//                         - Expression::Constant(F::ONE))  // or
//                         * (config1.expr() * config2.expr() * config3.expr() * config_lt_3.is_lt(meta, None)
//                         - Expression::Constant(F::ONE))
//                 ]

//             },
//         );

//         // gate s_acctbal[i] = s_acctbal[i-1] and n_name[i] = n_name[i-1]

//         TestCircuitConfig {
//             q_enable,
//             q_cond,
//             q_sort,

//             part,
//             supplier,
//             partsupp,
//             nation,
//             region,
//             condition,
//             equal_check,
//             join_and_disjoin_part,
//             join_and_disjoin_supplier,
//             join_and_disjoin_partsupp,
//             join_and_disjoin_nation,
//             join_and_disjoin_region,
//             join_and_disjoin_part1,
//             join_and_disjoin_partsupp1,
//             orderby,
//             equal_condition,
//             compare_condition,
//             perm,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         part: Vec<Vec<F>>,
//         supplier: Vec<Vec<F>>,
//         partsupp: Vec<Vec<F>>,
//         nation: Vec<Vec<F>>,
//         regions: Vec<Vec<F>>,
//         condition: [F; 3],
//     ) -> Result<(), Error> {
//         // let mut compare_chip = Vec::with_capacity(3);
//         let mut equal_chip = Vec::with_capacity(3);
//         // for i in 0..3 {
//         //     let chip = LtEqGenericChip::construct(self.config.compare_condition[i].clone());
//         //     chip.load(layouter)?;
//         //     compare_chip.push(chip);
//         // }
//         for i in 0..3 {
//             let chip = IsZeroChip::construct(self.config.equal_condition[i].clone());
//             equal_chip.push(chip);
//         }

//         // do not need this chip to assign, I will assign them directly in this part
//         // let mut perm_chip = Vec::new();
//         // for i in 0..7 {
//         //     let chip: PermAnyChip<F> = PermAnyChip::construct(self.config.perm[i].clone());
//         //     perm_chip.push(chip);
//         // }

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 //assign input values

//                 for i in 0..part.len() {
//                     for j in 0..part[0].len() {
//                         region.assign_advice(
//                             || "p",
//                             self.config.part[j],
//                             i,
//                             || Value::known(part.clone()[i][j]),
//                         )?;
//                     }
//                 }

//                 for i in 0..supplier.len() {
//                     for j in 0..supplier[0].len() {
//                         region.assign_advice(
//                             || "s",
//                             self.config.supplier[j],
//                             i,
//                             || Value::known(supplier[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..partsupp.len() {
//                     for j in 0..partsupp[0].len() {
//                         region.assign_advice(
//                             || "ps",
//                             self.config.partsupp[j],
//                             i,
//                             || Value::known(partsupp[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..nation.len() {
//                     for j in 0..nation[0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.nation[j],
//                             i,
//                             || Value::known(nation[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..regions.len() {
//                     for j in 0..regions[0].len() {
//                         region.assign_advice(
//                             || "r",
//                             self.config.region[j],
//                             i,
//                             || Value::known(regions[i][j]),
//                         )?;
//                         // iszero chip assigns values
//                         equal_chip[0].assign(
//                             &mut region,
//                             i,
//                             Value::known(regions[i][0] - condition[0]),
//                         )?;
//                     }
//                 }

//                 let mut p_combined = part.clone();

//                 let s_combined = supplier.clone();
//                 let mut ps_combined = partsupp.clone();
//                 let n_combined = nation.clone();
//                 let mut r_combined = regions.clone();

//                 // println!("1:  {:?}", r_combined);

//                 let r_combined: Vec<Vec<_>> = r_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|row| row[0] == condition[2]) // r_name = ':3'
//                     .collect();

//                 // println!("2:  {:?}", r_combined);

//                 //     select
//                 //     min(ps_supplycost)
//                 // from
//                 //     partsupp,
//                 //     supplier,
//                 //     nation,
//                 //     region
//                 // where
//                 //     p_partkey = ps_partkey
//                 //     and s_suppkey = ps_suppkey
//                 //     and s_nationkey = n_nationkey
//                 //     and n_regionkey = r_regionkey
//                 //     and r_name = ':3'

//                 //create the values for join and disjoin
//                 let mut join_value: Vec<Vec<_>> = vec![vec![]; 8];
//                 let mut disjoin_value: Vec<Vec<_>> = vec![vec![]; 8];

//                 let mut combined = Vec::new();
//                 combined.push(p_combined.clone()); // its length is 4
//                 combined.push(ps_combined.clone()); // 3
//                 combined.push(s_combined.clone()); // 7
//                 combined.push(n_combined.clone()); // 3
//                 combined.push(r_combined.clone()); // 2

//                 // (input1 index, input2 index, join attribute index of input1, join attribute of input2)
//                 let index = [
//                     (0, 1, 0, 0), //   p_partkey = ps_partkey
//                     (1, 2, 1, 5), // s_suppkey = ps_suppkey
//                     (2, 3, 6, 1), //  s_nationkey = n_nationkey
//                     (3, 4, 2, 1), // n_regionkey = r_regionkey
//                 ];

//                 for i in 0..index.len() {
//                     for val in combined[index[i].0].iter() {
//                         if let Some(_) = combined[index[i].1]
//                             .iter()
//                             .find(|v| v[index[i].3] == val[index[i].2])
//                         {
//                             join_value[i * 2].push(val.clone()); // join values
//                         } else {
//                             disjoin_value[i * 2].push(val); // disjoin values
//                         }
//                     }
//                     for val in combined[index[i].1].iter() {
//                         if let Some(_) = combined[index[i].0]
//                             .iter()
//                             .find(|v| v[index[i].2] == val[index[i].3])
//                         {
//                             join_value[i * 2 + 1].push(val.clone());
//                         } else {
//                             disjoin_value[i * 2 + 1].push(val);
//                         }
//                     }
//                 }

//                 // for i in 0..perm_chip.len() {
//                 //     perm_chip[i].assign(
//                 //         layouter,
//                 //         join_value[0].clone(),
//                 //         join_value[1].clone(),
//                 //         p_combined.clone(),
//                 //     );
//                 // }

//                 // assign join and disjoin values
//                 // println!("Join 0 {:?}", join_value[6]);
//                 // println!("Join 0 {:?}", join_value[7]);
//                 // println!("Join 1 {:?}", join_value[1].len());
//                 // println!("Join 2 {:?}", join_value[2].len());
//                 // println!("Join 3 {:?}", join_value[3].len());

//                 for i in 0..join_value[0].len() {
//                     for j in 0..join_value[0][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_part[0][j],
//                             i,
//                             || Value::known(join_value[0][i][j]),
//                         )?;
//                     }
//                 }

//                 for i in 0..disjoin_value[0].len() {
//                     for j in 0..disjoin_value[0][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_part[1][j],
//                             i,
//                             || Value::known(disjoin_value[0][i][j]),
//                         )?;
//                     }
//                 }

//                 for (idx, x) in (1..3).enumerate() {
//                     for i in 0..join_value[x].len() {
//                         for j in 0..join_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_partsupp[idx * 2][j],
//                                 i,
//                                 || Value::known(join_value[x][i][j]),
//                             )?;
//                         }
//                     }

//                     for i in 0..disjoin_value[x].len() {
//                         for j in 0..disjoin_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_partsupp[idx * 2 + 1][j],
//                                 i,
//                                 || Value::known(disjoin_value[x][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 for (idx, x) in (3..5).enumerate() {
//                     for i in 0..join_value[x].len() {
//                         for j in 0..join_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_supplier[idx * 2][j],
//                                 i,
//                                 || Value::known(join_value[x][i][j]),
//                             )?;
//                         }
//                     }

//                     for i in 0..disjoin_value[x].len() {
//                         for j in 0..disjoin_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_supplier[idx * 2 + 1][j],
//                                 i,
//                                 || Value::known(disjoin_value[x][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 for (idx, x) in (5..7).enumerate() {
//                     for i in 0..join_value[x].len() {
//                         for j in 0..join_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_nation[idx * 2][j],
//                                 i,
//                                 || Value::known(join_value[x][i][j]),
//                             )?;
//                         }
//                     }

//                     for i in 0..disjoin_value[x].len() {
//                         for j in 0..disjoin_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_nation[idx * 2 + 1][j],
//                                 i,
//                                 || Value::known(disjoin_value[x][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 for i in 0..join_value[7].len() {
//                     for j in 0..join_value[7][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_region[0][j],
//                             i,
//                             || Value::known(join_value[7][i][j]),
//                         )?;
//                     }
//                 }

//                 for i in 0..disjoin_value[7].len() {
//                     for j in 0..disjoin_value[7][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_region[1][j],
//                             i,
//                             || Value::known(disjoin_value[7][i][j]),
//                         )?;
//                     }
//                 }

//                 // compute final table by applying all joins
//                 let join_index = [
//                     (0, 1, 0, 0),             //   p_partkey = ps_partkey
//                     (1, 2, 4 + 1, 5),         // s_suppkey = ps_suppkey
//                     (2, 3, 4 + 3 + 6, 1),     //  s_nationkey = n_nationkey
//                     (3, 4, 4 + 3 + 7 + 2, 1), // n_regionkey = r_regionkey
//                 ];
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

//                 let mut cartesian_product = join_vectors(&combined, &join_index);

//                 // println!(
//                 //     "product: {:?}",
//                 //     cartesian_product,
//                 //     // cartesian_product[0].len()
//                 // );
//                 let min_ps_supplycost = cartesian_product
//                     .iter()
//                     .map(|row| row[6])
//                     .min()
//                     .unwrap_or_default();

//                 // p_partkey = ps_partkey
//                 // and s_suppkey = ps_suppkey
//                 // and p_size = :1
//                 // and p_type like '%:2'
//                 // and s_nationkey = n_nationkey
//                 // and n_regionkey = r_regionkey
//                 // and r_name = ':3'
//                 // and ps_supplycost = ps_supplycost_min:
//                 let p_combined: Vec<Vec<_>> = p_combined
//                     .clone() // due to p_name like '%:1%'
//                     .into_iter()
//                     .filter(|row| row[1] == condition[0] && row[2] == condition[1]) // p_size = :1 and p_type like '%:2'
//                     .collect();
//                 let ps_combined: Vec<Vec<_>> = ps_combined
//                     .clone() // due to p_name like '%:1%'
//                     .into_iter()
//                     .filter(|row| row[2] == min_ps_supplycost)
//                     .collect();

//                 println!("Join 01 {:?}", p_combined);
//                 // println!("Join 11 {:?}", ps_combined);
//                 // assign min_ps_supplycost values to the condition[4] column
//                 for i in 0..ps_combined.len() {
//                     region.assign_advice(
//                         || "p",
//                         self.config.condition[3],
//                         i,
//                         || Value::known(min_ps_supplycost.clone()),
//                     )?;
//                 }

//                 // p_partkey = ps_partkey
//                 // and s_suppkey = ps_suppkey  only with these two different conditions
//                 let mut combined_1 = Vec::new();
//                 let mut join_value_1: Vec<Vec<_>> = vec![Default::default(); 4];
//                 let mut disjoin_value_1: Vec<Vec<_>> = vec![Default::default(); 4];
//                 combined_1.push(p_combined); // its length is 4
//                 combined_1.push(ps_combined); // 3
//                 combined_1.push(s_combined); // 7
//                 combined_1.push(n_combined); // 3
//                 combined_1.push(r_combined); // 2

//                 for i in 0..2 {
//                     // only update the first two of index
//                     for val in combined_1[index[i].0].iter() {
//                         if let Some(_) = combined_1[index[i].1]
//                             .iter()
//                             .find(|v| v[index[i].3] == val[index[i].2])
//                         {
//                             join_value_1[i * 2].push(val.clone()); // join values
//                         } else {
//                             disjoin_value_1[i * 2].push(val); // disjoin values
//                         }
//                     }
//                     for val in combined_1[index[i].1].iter() {
//                         if let Some(_) = combined_1[index[i].0]
//                             .iter()
//                             .find(|v| v[index[i].2] == val[index[i].3])
//                         {
//                             join_value_1[i * 2 + 1].push(val.clone());
//                         } else {
//                             disjoin_value_1[i * 2 + 1].push(val);
//                         }
//                     }
//                 }

//                 // println!("Join 0 {:?}", join_value_1[0]);
//                 // println!("Join 1 {:?}", join_value_1[1]);

//                 // assign update join values
//                 for (idx, x) in (0..1).enumerate() {
//                     for i in 0..join_value_1[x].len() {
//                         for j in 0..join_value_1[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_part1[idx * 2][j],
//                                 i,
//                                 || Value::known(join_value_1[x][i][j]),
//                             )?;
//                         }
//                     }

//                     for i in 0..disjoin_value_1[x].len() {
//                         for j in 0..disjoin_value_1[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_part1[idx * 2 + 1][j],
//                                 i,
//                                 || Value::known(disjoin_value_1[x][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 for (idx, x) in (2..3).enumerate() {
//                     for i in 0..join_value_1[x].len() {
//                         for j in 0..join_value_1[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_partsupp1[idx * 2][j],
//                                 i,
//                                 || Value::known(join_value_1[x][i][j]),
//                             )?;
//                         }
//                     }

//                     for i in 0..disjoin_value_1[x].len() {
//                         for j in 0..disjoin_value_1[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_partsupp1[idx * 2 + 1][j],
//                                 i,
//                                 || Value::known(disjoin_value_1[x][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 // generate the new join table
//                 let mut cartesian_product_1 = join_vectors(&combined_1, &join_index);

//                 // order by
//                 // s_acctbal desc,
//                 // n_name,
//                 // s_name,
//                 // p_partkey;
//                 cartesian_product_1.sort_by(|a, b| {
//                     match b[8].cmp(&a[8]) {
//                         // s_acctbal desc,
//                         Ordering::Equal => {
//                             a[14]
//                                 .cmp(&b[14]) // n_name
//                                 .then(a[7].cmp(&b[7])) // s_name
//                                 .then(a[0].cmp(&b[0])) // p_partkey
//                         }
//                         other => other,
//                     }
//                 });

//                 // assign selectors for apply the sort constraints
//                 for (i, row) in cartesian_product_1.iter().enumerate() {
//                     if i > 0 {
//                         self.config.q_cond[0].enable(&mut region, i)?;

//                         if row[8] == cartesian_product_1[i - 1][8] {

//                             region.assign_advice(
//                                 || "equal_check",
//                                 self.config.equal_check[0],
//                                 i,
//                                 || Value::known(F::from(1)),
//                             )?;
//                         }
//                         else {
//                             region.assign_advice(
//                                 || "equal_check",
//                                 self.config.equal_check[0],
//                                 i,
//                                 || Value::known(F::from(0)),
//                             )?;
//                             if row[14] == cartesian_product_1[i - 1][14] {
//                                 region.assign_advice(
//                                     || "equal_check",
//                                     self.config.equal_check[1],
//                                     i,
//                                     || Value::known(F::from(1)),
//                                 )?;
//                             }
//                             else{
//                                 region.assign_advice(
//                                     || "equal_check",
//                                     self.config.equal_check[1],
//                                     i,
//                                     || Value::known(F::from(0)),
//                                 )?;

//                                 if row[14] == cartesian_product_1[i - 1][7] {
//                                     region.assign_advice(
//                                         || "equal_check",
//                                         self.config.equal_check[2],
//                                         i,
//                                         || Value::known(F::from(1)),
//                                     )?;
//                                 }
//                                 else{
//                                     region.assign_advice(
//                                         || "equal_check",
//                                         self.config.equal_check[2],
//                                         i,
//                                         || Value::known(F::from(0)),
//                                     )?;
//                                 }
//                             }
//                         }
//                     }

//                 }

//                 // println!(
//                 //     "product: {:?}",
//                 //     cartesian_product_1.len(),
//                 //     // cartesian_product[0].len()
//                 // );

//                 for i in 0..cartesian_product_1.clone().len() {
//                     region.assign_advice(
//                         || "s_acctbal",
//                         self.config.orderby[0],
//                         i,
//                         || Value::known(cartesian_product_1[i][8]),
//                     )?;
//                     region.assign_advice(
//                         || "s_name",
//                         self.config.orderby[1],
//                         i,
//                         || Value::known(cartesian_product_1[i][7]),
//                     )?;
//                     region.assign_advice(
//                         || "n_name",
//                         self.config.orderby[2],
//                         i,
//                         || Value::known(cartesian_product_1[i][14]),
//                     )?;
//                     region.assign_advice(
//                         || "p_partkey",
//                         self.config.orderby[3],
//                         i,
//                         || Value::known(cartesian_product_1[i][0]),
//                     )?;
//                     region.assign_advice(
//                         || "p_mfgr",
//                         self.config.orderby[4],
//                         i,
//                         || Value::known(cartesian_product_1[i][3]),
//                     )?;
//                     region.assign_advice(
//                         || "s_address",
//                         self.config.orderby[5],
//                         i,
//                         || Value::known(cartesian_product_1[i][9]),
//                     )?;
//                     region.assign_advice(
//                         || "s_phone",
//                         self.config.orderby[6],
//                         i,
//                         || Value::known(cartesian_product_1[i][10]),
//                     )?;
//                     region.assign_advice(
//                         || "s_phone",
//                         self.config.orderby[7],
//                         i,
//                         || Value::known(cartesian_product_1[i][11]),
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
// }

// struct MyCircuit<F: Copy> {
//     part: Vec<Vec<F>>,
//     supplier: Vec<Vec<F>>,
//     partsupp: Vec<Vec<F>>,
//     nation: Vec<Vec<F>>,
//     regions: Vec<Vec<F>>,
//     condition: [F; 3],

//     _marker: PhantomData<F>,
// }

// impl<F: Copy + Default> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             part: Vec::new(),
//             supplier: Vec::new(),
//             partsupp: Vec::new(),
//             nation: Vec::new(),
//             regions: Vec::new(),
//             condition: [Default::default(); 3],

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
//             self.part.clone(),
//             self.supplier.clone(),
//             self.partsupp.clone(),
//             self.nation.clone(),
//             self.regions.clone(),
//             self.condition.clone(),
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

//     use crate::data::data_processing;

//     #[test]
//     fn test_1() {
//         let k = 16;

//         // if let Ok(records) =
//         //     data_processing::read_records_from_csv("/Users/binbingu/halo2-TPCH/src/data/region.cvs")
//         // {
//         //     println!("{:?}", records[0].name);
//         // } else {
//         //     println!("Failed to read records from file2");
//         // }
//         fn string_to_u64(s: &str) -> u64 {
//             let mut result = 0;

//             for (i, c) in s.chars().enumerate() {
//                 result += (i as u64 + 1) * (c as u64);
//             }

//             result
//         }
//         fn scale_by_1000(x: f64) -> u64 {
//             (1000.0 * x) as u64
//         }

//         let part_file_path = "/Users/binbingu/halo2-TPCH/src/data/part.tbl";
//         let supplier_file_path = "/Users/binbingu/halo2-TPCH/src/data/supplier.tbl";
//         let partsupp_file_path = "/Users/binbingu/halo2-TPCH/src/data/partsupp.tbl";
//         let nation_file_path = "/Users/binbingu/halo2-TPCH/src/data/nation.tbl";
//         let region_file_path = "/Users/binbingu/halo2-TPCH/src/data/region.cvs";
//         let mut part: Vec<Vec<Fp>> = Vec::new();
//         let mut supplier: Vec<Vec<Fp>> = Vec::new();
//         let mut partsupp: Vec<Vec<Fp>> = Vec::new();
//         let mut nation: Vec<Vec<Fp>> = Vec::new();
//         let mut regions: Vec<Vec<Fp>> = Vec::new();

//         if let Ok(records) = data_processing::part_read_records_from_file(part_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             part = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.p_partkey),
//                         // Fp::from(string_to_u64(&record.p_name)),
//                         // Fp::from(string_to_u64(&record.p_brand)),
//                         Fp::from(record.p_size),
//                         Fp::from(string_to_u64(&record.p_type)),
//                         Fp::from(string_to_u64(&record.p_mfgr)),
//                         // Fp::from(string_to_u64(&record.p_container)),
//                         // Fp::from(scale_by_1000(record.p_retailprice)),
//                         // Fp::from(string_to_u64(&record.p_comment)),
//                     ]
//                 })
//                 .collect();
//         }

//         if let Ok(records) = data_processing::supplier_read_records_from_file(supplier_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             supplier = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(string_to_u64(&record.s_name)),
//                         Fp::from(scale_by_1000(record.s_acctbal)),
//                         Fp::from(string_to_u64(&record.s_address)),
//                         Fp::from(string_to_u64(&record.s_phone)),
//                         Fp::from(string_to_u64(&record.s_comment)),
//                         Fp::from(record.s_suppkey),
//                         Fp::from(record.s_nationkey),
//                     ]
//                 })
//                 .collect();
//         }

//         if let Ok(records) = data_processing::partsupp_read_records_from_file(partsupp_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             partsupp = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.ps_partkey),
//                         Fp::from(record.ps_suppkey),
//                         // Fp::from(record.ps_availqty),
//                         Fp::from(scale_by_1000(record.ps_supplycost)),
//                         // Fp::from(string_to_u64(&record.ps_comment)),
//                     ]
//                 })
//                 .collect();
//         }

//         if let Ok(records) = data_processing::nation_read_records_from_file(nation_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             nation = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(string_to_u64(&record.n_name)),
//                         Fp::from(record.n_nationkey),
//                         Fp::from(record.n_regionkey),
//                         // Fp::from(string_to_u64(&record.n_comment)),
//                     ]
//                 })
//                 .collect();
//         }

//         if let Ok(records) = data_processing::region_read_records_from_csv(region_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             regions = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(string_to_u64(&record.r_name)),
//                         Fp::from(record.r_regionkey),
//                     ]
//                 })
//                 .collect();
//         }

//         // Create a new variable for the first 3 rows
//         // let part: Vec<Vec<Fp>> = part.iter().take(1000).cloned().collect();
//         // let supplier: Vec<Vec<Fp>> = supplier.iter().take(1000).cloned().collect();
//         // let partsupp: Vec<Vec<Fp>> = partsupp.iter().take(1000).cloned().collect();

//         // let nation: Vec<Vec<Fp>> = nation.iter().take(4).cloned().collect();
//         // let regions: Vec<Vec<Fp>> = regions.iter().take(3).cloned().collect();
//         // for row in &part {
//         //     println!("{:?}", row);
//         // }
//         // for row in &regions {
//         //     println!("{:?}", row);
//         // }

//         let condition = [Fp::from(7), Fp::from(18203), Fp::from(1468)];
//         // p_size = :1   ->  7
//         // p_type like '%:2'   -> PROMO BURNISHED COPPER  -> 18203
//         // r_name = ':3'   ->    AFRICA: 1468

//         let circuit = MyCircuit::<Fp> {
//             part,
//             supplier,
//             partsupp,
//             nation,
//             regions,

//             condition,
//             _marker: PhantomData,
//         };

//         // // let prover = MockProver::run(k, &circuit, vec![z.to_vec()]).unwrap();
//         let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//         prover.assert_satisfied();
//     }
// }
// // time cargo test --package halo2-experiments --lib -- sql::q3_final_v1::tests::test_1 --exact --nocapture
