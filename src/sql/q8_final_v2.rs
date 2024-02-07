// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};
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
// use std::time::Instant;

// use std::mem;

// const N: usize = 1;
// const NUM_BYTES: usize = 5;

// // #[derive(Default)]
// // We should use the selector to skip the row which does not satisfy shipdate values

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: Vec<Selector>,
//     q_sort: Vec<Selector>,
//     q_accu: Selector,
//     q_perm: Vec<Selector>,

//     part: Vec<Column<Advice>>,      // p_partkey, p_type
//     supplier: Vec<Column<Advice>>,  // s_suppkey, s_nationkey
//     lineitem: Vec<Column<Advice>>, // l_extendedprice, l_discount, l_partkey, l_suppkey, l_orderkey,
//     orders: Vec<Column<Advice>>, // o_year , o_orderdate, o_orderkey, o_custkey      (o_year needs to be generated as an extra column)
//     customer: Vec<Column<Advice>>, // c_custkey, c_nationkey,
//     nation_n1: Vec<Column<Advice>>, // n_nationkey, n_regionkey
//     nation_n2: Vec<Column<Advice>>, // n_nationkey, n_regionkey, n_name
//     regions: Vec<Column<Advice>>, // r_name, r_regionkey

//     condition: Vec<Column<Advice>>, // r_name = ':2', o_orderdate between date '1995-01-01' and date '1996-12-31', p_type = ':3'

//     check: Vec<Column<Advice>>,

//     groupby: Vec<Column<Advice>>,

//     join_group: Vec<Vec<Column<Advice>>>, // its length is 12
//     disjoin_group: Vec<Vec<Column<Advice>>>,

//     deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey

//     dedup_sort: Vec<Column<Advice>>,
//     join: Vec<Column<Advice>>,

//     sum_col: Vec<Column<Advice>>, // its length is 2, sum(case	when nation = ':1' then volumn	else 0 end), sum(volume), mkt_share is represented with the previous two
//     equal_check: Column<Advice>,

//     equal_condition: Vec<IsZeroConfig<F>>,
//     compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
//     lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,
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
//         let mut q_enable = Vec::new();
//         for i_ in 0..5 {
//             q_enable.push(meta.complex_selector());
//         }
//         let mut q_sort = Vec::new();
//         for i_ in 0..8 {
//             q_sort.push(meta.complex_selector());
//         }
//         let mut q_perm = Vec::new();
//         for i_ in 0..1 {
//             q_perm.push(meta.complex_selector());
//         }
//         let q_accu = meta.complex_selector();

//         let mut part = Vec::new();
//         let mut supplier = Vec::new();
//         let mut lineitem = Vec::new();
//         let mut orders = Vec::new();
//         let mut customer = Vec::new();
//         let mut nation_n1 = Vec::new();
//         let mut nation_n2 = Vec::new();
//         let mut regions = Vec::new();

//         for _ in 0..2 {
//             part.push(meta.advice_column());
//             supplier.push(meta.advice_column());
//             customer.push(meta.advice_column());
//             nation_n1.push(meta.advice_column());
//             regions.push(meta.advice_column());
//         }

//         for _ in 0..3 {
//             nation_n2.push(meta.advice_column());
//         }
//         for _ in 0..4 {
//             orders.push(meta.advice_column());
//         }
//         for _ in 0..5 {
//             lineitem.push(meta.advice_column());
//         }

//         let mut condition = Vec::new();
//         let mut check = Vec::new();
//         for _ in 0..5 {
//             condition.push(meta.advice_column());
//             check.push(meta.advice_column());
//         }

//         let mut join_group = Vec::new();
//         let mut disjoin_group = Vec::new();

//         for l in [2, 5, 5, 2, 5, 4, 4, 2, 2, 2, 2, 2, 2, 3] {
//             let mut col = Vec::new();
//             for _ in 0..l {
//                 col.push(meta.advice_column());
//             }
//             join_group.push(col.clone());
//             disjoin_group.push(col.clone());
//         }

//         let mut deduplicate = Vec::new();
//         let mut dedup_sort = Vec::new();

//         for _ in 0..7 {
//             dedup_sort.push(meta.advice_column());
//         }
//         for _ in 0..14 {
//             deduplicate.push(meta.advice_column());
//         }

//         let mut join = Vec::new();
//         let mut groupby = Vec::new();
//         for _ in 0..3 {
//             join.push(meta.advice_column());
//             groupby.push(meta.advice_column());
//         }
//         let mut sum_col = Vec::new();
//         for _ in 0..2 {
//             sum_col.push(meta.advice_column());
//         }
//         let equal_check = meta.advice_column();
//         let mut is_zero_advice_column = Vec::new();
//         for _ in 0..2 {
//             is_zero_advice_column.push(meta.advice_column());
//         }

