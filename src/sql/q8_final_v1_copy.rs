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
// use std::cmp::Reverse;
// use std::collections::HashSet;

// use std::mem;

// const N: usize = 10;
// const NUM_BYTES: usize = 3;

// // #[derive(Default)]
// // We should use the selector to skip the row which does not satisfy shipdate values

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: [Selector; 5],

//     part: [Column<Advice>; 2],      // p_partkey, p_type
//     supplier: [Column<Advice>; 2],  // s_suppkey, s_nationkey
//     lineitem: [Column<Advice>; 5], // l_extendedprice, l_discount, l_partkey, l_suppkey, l_orderkey,
//     orders: [Column<Advice>; 4],   // o_year, o_orderdate, o_orderkey, o_custkey
//     customer: [Column<Advice>; 2], // c_custkey, c_nationkey,
//     nation_n1: [Column<Advice>; 2], // n_nationkey, n_regionkey
//     nation_n2: [Column<Advice>; 2], // n_nationkey, n_regionkey
//     region: [Column<Advice>; 2],   // r_name, r_regionkey

//     condition: [Column<Advice>; 4], // r_name = ':2', o_orderdate between date '1995-01-01' and date '1996-12-31', p_type = ':3'

//     check: [Column<Advice>; 3],

//     groupby: [Column<Advice>; 2],

//     join_column: [Column<Advice>; 5],
//     disjoin_column: [Column<Advice>; 5],

//     revenue: [Column<Advice>; 2],

//     lt_o1_condition: LtEqGenericConfig<F, NUM_BYTES>,
//     lt_o2_condition: LtEqGenericConfig<F, NUM_BYTES>,
//     r_condition: IsZeroConfig<F>,
//     p_condition: IsZeroConfig<F>,
//     // groupby_sort: LtEqGenericConfig<F, NUM_BYTES>,
//     // revenue_final: LtEqGenericConfig<F, NUM_BYTES>,
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
//         let q_enable = [
//             meta.complex_selector(),
//             meta.complex_selector(),
//             meta.complex_selector(),
//             meta.complex_selector(),
//             meta.complex_selector(),
//         ];
//         let part = [meta.advice_column(), meta.advice_column()];
//         let supplier = [meta.advice_column(), meta.advice_column()];
//         let lineitem = [
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//         ];
//         let orders = [
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//         ];
//         let customer = [meta.advice_column(), meta.advice_column()];
//         let nation_n1 = [meta.advice_column(), meta.advice_column()];
//         let nation_n2 = [meta.advice_column(), meta.advice_column()];
//         let region = [meta.advice_column(), meta.advice_column()];

//         let condition = [
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//         ];
//         let check = [
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//         ];
//         let groupby = [meta.advice_column(), meta.advice_column()];

//         let join_column = [
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//         ];

//         let disjoin_column = [
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//             meta.advice_column(),
//         ];
//         let revenue = [meta.advice_column(), meta.advice_column()];
//         let is_zero_advice_column1 = meta.advice_column();
//         let is_zero_advice_column2 = meta.advice_column();

//         for i in 0..2 {
//             meta.enable_equality(part[i]);
//             meta.enable_equality(supplier[i]);
//             meta.enable_equality(customer[i]);
//             meta.enable_equality(nation_n1[i]);
//             meta.enable_equality(nation_n2[i]);
//             meta.enable_equality(region[i]);
//             meta.enable_equality(revenue[i]);
//         }
//         for i in 0..4 {
//             meta.enable_equality(orders[i]);
//             meta.enable_equality(condition[i]);
//         }
//         for i in 0..5 {
//             meta.enable_equality(lineitem[i]);
//         }
//         for i in 0..join_column.len() {
//             meta.enable_equality(join_column[i]);
//             meta.enable_equality(disjoin_column[i]);
//         }

//         let lt_o1_condition = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]),
//             |meta| vec![meta.query_advice(orders[1], Rotation::cur())],
//             |meta| vec![meta.query_advice(condition[1], Rotation::cur())],
//         );

//         let lt_o2_condition = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]),
//             |meta| vec![meta.query_advice(condition[2], Rotation::cur())],
//             |meta| vec![meta.query_advice(orders[1], Rotation::cur())],
//         );

