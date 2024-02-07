// use eth_types::Field;
// use gadgets::util::or;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};
// use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::lessthan_or_equal::{LtEqChip, LtEqConfig, LtEqInstruction};
// use gadgets::lessthan_or_equal_generic::{
//     LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
// };

// use std::thread::sleep;
// use std::{default, marker::PhantomData};

// // use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
// use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
// use itertools::iproduct;
// use std::cmp::Ordering;
// use std::collections::HashMap;
// use std::collections::HashSet;
// use std::mem;
// use std::time::Instant;

// const N: usize = 10;
// const NUM_BYTES: usize = 5;

// // #[derive(Default)]
// // We should use the selector to skip the row which does not satisfy shipdate values

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: Vec<Selector>,
//     q_join: Vec<Selector>,
//     q_dedup: Vec<Selector>,
//     q_perm: Vec<Selector>,

//     q_sort: Vec<Selector>,
//     q_accu: Selector,

//     customer: Vec<Column<Advice>>, // c_mkt, c_custkey
//     orders: Vec<Column<Advice>>,   // o_orderdate, o_shippri, o_custkey, o_orderkey
//     lineitem: Vec<Column<Advice>>, // l_order, l_extened, l_dis, l_ship

//     join_and_disjoin_customer: Vec<Vec<Column<Advice>>>,
//     join_and_disjoin_orders: Vec<Vec<Column<Advice>>>,
//     join_and_disjoin_lineitem: Vec<Vec<Column<Advice>>>,

//     check: Vec<Column<Advice>>,

//     deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey

//     dedup_sort: Vec<Column<Advice>>,

//     condition: Vec<Column<Advice>>, // conditional value for l, c and o

//     join: Vec<Column<Advice>>,
//     groupby: Vec<Column<Advice>>,
//     orderby: Vec<Column<Advice>>,

//     equal_check: Column<Advice>, // check if sorted_revenue[i-1] = sorted_revenue[i]
//     revenue: Column<Advice>,

//     lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,

//     equal_condition: Vec<IsZeroConfig<F>>,
//     compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
//     perm: Vec<PermAnyConfig>,
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
//         let mut q_enable = Vec::new();
//         for i_ in 0..3 {
//             q_enable.push(meta.complex_selector());
//         }

//         let mut q_sort = Vec::new();
//         for i_ in 0..4 {
//             q_sort.push(meta.complex_selector());
//         }

//         let mut q_join = Vec::new();
//         for i_ in 0..8 {
//             q_join.push(meta.complex_selector());
//         }
//         let mut q_dedup = Vec::new();
//         for i_ in 0..2 {
//             q_dedup.push(meta.complex_selector());
//         }
//         let mut q_perm = Vec::new();
//         for i_ in 0..1 {
//             q_perm.push(meta.complex_selector());
//         }

//         let q_accu = meta.complex_selector();

//         let mut customer = Vec::new();
//         let mut orders = Vec::new();
//         let mut lineitem = Vec::new();

//         for _ in 0..2 {
//             customer.push(meta.advice_column());
//         }
//         for _ in 0..4 {
//             orders.push(meta.advice_column());
//             lineitem.push(meta.advice_column());
//         }

//         let mut join_and_disjoin_customer = Vec::new();
//         let mut join_and_disjoin_orders = Vec::new();
//         let mut join_and_disjoin_lineitem = Vec::new();
//         let mut deduplicate = Vec::new();
//         let mut dedup_sort = Vec::new();
//         let mut condition = Vec::new();

//         for _ in 0..2 {
//             dedup_sort.push(meta.advice_column());
//         }
//         for _ in 0..3 {
//             condition.push(meta.advice_column());
//         }

//         for _ in 0..4 {
//             deduplicate.push(meta.advice_column());
//         }

//         for _ in 0..2 {
//             let mut col = Vec::new();
//             for _ in 0..2 {
//                 col.push(meta.advice_column());
//             }
//             join_and_disjoin_customer.push(col.clone());
//         }

//         for _ in 0..4 {
//             let mut col = Vec::new();
//             for _ in 0..4 {
//                 col.push(meta.advice_column());
//             }
//             join_and_disjoin_orders.push(col.clone());
//         }

//         for _ in 0..2 {
//             let mut col = Vec::new();
//             for _ in 0..4 {
//                 col.push(meta.advice_column());
//             }
//             join_and_disjoin_lineitem.push(col.clone());
//         }

//         let mut join = Vec::new();
//         let mut groupby = Vec::new();
//         let mut orderby = Vec::new();
//         for _ in 0..5 {
//             join.push(meta.advice_column());
//             groupby.push(meta.advice_column());
//         }
//         for _ in 0..4 {
//             orderby.push(meta.advice_column());
//         }
//         let equal_check = meta.advice_column();
//         let revenue = meta.advice_column();