//         let vectors = [
//             &part,
//             &supplier,
//             &lineitem,
//             &orders,
//             &customer,
//             &nation_n1,
//             &nation_n2,
//             &regions,
//             &condition,
//             &check,
//             &deduplicate,
//             &dedup_sort,
//             &join,
//             &sum_col,
//         ];

//         // Applying meta.enable_equality() to each element of each vector
//         for vec in vectors {
//             for element in vec.iter() {
//                 meta.enable_equality(*element);
//             }
//         }
//         // For 'join_group' and 'disjoin_group' (assuming you want to apply it to these as well)
//         for vec in join_group.iter().chain(disjoin_group.iter()) {
//             for &element in vec.iter() {
//                 meta.enable_equality(element);
//             }
//         }

//         // Apply to `equal_check` separately since it's not in a vector
//         meta.enable_equality(equal_check);

//         // r_name = ':1'
//         let mut equal_condition = Vec::new();
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]), // this is the q_enable
//             |meta| {
//                 meta.query_advice(regions[0], Rotation::cur())
//                     - meta.query_advice(condition[0], Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column[0], // this is the advice column that stores value_inv
//         );
//         equal_condition.push(config.clone());

//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(q_enable[0]);
//             let output = meta.query_advice(check[0], Rotation::cur());
//             vec![
//                 s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
//             ]
//         });
//         // o_orderdate between date '1995-01-01' and date '1996-12-31'
//         let mut compare_condition = Vec::new();
//         let config: LtEqGenericConfig<F, NUM_BYTES> = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[1]),
//             |meta| vec![meta.query_advice(condition[1], Rotation::cur())],
//             |meta| vec![meta.query_advice(orders[1], Rotation::cur())],
//         );
//         meta.create_gate(
//             "verifies 1995-01-01 <= o_orderdate", // just use less_than for testing here
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable[1]);
//                 let check = meta.query_advice(check[1], Rotation::cur());
//                 vec![q_enable * (config.clone().is_lt(meta, None) - check)]
//             },
//         );
//         compare_condition.push(config);
//         let config: LtEqGenericConfig<F, NUM_BYTES> = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[2]),
//             |meta| vec![meta.query_advice(orders[1], Rotation::cur())],
//             |meta| vec![meta.query_advice(condition[2], Rotation::cur())],
//         );
//         meta.create_gate(
//             "verifies  o_orderdate <= 1996-12-31", // just use less_than for testing here
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable[2]);
//                 let check = meta.query_advice(check[2], Rotation::cur());
//                 vec![q_enable * (config.clone().is_lt(meta, None) - check)]
//             },
//         );
//         compare_condition.push(config);
//         // p_type = ':3'
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[3]), // this is the q_enable
//             |meta| {
//                 meta.query_advice(part[1], Rotation::cur())
//                     - meta.query_advice(condition[3], Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column[0], // this is the advice column that stores value_inv
//         );
//         equal_condition.push(config.clone());

//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(q_enable[3]);
//             let output = meta.query_advice(check[3], Rotation::cur());
//             vec![
//                 s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
//             ]
//         });
//         // nation = ':1'
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[4]), // this is the q_enable
//             |meta| {
//                 meta.query_advice(nation_n2[2], Rotation::cur())
//                     - meta.query_advice(condition[4], Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column[0], // this is the advice column that stores value_inv
//         );
//         equal_condition.push(config.clone());

//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(q_enable[4]);
//             let output = meta.query_advice(check[4], Rotation::cur());
//             vec![
//                 s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
//             ]
//         });

//         // join sort check
//         let mut lt_compare_condition = Vec::new();
//         for i in 0..7 {
//             let config = LtChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_sort[i]),
//                 |meta| meta.query_advice(dedup_sort[i], Rotation::prev()),
//                 |meta| meta.query_advice(dedup_sort[i], Rotation::cur()), // we put the left and right value at the first two positions of value_l
//             );
//             lt_compare_condition.push(config.clone());
//             meta.create_gate("t[i-1]<t[i]'", |meta| {
//                 let q_enable = meta.query_selector(q_sort[i]);
//                 vec![q_enable * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
//             });
//         }

//         // group by o_year
//         let config = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[7]),
//             |meta| vec![meta.query_advice(groupby[0], Rotation::prev())],
//             |meta| vec![meta.query_advice(groupby[0], Rotation::cur())],
//         );
//         compare_condition.push(config);