//         let r_condition = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]), // this is the q_enable
//             |meta| {
//                 meta.query_advice(region[0], Rotation::cur())
//                     - meta.query_advice(condition[0], Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column1, // this is the advice column that stores value_inv
//         );

//         let p_condition = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]), // this is the q_enable
//             |meta| {
//                 meta.query_advice(part[0], Rotation::cur())
//                     - meta.query_advice(condition[3], Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column2, // this is the advice column that stores value_inv
//         );

//         // // gate for o_orderdate < date ':2' + interval '1' year
//         // meta.create_gate(
//         //     "verifies o_orderdate < date ':2' + interval '1' year", // just use less_than for testing here
//         //     |meta| {
//         //         let q_enable = meta.query_selector(q_enable);
//         //         let check = meta.query_advice(o1_check, Rotation::cur());
//         //         vec![q_enable * (lt_o1_condition.is_lt(meta, None) - check)]
//         //     },
//         // );

//         // // gate for o_orderdate >= date ':2'
//         // meta.create_gate("verifies o_orderdate < date ':2'", |meta| {
//         //     let q_enable = meta.query_selector(q_enable);
//         //     let check = meta.query_advice(o2_check, Rotation::cur());
//         //     vec![q_enable * (lt_o2_condition.is_lt(meta, None) - check)]
//         // });

//         // // gate for r_name = ':1'
//         // meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//         //     let s = meta.query_selector(q_enable);
//         //     let output = meta.query_advice(r_check, Rotation::cur());
//         //     vec![
//         //         s.clone()
//         //             * (lt_r_condition.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//         //         s * (Expression::Constant(F::ONE) - lt_r_condition.expr()) * (output), // in this case output == 0
//         //     ]
//         // });

//         // // groupby
//         // let groupby_sort = LtEqGenericChip::configure(
//         //     meta,
//         //     |meta| meta.query_selector(q_sort),
//         //     |meta| vec![meta.query_advice(groupby_name, Rotation::prev())],
//         //     |meta| vec![meta.query_advice(groupby_name, Rotation::cur())],
//         // );

//         // let revenue_final = LtEqGenericChip::configure(
//         //     meta,
//         //     |meta| meta.query_selector(q_sort),
//         //     |meta| vec![meta.query_advice(sorted_revenue, Rotation::prev())],
//         //     |meta| vec![meta.query_advice(sorted_revenue, Rotation::cur())],
//         // );

//         TestCircuitConfig {
//             q_enable,
//             part,
//             supplier,
//             lineitem,
//             orders,
//             customer,
//             nation_n1,
//             nation_n2,
//             region,
//             condition,
//             check,
//             groupby,
//             join_column,
//             disjoin_column,
//             revenue,
//             lt_o1_condition,
//             lt_o2_condition,
//             r_condition,
//             p_condition,
//             // groupby_sort,
//             // revenue_final,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         part: [[F; N]; 2],
//         supplier: [[F; N]; 2],
//         lineitem: [[F; N]; 5],
//         orders: [[F; N]; 4],
//         customer: [[F; N]; 2],
//         nation_n1: [[F; N]; 2],
//         nation_n2: [[F; N]; 2],
//         regions: [[F; N]; 2], // i.e. region, since halo2 has region keyword, we use different word here

//         o1_condition: F,
//         o2_condition: F,
//         r_condition: F,
//         p_condition: F,
//     ) -> Result<(), Error> {
//         // Result<AssignedCell<F, F>, Error> {
//         // load the chips for the filtering conditions of the three tables
//         let lt_o1_cond_chip = LtEqGenericChip::construct(self.config.lt_o1_condition.clone());
//         let lt_o2_cond_chip = LtEqGenericChip::construct(self.config.lt_o2_condition.clone());
//         let r_cond_chip = IsZeroChip::construct(self.config.r_condition.clone());
//         let p_cond_chip = IsZeroChip::construct(self.config.p_condition.clone());
//         // let groupby_sort_chip = LtEqGenericChip::construct(self.config.groupby_sort.clone());
//         // let lt_revenue_final_chip = LtEqGenericChip::construct(self.config.revenue_final.clone());