//         let mut is_zero_advice_column = Vec::new();
//         for _ in 0..2 {
//             is_zero_advice_column.push(meta.advice_column());
//         }

//         let mut check = Vec::new();
//         for _ in 0..3 {
//             check.push(meta.advice_column());
//         }
//         // equality
//         for i in 0..2 {
//             meta.enable_equality(customer[i]);

//             meta.enable_equality(deduplicate[i]);
//             meta.enable_equality(dedup_sort[i]);
//             meta.enable_equality(condition[i]);
//         }
//         for i in 0..4 {
//             meta.enable_equality(orders[i]);
//             meta.enable_equality(lineitem[i]);
//             meta.enable_equality(orderby[i]);
//         }
//         for i in 0..5 {
//             meta.enable_equality(groupby[i]);
//         }

//         for i in 0..2 {
//             for j in 0..2 {
//                 meta.enable_equality(join_and_disjoin_customer[i][j]);
//             }
//         }
//         for i in 0..4 {
//             for j in 0..4 {
//                 meta.enable_equality(join_and_disjoin_orders[i][j]);
//             }
//         }
//         for i in 0..2 {
//             for j in 0..4 {
//                 meta.enable_equality(join_and_disjoin_lineitem[i][j]);
//             }
//         }

//         meta.enable_equality(equal_check);
//         meta.enable_equality(revenue);

//         // c_mktsegment = ':1'
//         let mut equal_condition = Vec::new();
//         let config = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[0]), // this is the q_enable
//             |meta| {
//                 meta.query_advice(customer[0], Rotation::cur())
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

//         let mut lt_compare_condition = Vec::new();
//         // o_orderdate < date ':2'
//         let config = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[1]),
//             |meta| meta.query_advice(orders[0], Rotation::cur()),
//             |meta| meta.query_advice(condition[1], Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );
//         meta.create_gate(
//             "verifies o_orderdate < date ':2'", // just use less_than for testing here
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable[1]);
//                 let check = meta.query_advice(check[1], Rotation::cur());
//                 vec![q_enable * (config.is_lt(meta, None) - check)]
//             },
//         );
//         lt_compare_condition.push(config);
//         // l_shipdate > date ':2'
//         let config = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable[2]),
//             |meta| meta.query_advice(condition[2], Rotation::cur()),
//             |meta| meta.query_advice(lineitem[3], Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         meta.create_gate(
//             "verifies l_shipdate > date ':2'", // just use less_than for testing here
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable[2]);
//                 let check = meta.query_advice(check[2], Rotation::cur());
//                 vec![q_enable * (config.is_lt(meta, None) - check)]
//             },
//         );
//         lt_compare_condition.push(config);

//         // // join
//         // meta.shuffle("permutation check", |meta| {
//         //     // Inputs
//         //     let q0 = meta.query_selector(q_join[0]);
//         //     let q1 = meta.query_selector(q_join[1]);
//         //     let q2 = meta.query_selector(q_join[2]);
//         //     let q3 = meta.query_selector(q_join[3]);
//         //     let q4 = meta.query_selector(q_join[4]);
//         //     let q5 = meta.query_selector(q_join[5]);
//         //     let q6 = meta.query_selector(q_join[6]);
//         //     let q7 = meta.query_selector(q_join[7]);

//         //     let input1: Vec<_> = customer
//         //         .iter()
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();

//         //     let table1: Vec<_> = join_and_disjoin_customer
//         //         .iter()
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();

//         //     let input2: Vec<_> = orders
//         //         .iter()
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();

//         //     let table2: Vec<_> = join_and_disjoin_orders
//         //         .iter()
//         //         .take(2)
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();
//         //     let input3: Vec<_> = orders
//         //         .iter()
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();

//         //     let table3: Vec<_> = join_and_disjoin_orders
//         //         .iter()
//         //         .skip(2) // Skip the first two elements
//         //         .take(2)
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();
//         //     let input4: Vec<_> = lineitem
//         //         .iter()
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();

//         //     let table4: Vec<_> = join_and_disjoin_lineitem
//         //         .iter()
//         //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//         //         .collect();

//         //     let constraint0: Vec<_> = input1
//         //         .iter()
//         //         .zip(table1.iter())
//         //         .map(|(input, table)| (q0 * input.clone(), q1 * table.clone()))
//         //         .collect();
//         //     let constraint1: Vec<_> = input2
//         //         .iter()
//         //         .zip(table2.iter())
//         //         .map(|(input, table)| (q2 * input.clone(), q3 * table.clone()))
//         //         .collect();
//         //     let constraint2: Vec<_> = input3
//         //         .iter()
//         //         .zip(table3.iter())
//         //         .map(|(input, table)| (q4 * input.clone(), q5 * table.clone()))
//         //         .collect();
//         //     let constraint3: Vec<_> = input4
//         //         .iter()
//         //         .zip(table4.iter())
//         //         .map(|(input, table)| (q6 * input.clone(), q7 * table.clone()))
//         //         .collect();
//         //     let mut constraints = Vec::new();
//         //     constraints.extend(constraint0);
//         //     constraints.extend(constraint1);
//         //     constraints.extend(constraint2);
//         //     constraints.extend(constraint3);
//         //     constraints
//         // });