//         // groupby permutation check
//         let mut perm = Vec::new();
//         let config = PermAnyChip::configure(meta, q_perm[0], join.clone(), groupby.clone());
//         perm.push(config);

//         // sum gate: sum(l_extendedprice * (1 - l_discount)) as revenue, note that revenue column starts by zero and its length is 1 more than others
//         meta.create_gate("accumulate constraint", |meta| {
//             let q_accu = meta.query_selector(q_accu);
//             let prev_nation_volumn = meta.query_advice(sum_col[0].clone(), Rotation::cur());
//             let prev_volumn = meta.query_advice(sum_col[1].clone(), Rotation::cur());
//             let extendedprice = meta.query_advice(groupby[1], Rotation::cur());
//             let discount = meta.query_advice(groupby[2], Rotation::cur());
//             let sum_nation_volumn = meta.query_advice(sum_col[0], Rotation::next());
//             let sum_volumn = meta.query_advice(sum_col[1], Rotation::next());
//             let equal_check = meta.query_advice(equal_check, Rotation::cur());
//             let n_check = meta.query_advice(check[4], Rotation::cur());
//             let volumn_value =
//                 extendedprice.clone() * (Expression::Constant(F::from(1000)) - discount.clone());
//             vec![
//                 q_accu.clone()
//                     * (equal_check.clone() * prev_nation_volumn + volumn_value.clone()
//                         - sum_nation_volumn),
//                 q_accu * (equal_check * prev_volumn + volumn_value * n_check - sum_volumn),
//             ]
//         });

//         TestCircuitConfig {
//             q_enable,
//             q_accu,
//             q_perm,
//             q_sort,
//             part,
//             supplier,
//             lineitem,
//             orders,
//             customer,
//             nation_n1,
//             nation_n2,
//             regions,
//             condition,
//             check,
//             groupby,
//             join,
//             join_group,
//             disjoin_group,
//             dedup_sort,
//             deduplicate,
//             sum_col,
//             equal_check,
//             equal_condition,
//             compare_condition,
//             lt_compare_condition,
//             perm,
//         }
//     }

//     pub fn assign(
//         &self,
//         layouter: &mut impl Layouter<F>,
//         part: Vec<Vec<F>>,
//         supplier: Vec<Vec<F>>,
//         lineitem: Vec<Vec<F>>,
//         orders: Vec<Vec<F>>,
//         customer: Vec<Vec<F>>,
//         nation_n1: Vec<Vec<F>>,
//         nation_n2: Vec<Vec<F>>,
//         regions: Vec<Vec<F>>, // i.e. region, since halo2 has region keyword, we use different word here

//         condition: Vec<F>, // its last value is "nation = ':1' then volume"
//     ) -> Result<(), Error> {
//         let mut equal_chip = Vec::new();
//         let mut compare_chip = Vec::new();
//         let mut lt_compare_chip = Vec::new();
//         let mut perm_chip: Vec<PermAnyChip<F>> = Vec::new();

//         for i in 0..self.config.equal_condition.len() {
//             let chip = IsZeroChip::construct(self.config.equal_condition[i].clone());
//             equal_chip.push(chip);
//         }
//         for i in 0..self.config.compare_condition.len() {
//             let chip = LtEqGenericChip::construct(self.config.compare_condition[i].clone());
//             chip.load(layouter)?;
//             compare_chip.push(chip);
//         }

//         for i in 0..self.config.lt_compare_condition.len() {
//             let chip = LtChip::construct(self.config.lt_compare_condition[i].clone());
//             chip.load(layouter)?;
//             lt_compare_chip.push(chip);
//         }

//         println!("equal chip: {:?}", equal_chip.len());
//         println!("compare chip: {:?}", compare_chip.len());
//         println!("lt compre chip: {:?}", lt_compare_chip.len());

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 // locally compute the values for conditional check
//                 let mut o1_check = Vec::new();
//                 let mut o2_check = Vec::new();
//                 let mut r_check = Vec::new();
//                 let mut p_check = Vec::new();

//                 //assign input values
//                 for i in 0..part.len() {
//                     for j in 0..part[0].len() {
//                         region.assign_advice(
//                             || "p",
//                             self.config.part[j],
//                             i,
//                             || Value::known(part[i][j]),
//                         )?;
//                     }
//                     if part[i][1] == condition[3] {
//                         // p_type = ':3'
//                         p_check.push(F::from(1));
//                     } else {
//                         p_check.push(F::from(0));
//                     }
//                     region.assign_advice(
//                         || "check0",
//                         self.config.check[3],
//                         i,
//                         || Value::known(p_check[i]),
//                     )?;

