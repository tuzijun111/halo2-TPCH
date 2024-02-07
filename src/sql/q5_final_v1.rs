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
//     q_enable: Selector,
//     q_first: Selector,
//     q_nonfirst: Selector,
//     q_sort: Selector,
//     q_sort_final: Selector,

//     l_orderkey: Column<Advice>,
//     l_extendedprice: Column<Advice>,
//     l_discount: Column<Advice>,
//     l_suppkey: Column<Advice>,

//     o_custkey: Column<Advice>,
//     o_orderdate: Column<Advice>,
//     o_orderkey: Column<Advice>,

//     c_custkey: Column<Advice>,
//     c_nationkey: Column<Advice>,

//     s_nationkey: Column<Advice>,
//     s_suppkey: Column<Advice>,

//     n_nationkey: Column<Advice>,
//     n_regionkey: Column<Advice>,
//     n_name: Column<Advice>,

//     r_regionkey: Column<Advice>,
//     r_name: Column<Advice>,

//     o1_condition: Column<Advice>,
//     o2_condition: Column<Advice>,
//     r_condition: Column<Advice>,

//     o1_check: Column<Advice>,
//     o2_check: Column<Advice>,
//     r_check: Column<Advice>,

//     lt_o1_condition: LtConfig<F, NUM_BYTES>,
//     lt_o2_condition: LtEqGenericConfig<F, NUM_BYTES>,
//     lt_r_condition: IsZeroConfig<F>,
//     groupby_sort: LtEqGenericConfig<F, NUM_BYTES>,
//     revenue_final: LtEqGenericConfig<F, NUM_BYTES>,

//     groupby_name: Column<Advice>,
//     groupby_extendedprice: Column<Advice>,
//     groupby_discount: Column<Advice>,
//     revenue: Column<Advice>,
//     sorted_revenue: Column<Advice>,
//     equal_check: Column<Advice>,

//     join_column: [Column<Advice>; 5],
//     disjoin_column: [Column<Advice>; 5],
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
//         let q_enable = meta.complex_selector();
//         let q_first = meta.complex_selector();
//         let q_nonfirst = meta.complex_selector();
//         let q_sort = meta.complex_selector();
//         let q_sort_final = meta.complex_selector();

//         let l_orderkey = meta.advice_column();
//         let l_extendedprice = meta.advice_column();
//         let l_discount = meta.advice_column();
//         let l_suppkey = meta.advice_column();

//         let o_custkey = meta.advice_column();
//         let o_orderdate = meta.advice_column();
//         let o_orderkey = meta.advice_column();

//         let c_custkey = meta.advice_column();
//         let c_nationkey = meta.advice_column();

//         let s_nationkey = meta.advice_column();
//         let s_suppkey = meta.advice_column();

//         let n_nationkey = meta.advice_column();
//         let n_regionkey = meta.advice_column();
//         let n_name = meta.advice_column();

//         let r_regionkey = meta.advice_column();
//         let r_name = meta.advice_column();

//         let o1_condition = meta.advice_column();
//         let o2_condition = meta.advice_column();
//         let r_condition = meta.advice_column();

//         let o1_check = meta.advice_column();
//         let o2_check = meta.advice_column();
//         let r_check = meta.advice_column();

//         let is_zero_advice_column = meta.advice_column();
//         // let constant = meta.fixed_column();
//         // let instance = meta.instance_column();
//         let groupby_name = meta.advice_column();
//         let groupby_extendedprice = meta.advice_column();
//         let groupby_discount = meta.advice_column();
//         let revenue = meta.advice_column();
//         let sorted_revenue = meta.advice_column();
//         let equal_check = meta.advice_column();

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