//         // disjoin check
//         // we can use copy_from copy_advice() to do the permutation check easily
//         // https://github.com/tuzijun111/halo2-example/blob/main/src/fibonacci/example1.rs#L108C24-L108C36

//         // permutation check after merging two deduplicate columns

//         // meta.shuffle(
//         //     "deduplicate permutation check'", // just use less_than for testing here
//         //     |meta| {
//         //         let q1 = meta.query_selector(q_dedup[4]);

//         //     },
//         // );

//         // join sort check

//         for i in 0..2 {
//             let config = LtChip::configure(
//                 meta,
//                 |meta| meta.query_selector(q_sort[i]),
//                 |meta| meta.query_advice(dedup_sort[i], Rotation::prev()),
//                 |meta| meta.query_advice(dedup_sort[i], Rotation::cur()), // we put the left and right value at the first two positions of value_l
//             );
//             lt_compare_condition.push(config.clone());
//             // meta.create_gate("t[i-1]<t[i]'", |meta| {
//             //     let q_enable = meta.query_selector(q_sort[i]);
//             //     vec![q_enable * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
//             // });
//         }

//         // group by
//         let mut compare_condition = Vec::new();
//         let config = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[2]),
//             |meta| {
//                 vec![
//                     meta.query_advice(groupby[0], Rotation::prev()),
//                     meta.query_advice(groupby[1], Rotation::prev()),
//                     meta.query_advice(groupby[2], Rotation::prev()),
//                 ]
//             },
//             |meta| {
//                 vec![
//                     meta.query_advice(groupby[0], Rotation::cur()),
//                     meta.query_advice(groupby[1], Rotation::cur()),
//                     meta.query_advice(groupby[2], Rotation::cur()),
//                 ]
//             },
//         );
//         compare_condition.push(config);

//         // groupby permutation check
//         let mut perm = Vec::new();
//         let config = PermAnyChip::configure(meta, q_perm[0], join.clone(), groupby.clone());
//         perm.push(config);

//         // sum gate: sum(l_extendedprice * (1 - l_discount)) as revenue, note that revenue column starts by zero and its length is 1 more than others
//         meta.create_gate("accumulate constraint", |meta| {
//             let q_accu = meta.query_selector(q_accu);
//             let prev_revenue = meta.query_advice(revenue.clone(), Rotation::cur());
//             let extendedprice = meta.query_advice(groupby[3], Rotation::cur());
//             let discount = meta.query_advice(groupby[4], Rotation::cur());
//             let sum_revenue = meta.query_advice(revenue, Rotation::next());
//             let check = meta.query_advice(equal_check, Rotation::cur());

//             vec![
//                 q_accu.clone()
//                     * (check.clone() * prev_revenue
//                         + extendedprice.clone()
//                             * (Expression::Constant(F::from(1000)) - discount.clone())
//                         - sum_revenue),
//             ]
//         });

//         // orderby

//         // (1) revenue[i-1] > revenue[i]
//         let config = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[3]), // q_sort[1] should start from index 1
//             |meta| vec![meta.query_advice(orderby[3], Rotation::cur())], // revenue
//             |meta| vec![meta.query_advice(orderby[3], Rotation::prev())],
//         );
//         compare_condition.push(config.clone());
//         // revenue[i-1] = revenue[i]
//         let equal_v1 = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[3]),
//             |meta| {
//                 meta.query_advice(orderby[3], Rotation::prev())
//                     - meta.query_advice(orderby[3], Rotation::cur())
//             },
//             is_zero_advice_column[1],
//         );
//         equal_condition.push(equal_v1.clone());

//         // o_orderdate[i-1] <= o_orderdate[i]
//         let config_lt = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort[3]), // q_sort[2] should start from index 1
//             |meta| vec![meta.query_advice(orderby[0], Rotation::prev())],
//             |meta| vec![meta.query_advice(orderby[0], Rotation::cur())],
//         );
//         compare_condition.push(config_lt.clone());

//         meta.create_gate("verifies orderby scenarios", |meta| {
//             let q_sort = meta.query_selector(q_sort[3]);

//             vec![
//                 q_sort.clone() *
//                 (config.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
//                         * (equal_v1.expr() * config_lt.is_lt(meta, None)
//                             - Expression::Constant(F::ONE)),
//             ]
//         });

//         TestCircuitConfig {
//             q_enable,
//             q_join,
//             q_dedup,
//             q_perm,
//             q_sort,
//             q_accu,

//             customer,
//             orders,
//             lineitem,