//                     region.assign_advice(
//                         || "condition for part",
//                         self.config.condition[3],
//                         i,
//                         || Value::known(condition[3]),
//                     )?;

//                     equal_chip[0].assign(
//                         &mut region,
//                         i,
//                         Value::known(part[i][1] - condition[3]),
//                     )?; //p_type = ':3'
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
//                 for i in 0..lineitem.len() {
//                     for j in 0..lineitem[0].len() {
//                         region.assign_advice(
//                             || "l",
//                             self.config.lineitem[j],
//                             i,
//                             || Value::known(lineitem[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..orders.len() {
//                     for j in 0..orders[0].len() {
//                         region.assign_advice(
//                             || "o",
//                             self.config.orders[j],
//                             i,
//                             || Value::known(orders[i][j]),
//                         )?;
//                     }
//                     if condition[1] < orders[i][1] {
//                         o1_check.push(true);
//                     } else {
//                         o1_check.push(false);
//                     }
//                     if orders[i][0] < condition[2] {
//                         o2_check.push(true);
//                     } else {
//                         o2_check.push(false);
//                     }

//                     region.assign_advice(
//                         || "check1",
//                         self.config.check[1],
//                         i,
//                         || Value::known(F::from(o1_check[i] as u64)),
//                     )?;
//                     region.assign_advice(
//                         || "check1",
//                         self.config.check[2],
//                         i,
//                         || Value::known(F::from(o2_check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "condition for orders",
//                         self.config.condition[1],
//                         i,
//                         || Value::known(condition[1]),
//                     )?;
//                     region.assign_advice(
//                         || "condition for orders",
//                         self.config.condition[2],
//                         i,
//                         || Value::known(condition[2]),
//                     )?;

//                     compare_chip[0].assign(&mut region, i, &[condition[1]], &[orders[i][1]])?;
//                     compare_chip[1].assign(&mut region, i, &[orders[i][1]], &[condition[2]])?;
//                 }
//                 for i in 0..customer.len() {
//                     for j in 0..customer[0].len() {
//                         region.assign_advice(
//                             || "c",
//                             self.config.customer[j],
//                             i,
//                             || Value::known(customer[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..nation_n1.len() {
//                     for j in 0..nation_n1[0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.nation_n1[j],
//                             i,
//                             || Value::known(nation_n1[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..nation_n2.len() {
//                     for j in 0..nation_n2[0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.nation_n2[j],
//                             i,
//                             || Value::known(nation_n2[i][j]),
//                         )?;
//                     }
//                 }
//                 for i in 0..regions.len() {
//                     for j in 0..regions[0].len() {
//                         region.assign_advice(
//                             || "region",
//                             self.config.regions[j],
//                             i,
//                             || Value::known(regions[i][j]),
//                         )?;
//                     }
//                     if regions[i][0] == condition[0] {
//                         // r_name = ':2'
//                         r_check.push(F::from(1));
//                     } else {
//                         r_check.push(F::from(0));
//                     }
//                     region.assign_advice(
//                         || "check0",
//                         self.config.check[0],
//                         i,
//                         || Value::known(r_check[i]),
//                     )?;

//                     region.assign_advice(
//                         || "condition for customer",
//                         self.config.condition[0],
//                         i,
//                         || Value::known(condition[0]),
//                     )?;

//                     equal_chip[1].assign(
//                         &mut region,
//                         i,
//                         Value::known(regions[i][0] - condition[0]),
//                     )?; // r_name = ':1'
//                 }

//                 let mut p_combined = part.clone();
//                 let mut s_combined = supplier.clone();
//                 let mut l_combined = lineitem.clone();
//                 let mut o_combined = orders.clone();
//                 let mut c_combined = customer.clone();
//                 let mut n1_combined = nation_n1.clone();
//                 let mut n2_combined = nation_n2.clone();
//                 let mut r_combined = regions.clone();

//                 let mut p_combined: Vec<Vec<_>> = p_combined // due to p_type = ':3'
//                     .into_iter()
//                     .filter(|row| row[1] == condition[3])
//                     .collect();

//                 let o_combined: Vec<Vec<_>> = o_combined // due to and o_orderdate between date '1995-01-01' and date '1996-12-31'
//                     .into_iter()
//                     .filter(|row| condition[1] <= row[1] && row[1] <= condition[2])
//                     .collect();