//         lt_o1_cond_chip.load(layouter)?;
//         lt_o2_cond_chip.load(layouter)?;

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 // locally compute the values for conditional check
//                 let mut o1_check: [bool; N] = [false; N];
//                 let mut o2_check: [bool; N] = [false; N];
//                 let mut r_check: [F; N] = [F::from(0); N];
//                 let mut p_check: [F; N] = [F::from(0); N];
//                 // o2_condition<= x <= o1_condition
//                 for i in 0..orders[0].len() {
//                     if orders[1][i] <= o1_condition {
//                         o1_check[i] = true;
//                     }
//                     if orders[1][i] >= o2_condition {
//                         o2_check[i] = true;
//                     }
//                 }
//                 for i in 0..regions[0].len() {
//                     if regions[0][i] == r_condition {
//                         r_check[i] = F::from(1);
//                     }
//                 }
//                 for i in 0..part[0].len() {
//                     if part[1][i] == p_condition {
//                         p_check[i] = F::from(1);
//                     }
//                 }

//                 //assign input values
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
//                 for i in 0..supplier.len() {
//                     for j in 0..supplier[0].len() {
//                         region.assign_advice(
//                             || "s",
//                             self.config.supplier[i],
//                             i,
//                             || Value::known(supplier[i][j]),
//                         )?;
//                     }
//                 }
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
//                 for i in 0..orders.len() {
//                     for j in 0..orders[0].len() {
//                         region.assign_advice(
//                             || "o",
//                             self.config.orders[i],
//                             i,
//                             || Value::known(orders[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..customer.len() {
//                     for j in 0..customer[0].len() {
//                         region.assign_advice(
//                             || "c",
//                             self.config.customer[i],
//                             i,
//                             || Value::known(customer[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..nation_n1.len() {
//                     for j in 0..nation_n1[0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.nation_n1[i],
//                             i,
//                             || Value::known(nation_n1[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..nation_n2.len() {
//                     for j in 0..nation_n2[0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.nation_n2[i],
//                             i,
//                             || Value::known(nation_n2[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..regions.len() {
//                     for j in 0..regions[0].len() {
//                         region.assign_advice(
//                             || "region",
//                             self.config.region[i],
//                             i,
//                             || Value::known(regions[i][j]),
//                         )?;
//                     }
//                 }

//                 // assign values for loaded chips and checks
//                 for i in 0..orders[0].len() {
//                     lt_o1_cond_chip.assign(&mut region, i, &[orders[1][i]], &[o1_condition])?;
//                     lt_o2_cond_chip.assign(&mut region, i, &[o2_condition], &[orders[1][i]])?;
//                     region.assign_advice(
//                         || "",
//                         self.config.condition[0],
//                         i,
//                         || Value::known(o1_condition),
//                     )?;
//                     region.assign_advice(
//                         || "",
//                         self.config.condition[1],
//                         i,
//                         || Value::known(o2_condition),
//                     )?;
//                     region.assign_advice(
//                         || "",
//                         self.config.check[0],
//                         i,
//                         || Value::known(F::from(o1_check[i] as u64)),
//                     )?;
//                     region.assign_advice(
//                         || "",
//                         self.config.check[1],
//                         i,
//                         || Value::known(F::from(o2_check[i] as u64)),
//                     )?;
//                 }
//                 for i in 0..regions[0].len() {
//                     r_cond_chip.assign(
//                         &mut region,
//                         i,
//                         Value::known(regions[0][i] - r_condition),
//                     )?;
//                     region.assign_advice(
//                         || "",
//                         self.config.condition[2],
//                         i,
//                         || Value::known(r_condition),
//                     )?;
//                     region.assign_advice(
//                         || "",
//                         self.config.check[2],
//                         i,
//                         || Value::known(r_check[i]),
//                     )?;
//                 }
//                 for i in 0..part[0].len() {
//                     p_cond_chip.assign(&mut region, i, Value::known(part[1][i] - p_condition))?;
//                     region.assign_advice(
//                         || "",
//                         self.config.condition[3],
//                         i,
//                         || Value::known(p_condition),
//                     )?;
//                     region.assign_advice(
//                         || "",
//                         self.config.check[3],
//                         i,
//                         || Value::known(p_check[i]),
//                     )?;
//                 }