//             join_and_disjoin_customer,
//             join_and_disjoin_orders,
//             join_and_disjoin_lineitem,

//             check,
//             deduplicate,
//             dedup_sort,
//             condition,
//             join,
//             groupby,
//             orderby,
//             equal_check,

//             revenue,
//             lt_compare_condition,
//             equal_condition,
//             compare_condition,
//             perm,
//         }
//     }

//     pub fn assign(
//         &self,
//         // layouter: &mut impl Layouter<F>,
//         layouter: &mut impl Layouter<F>,

//         customer: Vec<Vec<F>>,
//         orders: Vec<Vec<F>>,
//         lineitem: Vec<Vec<F>>,

//         condition: [F; 2],
//     ) -> Result<(), Error> {
//         let mut equal_chip = Vec::new();
//         let mut compare_chip = Vec::new();
//         let mut lt_compare_chip = Vec::new();
//         let mut perm_chip = Vec::new();

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

//         // println!("equal chip: {:?}", equal_chip.len());
//         // println!("compare chip: {:?}", compare_chip.len());
//         // println!("lt compre chip: {:?}", lt_compare_chip.len());

//         for i in 0..self.config.perm.len() {
//             let config = PermAnyChip::construct(self.config.perm[i].clone());
//             perm_chip.push(config);
//         }

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 let start = Instant::now();

//                 let mut l_check = Vec::new(); // t/f
//                 let mut o_check = Vec::new(); // t/f
//                 let mut c_check = Vec::new(); // 0, 1

//                 //assign input values
//                 for i in 0..customer.len() {
//                     self.config.q_enable[0].enable(&mut region, i)?;
//                     for j in 0..customer[0].len() {
//                         region.assign_advice(
//                             || "customer",
//                             self.config.customer[j],
//                             i,
//                             || Value::known(customer[i][j]),
//                         )?;
//                     }
//                     if customer[i][0] == condition[0] {
//                         c_check.push(F::from(1));
//                     } else {
//                         c_check.push(F::from(0));
//                     }
//                     region.assign_advice(
//                         || "check0",
//                         self.config.check[0],
//                         i,
//                         || Value::known(c_check[i]),
//                     )?;

//                     region.assign_advice(
//                         || "condition for customer",
//                         self.config.condition[0],
//                         i,
//                         || Value::known(condition[0]),
//                     )?;

//                     equal_chip[0].assign(
//                         &mut region,
//                         i,
//                         Value::known(customer[i][0] - condition[0]),
//                     )?; // c_mktsegment = ':1'
//                 }
//                 for i in 0..orders.len() {
//                     self.config.q_enable[1].enable(&mut region, i)?;
//                     for j in 0..orders[0].len() {
//                         region.assign_advice(
//                             || "orders",
//                             self.config.orders[j],
//                             i,
//                             || Value::known(orders[i][j]),
//                         )?;
//                     }
//                     if orders[i][0] < condition[1] {
//                         o_check.push(true);
//                     } else {
//                         o_check.push(false);
//                     }
//                     region.assign_advice(
//                         || "check1",
//                         self.config.check[1],
//                         i,
//                         || Value::known(F::from(o_check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "condition for orders",
//                         self.config.condition[1],
//                         i,
//                         || Value::known(condition[1]),
//                     )?;

//                     lt_compare_chip[0].assign(
//                         &mut region,
//                         i,
//                         Value::known(orders[i][0]),
//                         Value::known(condition[1]),
//                     )?;
//                 }
//                 for i in 0..lineitem.len() {
//                     self.config.q_enable[2].enable(&mut region, i)?;
//                     for j in 0..lineitem[0].len() {
//                         region.assign_advice(
//                             || "lineitem",
//                             self.config.lineitem[j],
//                             i,
//                             || Value::known(lineitem[i][j]),
//                         )?;
//                     }
//                     if lineitem[i][3] > condition[1] {
//                         l_check.push(true);
//                     } else {
//                         l_check.push(false);
//                     }

//                     region.assign_advice(
//                         || "check2",
//                         self.config.check[2],
//                         i,
//                         || Value::known(F::from(l_check[i] as u64)),
//                     )?;
//                     region.assign_advice(
//                         || "condition for lineitem",
//                         self.config.condition[2],
//                         i,
//                         || Value::known(condition[1]),
//                     )?;
//                     lt_compare_chip[1].assign(
//                         &mut region,
//                         i,
//                         Value::known(condition[1]),
//                         Value::known(lineitem[i][3]),
//                     )?;
//                 }

//                 // for i in c_check.len(){
//                 //     if c_check == F::from(1) && o_check == true && l_check == true{
//                 //         self.config.q_perm[0].enable(&mut region, i)?; // selectors for permutation check after
//                 //     }

//                 // }

//                 // compute values related to the join operation locally
//                 // store the attribtues of the tables that will be used in the SQL query in tuples
//                 let mut c_combined = customer.clone();
//                 let mut o_combined = orders.clone();
//                 let mut l_combined = lineitem.clone();