//         meta.enable_equality(l_orderkey);
//         meta.enable_equality(l_extendedprice);
//         meta.enable_equality(l_discount);
//         meta.enable_equality(l_suppkey);
//         meta.enable_equality(o_custkey);
//         meta.enable_equality(o_orderdate);
//         meta.enable_equality(o_orderkey);
//         meta.enable_equality(c_custkey);
//         meta.enable_equality(c_nationkey);
//         meta.enable_equality(s_nationkey);
//         meta.enable_equality(s_suppkey);
//         meta.enable_equality(n_nationkey);
//         meta.enable_equality(n_regionkey);
//         meta.enable_equality(n_name);
//         meta.enable_equality(r_regionkey);
//         meta.enable_equality(r_name);
//         meta.enable_equality(groupby_name);
//         meta.enable_equality(groupby_extendedprice);
//         meta.enable_equality(groupby_discount);
//         meta.enable_equality(revenue);
//         meta.enable_equality(sorted_revenue);
//         meta.enable_equality(equal_check);
//         for i in 0..join_column.len() {
//             meta.enable_equality(join_column[i]);
//             meta.enable_equality(disjoin_column[i]);
//         }

//         let lt_o1_condition = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(o_orderdate, Rotation::cur()),
//             |meta| meta.query_advice(o1_condition, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );
//         // larger or equal to
//         let lt_o2_condition = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| vec![meta.query_advice(o2_condition, Rotation::cur())],
//             |meta| vec![meta.query_advice(o_orderdate, Rotation::cur())],
//         );

//         let lt_r_condition = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable), // this is the q_enable
//             |meta| {
//                 meta.query_advice(r_name, Rotation::cur())
//                     - meta.query_advice(r_condition, Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column,                // this is the advice column that stores value_inv
//         );

//         // gate for o_orderdate < date ':2' + interval '1' year
//         meta.create_gate(
//             "verifies o_orderdate < date ':2' + interval '1' year", // just use less_than for testing here
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);
//                 let check = meta.query_advice(o1_check, Rotation::cur());
//                 vec![q_enable * (lt_o1_condition.is_lt(meta, None) - check)]
//             },
//         );

//         // gate for o_orderdate >= date ':2'
//         meta.create_gate("verifies o_orderdate < date ':2'", |meta| {
//             let q_enable = meta.query_selector(q_enable);
//             let check = meta.query_advice(o2_check, Rotation::cur());
//             vec![q_enable * (lt_o2_condition.is_lt(meta, None) - check)]
//         });

//         // gate for r_name = ':1'
//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(q_enable);
//             let output = meta.query_advice(r_check, Rotation::cur());
//             vec![
//                 s.clone()
//                     * (lt_r_condition.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 s * (Expression::Constant(F::ONE) - lt_r_condition.expr()) * (output), // in this case output == 0
//             ]
//         });

//         // groupby
//         let groupby_sort = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| vec![meta.query_advice(groupby_name, Rotation::prev())],
//             |meta| vec![meta.query_advice(groupby_name, Rotation::cur())],
//         );

//         let revenue_final = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| vec![meta.query_advice(sorted_revenue, Rotation::prev())],
//             |meta| vec![meta.query_advice(sorted_revenue, Rotation::cur())],
//         );

//         TestCircuitConfig {
//             q_enable,
//             q_first,
//             q_nonfirst,
//             q_sort,
//             q_sort_final,

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
//             o1_check,
//             o2_check,
//             r_check,

//             lt_o1_condition,
//             lt_o2_condition,
//             lt_r_condition,
//             groupby_sort,
//             revenue_final,

//             groupby_name,
//             groupby_extendedprice,
//             groupby_discount,
//             revenue,
//             sorted_revenue,
//             equal_check,

//             join_column,
//             disjoin_column,
//         }
//     }

//     pub fn assign(
//         &self,
//         // layouter: &mut impl Layouter<F>,
//         layouter: &mut impl Layouter<F>,
//         l_orderkey: [F; N],
//         l_extendedprice: [F; N],
//         l_discount: [F; N],
//         l_suppkey: [F; N],
//         o_custkey: [F; N],
//         o_orderdate: [F; N],
//         o_orderkey: [F; N],
//         c_custkey: [F; N],
//         c_nationkey: [F; N],
//         s_nationkey: [F; N],
//         s_suppkey: [F; N],
//         n_nationkey: [F; N],
//         n_regionkey: [F; N],
//         n_name: [F; N],
//         r_regionkey: [F; N],
//         r_name: [F; N],