//                 // compute values related to the join operation locally
//                 // translate the input into row-based values
//                 let mut p_combined: Vec<Vec<_>> = (0..part[0].len())
//                     .map(|i| part.iter().map(|row| row[i]).collect())
//                     .collect();
//                 let p_combined: Vec<Vec<_>> = p_combined // due to p_type = ':3'
//                     .into_iter()
//                     .filter(|row| row[1] == p_condition)
//                     .collect();

//                 let s_combined: Vec<Vec<_>> = (0..supplier[0].len())
//                     .map(|i| supplier.iter().map(|row| row[i]).collect())
//                     .collect();

//                 let l_combined: Vec<Vec<_>> = (0..lineitem[0].len())
//                     .map(|i| lineitem.iter().map(|row| row[i]).collect())
//                     .collect();

//                 let mut o_combined: Vec<Vec<_>> = (0..orders[0].len())
//                     .map(|i| orders.iter().map(|row| row[i]).collect())
//                     .collect();
//                 let o_combined: Vec<Vec<_>> = o_combined // due to and o_orderdate between date '1995-01-01' and date '1996-12-31'
//                     .into_iter()
//                     .filter(|row| o2_condition <= row[1] && row[1] <= o1_condition)
//                     .collect();

//                 let c_combined: Vec<Vec<_>> = (0..customer[0].len())
//                     .map(|i| customer.iter().map(|row| row[i]).collect())
//                     .collect();

//                 let n1_combined: Vec<Vec<_>> = (0..nation_n1[0].len())
//                     .map(|i| nation_n1.iter().map(|row| row[i]).collect())
//                     .collect();

//                 let n2_combined: Vec<Vec<_>> = (0..nation_n2[0].len())
//                     .map(|i| nation_n2.iter().map(|row| row[i]).collect())
//                     .collect();

//                 let mut r_combined: Vec<Vec<_>> =
//                     (0..regions[0].len()) // due to and p_type = ':3'
//                         .map(|i| regions.iter().map(|row| row[i]).collect())
//                         .collect();
//                 let r_combined: Vec<Vec<_>> = r_combined // due to p_type = ':3'
//                     .into_iter()
//                     .filter(|row| row[1] == r_condition)
//                     .collect();

//                 //create the values for join and disjoin

//                 let mut join_value: Vec<Vec<_>> = vec![Default::default(); 8];
//                 let mut disjoin_value: Vec<Vec<_>> = vec![Default::default(); 8];
//                 // p_partkey = l_partkey, s_suppkey = l_suppkey, l_orderkey = o_orderkey, o_custkey = c_custkey,
//                 // c_nationkey = n1.n_nationkey, n1.n_regionkey = r_regionkey, s_nationkey = n2.n_nationkey
//                 let mut combined = Vec::new();
//                 combined.push(p_combined);
//                 combined.push(s_combined);
//                 combined.push(l_combined);
//                 combined.push(o_combined);
//                 combined.push(c_combined);
//                 combined.push(n1_combined);
//                 combined.push(n2_combined);
//                 combined.push(r_combined);
//                 // (input1 index, input2 index, join attribute index of input1, join attribute of input2)
//                 let index = [
//                     (0, 1, 2, 0),
//                     (1, 2, 3, 0),
//                     (2, 3, 2, 4),
//                     (3, 4, 0, 3),
//                     (4, 5, 0, 1),
//                     (5, 7, 1, 1),
//                     (1, 6, 0, 1),
//                 ];

//                 for i in 0..index.len() {
//                     for val in combined[index[i].0].iter() {
//                         if let Some(_) = combined[index[i].1]
//                             .iter()
//                             .find(|v| v[index[i].2] == val[index[i].3])
//                         {
//                             join_value[i * 2].push(val);
//                         } else {
//                             disjoin_value[i * 2].push(val);
//                         }
//                     }
//                     for val in combined[index[i].1].iter() {
//                         if let Some(_) = combined[index[i].0]
//                             .iter()
//                             .find(|v| v[index[i].3] == val[index[i].2])
//                         {
//                             join_value[i * 2 + 1].push(val);
//                         } else {
//                             disjoin_value[i * 2 + 1].push(val);
//                         }
//                     }
//                 }
//                 // // p_partkey = l_partkey
//                 // for val in p_combined.iter() {
//                 //     if let Some(_) = l_combined.iter().find(|v| v[2] == val[0]) {
//                 //         join_value[0].push(val);
//                 //     } else {
//                 //         disjoin_value[0].push(val);
//                 //     }
//                 // }
//                 // for val in l_combined.iter() {
//                 //     if let Some(_) = p_combined.iter().find(|v| v[0] == val[2]) {
//                 //         join_value[1].push(val);
//                 //     } else {
//                 //         join_value[1].push(val);
//                 //     }
//                 // }