//                 let duration_block = start.elapsed();
//                 println!("Time elapsed for block: {:?}", duration_block);

//                 let c_combined: Vec<Vec<_>> = c_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|row| row[0] == condition[0]) // r_name = ':3'
//                     .collect();

//                 let o_combined: Vec<Vec<_>> = o_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|row| row[0] < condition[1]) // r_name = ':3'
//                     .collect();

//                 let l_combined: Vec<Vec<_>> = l_combined
//                     .clone()
//                     .into_iter()
//                     .filter(|row| row[3] > condition[1]) // r_name = ':3'
//                     .collect();

//                 let mut join_value: Vec<Vec<_>> = vec![vec![]; 4];
//                 let mut disjoin_value: Vec<Vec<_>> = vec![vec![]; 4];

//                 let mut combined = Vec::new();
//                 combined.push(c_combined.clone()); // its length is 2
//                 combined.push(o_combined.clone()); // 4
//                 combined.push(l_combined.clone()); // 4

//                 let index = [
//                     (0, 1, 1, 2), //   c_custkey = o_custkey
//                     (1, 2, 3, 0), //   l_orderkey = o_orderkey
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
//                 // assign join and disjoin values
//                 for i in 0..join_value[0].len() {
//                     for j in 0..join_value[0][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_customer[0][j],
//                             i,
//                             || Value::known(join_value[0][i][j]),
//                         )?;
//                     }
//                 }

//                 for i in 0..disjoin_value[0].len() {
//                     for j in 0..disjoin_value[0][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_customer[1][j],
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
//                                 self.config.join_and_disjoin_orders[idx * 2][j],
//                                 i,
//                                 || Value::known(join_value[x][i][j]),
//                             )?;
//                         }
//                     }

//                     for i in 0..disjoin_value[x].len() {
//                         for j in 0..disjoin_value[x][0].len() {
//                             region.assign_advice(
//                                 || "n",
//                                 self.config.join_and_disjoin_orders[idx * 2 + 1][j],
//                                 i,
//                                 || Value::known(disjoin_value[x][i][j]),
//                             )?;
//                         }
//                     }
//                 }

//                 for i in 0..join_value[3].len() {
//                     for j in 0..join_value[3][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_lineitem[0][j],
//                             i,
//                             || Value::known(join_value[3][i][j]),
//                         )?;
//                     }
//                 }

//                 for i in 0..disjoin_value[3].len() {
//                     for j in 0..disjoin_value[3][0].len() {
//                         region.assign_advice(
//                             || "n",
//                             self.config.join_and_disjoin_lineitem[1][j],
//                             i,
//                             || Value::known(disjoin_value[3][i][j]),
//                         )?;
//                     }
//                 }

//                 // compute final table by applying all joins
//                 let join_index = [
//                     (0, 1, 1, 2),     //   c_custkey = o_custkey
//                     (1, 2, 2 + 3, 0), //   l_orderkey = o_orderkey
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

//                 // assign join: cartesian_product
//                 for i in 0..cartesian_product.len() {
//                     for (idx, &j) in [2, 3, 6, 7, 8].iter().enumerate() {
//                         region.assign_advice(
//                             || "groupby",
//                             self.config.join[idx],
//                             i,
//                             || Value::known(cartesian_product[i][j]),
//                         )?;
//                     }
//                 }
//                 let input = cartesian_product.clone(); // for permanychip inputs

//                 let mut dis_c_custkey: Vec<F> = disjoin_value[0].iter().map(|v| v[1]).collect();
//                 let mut dis_o_custkey: Vec<F> = disjoin_value[1].iter().map(|v| v[2]).collect();
//                 let mut dis_o_orderkey: Vec<F> = disjoin_value[2].iter().map(|v| v[3]).collect();
//                 let mut dis_l_orderkey: Vec<F> = disjoin_value[3].iter().map(|v| v[0]).collect();

//                 // generate deduplicated columns for disjoin values
//                 dis_c_custkey.sort_by(|a, b| a.partial_cmp(b).unwrap());
//                 dis_c_custkey.dedup();
//                 dis_o_custkey.sort_by(|a, b| a.partial_cmp(b).unwrap());
//                 dis_o_custkey.dedup();
//                 dis_o_orderkey.sort_by(|a, b| a.partial_cmp(b).unwrap());
//                 dis_o_orderkey.dedup();
//                 dis_l_orderkey.sort_by(|a, b| a.partial_cmp(b).unwrap());
//                 dis_l_orderkey.dedup();