//         o1_condition: F,
//         o2_condition: F,
//         r_condition: F,
//         // l_orderkey: [u64; N],
//         // l_extendedprice: [u64; N],
//         // l_discount: [u64; N],
//         // l_suppkey: [u64; N],
//         // o_orderdate: [u64; N],
//         // o_orderkey: [u64; N],
//         // c_nationkey: [u64; N],
//         // s_nationkey: [u64; N],
//         // s_suppkey: [u64; N],
//         // n_nationkey: [u64; N],
//         // n_regionkey: [u64; N],
//         // n_name: [u64; N],
//         // r_regionkey: [u64; N],
//         // r_name: [u64; N],

//         // o1_condition: u64,
//         // o2_condition: u64,
//         // r_condition: u64,
//     ) -> Result<(), Error> {
//         // Result<AssignedCell<F, F>, Error> {
//         // load the chips for the filtering conditions of the three tables
//         let o1_cond_chip = LtChip::construct(self.config.lt_o1_condition);
//         let o2_cond_chip = LtEqGenericChip::construct(self.config.lt_o2_condition.clone());
//         let r_cond_chip = IsZeroChip::construct(self.config.lt_r_condition.clone());
//         let groupby_sort_chip = LtEqGenericChip::construct(self.config.groupby_sort.clone());
//         let lt_revenue_final_chip = LtEqGenericChip::construct(self.config.revenue_final.clone());

//         o1_cond_chip.load(layouter)?;
//         o2_cond_chip.load(layouter)?;

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 // locally compute the values for conditional check
//                 let mut o1_check: [bool; N] = [false; N];
//                 let mut o2_check: [bool; N] = [false; N];
//                 let mut r_check: [F; N] = [F::from(0); N];
//                 for i in 0..N {
//                     if o_orderdate[i] < o1_condition {
//                         o1_check[i] = true;
//                         // self.config.q_cond_l.enable(&mut region, i)?;
//                     }
//                     if o_orderdate[i] >= o2_condition {
//                         o2_check[i] = true;
//                         // self.config.q_cond_o.enable(&mut region, i)?;
//                     }
//                     if r_name[i] == r_condition {
//                         r_check[i] = F::from(1);
//                         // self.config.q_cond_c.enable(&mut region, i)?;
//                     }
//                 }

//                 //assign input values
//                 for i in 0..N {
//                     // enable selectors for q_enable
//                     self.config.q_enable.enable(&mut region, i)?;

//                     // assign the input values with the below codes
//                     region.assign_advice(
//                         || "l_orderkey value",
//                         self.config.l_orderkey,
//                         i,
//                         || Value::known(l_orderkey[i]),
//                     )?;
//                     region.assign_advice(
//                         || "l_extendedprice value",
//                         self.config.l_extendedprice,
//                         i,
//                         || Value::known(l_extendedprice[i]),
//                     )?;
//                     region.assign_advice(
//                         || "l_discount value",
//                         self.config.l_discount,
//                         i,
//                         || Value::known(l_discount[i]),
//                     )?;

//                     region.assign_advice(
//                         || "l_suppkey",
//                         self.config.l_suppkey,
//                         i,
//                         || Value::known(l_suppkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "o_custkey value",
//                         self.config.o_custkey,
//                         i,
//                         || Value::known(o_custkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "o_orderdate value",
//                         self.config.o_orderdate,
//                         i,
//                         || Value::known(o_orderdate[i]),
//                     )?;

//                     region.assign_advice(
//                         || "o_orderkey value",
//                         self.config.o_orderkey,
//                         i,
//                         || Value::known(o_orderkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "c_custkey value",
//                         self.config.c_custkey,
//                         i,
//                         || Value::known(c_custkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "c_nationkey value",
//                         self.config.c_nationkey,
//                         i,
//                         || Value::known(c_nationkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "s_nationkey value",
//                         self.config.s_nationkey,
//                         i,
//                         || Value::known(s_nationkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "s_suppkey value",
//                         self.config.s_suppkey,
//                         i,
//                         || Value::known(s_suppkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "n_nationkey value",
//                         self.config.n_nationkey,
//                         i,
//                         || Value::known(n_nationkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "n_regionkey value",
//                         self.config.n_regionkey,
//                         i,
//                         || Value::known(n_regionkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "n_name value",
//                         self.config.n_name,
//                         i,
//                         || Value::known(n_name[i]),
//                     )?;