//                 // // s_suppkey = l_suppkey
//                 // for val in s_combined.iter() {
//                 //     if let Some(_) = l_combined.iter().find(|v| v[3] == val[0]) {
//                 //         join_value[2].push(val);
//                 //     } else {
//                 //         disjoin_value[2].push(val);
//                 //     }
//                 // }
//                 // for val in l_combined.iter() {
//                 //     if let Some(_) = s_combined.iter().find(|v| v[0] == val[3]) {
//                 //         join_value[3].push(val);
//                 //     } else {
//                 //         disjoin_value[3].push(val);
//                 //     }
//                 // }

//                 // // and l_orderkey = o_orderkey
//                 // for val in l_combined.iter() {
//                 //     if let Some(_) = o_combined.iter().find(|v| v[2] == val[4]) {
//                 //         join_value[4].push(val);
//                 //     } else {
//                 //         disjoin_value[4].push(val);
//                 //     }
//                 // }
//                 // for val in o_combined.iter() {
//                 //     if let Some(_) = l_combined.iter().find(|v| v[4] == val[2]) {
//                 //         join_value[5].push(val);
//                 //     } else {
//                 //         disjoin_value[5].push(val);
//                 //     }
//                 // }

//                 // // and o_custkey = c_custkey
//                 // for val in o_combined.iter() {
//                 //     if let Some(_) = c_combined.iter().find(|v| v[0] == val[3]) {
//                 //         join_value[6].push(val);
//                 //     } else {
//                 //         disjoin_value[6].push(val);
//                 //     }
//                 // }
//                 // for val in c_combined.iter() {
//                 //     if let Some(_) = o_combined.iter().find(|v| v[3] == val[0]) {
//                 //         join_value[7].push(val);
//                 //     } else {
//                 //         disjoin_value[7].push(val);
//                 //     }
//                 // }

//                 // //   and c_nationkey = n1.n_nationkey
//                 // for val in c_combined.iter() {
//                 //     if let Some(_) = n1_combined.iter().find(|v| v[0] == val[1]) {
//                 //         join_value[8].push(val);
//                 //     } else {
//                 //         disjoin_value[8].push(val);
//                 //     }
//                 // }
//                 // for val in n1_combined.iter() {
//                 //     if let Some(_) = c_combined.iter().find(|v| v[1] == val[0]) {
//                 //         join_value[9].push(val);
//                 //     } else {
//                 //         disjoin_value[9].push(val);
//                 //     }
//                 // }

//                 // //   and n1.n_regionkey = r_regionkey
//                 // for val in n1_combined.iter() {
//                 //     if let Some(_) = r_combined.iter().find(|v| v[1] == val[1]) {
//                 //         join_value[10].push(val);
//                 //     } else {
//                 //         disjoin_value[10].push(val);
//                 //     }
//                 // }
//                 // for val in r_combined.iter() {
//                 //     if let Some(_) = n1_combined.iter().find(|v| v[1] == val[1]) {
//                 //         join_value[11].push(val);
//                 //     } else {
//                 //         disjoin_value[11].push(val);
//                 //     }
//                 // }

//                 // //  and s_nationkey = n2.n_nationkey
//                 // for val in s_combined.iter() {
//                 //     if let Some(_) = n2_combined.iter().find(|v| v[0] == val[1]) {
//                 //         join_value[12].push(val);
//                 //     } else {
//                 //         disjoin_value[12].push(val);
//                 //     }
//                 // }
//                 // for val in n2_combined.iter() {
//                 //     if let Some(_) = s_combined.iter().find(|v| v[1] == val[0]) {
//                 //         join_value[13].push(val);
//                 //     } else {
//                 //         disjoin_value[13].push(val);
//                 //     }
//                 // }