//                 let r_combined: Vec<Vec<_>> = r_combined // due to p_type = ':3'
//                     .into_iter()
//                     .filter(|row| row[0] == condition[0])
//                     .collect();

//                 //create the values for join and disjoin

//                 let mut join_value: Vec<Vec<_>> = vec![Default::default(); 14];
//                 let mut disjoin_value: Vec<Vec<_>> = vec![Default::default(); 14];
//                 // p_partkey = l_partkey, s_suppkey = l_suppkey, l_orderkey = o_orderkey, o_custkey = c_custkey,
//                 // c_nationkey = n1.n_nationkey, n1.n_regionkey = r_regionkey, s_nationkey = n2.n_nationkey
//                 let mut combined = Vec::new();
//                 combined.push(p_combined); // 2
//                 combined.push(l_combined); // its length is 5
//                 combined.push(s_combined); // 2
//                 combined.push(o_combined); // 4
//                 combined.push(c_combined); // 2
//                 combined.push(n1_combined); // 2
//                 combined.push(r_combined); // 2
//                 combined.push(n2_combined); // 3
//                                             // (input1 index, input2 index, join attribute index of input1, join attribute of input2)
//                 let index = [
//                     (0, 1, 0, 2),
//                     (1, 2, 3, 0),
//                     (1, 3, 4, 2),
//                     (3, 4, 3, 0),
//                     (4, 5, 1, 0),
//                     (5, 6, 1, 1),
//                     (2, 7, 1, 0),
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

//                 // assign join and disjoin values
//                 for k in 0..join_value.len() {
//                     // Assuming self.config has an array or vector of configurations corresponding to k

//                     let join_config = &self.config.join_group[k]; // Adjust this based on your actual config structure

//                     // Process join_value[k]
//                     for i in 0..join_value[k].len() {
//                         for j in 0..join_value[k][i].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 join_config[j],
//                                 i,
//                                 || Value::known(join_value[k][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 for k in 0..disjoin_value.len() {
//                     // Assuming self.config has an array or vector of configurations corresponding to k
//                     if k > 0 {
//                         let disjoin_config = &self.config.disjoin_group[k]; // Adjust this as well
//                                                                             // Process disjoin_value[k]
//                         for i in 0..disjoin_value[k].len() {
//                             for j in 0..disjoin_value[k][i].len() {
//                                 region.assign_advice(
//                                     || "n",
//                                     disjoin_config[j],
//                                     i,
//                                     || Value::known(disjoin_value[k][i][j]),
//                                 )?;
//                             }
//                         }
//                     }
//                 }

//                 // compute final table by applying all joins
//                 let join_index = [
//                     (0, 1, 0, 2),
//                     (1, 2, 2 + 3, 0),
//                     (1, 3, 2 + 4, 2),
//                     (3, 4, 2 + 5 + 2 + 3, 0),
//                     (4, 5, 2 + 5 + 2 + 4 + 1, 0),
//                     (5, 6, 2 + 5 + 2 + 4 + 2 + 1, 1),
//                     (2, 7, 2 + 5 + 1, 0),
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
//                 //     "product: {:?}, {:?}",
//                 //     cartesian_product.len(),
//                 //     cartesian_product[0].len()
//                 // );

//                 let input = cartesian_product.clone(); // for permanychip inputs

//                 let mut dis_vectors: Vec<Vec<F>> = Vec::new();

//                 for i in 0..disjoin_value.len() {
//                     let idx = if i % 2 == 0 {
//                         index[i / 2].2
//                     } else {
//                         index[i / 2].3
//                     };
//                     let mut column: Vec<F> = disjoin_value[i].iter().map(|v| v[idx]).collect();
//                     dis_vectors.push(column);
//                 }

//                 for dis_vector in dis_vectors.iter_mut() {
//                     dis_vector.sort_by(|a, b| a.partial_cmp(b).unwrap());
//                     dis_vector.dedup();
//                 }

//                 let deduplicate_indices = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

//                 for (index, dis_vector) in dis_vectors.iter().enumerate() {
//                     for (i, value) in dis_vector.iter().enumerate() {
//                         region.assign_advice(
//                             || "deduplicated",
//                             self.config.deduplicate[deduplicate_indices[index]],
//                             i,
//                             || Value::known(*value),
//                         )?;
//                     }
//                 }

//                 // concatenate two vectors for sorting
//                 let mut concatenated_vectors = Vec::new();
//                 for i in (0..dis_vectors.len()).step_by(2) {
//                     if let (Some(first), Some(second)) =
//                         (dis_vectors.get(i), dis_vectors.get(i + 1))
//                     {
//                         let concatenated = first
//                             .iter()
//                             .cloned()
//                             .chain(second.iter().cloned())
//                             .collect::<Vec<F>>();
//                         concatenated_vectors.push(concatenated);
//                     }
//                 }