//                     region.assign_advice(
//                         || "r_regionkey value",
//                         self.config.r_regionkey,
//                         i,
//                         || Value::known(r_regionkey[i]),
//                     )?;

//                     region.assign_advice(
//                         || "r_name value",
//                         self.config.r_name,
//                         i,
//                         || Value::known(r_name[i]),
//                     )?;

//                     // assign conditions for l,o,c
//                     region.assign_advice(
//                         || "o1_condition",
//                         self.config.o1_condition,
//                         i,
//                         || Value::known(o1_condition),
//                     )?;

//                     region.assign_advice(
//                         || "o2_condition",
//                         self.config.o2_condition,
//                         i,
//                         || Value::known(o2_condition),
//                     )?;

//                     region.assign_advice(
//                         || "r_condition",
//                         self.config.r_condition,
//                         i,
//                         || Value::known(r_condition),
//                     )?;

//                     region.assign_advice(
//                         || "o1_check",
//                         self.config.o1_check,
//                         i,
//                         || Value::known(F::from(o1_check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "o2_check",
//                         self.config.o2_check,
//                         i,
//                         || Value::known(F::from(o2_check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "r_check",
//                         self.config.r_check,
//                         i,
//                         || Value::known(r_check[i]),
//                     )?;

//                     // assign values for loaded chips
//                     o1_cond_chip.assign(
//                         &mut region,
//                         i,
//                         Value::known(o_orderdate[i]),
//                         Value::known(o1_condition),
//                     )?;

//                     o2_cond_chip.assign(&mut region, i, &[o2_condition], &[o_orderdate[i]])?;

//                     r_cond_chip.assign(&mut region, i, Value::known(r_name[i] - r_condition))?;
//                 }

//                 // compute values related to the join operation locally
//                 // store the attribtues of the tables that will be used in the SQL query in tuples
//                 let l_combined: Vec<_> = l_orderkey
//                     .iter()
//                     .zip(l_extendedprice.iter())
//                     .zip(l_discount.iter())
//                     .zip(l_suppkey.iter())
//                     .map(|(((&val1, &val2), &val3), &val4)| (val1, val2, val3, val4))
//                     .collect();
//                 let o_combined: Vec<_> = o_custkey
//                     .iter()
//                     .zip(o_orderdate.iter())
//                     .zip(o_orderkey.iter())
//                     .zip(o1_check.iter())
//                     .zip(o2_check.iter())
//                     .map(|((((&val1, &val2), &val3), &val4), &val5)| (val1, val2, val3, val4, val5))
//                     .collect();
//                 let c_combined: Vec<_> = c_custkey
//                     .iter()
//                     .zip(c_nationkey.iter())
//                     .map(|(&val1, &val2)| (val1, val2))
//                     .collect();
//                 let s_combined: Vec<_> = s_nationkey
//                     .iter()
//                     .zip(s_suppkey.iter())
//                     .map(|(&val1, &val2)| (val1, val2))
//                     .collect();

//                 let n_combined: Vec<_> = n_nationkey
//                     .iter()
//                     .zip(n_regionkey.iter())
//                     .zip(n_name.iter())
//                     .map(|((&val1, &val2), &val3)| (val1, val2, val3))
//                     .collect();

//                 let r_combined: Vec<_> = r_regionkey
//                     .iter()
//                     .zip(r_name.iter())
//                     .zip(r_check.iter())
//                     .map(|((&val1, &val2), &val3)| (val1, val2, val3))
//                     .collect();

//                 //create the values for join and disjoin

//                 let mut join_c_custkey = Vec::new();
//                 let mut join_o_custkey = Vec::new();
//                 let mut disjoin_c_custkey = Vec::new();
//                 let mut disjoin_o_custkey = Vec::new();