//                 // //assign values for the result of join i.e. l_orderkey = o_orderkey
//                 // let mut cartesian_product1 = Vec::new();
//                 // for val1 in &l_combined {
//                 //     for val2 in &o_combined {
//                 //         if val1.0 == val2.1 && val2.3 == true && val2.4 == true {
//                 //             cartesian_product1
//                 //                 .push((val1.0, val1.1, val1.2, val1.3, val2.0, val2.1));
//                 //             // cartesian_product1.4 = o_custkey
//                 //         }
//                 //     }
//                 // }

//                 // //assign values for the result of join i.e. l_suppkey = s_suppkey
//                 // let mut cartesian_product2 = Vec::new();
//                 // for val1 in &cartesian_product1 {
//                 //     for val2 in &s_combined {
//                 //         if val1.3 == val2.1 {
//                 //             cartesian_product2
//                 //                 .push((val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val2.0));
//                 //             // nationkey = cartesian_product2.5
//                 //         }
//                 //     }
//                 // }
//                 // //assign values for the result of join i.e. c_nationkey = s_nationkey and c_custkey = o_custkey
//                 // let mut cartesian_product3 = Vec::new();
//                 // for val1 in &cartesian_product2 {
//                 //     for val2 in &c_combined {
//                 //         if val1.5 == val2.1 && val1.4 == val2.0 {
//                 //             cartesian_product3.push((
//                 //                 val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val1.6, val2.0,
//                 //                 val2.1,
//                 //             ));
//                 //             // cartesian_product3.5 = c_nationkey = s_nationkey
//                 //         }
//                 //     }
//                 // }
//                 // //assign values for the result of join i.e. s_nationkey = n_nationkey
//                 // let mut cartesian_product4 = Vec::new();
//                 // for val1 in &cartesian_product3 {
//                 //     for val2 in &n_combined {
//                 //         if val1.5 == val2.0 {
//                 //             cartesian_product4.push((
//                 //                 val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val1.6, val1.7,
//                 //                 val1.8, val2.1, val2.2,
//                 //             ));
//                 //             // cartesian_product4.7 = n_regionkey
//                 //         }
//                 //     }
//                 // }
//                 // //assign values for the result of join i.e. n_regionkey = r_regionkey
//                 // let mut cartesian_product5 = Vec::new();
//                 // for val1 in &cartesian_product4 {
//                 //     for val2 in &r_combined {
//                 //         if val1.7 == val2.0 && val2.2 == F::from(1) {
//                 //             cartesian_product5.push((
//                 //                 val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val1.6, val1.7,
//                 //                 val1.8, val1.9, val1.10, val2.1,
//                 //             ));
//                 //             // cartesian_product5.8 = r_name
//                 //         }
//                 //     }
//                 // }

//                 // // the order of attributes in cartesian_product: l_orderkey/o_orderkey, c_custkey/o_custkey, l_extendedprice, l_discount, ...
//                 // //sort by l_orderkey, o_orderdate, o_shippriority
//                 // cartesian_product5.sort_by_key(|element| element.10);

//                 // let mut revenue = Vec::new();

//                 // for i in 0..cartesian_product5.len() {
//                 //     if i == 0 {
//                 //         self.config.q_first.enable(&mut region, i)?;
//                 //         revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
//                 //     // l_extendedprice * (1 - l_discount)
//                 //     } else {
//                 //         self.config.q_sort.enable(&mut region, i)?;
//                 //         groupby_sort_chip.assign(
//                 //             &mut region,
//                 //             i,
//                 //             &[cartesian_product5[i - 1].10],
//                 //             &[cartesian_product5[i].10],
//                 //         )?;

//                 //         // check if it is the first value
//                 //         if cartesian_product5[i - 1].10 == cartesian_product5[i].10 {
//                 //             self.config.q_first.enable(&mut region, i)?;
//                 //             revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
//                 //         } else {
//                 //             self.config.q_nonfirst.enable(&mut region, i)?;
//                 //             revenue.push(
//                 //                 revenue[i - 1] + cartesian_product5[i].1 * cartesian_product5[i].2,
//                 //             );
//                 //         }
//                 //     }
//                 //     // assign revenue column
//                 //     region.assign_advice(
//                 //         || "revenue",
//                 //         self.config.revenue,
//                 //         i,
//                 //         || Value::known(revenue[i]),
//                 //     )?;