//                 for mut element in &mut concatenated_vectors {
//                     element.sort();
//                 }

//                 // assign the new dedup
//                 for k in 0..concatenated_vectors.len() {
//                     for i in 0..concatenated_vectors[k].len() {
//                         if i > 0 {
//                             self.config.q_sort[k].enable(&mut region, i)?; // start at index 1

//                             lt_compare_chip[k].assign(
//                                 // dedup_sort[][i-1] < dedup_sort[][i]
//                                 &mut region,
//                                 i,
//                                 Value::known(concatenated_vectors[k][i - 1]),
//                                 Value::known(concatenated_vectors[k][i]),
//                             )?;
//                         }
//                         region.assign_advice(
//                             || "new_dedup",
//                             self.config.dedup_sort[k],
//                             i,
//                             || Value::known(concatenated_vectors[k][i]),
//                         )?;
//                     }
//                 }

//                 // select
//                 // extract(year from o_orderdate) as o_year,
//                 // l_extendedprice * (1 - l_discount) as volume,
//                 // n2.n_name as nation

//                 // join1 is used for permutation cheeck and later operations
//                 let mut join1: Vec<Vec<F>> = cartesian_product
//                     .iter()
//                     .map(|v| {
//                         let mut new_vec = Vec::new();
//                         if v.len() >= 1 {
//                             new_vec.push(v[9]); // extract(year from o_orderdate) as o_year,
//                             new_vec.push(v[4] * (F::from(1000) - v[5])); // l_extendedprice * (1 - l_discount) as volume,
//                             new_vec.push(v[19]); // n2.n_name as nation
//                         }
//                         new_vec
//                     })
//                     .collect();
//                 // assign join:
//                 for i in 0..join1.len() {
//                     for j in 0..join1[0].len() {
//                         region.assign_advice(
//                             || "join",
//                             self.config.join[j],
//                             i,
//                             || Value::known(join1[i][j]),
//                         )?;
//                     }
//                 }

//                 // group by o_year
//                 join1.sort_by_key(|v| v[0]);

//                 for i in 0..join1.len() {
//                     for j in 0..join1[0].len() {
//                         region.assign_advice(
//                             || "groupby",
//                             self.config.groupby[j],
//                             i,
//                             || Value::known(join1[i][j]),
//                         )?;
//                     }
//                 }

//                 // add a new column "nation = '1' " into join1
//                 for vec in &mut join1 {
//                     if vec[2] == condition[4] {
//                         vec.push(F::from(1));
//                     } else {
//                         vec.push(F::from(0));
//                     }
//                 }

//                 for i in 0..join1.len() {
//                     // if join1[i][2] == condition[4] {
//                     //     n_check.push(F::from(1));
//                     // } else {
//                     //     n_check.push(F::from(0));
//                     // }
//                     region.assign_advice(
//                         || "check4",
//                         self.config.check[4],
//                         i,
//                         || Value::known(join1[i][3]), // nation = ':1'
//                     )?;

//                     region.assign_advice(
//                         || "condition for nation",
//                         self.config.condition[4],
//                         i,
//                         || Value::known(condition[4]),
//                     )?;

//                     equal_chip[2].assign(
//                         &mut region,
//                         i,
//                         Value::known(join1[i][2] - condition[4]),
//                     )?; //p_type = ':3'
//                     if i > 0 {
//                         self.config.q_sort[7].enable(&mut region, i)?;
//                         compare_chip[2].assign(
//                             &mut region,
//                             i,
//                             &[join1[i - 1][0]],
//                             &[join1[i][1]],
//                         )?;
//                     }
//                 }

//                 let mut equal_check: Vec<F> = Vec::new();

//                 if join1.len() > 0 {
//                     equal_check.push(F::from(0)); // add the the first one must be 0
//                 }

//                 for row in 1..join1.len() {
//                     // self.config.q_sort[6].enable(&mut region, row)?;
//                     if join1[row][0] == join1[row - 1][0] {
//                         // check if o_year[i-1] = o_year[i]
//                         equal_check.push(F::from(1));
//                     } else {
//                         equal_check.push(F::from(0));
//                     }
//                 }

//                 for i in 0..equal_check.len() {
//                     self.config.q_accu.enable(&mut region, i)?;
//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.equal_check,
//                         i,
//                         || Value::known(equal_check[i]),
//                     )?;
//                 }