//                 let mut join_l_orderkey = Vec::new();
//                 let mut join_o_orderkey = Vec::new();
//                 let mut disjoin_l_orderkey = Vec::new();
//                 let mut disjoin_o_orderkey = Vec::new();

//                 let mut join_l_suppkey = Vec::new();
//                 let mut join_s_suppkey = Vec::new();
//                 let mut disjoin_l_suppkey = Vec::new();
//                 let mut disjoin_s_suppkey = Vec::new();

//                 let mut join_c_nationkey = Vec::new();
//                 let mut join_s_nationkey = Vec::new();
//                 let mut disjoin_c_nationkey = Vec::new();
//                 let mut disjoin_s_nationkey = Vec::new();

//                 let mut join_s1_nationkey = Vec::new();
//                 let mut join_n_nationkey = Vec::new();
//                 let mut disjoin_s1_nationkey = Vec::new();
//                 let mut disjoin_n_nationkey = Vec::new();

//                 let mut join_n_regionkey = Vec::new();
//                 let mut join_r_regionkey = Vec::new();
//                 let mut disjoin_n_regionkey = Vec::new();
//                 let mut disjoin_r_regionkey = Vec::new();

//                 // c_custkey = o_custkey
//                 for &(val1, _) in &c_combined {
//                     if o_combined
//                         .iter()
//                         .any(|(v0, _, _, v3, v4)| v0 == &val1 && v3 == &true && v4 == &true)
//                     {
//                         join_c_custkey.push(val1);
//                     } else {
//                         disjoin_c_custkey.push(val1);
//                     }
//                 }

//                 for &(val1, _, _, _, _) in &o_combined {
//                     if o_combined
//                         .iter()
//                         .any(|(_, _, _, v3, v4)| v3 == &true && v4 == &true)
//                         && c_combined.iter().any(|(v0, _)| v0 == &val1)
//                     {
//                         join_o_custkey.push(val1);
//                     } else {
//                         disjoin_o_custkey.push(val1);
//                     }
//                 }

//                 // l_orderkey = o_orderkey
//                 for &(val1, _, _, _) in &l_combined {
//                     if o_combined
//                         .iter()
//                         .any(|(_, v1, _, v3, v4)| v1 == &val1 && v3 == &true && v4 == &true)
//                     {
//                         join_l_orderkey.push(val1);
//                     } else {
//                         disjoin_l_orderkey.push(val1);
//                     }
//                 }

//                 for &(_, val1, _, _, _) in &o_combined {
//                     if o_combined
//                         .iter()
//                         .any(|(_, _, _, v3, v4)| v3 == &true && v4 == &true)
//                         && l_combined.iter().any(|(v0, _, _, _)| v0 == &val1)
//                     {
//                         join_o_orderkey.push(val1);
//                     } else {
//                         disjoin_o_orderkey.push(val1);
//                     }
//                 }

//                 // l_suppkey = s_suppkey
//                 for &(_, _, v, _) in &l_combined {
//                     if s_combined.iter().any(|(_, v1)| v1 == &v) {
//                         join_l_suppkey.push(v);
//                     } else {
//                         disjoin_l_suppkey.push(v);
//                     }
//                 }

//                 for &(_, v) in &s_combined {
//                     if l_combined.iter().any(|(_, _, _, v3)| v3 == &v) {
//                         join_s_suppkey.push(v);
//                     } else {
//                         disjoin_s_suppkey.push(v);
//                     }
//                 }

//                 // c_nationkey = s_nationkey
//                 for &(_, v) in &c_combined {
//                     if s_combined.iter().any(|(v0, _)| v0 == &v) {
//                         join_c_nationkey.push(v);
//                     } else {
//                         disjoin_c_nationkey.push(v);
//                     }
//                 }

//                 for &(v, _) in &s_combined {
//                     if c_combined.iter().any(|(_, v1)| v1 == &v) {
//                         join_s_nationkey.push(v);
//                     } else {
//                         disjoin_s_nationkey.push(v);
//                     }
//                 }

//                 // s_nationkey = n_nationkey
//                 for &(v, _) in &s_combined {
//                     if n_combined.iter().any(|(v0, _, _)| v0 == &v) {
//                         join_s1_nationkey.push(v);
//                     } else {
//                         disjoin_s1_nationkey.push(v);
//                     }
//                 }