//                 //     region.assign_advice(
//                 //         || "groupby_name",
//                 //         self.config.groupby_name,
//                 //         i,
//                 //         || Value::known(cartesian_product5[i].10),
//                 //     )?;

//                 //     region.assign_advice(
//                 //         || "groupby_l_extendedprice",
//                 //         self.config.groupby_extendedprice,
//                 //         i,
//                 //         || Value::known(cartesian_product5[i].1),
//                 //     )?;

//                 //     region.assign_advice(
//                 //         || "groupby_l_discount",
//                 //         self.config.groupby_discount,
//                 //         i,
//                 //         || Value::known(cartesian_product5[i].2),
//                 //     )?;
//                 // }

//                 // // generate revenue_final
//                 // // println!("product: {:?}", cartesian_product5);
//                 // let mut revenue_final = Vec::new(); // by removing intermediate revenue values, i.e. only keep the final revenue of each group
//                 // if revenue.len() > 0 {
//                 //     for i in 0..revenue.len() - 1 {
//                 //         if cartesian_product5[i].10 != cartesian_product5[i + 1].10 {
//                 //             revenue_final.push((cartesian_product5[i].10, revenue[i]))
//                 //         }
//                 //     }

//                 //     revenue_final.push((
//                 //         cartesian_product5[revenue.len() - 1].10,
//                 //         revenue[revenue.len() - 1],
//                 //     ));
//                 // }

//                 // // order by revenue desc

//                 // revenue_final.sort_by_key(|&(value, _)| Reverse(value));

//                 // // assign values of equal check for verifying if revenue_final is sorted
//                 // let mut equal_check: Vec<F> = Vec::new();

//                 // if revenue_final.len() == 1 {
//                 //     equal_check.push(F::from(0)); // 0 assigned to the first value in equal_check
//                 // } else {
//                 //     equal_check.push(F::from(0));
//                 //     for i in 1..revenue_final.len() {
//                 //         if revenue_final[i] == revenue_final[i - 1] {
//                 //             equal_check.push(F::from(1));
//                 //         } else {
//                 //             equal_check.push(F::from(0))
//                 //         }
//                 //     }
//                 // }
//                 // // println!("revenue: {:?}", revenue_final);
//                 // // println!("equal check: {:?}", equal_check);

//                 // // assign sorted revenue and orderdate
//                 // for i in 0..revenue_final.len() {
//                 //     region.assign_advice(
//                 //         || "sorted_revenue",
//                 //         self.config.sorted_revenue,
//                 //         i,
//                 //         || Value::known(revenue_final[i].1),
//                 //     )?;

//                 //     region.assign_advice(
//                 //         || "equal_check",
//                 //         self.config.equal_check,
//                 //         i,
//                 //         || Value::known(equal_check[i]),
//                 //     )?;

//                 //     if i != 0 {
//                 //         self.config.q_sort_final.enable(&mut region, i)?;
//                 //         lt_revenue_final_chip.assign(
//                 //             &mut region,
//                 //             i,
//                 //             &[revenue_final[i].1],
//                 //             &[revenue_final[i - 1].1],
//                 //         )?;
//                 //     }
//                 // }

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

// struct MyCircuit<F> {
//     l_orderkey: [F; N],
//     l_extendedprice: [F; N],
//     l_discount: [F; N],
//     l_suppkey: [F; N],
//     o_custkey: [F; N],
//     o_orderdate: [F; N],
//     o_orderkey: [F; N],
//     c_custkey: [F; N],
//     c_nationkey: [F; N],
//     s_nationkey: [F; N],
//     s_suppkey: [F; N],
//     n_nationkey: [F; N],
//     n_regionkey: [F; N],
//     n_name: [F; N],
//     r_regionkey: [F; N],
//     r_name: [F; N],

//     pub o1_condition: F,
//     pub o2_condition: F,
//     pub r_condition: F,

//     _marker: PhantomData<F>,
// }