//                 for i in 0..dis_c_custkey.len() {
//                     region.assign_advice(
//                         || "deduplicated_a2_vec",
//                         self.config.deduplicate[0],
//                         i,
//                         || Value::known(dis_c_custkey[i]),
//                     )?;
//                 }
//                 for i in 0..dis_o_custkey.len() {
//                     region.assign_advice(
//                         || "deduplicated_b2_vec",
//                         self.config.deduplicate[1],
//                         i,
//                         || Value::known(dis_o_custkey[i]),
//                     )?;
//                 }
//                 for i in 0..dis_o_orderkey.len() {
//                     region.assign_advice(
//                         || "deduplicated_c2_vec",
//                         self.config.deduplicate[2],
//                         i,
//                         || Value::known(dis_o_orderkey[i]),
//                     )?;
//                 }
//                 for i in 0..dis_l_orderkey.len() {
//                     region.assign_advice(
//                         || "deduplicated_d2_vec",
//                         self.config.deduplicate[3],
//                         i,
//                         || Value::known(dis_l_orderkey[i]),
//                     )?;
//                 }

//                 // concatenate two vectors for sorting
//                 let mut new_dedup_1: Vec<F> =
//                     dis_c_custkey.into_iter().chain(dis_o_custkey).collect();
//                 let mut new_dedup_2: Vec<F> =
//                     dis_o_orderkey.into_iter().chain(dis_l_orderkey).collect();
//                 // sort them
//                 new_dedup_1.sort();
//                 new_dedup_2.sort();
//                 // assign the new dedup
//                 // println!("new_dedup_1 {:?}", new_dedup_1.len());
//                 for i in 0..new_dedup_1.len() {
//                     if i > 0 {
//                         self.config.q_sort[0].enable(&mut region, i)?; // start at index 1

//                         lt_compare_chip[2].assign(
//                             // dedup_sort[][i-1] < dedup_sort[][i]
//                             &mut region,
//                             i,
//                             Value::known(new_dedup_1[i - 1]),
//                             Value::known(new_dedup_1[i]),
//                         )?;
//                     }
//                     region.assign_advice(
//                         || "new_dedup_1",
//                         self.config.dedup_sort[0],
//                         i,
//                         || Value::known(new_dedup_1[i]),
//                     )?;
//                 }
//                 // println!("new_dedup_2 {:?}", new_dedup_2.len());
//                 for i in 0..new_dedup_2.len() {
//                     if i > 0 {
//                         self.config.q_sort[1].enable(&mut region, i)?; // start at index 1

//                         lt_compare_chip[3].assign(
//                             &mut region,
//                             i,
//                             Value::known(new_dedup_2[i - 1]),
//                             Value::known(new_dedup_2[i]),
//                         )?;
//                     }
//                     region.assign_advice(
//                         || "new_dedup_2",
//                         self.config.dedup_sort[1],
//                         i,
//                         || Value::known(new_dedup_2[i]),
//                     )?;
//                 }

//                 let join: Vec<Vec<F>> = cartesian_product
//                     .iter()
//                     .map(|v| {
//                         let mut new_vec = Vec::new();
//                         if v.len() >= 3 {
//                             new_vec.push(v[2]);
//                             new_vec.push(v[3]);
//                             new_vec.push(v[6]);
//                             new_vec.push(v[7]);
//                             new_vec.push(v[8]);
//                         }
//                         new_vec
//                     })
//                     .collect();

//                 // group by
//                 // l_orderkey,
//                 // o_orderdate,
//                 // o_shippriority
//                 cartesian_product.sort_by_key(|element| {
//                     element[2].clone() + element[3].clone() + element[6].clone()
//                 });

//                 let groupby: Vec<Vec<F>> = cartesian_product
//                     .iter()
//                     .map(|v| {
//                         let mut new_vec = Vec::new();
//                         if v.len() >= 3 {
//                             new_vec.push(v[2]);
//                             new_vec.push(v[3]);
//                             new_vec.push(v[6]);
//                             new_vec.push(v[7]);
//                             new_vec.push(v[8]);
//                         }
//                         new_vec
//                     })
//                     .collect();

//                 // region.constrain_equal(left, right)

//                 let mut equal_check: Vec<F> = Vec::new();
//                 if cartesian_product.len() > 0 {
//                     equal_check.push(F::from(0)); // add the the first one must be 0
//                 }

//                 for row in 1..cartesian_product.len() {
//                     self.config.q_sort[2].enable(&mut region, row)?; // q_sort[2]
//                     if cartesian_product[row][2] == cartesian_product[row - 1][2]
//                         && cartesian_product[row][3] == cartesian_product[row - 1][3]
//                         && cartesian_product[row][6] == cartesian_product[row - 1][6]
//                     {
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

//                 let n = cartesian_product.len() + 1;
//                 let mut revenue: Vec<F> = vec![F::from(0); n];
//                 for i in 1..n {
//                     revenue[i] = revenue[i - 1] * equal_check[i - 1]  // sum(l_extendedprice * (1 - l_discount)) as revenue,
//                         + cartesian_product[i - 1][7] * (F::from(1000) - cartesian_product[i - 1][8]);
//                     // cartesian_product[i - 1].push(revenue[i].clone()); // add this value to correct row of cartesian product
//                 }