//                 // volumn
//                 let n = join1.len();
//                 let mut nation_volumn: Vec<F> = vec![F::from(0); n]; // when nation = ':1' then
//                 let mut volumn: Vec<F> = vec![F::from(0); n]; // sum(volumn)
//                 for i in 1..n {
//                     nation_volumn[i] =
//                         nation_volumn[i - 1] * equal_check[i - 1] + join1[i - 1][1] * join1[i][3];
//                     volumn[i] = volumn[i - 1] * equal_check[i - 1] + join1[i - 1][1];
//                 }

//                 for i in 0..n {
//                     region.assign_advice(
//                         || "nation_volumn",
//                         self.config.sum_col[0],
//                         i,
//                         || Value::known(nation_volumn[i]),
//                     )?;
//                     region.assign_advice(
//                         || "volumn",
//                         self.config.sum_col[1],
//                         i,
//                         || Value::known(volumn[i]),
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
//     lineitem: Vec<Vec<F>>,
//     orders: Vec<Vec<F>>,
//     customer: Vec<Vec<F>>,
//     nation_n1: Vec<Vec<F>>,
//     nation_n2: Vec<Vec<F>>,
//     regions: Vec<Vec<F>>,

//     pub condition: Vec<F>,
//     _marker: PhantomData<F>,
// }

// impl<F: Copy + Default> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             part: Vec::new(),
//             supplier: Vec::new(),
//             lineitem: Vec::new(),
//             orders: Vec::new(),
//             customer: Vec::new(),
//             nation_n1: Vec::new(),
//             nation_n2: Vec::new(),
//             regions: Vec::new(),

//             condition: vec![Default::default(); 4],

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
//             self.lineitem.clone(),
//             self.orders.clone(),
//             self.customer.clone(),
//             self.nation_n1.clone(),
//             self.nation_n2.clone(),
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
//     use crate::data::data_processing;
//     use chrono::{DateTime, NaiveDate, Utc};
//     use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};
//     use std::marker::PhantomData;

//     #[test]
//     fn test_1() {
//         let k = 16;

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
//         fn date_to_timestamp(date_str: &str) -> u64 {
//             match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
//                 Ok(date) => {
//                     let datetime: DateTime<Utc> =
//                         DateTime::<Utc>::from_utc(date.and_hms(0, 0, 0), Utc);
//                     datetime.timestamp() as u64
//                 }
//                 Err(_) => 0, // Return a default value like 0 in case of an error
//             }
//         }

//         // let customer_file_path = "/Users/binbingu/halo2-TPCH/src/data/customer.tbl";
//         // let orders_file_path = "/Users/binbingu/halo2-TPCH/src/data/orders.tbl";
//         // let lineitem_file_path = "/Users/binbingu/halo2-TPCH/src/data/lineitem.tbl";
//         // let supplier_file_path = "/Users/binbingu/halo2-TPCH/src/data/supplier.tbl";
//         // let nation_file_path = "/Users/binbingu/halo2-TPCH/src/data/nation.tbl";
//         // let region_file_path = "/Users/binbingu/halo2-TPCH/src/data/region.csv";

//         let part_file_path = "/home/cc/halo2-TPCH/src/data/part.tbl";
//         let supplier_file_path = "/home/cc/halo2-TPCH/src/data/supplier.tbl";
//         let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";
//         let orders_file_path = "/home/cc/halo2-TPCH/src/data/orders.tbl";
//         let customer_file_path = "/home/cc/halo2-TPCH/src/data/customer.tbl";
//         let nation_n1_file_path = "/home/cc/halo2-TPCH/src/data/nation.tbl";
//         let nation_n2_file_path = "/home/cc/halo2-TPCH/src/data/nation.tbl";
//         let region_file_path = "/home/cc/halo2-TPCH/src/data/region.cvs";

//         let mut part: Vec<Vec<Fp>> = Vec::new();
//         let mut supplier: Vec<Vec<Fp>> = Vec::new();
//         let mut lineitem: Vec<Vec<Fp>> = Vec::new();
//         let mut orders: Vec<Vec<Fp>> = Vec::new();
//         let mut customer: Vec<Vec<Fp>> = Vec::new();
//         let mut nation_n1: Vec<Vec<Fp>> = Vec::new();
//         let mut nation_n2: Vec<Vec<Fp>> = Vec::new();
//         let mut regions: Vec<Vec<Fp>> = Vec::new();