// impl<F: Default> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             l_orderkey: Default::default(),
//             l_extendedprice: Default::default(),
//             l_discount: Default::default(),
//             l_suppkey: Default::default(),
//             o_custkey: Default::default(),
//             o_orderdate: Default::default(),
//             o_orderkey: Default::default(),
//             c_custkey: Default::default(),
//             c_nationkey: Default::default(),
//             s_nationkey: Default::default(),
//             s_suppkey: Default::default(),
//             n_nationkey: Default::default(),
//             n_regionkey: Default::default(),
//             n_name: Default::default(),
//             r_regionkey: Default::default(),
//             r_name: Default::default(),

//             o1_condition: Default::default(),
//             o2_condition: Default::default(),
//             r_condition: Default::default(),
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
//             self.l_orderkey,
//             self.l_extendedprice,
//             self.l_discount,
//             self.l_suppkey,
//             self.o_custkey,
//             self.o_orderdate,
//             self.o_orderkey,
//             self.c_custkey,
//             self.c_nationkey,
//             self.s_nationkey,
//             self.s_suppkey,
//             self.n_nationkey,
//             self.n_regionkey,
//             self.n_name,
//             self.r_regionkey,
//             self.r_name,
//             self.o1_condition,
//             self.o2_condition,
//             self.r_condition,
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
//         let l_orderkey = [Fp::from(1); N];
//         let l_extendedprice = [Fp::from(1); N];
//         let l_discount = [Fp::from(1); N];
//         let l_suppkey = [Fp::from(1); N];
//         let o_custkey = [Fp::from(1); N];
//         let o_orderdate = [Fp::from(1); N];
//         let o_orderkey = [Fp::from(1); N];
//         let c_custkey = [Fp::from(1); N];
//         let c_nationkey = [Fp::from(1); N];
//         let s_nationkey = [Fp::from(1); N];
//         let s_suppkey = [Fp::from(1); N];
//         let n_nationkey = [Fp::from(1); N];
//         let n_regionkey = [Fp::from(1); N];
//         let n_name = [Fp::from(1); N];
//         let r_regionkey = [Fp::from(1); N];
//         let r_name = [Fp::from(1); N];
//         let o1_condition = Fp::from(1);
//         let o2_condition = Fp::from(1);
//         let r_condition = Fp::from(1);

//         // let mut l_orderkey: [u64; N] = [1; N];
//         // l_orderkey[5] = 3;
//         // l_orderkey[6] = 3;
//         // let mut l_extendedprice: [u64; N] = [1; N];
//         // let mut l_discount: [u64; N] = [1; N];
//         // let mut l_shipdate: [u64; N] = [1; N];
//         // let mut o_orderdate: [u64; N] = [2; N];
//         // let mut o_shippriority: [u64; N] = [3; N];
//         // let mut o_custkey: [u64; N] = [2; N];
//         // o_custkey[5] = 3;
//         // o_custkey[6] = 3;

//         // let mut o_orderkey: [u64; N] = [3; N];
//         // let mut c_mktsegment: [u64; N] = [13; N];
//         // c_mktsegment[5] = 11;
//         // c_mktsegment[6] = 11;
//         // let mut c_custkey: [u64; N] = [3; N];

//         // let mut l_condition: u64 = 5;
//         // let mut o_condition: u64 = 10;
//         // let mut c_condition: u64 = 11;

//         let circuit = MyCircuit::<Fp> {
//             l_orderkey,
//             l_extendedprice,
//             l_discount,
//             l_suppkey,
//             o_custkey,
//             o_orderdate,
//             o_orderkey,
//             c_custkey,
//             c_nationkey,
//             s_nationkey,
//             s_suppkey,
//             n_nationkey,
//             n_regionkey,
//             n_name,
//             r_regionkey,
//             r_name,
//             o1_condition,
//             o2_condition,
//             r_condition,
//             _marker: PhantomData,
//         };

//         // let z = [Fp::from(1 * (N as u64))];
//         // let z = [
//         //     Fp::from(0),
//         //     Fp::from(1),
//         //     Fp::from(0),
//         //     Fp::from(0),
//         //     Fp::from(1),
//         // ];

//         // let prover = MockProver::run(k, &circuit, vec![z.to_vec()]).unwrap();
//         let prover = MockProver::run(k, &circuit, vec![]).unwrap();
//         prover.assert_satisfied();
//     }
// }
// // time cargo test --package halo2-experiments --lib -- sql::q3_final_v1::tests::test_1 --exact --nocapture