//                 for i in 0..n {
//                     region.assign_advice(
//                         || "revenue",
//                         self.config.revenue,
//                         i,
//                         || Value::known(revenue[i]),
//                     )?;
//                 }

//                 for i in 0..cartesian_product.len() {
//                     self.config.q_perm[0].enable(&mut region, i)?; // q_perm[0]
//                     for (idx, &j) in [2, 3, 6, 7, 8].iter().enumerate() {
//                         region.assign_advice(
//                             || "groupby",
//                             self.config.groupby[idx],
//                             i,
//                             || Value::known(cartesian_product[i][j]),
//                         )?;
//                     }
//                     if i > 0 {
//                         compare_chip[0].assign(
//                             &mut region,
//                             i, // assign groupby compare chip
//                             &[
//                                 cartesian_product[i - 1][2],
//                                 cartesian_product[i - 1][3],
//                                 cartesian_product[i - 1][6],
//                             ],
//                             &[
//                                 cartesian_product[i][2],
//                                 cartesian_product[i][3],
//                                 cartesian_product[i][6],
//                             ],
//                         )?;
//                     }
//                 }

//                 let duration_block = start.elapsed();
//                 println!("Time elapsed for block: {:?}", duration_block);

//                 let _ = perm_chip[0].assign1(&mut region, join, groupby); // permutation between join and groupby

//                 // order by
//                 // revenue desc,
//                 // o_orderdate;
//                 let mut grouped_data: Vec<Vec<F>> = Vec::new();
//                 for row in &cartesian_product {
//                     // Check if the group (a1 value) already exists
//                     match grouped_data
//                         .iter_mut()
//                         .find(|g| g[0] == row[2] && g[1] == row[3] && g[2] == row[6])
//                     {
//                         Some(group) => {
//                             group[3] += row[7] * (F::from(1000) - row[8]); // Add to the existing sum
//                         }
//                         None => {
//                             grouped_data.push(vec![
//                                 row[2],
//                                 row[3],
//                                 row[6],
//                                 row[7] * (F::from(1000) - row[8]),
//                             ]); // Create a new group
//                         }
//                     }
//                 }
//                 // println!("cartesian {:?}", cartesian_product);
//                 // println!("grouped data {:?}", grouped_data);

//                 grouped_data.sort_by(|a, b| match b[3].cmp(&a[3]) {
//                     Ordering::Equal => a[1].cmp(&b[1]),
//                     other => other,
//                 });

//                 // println!("grouped data 1 {:?}", grouped_data.len());
//                 for i in 0..grouped_data.len() {
//                     if i > 0 {
//                         self.config.q_sort[3].enable(&mut region, i)?; // start at the index 1
//                         equal_chip[1].assign(
//                             // iszerochip assignment
//                             &mut region,
//                             i,
//                             Value::known(grouped_data[i - 1][3] - grouped_data[i][3]),
//                         )?; // revenue[i-1] = revenue [i]
//                         compare_chip[1].assign(
//                             // revenue[i-1] > revenue[i]
//                             &mut region,
//                             i, // assign groupby compare chip
//                             &[grouped_data[i][3]],
//                             &[grouped_data[i - 1][3]],
//                         )?;
//                         compare_chip[2].assign(
//                             // o_orderdate[i-1] <= o_orderdate[i]
//                             &mut region,
//                             i,
//                             &[grouped_data[i - 1][0]],
//                             &[grouped_data[i][0]],
//                         )?;
//                     }
//                     for j in 0..4 {
//                         region.assign_advice(
//                             || "orderby",
//                             self.config.orderby[j],
//                             i,
//                             || Value::known(grouped_data[i][j]),
//                         )?;
//                     }
//                 }
//                 let duration_block = start.elapsed();
//                 println!("Time elapsed for block: {:?}", duration_block);

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
//     customer: Vec<Vec<F>>,
//     orders: Vec<Vec<F>>,
//     lineitem: Vec<Vec<F>>,

//     pub condition: [F; 2],

//     _marker: PhantomData<F>,
// }

// impl<F: Copy + Default> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             customer: Vec::new(),
//             orders: Vec::new(),
//             lineitem: Vec::new(),

//             condition: [Default::default(); 2],
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
//             self.customer.clone(),
//             self.orders.clone(),
//             self.lineitem.clone(),
//             self.condition.clone(),
//         )?;

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

//         let customer_file_path = "/home/cc/halo2-TPCH/src/data/customer.tbl";
//         let orders_file_path = "/home/cc/halo2-TPCH/src/data/orders.tbl";
//         let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";

//         let mut customer: Vec<Vec<Fp>> = Vec::new();
//         let mut orders: Vec<Vec<Fp>> = Vec::new();
//         let mut lineitem: Vec<Vec<Fp>> = Vec::new();