//                 for &(v, _, _) in &n_combined {
//                     if s_combined.iter().any(|(v0, _)| v0 == &v) {
//                         join_n_nationkey.push(v);
//                     } else {
//                         disjoin_n_nationkey.push(v);
//                     }
//                 }

//                 // n_regionkey = r_regionkey
//                 for &(_, v, _) in &n_combined {
//                     if r_combined
//                         .iter()
//                         .any(|(v0, _, v2)| v0 == &v && v2 == &F::from(1))
//                     {
//                         join_n_regionkey.push(v);
//                     } else {
//                         disjoin_n_regionkey.push(v);
//                     }
//                 }

//                 for &(v, _, _) in &r_combined {
//                     if r_combined.iter().any(|(_, _, v2)| v2 == &F::from(1))
//                         && n_combined.iter().any(|(_, v1, _)| v1 == &v)
//                     {
//                         join_r_regionkey.push(v);
//                     } else {
//                         disjoin_r_regionkey.push(v);
//                     }
//                 }

//                 // assign join and disjoin values
//                 for i in 0..join_c_custkey.len() {
//                     region.assign_advice(
//                         || " ",
//                         self.config.join_column[0],
//                         i,
//                         || Value::known(join_c_custkey[i]),
//                     )?;
//                 }

//                 for i in 0..disjoin_c_custkey.len() {
//                     region.assign_advice(
//                         || " ",
//                         self.config.join_column[1],
//                         i,
//                         || Value::known(disjoin_c_custkey[i]),
//                     )?;
//                 }

//                 // ...

//                 //assign values for the result of join i.e. l_orderkey = o_orderkey
//                 let mut cartesian_product1 = Vec::new();
//                 for val1 in &l_combined {
//                     for val2 in &o_combined {
//                         if val1.0 == val2.1 && val2.3 == true && val2.4 == true {
//                             cartesian_product1
//                                 .push((val1.0, val1.1, val1.2, val1.3, val2.0, val2.1));
//                             // cartesian_product1.4 = o_custkey
//                         }
//                     }
//                 }

//                 //assign values for the result of join i.e. l_suppkey = s_suppkey
//                 let mut cartesian_product2 = Vec::new();
//                 for val1 in &cartesian_product1 {
//                     for val2 in &s_combined {
//                         if val1.3 == val2.1 {
//                             cartesian_product2
//                                 .push((val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val2.0));
//                             // nationkey = cartesian_product2.5
//                         }
//                     }
//                 }
//                 //assign values for the result of join i.e. c_nationkey = s_nationkey and c_custkey = o_custkey
//                 let mut cartesian_product3 = Vec::new();
//                 for val1 in &cartesian_product2 {
//                     for val2 in &c_combined {
//                         if val1.5 == val2.1 && val1.4 == val2.0 {
//                             cartesian_product3.push((
//                                 val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val1.6, val2.0,
//                                 val2.1,
//                             ));
//                             // cartesian_product3.5 = c_nationkey = s_nationkey
//                         }
//                     }
//                 }
//                 //assign values for the result of join i.e. s_nationkey = n_nationkey
//                 let mut cartesian_product4 = Vec::new();
//                 for val1 in &cartesian_product3 {
//                     for val2 in &n_combined {
//                         if val1.5 == val2.0 {
//                             cartesian_product4.push((
//                                 val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val1.6, val1.7,
//                                 val1.8, val2.1, val2.2,
//                             ));
//                             // cartesian_product4.7 = n_regionkey
//                         }
//                     }
//                 }
//                 //assign values for the result of join i.e. n_regionkey = r_regionkey
//                 let mut cartesian_product5 = Vec::new();
//                 for val1 in &cartesian_product4 {
//                     for val2 in &r_combined {
//                         if val1.7 == val2.0 && val2.2 == F::from(1) {
//                             cartesian_product5.push((
//                                 val1.0, val1.1, val1.2, val1.3, val1.4, val1.5, val1.6, val1.7,
//                                 val1.8, val1.9, val1.10, val2.1,
//                             ));
//                             // cartesian_product5.8 = r_name
//                         }
//                     }
//                 }