//         if let Ok(records) = data_processing::part_read_records_from_file(part_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             part = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.p_partkey),
//                         Fp::from(string_to_u64(&record.p_type)),
//                     ]
//                 })
//                 .collect();
//         }
//         if let Ok(records) = data_processing::supplier_read_records_from_file(supplier_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             supplier = records
//                 .iter()
//                 .map(|record| vec![Fp::from(record.s_suppkey), Fp::from(record.s_nationkey)])
//                 .collect();
//         }
//         if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             lineitem = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(scale_by_1000(record.l_extendedprice)),
//                         Fp::from(scale_by_1000(record.l_discount)),
//                         Fp::from(record.l_partkey),
//                         Fp::from(record.l_suppkey),
//                         Fp::from(record.l_orderkey),
//                     ]
//                 })
//                 .collect();
//         }
//         if let Ok(records) = data_processing::orders_read_records_from_file(orders_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             orders = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.o_orderdate[..4].parse::<u64>().unwrap()), // o_year
//                         Fp::from(date_to_timestamp(&record.o_orderdate)),
//                         Fp::from(record.o_orderkey),
//                         Fp::from(record.o_custkey),
//                     ]
//                 })
//                 .collect();
//         }
//         if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             customer = records
//                 .iter()
//                 .map(|record| vec![Fp::from(record.c_custkey), Fp::from(record.c_nationkey)])
//                 .collect();
//         }

//         if let Ok(records) = data_processing::nation_read_records_from_file(nation_n1_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             nation_n1 = records
//                 .iter()
//                 .map(|record| vec![Fp::from(record.n_nationkey), Fp::from(record.n_regionkey)])
//                 .collect();
//         }
//         if let Ok(records) = data_processing::nation_read_records_from_file(nation_n2_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             nation_n2 = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.n_nationkey),
//                         Fp::from(record.n_regionkey),
//                         Fp::from(string_to_u64(&record.n_name)),
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
//                         Fp::from(record.r_regionkey),
//                         Fp::from(string_to_u64(&record.r_name)),
//                     ]
//                 })
//                 .collect();
//         }

//         let condition = vec![
//             Fp::from(string_to_u64("MIDDLE EAST")),
//             Fp::from(date_to_timestamp("1995-01-01")),
//             Fp::from(date_to_timestamp("1996-12-31")),
//             Fp::from(string_to_u64("PROMO BRUSHED COPPER")),
//             Fp::from(string_to_u64("EGYPT")),
//         ];

//         // r_name = "MIDDLE EAST";
//         // 1995-01-01 -> 788918400
//         // 1996-12-31 -> 851990400
//         // p_type = "PROMO BRUSHED COPPER"
//         // nation = "EGYPT"

//         // 1991-01-01 -> 662688000 just for testing
//         //
//         // for i in 0..customer.len(){
//         //     for j in 0..orders.len(){
//         //         if customer[i][0] == orders[j][1]{
//         //             println!("customer: {:?}", customer[i][0]);
//         //             break;
//         //         }

//         //     }
//         // }

//         // let part: Vec<Vec<Fp>> = part.iter().take(300).cloned().collect();
//         // let supplier: Vec<Vec<Fp>> = supplier.iter().take(300).cloned().collect();
//         // let lineitem: Vec<Vec<Fp>> = lineitem.iter().take(300).cloned().collect();
//         // let orders: Vec<Vec<Fp>> = orders.iter().take(300).cloned().collect();
//         // let customer: Vec<Vec<Fp>> = customer.iter().take(300).cloned().collect();

//         // let nation_n1: Vec<Vec<Fp>> = nation.iter().take(3).cloned().collect();
//         // let nation_n1: Vec<Vec<Fp>> = nation.iter().take(3).cloned().collect();
//         // let region: Vec<Vec<Fp>> = region.iter().take(3).cloned().collect();

//         // let lineitem: Vec<Vec<Fp>> = vec![
//         //     vec![Fp::from(4), Fp::from(2), Fp::from(1), Fp::from(11)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(12)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(13)],
//         // ];

//         // let supplier: Vec<Vec<Fp>> = vec![
//         //     vec![Fp::from(1), Fp::from(2)],
//         //     vec![Fp::from(1), Fp::from(3)],
//         //     vec![Fp::from(1), Fp::from(4)],
//         // ];

//         // let nation: Vec<Vec<Fp>> = vec![
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(1)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(2)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(6)],
//         // ];
//         // let condition = vec![Fp::from(1615), Fp::from(852076800), Fp::from(883612800)];

//         let circuit = MyCircuit::<Fp> {
//             part,
//             supplier,
//             lineitem,
//             orders,
//             customer,
//             nation_n1,
//             nation_n2,
//             regions,
//             condition,
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