//         if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             customer = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(string_to_u64(&record.c_mktsegment)),
//                         Fp::from(record.c_custkey),
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
//                         // Fp::from(string_to_u64(&record.o_orderdate)),
//                         Fp::from(date_to_timestamp(&record.o_orderdate)),
//                         Fp::from(record.o_shippriority),
//                         Fp::from(record.o_custkey),
//                         Fp::from(record.o_orderkey),
//                     ]
//                 })
//                 .collect();
//         }
//         if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             lineitem = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.l_orderkey),
//                         Fp::from(scale_by_1000(record.l_extendedprice)),
//                         Fp::from(scale_by_1000(record.l_discount)),
//                         // Fp::from(string_to_u64(&record.l_shipdate)),
//                         Fp::from(date_to_timestamp(&record.l_shipdate)),
//                     ]
//                 })
//                 .collect();
//         }

//         let condition = [Fp::from(3367), Fp::from(796089600)];
//         // c_mktsegment = 'HOUSEHOLD'   -> 3367
//         // o_orderdate < date '1995-03-25'and l_shipdate > date '1995-03-25'  ->796089600
//         //  BUILDING ->   2651;    1996-03-13 -> 2731

//         // let customer: Vec<Vec<Fp>> = customer.iter().take(10).cloned().collect();
//         // let orders: Vec<Vec<Fp>> = orders.iter().take(10).cloned().collect();
//         // let lineitem: Vec<Vec<Fp>> = lineitem.iter().take(10).cloned().collect();

//         // let customer: Vec<Vec<Fp>> = vec![
//         //     vec![Fp::from(1), Fp::from(2)],
//         //     vec![Fp::from(1), Fp::from(3)],
//         //     vec![Fp::from(1), Fp::from(4)],
//         // ];
//         // let orders: Vec<Vec<Fp>> = vec![
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(2)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(2), Fp::from(4)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(6), Fp::from(2)],
//         // ];
//         // let lineitem: Vec<Vec<Fp>> = vec![
//         //     vec![Fp::from(4), Fp::from(2), Fp::from(1), Fp::from(11)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(12)],
//         //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(13)],
//         // ];

//         let circuit = MyCircuit::<Fp> {
//             customer,
//             orders,
//             lineitem,
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
// // time cargo test --package halo2-experiments --lib -- sql::q3_final_v1::tests::test_1 --exact --nocapture

// // meta.shuffle("permutation check", |meta| {
// //     // Inputs
// //     let q0 = meta.query_selector(q_join[0]);
// //     let q1 = meta.query_selector(q_join[1]);
// //     let q2 = meta.query_selector(q_join[2]);
// //     let q3 = meta.query_selector(q_join[3]);
// //     let q4 = meta.query_selector(q_join[4]);
// //     let q5 = meta.query_selector(q_join[5]);
// //     let q6 = meta.query_selector(q_join[6]);
// //     let q7 = meta.query_selector(q_join[7]);

// //     let input1: Vec<_> = customer
// //         .iter()
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();

// //     let table1: Vec<_> = join_and_disjoin_customer
// //         .iter()
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();

// //     let input2: Vec<_> = orders
// //         .iter()
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();

// //     let table2: Vec<_> = join_and_disjoin_orders
// //         .iter()
// //         .take(2)
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();
// //     let input3: Vec<_> = orders
// //         .iter()
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();

// //     let table3: Vec<_> = join_and_disjoin_orders
// //         .iter()
// //         .skip(2) // Skip the first two elements
// //         .take(2)
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();
// //     let input4: Vec<_> = lineitem
// //         .iter()
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();

// //     let table4: Vec<_> = join_and_disjoin_lineitem
// //         .iter()
// //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
// //         .collect();

// //     let constraint0: Vec<_> = input1
// //         .iter()
// //         .zip(table1.iter())
// //         .map(|(input, table)| (q0 * input.clone(), q1 * table.clone()))
// //         .collect();
// //     let constraint1: Vec<_> = input2
// //         .iter()
// //         .zip(table2.iter())
// //         .map(|(input, table)| (q2 * input.clone(), q3 * table.clone()))
// //         .collect();
// //     let constraint2: Vec<_> = input3
// //         .iter()
// //         .zip(table3.iter())
// //         .map(|(input, table)| (q4 * input.clone(), q5 * table.clone()))
// //         .collect();
// //     let constraint3: Vec<_> = input4
// //         .iter()
// //         .zip(table4.iter())
// //         .map(|(input, table)| (q6 * input.clone(), q7 * table.clone()))
// //         .collect();
// //     let mut constraints = Vec::new();
// //     constraints.extend(constraint0);
// //     constraints.extend(constraint1);
// //     constraints.extend(constraint2);
// //     constraints.extend(constraint3);
// //     constraints
// // });