//                 // the order of attributes in cartesian_product: l_orderkey/o_orderkey, c_custkey/o_custkey, l_extendedprice, l_discount, ...
//                 //sort by l_orderkey, o_orderdate, o_shippriority
//                 cartesian_product5.sort_by_key(|element| element.10);

//                 let mut revenue = Vec::new();

//                 for i in 0..cartesian_product5.len() {
//                     if i == 0 {
//                         self.config.q_first.enable(&mut region, i)?;
//                         revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
//                     // l_extendedprice * (1 - l_discount)
//                     } else {
//                         self.config.q_sort.enable(&mut region, i)?;
//                         groupby_sort_chip.assign(
//                             &mut region,
//                             i,
//                             &[cartesian_product5[i - 1].10],
//                             &[cartesian_product5[i].10],
//                         )?;

//                         // check if it is the first value
//                         if cartesian_product5[i - 1].10 == cartesian_product5[i].10 {
//                             self.config.q_first.enable(&mut region, i)?;
//                             revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
//                         } else {
//                             self.config.q_nonfirst.enable(&mut region, i)?;
//                             revenue.push(
//                                 revenue[i - 1] + cartesian_product5[i].1 * cartesian_product5[i].2,
//                             );
//                         }
//                     }
//                     // assign revenue column
//                     region.assign_advice(
//                         || "revenue",
//                         self.config.revenue,
//                         i,
//                         || Value::known(revenue[i]),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_name",
//                         self.config.groupby_name,
//                         i,
//                         || Value::known(cartesian_product5[i].10),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_l_extendedprice",
//                         self.config.groupby_extendedprice,
//                         i,
//                         || Value::known(cartesian_product5[i].1),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_l_discount",
//                         self.config.groupby_discount,
//                         i,
//                         || Value::known(cartesian_product5[i].2),
//                     )?;
//                 }

//                 // generate revenue_final
//                 // println!("product: {:?}", cartesian_product5);
//                 let mut revenue_final = Vec::new(); // by removing intermediate revenue values, i.e. only keep the final revenue of each group
//                 if revenue.len() > 0 {
//                     for i in 0..revenue.len() - 1 {
//                         if cartesian_product5[i].10 != cartesian_product5[i + 1].10 {
//                             revenue_final.push((cartesian_product5[i].10, revenue[i]))
//                         }
//                     }

//                     revenue_final.push((
//                         cartesian_product5[revenue.len() - 1].10,
//                         revenue[revenue.len() - 1],
//                     ));
//                 }

//                 // order by revenue desc

//                 revenue_final.sort_by_key(|&(value, _)| Reverse(value));

//                 // assign values of equal check for verifying if revenue_final is sorted
//                 let mut equal_check: Vec<F> = Vec::new();

//                 if revenue_final.len() == 1 {
//                     equal_check.push(F::from(0)); // 0 assigned to the first value in equal_check
//                 } else {
//                     equal_check.push(F::from(0));
//                     for i in 1..revenue_final.len() {
//                         if revenue_final[i] == revenue_final[i - 1] {
//                             equal_check.push(F::from(1));
//                         } else {
//                             equal_check.push(F::from(0))
//                         }
//                     }
//                 }
//                 // println!("revenue: {:?}", revenue_final);
//                 // println!("equal check: {:?}", equal_check);

//                 // assign sorted revenue and orderdate
//                 for i in 0..revenue_final.len() {
//                     region.assign_advice(
//                         || "sorted_revenue",
//                         self.config.sorted_revenue,
//                         i,
//                         || Value::known(revenue_final[i].1),
//                     )?;

//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.equal_check,
//                         i,
//                         || Value::known(equal_check[i]),
//                     )?;

//                     if i != 0 {
//                         self.config.q_sort_final.enable(&mut region, i)?;
//                         lt_revenue_final_chip.assign(
//                             &mut region,
//                             i,
//                             &[revenue_final[i].1],
//                             &[revenue_final[i - 1].1],
//                         )?;
//                     }
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
