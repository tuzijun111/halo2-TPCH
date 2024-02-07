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
// use std::collections::HashSet;

// use std::mem;

// const N: usize = 10;

// // #[derive(Default)]
// // We should use the selector to skip the row which does not satisfy shipdate values

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: Selector,
//     q_cond_l: Selector, // permutation check for l table
//     q_cond_o: Selector, // permutation check for o table
//     q_cond_c: Selector, // permutation check for c table
//     q_sort: Selector,
//     q_sort_final: Selector,
//     q_first: Selector,
//     q_nonfirst: Selector,
//     q_sort_l_o_join: Selector,
//     q_sort_o_c_join: Selector,

//     l_orderkey: Column<Advice>,
//     l_extendedprice: Column<Advice>,
//     l_discount: Column<Advice>,
//     l_shipdate: Column<Advice>,

//     o_orderdate: Column<Advice>,
//     o_shippriority: Column<Advice>,
//     o_custkey: Column<Advice>,
//     o_orderkey: Column<Advice>,

//     c_mktsegment: Column<Advice>,
//     c_custkey: Column<Advice>,

//     perm_l_orderkey: Column<Advice>,
//     perm_l_extendedprice: Column<Advice>,
//     perm_l_discount: Column<Advice>,

//     perm_o_orderdate: Column<Advice>,
//     perm_o_shippriority: Column<Advice>,
//     perm_o_custkey: Column<Advice>,
//     perm_o_orderkey: Column<Advice>,

//     perm_c_custkey: Column<Advice>,

//     deduplicated_a2_vec: Column<Advice>, // deduplicate disjoint values of l_orderkey
//     deduplicated_b2_vec: Column<Advice>, // o_orderkey
//     deduplicated_c2_vec: Column<Advice>, // o_custkey
//     deduplicated_d2_vec: Column<Advice>, // c_custkey
//     new_dedup_1: Column<Advice>,
//     new_dedup_2: Column<Advice>,

//     sorted_l_o_join: Column<Advice>,
//     sorted_o_c_join: Column<Advice>,

//     revenue: Column<Advice>,

//     l_condition: Column<Advice>, // conditional value for lineitem table
//     o_condition: Column<Advice>, // conditional value for orders table
//     c_condition: Column<Advice>, // conditional value for customer table

//     l_check: Column<Advice>,     // conditional check for lineitem table
//     o_check: Column<Advice>,     // conditional check for orders table
//     c_check: Column<Advice>,     // conditional check for customer table
//     equal_check: Column<Advice>, // check if sorted_revenue[i-1] = sorted_revenue[i]

//     lt_l_condition: LtConfig<F, 3>,
//     lt_o_condition: LtConfig<F, 3>,
//     lt_c_condition: IsZeroConfig<F>,
//     lt_l_orderkey_o_orderdate: LtEqGenericConfig<F, 3>,
//     lt_revenue_final: LtEqConfig<F, 3>,
//     lt_orderdate_final: LtEqConfig<F, 3>,
//     lt_sorted_l_o_join: LtEqGenericConfig<F, 3>,
//     lt_sorted_o_c_join: LtEqGenericConfig<F, 3>,
//     // pub instance: Column<Instance>,
//     groupby_l_orderkey: Column<Advice>,
//     groupby_o_custkey: Column<Advice>,
//     groupby_l_extendedprice: Column<Advice>,
//     groupby_l_discount: Column<Advice>,
//     groupby_o_orderdate: Column<Advice>,
//     groupby_o_shippriority: Column<Advice>,

//     // columns for the tuples that contribute to join predicate
//     join_l_orderkey: Column<Advice>,
//     join_l_extendedprice: Column<Advice>,
//     join_l_discount: Column<Advice>,
//     // join_l_check: Column<Advice>,
//     join_o_orderkey: Column<Advice>,
//     join_o_custkey: Column<Advice>,
//     join_o_orderdate: Column<Advice>,
//     join_o_shippriority: Column<Advice>,
//     // join_o_check: Column<Advice>,
//     join_c_custkey: Column<Advice>,
//     // join_c_check: Column<Advice>,
//     disjoin_l_orderkey: Column<Advice>,
//     disjoin_l_extendedprice: Column<Advice>,
//     disjoin_l_discount: Column<Advice>,
//     disjoin_o_orderkey: Column<Advice>,
//     disjoin_o_custkey: Column<Advice>,
//     disjoin_o_orderdate: Column<Advice>,
//     disjoin_o_shippriority: Column<Advice>,
//     disjoin_c_custkey: Column<Advice>,

//     // sorted revenue and orderdate
//     sorted_revenue: Column<Advice>,
//     sorted_orderdate: Column<Advice>,
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
//         let q_cond_l = meta.complex_selector();
//         let q_cond_o = meta.complex_selector();
//         let q_cond_c = meta.complex_selector();
//         let q_sort = meta.complex_selector();
//         let q_sort_final = meta.complex_selector();
//         let q_first = meta.complex_selector();
//         let q_nonfirst = meta.complex_selector();
//         let q_sort_l_o_join = meta.complex_selector();
//         let q_sort_o_c_join = meta.complex_selector();

//         let l_orderkey = meta.advice_column();
//         let l_extendedprice = meta.advice_column();
//         let l_discount = meta.advice_column();
//         let l_shipdate = meta.advice_column();

//         let o_orderkey = meta.advice_column();
//         let o_custkey = meta.advice_column();
//         let o_orderdate = meta.advice_column();
//         let o_shippriority = meta.advice_column();

//         let c_mktsegment = meta.advice_column();
//         let c_custkey = meta.advice_column();

//         let perm_l_orderkey = meta.advice_column();
//         let perm_l_extendedprice = meta.advice_column();
//         let perm_l_discount = meta.advice_column();

//         let perm_o_orderkey = meta.advice_column();
//         let perm_o_custkey = meta.advice_column();
//         let perm_o_orderdate = meta.advice_column();
//         let perm_o_shippriority = meta.advice_column();

//         let perm_c_custkey = meta.advice_column();

//         let deduplicated_a2_vec = meta.advice_column();
//         let deduplicated_b2_vec = meta.advice_column();
//         let deduplicated_c2_vec = meta.advice_column();
//         let deduplicated_d2_vec = meta.advice_column();
//         let new_dedup_1 = meta.advice_column();
//         let new_dedup_2 = meta.advice_column();

//         let sorted_l_o_join = meta.advice_column();
//         let sorted_o_c_join = meta.advice_column();

//         let revenue = meta.advice_column();

//         let l_condition = meta.advice_column();
//         let o_condition = meta.advice_column();
//         let c_condition = meta.advice_column();

//         let l_check = meta.advice_column();
//         let o_check = meta.advice_column();
//         let c_check = meta.advice_column();
//         let equal_check = meta.advice_column();

//         let is_zero_advice_column = meta.advice_column();

//         let groupby_l_orderkey = meta.advice_column();
//         let groupby_o_custkey = meta.advice_column();
//         let groupby_l_extendedprice = meta.advice_column();
//         let groupby_l_discount = meta.advice_column();
//         let groupby_o_orderdate = meta.advice_column();
//         let groupby_o_shippriority = meta.advice_column();

//         let join_l_orderkey = meta.advice_column();
//         let join_l_extendedprice = meta.advice_column();
//         let join_l_discount = meta.advice_column();
//         let join_o_orderkey = meta.advice_column();
//         let join_o_custkey = meta.advice_column();
//         let join_o_orderdate = meta.advice_column();
//         let join_o_shippriority = meta.advice_column();
//         let join_c_custkey = meta.advice_column();

//         let disjoin_l_orderkey = meta.advice_column();
//         let disjoin_l_extendedprice = meta.advice_column();
//         let disjoin_l_discount = meta.advice_column();
//         let disjoin_o_orderkey = meta.advice_column();
//         let disjoin_o_custkey = meta.advice_column();
//         let disjoin_o_orderdate = meta.advice_column();
//         let disjoin_o_shippriority = meta.advice_column();
//         let disjoin_c_custkey = meta.advice_column();

//         let sorted_revenue = meta.advice_column();
//         let sorted_orderdate = meta.advice_column();

//         // let constant = meta.fixed_column();
//         // let instance = meta.instance_column();

//         meta.enable_equality(l_orderkey);
//         meta.enable_equality(l_extendedprice);
//         meta.enable_equality(l_discount);

//         meta.enable_equality(o_orderdate);
//         meta.enable_equality(o_shippriority);
//         meta.enable_equality(o_custkey);
//         meta.enable_equality(o_orderkey);

//         meta.enable_equality(c_mktsegment);
//         meta.enable_equality(c_custkey);

//         meta.enable_equality(perm_l_orderkey);
//         meta.enable_equality(perm_l_extendedprice);
//         meta.enable_equality(perm_l_discount);

//         meta.enable_equality(perm_o_orderdate);
//         meta.enable_equality(perm_o_shippriority);
//         meta.enable_equality(perm_o_custkey);
//         meta.enable_equality(perm_o_orderkey);

//         meta.enable_equality(perm_c_custkey);

//         meta.enable_equality(revenue);

//         meta.enable_equality(l_condition);
//         meta.enable_equality(o_condition);
//         meta.enable_equality(c_condition);

//         meta.enable_equality(l_check);
//         meta.enable_equality(o_check);
//         meta.enable_equality(c_check);
//         meta.enable_equality(equal_check);

//         meta.enable_equality(groupby_l_orderkey);
//         meta.enable_equality(groupby_o_orderdate);
//         meta.enable_equality(groupby_o_shippriority);

//         meta.enable_equality(join_l_orderkey);
//         meta.enable_equality(join_l_extendedprice);
//         meta.enable_equality(join_l_discount);
//         meta.enable_equality(join_o_orderkey);
//         meta.enable_equality(join_o_custkey);
//         meta.enable_equality(join_o_orderdate);
//         meta.enable_equality(join_o_shippriority);
//         meta.enable_equality(join_c_custkey);
//         meta.enable_equality(disjoin_l_orderkey);
//         meta.enable_equality(disjoin_l_extendedprice);
//         meta.enable_equality(disjoin_l_discount);
//         meta.enable_equality(disjoin_o_orderkey);
//         meta.enable_equality(disjoin_o_custkey);
//         meta.enable_equality(disjoin_o_orderdate);
//         meta.enable_equality(disjoin_o_shippriority);
//         meta.enable_equality(disjoin_c_custkey);

//         meta.enable_equality(sorted_revenue);
//         meta.enable_equality(sorted_orderdate);

//         let lt_l_condition = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(l_shipdate, Rotation::cur()),
//             |meta| meta.query_advice(l_condition, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         let lt_o_condition = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(o_orderdate, Rotation::cur()),
//             |meta| meta.query_advice(o_condition, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         let lt_c_condition = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable), // this is the q_enable
//             |meta| {
//                 meta.query_advice(c_mktsegment, Rotation::cur())
//                     - meta.query_advice(c_condition, Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column,                // this is the advice column that stores value_inv
//         );

//         // gate for l_shipdate > date ':2'
//         meta.create_gate(
//             "verifies l_shipdate > date ':2'", // just use less_than for testing here
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);
//                 let check = meta.query_advice(l_check, Rotation::cur());
//                 vec![q_enable * (lt_l_condition.is_lt(meta, None) - check)]
//             },
//         );

//         // gate for o_orderdate < date ':2'
//         meta.create_gate("verifies o_orderdate < date ':2'", |meta| {
//             let q_enable = meta.query_selector(q_enable);
//             let check = meta.query_advice(o_check, Rotation::cur());
//             vec![q_enable * (lt_o_condition.is_lt(meta, None) - check)]
//         });

//         // gate for c_mktsegment = ':1'
//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(q_enable);
//             let output = meta.query_advice(c_check, Rotation::cur());
//             vec![
//                 s.clone()
//                     * (lt_c_condition.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
//                 s * (Expression::Constant(F::ONE) - lt_c_condition.expr()) * (output), // in this case output == 0
//             ]
//         });

//         // group by l_orderkey, o_orderdate, o_shippriority for sorting check
//         let lt_l_orderkey_o_orderdate = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| {
//                 vec![
//                     meta.query_advice(groupby_l_orderkey, Rotation::prev()),
//                     meta.query_advice(groupby_o_orderdate, Rotation::prev()),
//                     meta.query_advice(groupby_o_shippriority, Rotation::prev()),
//                 ]
//             },
//             |meta|
//                 // RHS vector
//                 vec![
//                     meta.query_advice(groupby_l_orderkey, Rotation::cur()),
//                     meta.query_advice(groupby_o_orderdate, Rotation::cur()),
//                     meta.query_advice(groupby_o_shippriority, Rotation::cur()),
//                 ],
//         );

//         // check the values in the two columns are sorted (i.e. tuple sorted)
//         meta.create_gate("verifies that t[i-1] <= t[i]", |meta| {
//             let q_sort = meta.query_selector(q_sort);
//             vec![
//                 q_sort
//                     * (lt_l_orderkey_o_orderdate.is_lt(meta, None)
//                         - Expression::Constant(F::from(1))),
//             ]
//         });

//         // larger than
//         let lt_revenue_final = LtEqChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort_final),
//             |meta| meta.query_advice(sorted_revenue, Rotation::cur()),
//             |meta| meta.query_advice(sorted_revenue, Rotation::prev()),
//         );
//         // less than
//         let lt_orderdate_final = LtEqChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort_final),
//             |meta| meta.query_advice(sorted_orderdate, Rotation::prev()),
//             |meta| meta.query_advice(sorted_orderdate, Rotation::cur()),
//         );
//         // Is zero for checking if sorted_revenue[i-1] = sorted_revenue[i]
//         let equal_condition = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort_final), // this is the q_enable
//             |meta| {
//                 meta.query_advice(sorted_revenue, Rotation::prev())
//                     - meta.query_advice(sorted_revenue, Rotation::cur())
//             }, // this is the value
//             is_zero_advice_column, // this is the advice column that stores value_inv
//         );

//         // check the values in the two columns sorted_revenue sorted_orderdate and are sorted (i.e. tuple sorted)
//         meta.create_gate("verifies that t[i-1] <= t[i]", |meta| {
//             let q_sort_final = meta.query_selector(q_sort_final);
//             let output = meta.query_advice(equal_check, Rotation::cur()); // for sorted_revenue
//             vec![
//                 q_sort_final.clone()
//                     * ((lt_revenue_final.is_lt(meta, None) - Expression::Constant(F::from(1)))
//                         * (equal_condition.expr()
//                             * (output.clone() - Expression::Constant(F::ONE)))
//                         * (lt_revenue_final.is_lt(meta, None) - Expression::Constant(F::from(1)))),
//                 q_sort_final * (Expression::Constant(F::ONE) - equal_condition.expr()) * (output),
//             ]
//         });

//         // permutation check for l table
//         meta.shuffle("l permutation check", |meta| {
//             // Inputs
//             let q = meta.query_selector(q_cond_l);
//             let input_1 = meta.query_advice(l_orderkey, Rotation::cur());
//             let input_2 = meta.query_advice(l_extendedprice, Rotation::cur());
//             let input_3 = meta.query_advice(l_discount, Rotation::cur());

//             let table_1 = meta.query_advice(perm_l_orderkey, Rotation::cur());
//             let table_2 = meta.query_advice(perm_l_extendedprice, Rotation::cur());
//             let table_3 = meta.query_advice(perm_l_discount, Rotation::cur());

//             // Constraints
//             vec![
//                 (q.clone() * input_1, table_1),
//                 (q.clone() * input_2, table_2),
//                 (q * input_3, table_3),
//             ]
//         });

//         // permutation check for o table
//         meta.shuffle("o permutation check", |meta| {
//             // Inputs
//             let q = meta.query_selector(q_cond_o);
//             let input_1 = meta.query_advice(o_orderkey, Rotation::cur());
//             let input_2 = meta.query_advice(o_custkey, Rotation::cur());
//             let input_3 = meta.query_advice(o_orderdate, Rotation::cur());
//             let input_4 = meta.query_advice(o_shippriority, Rotation::cur());

//             let table_1 = meta.query_advice(perm_o_orderkey, Rotation::cur());
//             let table_2 = meta.query_advice(perm_o_custkey, Rotation::cur());
//             let table_3 = meta.query_advice(perm_o_orderdate, Rotation::cur());
//             let table_4 = meta.query_advice(perm_o_shippriority, Rotation::cur());

//             vec![
//                 (q.clone() * input_1, table_1),
//                 (q.clone() * input_2, table_2),
//                 (q.clone() * input_3, table_3),
//                 (q * input_4, table_4),
//             ]
//         });

//         // permutation check for c table
//         meta.shuffle("c permutation check", |meta| {
//             // Inputs
//             let q = meta.query_selector(q_cond_c);
//             let input_1 = meta.query_advice(c_custkey, Rotation::cur());
//             let table_1 = meta.query_advice(perm_c_custkey, Rotation::cur());

//             // Constraints
//             vec![(q * input_1, table_1)]
//         });

//         // permutation check for new_dedup_1 and sorted_l_o_join
//         meta.shuffle(
//             "permutation check for new_dedup_1 and sorted_l_o_join",
//             |meta| {
//                 let input_1 = meta.query_advice(new_dedup_1, Rotation::cur());
//                 let table_1 = meta.query_advice(sorted_l_o_join, Rotation::cur());
//                 vec![(input_1, table_1)]
//             },
//         );
//         // permutation check for new_dedup_2 and sorted_o_c_join
//         meta.shuffle(
//             "permutation check for new_dedup_1 and sorted_l_o_join",
//             |meta| {
//                 let input_1 = meta.query_advice(new_dedup_2, Rotation::cur());
//                 let table_1 = meta.query_advice(sorted_o_c_join, Rotation::cur());
//                 vec![(input_1, table_1)]
//             },
//         );

//         let lt_sorted_l_o_join = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort_l_o_join),
//             |meta| vec![meta.query_advice(sorted_l_o_join, Rotation::prev())],
//             |meta| vec![meta.query_advice(sorted_l_o_join, Rotation::cur())],
//         );
//         let lt_sorted_o_c_join = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort_o_c_join),
//             |meta| vec![meta.query_advice(sorted_o_c_join, Rotation::prev())],
//             |meta| vec![meta.query_advice(sorted_o_c_join, Rotation::cur())],
//         );
//         // check the sort property for sorted_l_o_join and sorted_o_c_join
//         meta.create_gate("verifies that t[i-1] <= t[i] (l_o)", |meta| {
//             let q = meta.query_selector(q_sort_l_o_join);
//             vec![q * (lt_sorted_l_o_join.is_lt(meta, None) - Expression::Constant(F::ONE))]
//         });
//         meta.create_gate("verifies that t[i-1] <= t[i] (o_c)", |meta| {
//             let q = meta.query_selector(q_sort_o_c_join);
//             vec![q * (lt_sorted_o_c_join.is_lt(meta, None) - Expression::Constant(F::ONE))]
//         });

//         // groupby sum gate
//         meta.create_gate("first unique value for aggregation", |meta| {
//             let q_first = meta.query_selector(q_first);
//             let revenue = meta.query_advice(revenue, Rotation::cur());
//             let l_extendedprice = meta.query_advice(l_extendedprice, Rotation::cur());
//             let l_discount = meta.query_advice(l_discount, Rotation::cur());
//             vec![q_first * (l_extendedprice * l_discount - revenue)]
//         });

//         meta.create_gate("not first unique value for aggregation", |meta| {
//             let q_nonfirst = meta.query_selector(q_nonfirst);
//             let prev_revenue = meta.query_advice(revenue, Rotation::prev());
//             let revenue = meta.query_advice(revenue, Rotation::cur());
//             let l_extendedprice = meta.query_advice(l_extendedprice, Rotation::cur());
//             let l_discount = meta.query_advice(l_discount, Rotation::cur());
//             vec![q_nonfirst * (l_extendedprice * l_discount + prev_revenue - revenue)]
//         });

//         TestCircuitConfig {
//             q_enable,
//             q_cond_l,
//             q_cond_o,
//             q_cond_c,
//             q_sort,
//             q_sort_final,
//             q_first,
//             q_nonfirst,
//             q_sort_l_o_join,
//             q_sort_o_c_join,

//             l_orderkey,
//             l_extendedprice,
//             l_discount,
//             l_shipdate,

//             o_orderdate,
//             o_shippriority,
//             o_custkey,
//             o_orderkey,

//             c_mktsegment,
//             c_custkey,

//             perm_l_orderkey,
//             perm_l_extendedprice,
//             perm_l_discount,

//             perm_o_orderdate,
//             perm_o_shippriority,
//             perm_o_custkey,
//             perm_o_orderkey,

//             perm_c_custkey,

//             deduplicated_a2_vec,
//             deduplicated_b2_vec,
//             deduplicated_c2_vec,
//             deduplicated_d2_vec,
//             new_dedup_1,
//             new_dedup_2,
//             sorted_l_o_join,
//             sorted_o_c_join,

//             revenue,

//             l_condition,
//             o_condition,
//             c_condition,
//             l_check,
//             o_check,
//             c_check,
//             equal_check,

//             lt_l_condition,
//             lt_o_condition,
//             lt_c_condition,
//             lt_l_orderkey_o_orderdate,
//             lt_revenue_final,
//             lt_orderdate_final,
//             lt_sorted_l_o_join,
//             lt_sorted_o_c_join,

//             groupby_l_orderkey,
//             groupby_o_custkey,
//             groupby_l_extendedprice,
//             groupby_l_discount,
//             groupby_o_orderdate,
//             groupby_o_shippriority,

//             join_l_orderkey,
//             join_l_extendedprice,
//             join_l_discount,
//             join_o_orderkey,
//             join_o_custkey,
//             join_o_orderdate,
//             join_o_shippriority,
//             join_c_custkey,
//             disjoin_l_orderkey,
//             disjoin_l_extendedprice,
//             disjoin_l_discount,
//             disjoin_o_orderkey,
//             disjoin_o_custkey,
//             disjoin_o_orderdate,
//             disjoin_o_shippriority,
//             disjoin_c_custkey,

//             sorted_revenue,
//             sorted_orderdate,
//         }
//     }

//     pub fn assign(
//         &self,
//         // layouter: &mut impl Layouter<F>,
//         layouter: &mut impl Layouter<F>,

//         l_orderkey: [u64; N],
//         l_extendedprice: [u64; N],
//         l_discount: [u64; N],
//         l_shipdate: [u64; N],

//         o_orderdate: [u64; N],
//         o_shippriority: [u64; N],
//         o_custkey: [u64; N],
//         o_orderkey: [u64; N],

//         c_mktsegment: [u64; N],
//         c_custkey: [u64; N],

//         l_condition: u64,
//         o_condition: u64,
//         c_condition: u64,
//         // l_check: [bool; N],
//         // o_check: [bool; N],
//         // c_check: [F; N],
//         // groupby_l_orderkey: [u64; N],
//         // groupby_o_orderdate: [u64; N],
//         // groupby_o_shippriority: [u64; N],
//     ) -> Result<(), Error> {
//         // Result<AssignedCell<F, F>, Error> {
//         // load the chips for the filtering conditions of the three tables
//         let l_cond_chip = LtChip::construct(self.config.lt_l_condition);
//         let o_cond_chip = LtChip::construct(self.config.lt_o_condition);
//         let c_cond_chip = IsZeroChip::construct(self.config.lt_c_condition.clone());
//         let lt_revenue_final_chip = LtEqChip::construct(self.config.lt_revenue_final);
//         let lt_orderdate_final_chip = LtEqChip::construct(self.config.lt_orderdate_final);
//         let groupby_sort_chip =
//             LtEqGenericChip::construct(self.config.lt_l_orderkey_o_orderdate.clone());

//         l_cond_chip.load(layouter)?;
//         o_cond_chip.load(layouter)?;
//         lt_revenue_final_chip.load(layouter)?;
//         lt_orderdate_final_chip.load(layouter)?;
//         groupby_sort_chip.load(layouter)?;

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 // locally compute the values for l_check: [bool; N], o_check: [bool; N], c_check: [F; N],
//                 let mut l_check: [bool; N] = [false; N];
//                 let mut o_check: [bool; N] = [false; N];
//                 let mut c_check: [F; N] = [F::from(0); N];
//                 for k in 0..N {
//                     if l_shipdate[k] < l_condition {
//                         l_check[k] = true;
//                         self.config.q_cond_l.enable(&mut region, k)?;
//                     }
//                     if o_orderdate[k] < o_condition {
//                         o_check[k] = true;
//                         self.config.q_cond_o.enable(&mut region, k)?;
//                     }
//                     if c_mktsegment[k] == c_condition {
//                         c_check[k] = F::from(1);
//                         self.config.q_cond_c.enable(&mut region, k)?;
//                     }
//                 }

//                 //assign input values
//                 for i in 0..N {
//                     // enable selectors for q_enable
//                     self.config.q_enable.enable(&mut region, i)?;

//                     // assign the input values with the below codes
//                     region.assign_advice(
//                         || "l_shipdate value",
//                         self.config.l_shipdate,
//                         i,
//                         || Value::known(F::from(l_shipdate[i])),
//                     )?;

//                     region.assign_advice(
//                         || "l_orderkey value",
//                         self.config.l_orderkey,
//                         i,
//                         || Value::known(F::from(l_orderkey[i])),
//                     )?;
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
//                         || "o_orderkey value",
//                         self.config.o_orderkey,
//                         i,
//                         || Value::known(F::from(o_orderkey[i])),
//                     )?;

//                     region.assign_advice(
//                         || "o_custkey value",
//                         self.config.o_custkey,
//                         i,
//                         || Value::known(F::from(o_custkey[i])),
//                     )?;

//                     region.assign_advice(
//                         || "o_orderdate value",
//                         self.config.o_orderdate,
//                         i,
//                         || Value::known(F::from(o_orderdate[i])),
//                     )?;

//                     region.assign_advice(
//                         || "o_shippriority value",
//                         self.config.o_shippriority,
//                         i,
//                         || Value::known(F::from(o_shippriority[i])),
//                     )?;

//                     region.assign_advice(
//                         || "c_mktsegment",
//                         self.config.c_mktsegment,
//                         i,
//                         || Value::known(F::from(c_mktsegment[i])),
//                     )?;
//                     region.assign_advice(
//                         || "c_custkey",
//                         self.config.c_custkey,
//                         i,
//                         || Value::known(F::from(c_custkey[i])),
//                     )?;

//                     // assign conditions for l,o,c
//                     region.assign_advice(
//                         || "l_condition",
//                         self.config.l_condition,
//                         i,
//                         || Value::known(F::from(l_condition)),
//                     )?;

//                     region.assign_advice(
//                         || "o_condition",
//                         self.config.o_condition,
//                         i,
//                         || Value::known(F::from(o_condition)),
//                     )?;

//                     region.assign_advice(
//                         || "c_condition",
//                         self.config.c_condition,
//                         i,
//                         || Value::known(F::from(c_condition)),
//                     )?;

//                     region.assign_advice(
//                         || "l_check",
//                         self.config.l_check,
//                         i,
//                         || Value::known(F::from(l_check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "o_check",
//                         self.config.o_check,
//                         i,
//                         || Value::known(F::from(o_check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "c_check",
//                         self.config.c_check,
//                         i,
//                         || Value::known(c_check[i]),
//                     )?;

//                     // assign values for loaded chips
//                     l_cond_chip.assign(
//                         &mut region,
//                         i,
//                         Value::known(F::from(l_shipdate[i])),
//                         Value::known(F::from(l_condition)),
//                     )?;

//                     o_cond_chip.assign(
//                         &mut region,
//                         i,
//                         Value::known(F::from(o_orderdate[i])),
//                         Value::known(F::from(o_condition)),
//                     )?;

//                     c_cond_chip.assign(
//                         &mut region,
//                         i,
//                         Value::known(F::from(c_mktsegment[i] - c_condition)),
//                     )?;
//                 }

//                 // compute values related to the join operation locally
//                 // store the attribtues of the tables that will be used in the SQL query in tuples
//                 let l_combined: Vec<_> = l_orderkey
//                     .iter()
//                     .zip(l_extendedprice.iter())
//                     .zip(l_discount.iter())
//                     .zip(l_check.iter())
//                     .map(|(((&val1, &val2), &val3), &val4)| (val1, val2, val3, val4))
//                     .collect();
//                 let o_combined: Vec<_> = o_orderkey
//                     .iter()
//                     .zip(o_custkey.iter())
//                     .zip(o_orderdate.iter())
//                     .zip(o_shippriority.iter())
//                     .zip(o_check.iter())
//                     .map(|((((&val1, &val2), &val3), &val4), &val5)| (val1, val2, val3, val4, val5))
//                     .collect();
//                 let c_combined: Vec<_> = c_custkey
//                     .iter()
//                     .zip(c_check.iter())
//                     .map(|(&val1, &val2)| (val1, val2))
//                     .collect();

//                 // println!(
//                 //     "T----------- {:?}{:?}{:?}",
//                 //     l_combined, o_combined, c_combined
//                 // );
//                 let mut a1 = Vec::new(); // join l_orderkey
//                 let mut a2 = Vec::new(); // disjoin l_orderkey
//                 let mut b1 = Vec::new(); // join o_orderkey
//                 let mut b2 = Vec::new(); // disjoin o_orderkey
//                 let mut c1 = Vec::new(); // join o_custkey
//                 let mut c2 = Vec::new(); // disjoin o_custkey
//                 let mut d1 = Vec::new(); // join c_custkey
//                 let mut d2 = Vec::new(); // disjoin c_custkey

//                 let mut a1_indices = Vec::<usize>::new();
//                 let mut a2_indices = Vec::<usize>::new();
//                 let mut b1_indices = Vec::<usize>::new();
//                 let mut b2_indices = Vec::<usize>::new();
//                 // let mut c1_indices = Vec::<usize>::new();
//                 // let mut c2_indices = Vec::<usize>::new();
//                 let mut d1_indices = Vec::<usize>::new();
//                 let mut d2_indices = Vec::<usize>::new();

//                 for (i, (x, _, _, y1_val)) in l_combined.iter().enumerate() {
//                     if y1_val == &true {
//                         if o_combined
//                             .iter()
//                             .any(|(bx, _, _, _, by)| x == bx && by == &true)
//                         {
//                             a1.push(*x);
//                             a1_indices.push(i);
//                         } else {
//                             a2.push(*x);
//                             a2_indices.push(i);
//                         }
//                     }
//                 }

//                 for (i, (x, _, _, _, y2_val)) in o_combined.iter().enumerate() {
//                     if y2_val == &true {
//                         if l_combined
//                             .iter()
//                             .any(|(ax, _, _, ay)| x == ax && ay == &true)
//                         {
//                             b1.push(*x);
//                             b1_indices.push(i);
//                         } else {
//                             b2.push(*x);
//                             b2_indices.push(i);
//                         }
//                     }
//                 }

//                 for (i, (_, x, _, _, y3_val)) in o_combined.iter().enumerate() {
//                     if y3_val == &true {
//                         if c_combined
//                             .iter()
//                             .any(|(bx, by)| x == bx && by == &F::from(1))
//                         {
//                             c1.push(*x);
//                             // c1_indices.push(i);
//                         } else {
//                             c2.push(*x);
//                             // c2_indices.push(i);
//                         }
//                     }
//                 }

//                 for (i, (x, y4_val)) in c_combined.iter().enumerate() {
//                     if y4_val == &F::from(1) {
//                         if o_combined
//                             .iter()
//                             .any(|(_, ax, _, _, ay)| x == ax && ay == &true)
//                         {
//                             d1.push(*x);
//                             d1_indices.push(i);
//                         } else {
//                             d2.push(*x);
//                             d2_indices.push(i);
//                         }
//                     }
//                 }

//                 // assign join values

//                 for i in 0..a1.len() {
//                     region.assign_advice(
//                         || "join_l_orderkey",
//                         self.config.join_l_orderkey,
//                         i,
//                         || Value::known(F::from(a1[i])),
//                     )?;
//                     region.assign_advice(
//                         || "join_l_extendedprice",
//                         self.config.join_l_extendedprice,
//                         i,
//                         || Value::known(F::from(l_combined[a1_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "join_l_discount",
//                         self.config.join_l_discount,
//                         i,
//                         || Value::known(F::from(l_combined[a1_indices[i]].2)),
//                     )?;
//                     //assign perm values
//                     region.assign_advice(
//                         || "perm_l_orderkey",
//                         self.config.perm_l_orderkey,
//                         i,
//                         || Value::known(F::from(a1[i])),
//                     )?;
//                     region.assign_advice(
//                         || "perm_l_extendedprice",
//                         self.config.perm_l_extendedprice,
//                         i,
//                         || Value::known(F::from(l_combined[a1_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "perm_l_discount",
//                         self.config.perm_l_discount,
//                         i,
//                         || Value::known(F::from(l_combined[a1_indices[i]].2)),
//                     )?;
//                 }

//                 for i in 0..a2.len() {
//                     region.assign_advice(
//                         || "disjoin_l_orderkey",
//                         self.config.disjoin_l_orderkey,
//                         i,
//                         || Value::known(F::from(a2[i])),
//                     )?;
//                     region.assign_advice(
//                         || "disjoin_l_extendedprice",
//                         self.config.disjoin_l_extendedprice,
//                         i,
//                         || Value::known(F::from(l_combined[a2_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "disjoin_l_discount",
//                         self.config.disjoin_l_discount,
//                         i,
//                         || Value::known(F::from(l_combined[a2_indices[i]].2)),
//                     )?;
//                     // assign perm values
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_l_orderkey,
//                         i + a1.len(),
//                         || Value::known(F::from(a2[i])),
//                     )?;
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_l_extendedprice,
//                         i + a1.len(),
//                         || Value::known(F::from(l_combined[a2_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_l_discount,
//                         i + a1.len(),
//                         || Value::known(F::from(l_combined[a2_indices[i]].2)),
//                     )?;
//                 }

//                 for i in 0..b1.len() {
//                     region.assign_advice(
//                         || "join_o_orderkey",
//                         self.config.join_o_orderkey,
//                         i,
//                         || Value::known(F::from(b1[i])),
//                     )?;
//                     region.assign_advice(
//                         || "join_o_custkey",
//                         self.config.join_o_custkey,
//                         i,
//                         || Value::known(F::from(o_combined[b1_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "join_o_orderdate",
//                         self.config.join_o_orderdate,
//                         i,
//                         || Value::known(F::from(o_combined[b1_indices[i]].2)),
//                     )?;
//                     region.assign_advice(
//                         || "join_o_shippriority",
//                         self.config.join_o_shippriority,
//                         i,
//                         || Value::known(F::from(o_combined[b1_indices[i]].3)),
//                     )?;
//                     //assign perm
//                     region.assign_advice(
//                         || "perm_o_orderkey",
//                         self.config.perm_o_orderkey,
//                         i,
//                         || Value::known(F::from(b1[i])),
//                     )?;
//                     region.assign_advice(
//                         || "perm_o_custkey",
//                         self.config.perm_o_custkey,
//                         i,
//                         || Value::known(F::from(o_combined[b1_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "perm_o_orderdate",
//                         self.config.perm_o_orderdate,
//                         i,
//                         || Value::known(F::from(o_combined[b1_indices[i]].2)),
//                     )?;
//                     region.assign_advice(
//                         || "perm_o_shippriority",
//                         self.config.perm_o_shippriority,
//                         i,
//                         || Value::known(F::from(o_combined[b1_indices[i]].3)),
//                     )?;
//                 }

//                 for i in 0..b2.len() {
//                     region.assign_advice(
//                         || "disjoin_o_orderkey",
//                         self.config.disjoin_o_orderkey,
//                         i,
//                         || Value::known(F::from(b2[i])),
//                     )?;
//                     region.assign_advice(
//                         || "disjoin_o_custkey",
//                         self.config.disjoin_o_custkey,
//                         i,
//                         || Value::known(F::from(o_combined[b2_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "disjoin_o_orderdate",
//                         self.config.disjoin_o_orderdate,
//                         i,
//                         || Value::known(F::from(o_combined[b2_indices[i]].2)),
//                     )?;
//                     region.assign_advice(
//                         || "disjoin_o_shippriority",
//                         self.config.disjoin_o_shippriority,
//                         i,
//                         || Value::known(F::from(o_combined[b2_indices[i]].3)),
//                     )?;
//                     //assign perm values
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_o_orderkey,
//                         i + b1.len(),
//                         || Value::known(F::from(b2[i])),
//                     )?;
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_o_custkey,
//                         i + b1.len(),
//                         || Value::known(F::from(o_combined[b2_indices[i]].1)),
//                     )?;
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_o_orderdate,
//                         i + b1.len(),
//                         || Value::known(F::from(o_combined[b2_indices[i]].2)),
//                     )?;
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_o_shippriority,
//                         i + b1.len(),
//                         || Value::known(F::from(o_combined[b2_indices[i]].3)),
//                     )?;
//                 }

//                 for i in 0..d1.len() {
//                     region.assign_advice(
//                         || "join_c_custkey",
//                         self.config.join_c_custkey,
//                         i,
//                         || Value::known(F::from(d1[i])),
//                     )?;
//                     //assign perm
//                     region.assign_advice(
//                         || "perm_c_custkey",
//                         self.config.perm_c_custkey,
//                         i,
//                         || Value::known(F::from(d1[i])),
//                     )?;
//                 }

//                 for i in 0..d2.len() {
//                     region.assign_advice(
//                         || "disjoin_c_custkey",
//                         self.config.disjoin_c_custkey,
//                         i,
//                         || Value::known(F::from(d2[i])),
//                     )?;
//                     //assign perm
//                     region.assign_advice(
//                         || "perm",
//                         self.config.perm_c_custkey,
//                         i + d1.len(),
//                         || Value::known(F::from(d2[i])),
//                     )?;
//                 }

//                 // generate deduplicated columns for a2, b2 and d2 locally
//                 // Convert the vector to a HashSet to deduplicate
//                 let deduplicated_a2: HashSet<_> = a2.into_iter().collect();
//                 // Convert the HashSet back to a vector if needed
//                 let deduplicated_a2_vec: Vec<_> = deduplicated_a2.into_iter().collect();

//                 let deduplicated_b2: HashSet<_> = b2.into_iter().collect();
//                 let deduplicated_b2_vec: Vec<_> = deduplicated_b2.into_iter().collect();

//                 let deduplicated_c2: HashSet<_> = c2.into_iter().collect();
//                 let deduplicated_c2_vec: Vec<_> = deduplicated_c2.into_iter().collect();

//                 let deduplicated_d2: HashSet<_> = d2.into_iter().collect();
//                 let deduplicated_d2_vec: Vec<_> = deduplicated_d2.into_iter().collect();

//                 for i in 0..deduplicated_a2_vec.len() {
//                     region.assign_advice(
//                         || "deduplicated_a2_vec",
//                         self.config.deduplicated_a2_vec,
//                         i,
//                         || Value::known(F::from(deduplicated_a2_vec[i])),
//                     )?;
//                 }
//                 for i in 0..deduplicated_b2_vec.len() {
//                     region.assign_advice(
//                         || "deduplicated_b2_vec",
//                         self.config.deduplicated_b2_vec,
//                         i,
//                         || Value::known(F::from(deduplicated_b2_vec[i])),
//                     )?;
//                 }
//                 for i in 0..deduplicated_c2_vec.len() {
//                     region.assign_advice(
//                         || "deduplicated_c2_vec",
//                         self.config.deduplicated_c2_vec,
//                         i,
//                         || Value::known(F::from(deduplicated_c2_vec[i])),
//                     )?;
//                 }
//                 for i in 0..deduplicated_d2_vec.len() {
//                     region.assign_advice(
//                         || "deduplicated_d2_vec",
//                         self.config.deduplicated_d2_vec,
//                         i,
//                         || Value::known(F::from(deduplicated_d2_vec[i])),
//                     )?;
//                 }
//                 // concatenate two vectors for sorting
//                 let mut new_dedup_1: Vec<u64> = deduplicated_a2_vec
//                     .into_iter()
//                     .chain(deduplicated_b2_vec)
//                     .collect();
//                 let mut new_dedup_2: Vec<u64> = deduplicated_d2_vec
//                     .into_iter()
//                     .chain(deduplicated_c2_vec)
//                     .collect();
//                 // assign the new dedup
//                 for i in 0..new_dedup_1.len() {
//                     region.assign_advice(
//                         || "new_dedup_1",
//                         self.config.new_dedup_1,
//                         i,
//                         || Value::known(F::from(new_dedup_1[i])),
//                     )?;
//                 }
//                 for i in 0..new_dedup_2.len() {
//                     region.assign_advice(
//                         || "new_dedup_2",
//                         self.config.new_dedup_2,
//                         i,
//                         || Value::known(F::from(new_dedup_2[i])),
//                     )?;
//                 }
//                 // sort them
//                 new_dedup_1.sort();
//                 new_dedup_1.sort();
//                 // assign the sorted ones
//                 for i in 0..new_dedup_1.len() {
//                     if i != 0 {
//                         self.config.q_sort_l_o_join.enable(&mut region, i)?;
//                     }
//                     region.assign_advice(
//                         || "sorted_l_o_join",
//                         self.config.sorted_l_o_join,
//                         i,
//                         || Value::known(F::from(new_dedup_1[i])),
//                     )?;
//                 }
//                 for i in 0..new_dedup_2.len() {
//                     if i != 0 {
//                         self.config.q_sort_o_c_join.enable(&mut region, i)?;
//                     }
//                     region.assign_advice(
//                         || "sorted_o_c_join",
//                         self.config.sorted_o_c_join,
//                         i,
//                         || Value::known(F::from(new_dedup_2[i])),
//                     )?;
//                 }

//                 //assign values for the result of join i.e. l_orderkey = o_orderkey
//                 let mut cartesian_product1 = Vec::new();
//                 for (i, &val1) in a1.iter().enumerate() {
//                     for (j, &val2) in b1.iter().enumerate() {
//                         if val1 == val2 {
//                             cartesian_product1.push(vec![
//                                 val1,
//                                 l_combined[a1_indices[i]].1,
//                                 l_combined[a1_indices[i]].2,
//                                 o_combined[b1_indices[j]].1,
//                                 o_combined[b1_indices[j]].2,
//                                 o_combined[b1_indices[j]].3,
//                             ]);
//                         }
//                     }
//                 }
//                 // println!("cartesian product1 {:?}", cartesian_product1);
//                 //assign values for the result of join i.e. c_custkey = o_custkey
//                 let mut cartesian_product = Vec::new();
//                 for (i, &val1) in d1.iter().enumerate() {
//                     for v in &cartesian_product1 {
//                         if val1 == v[3] {
//                             cartesian_product.push(vec![
//                                 v[0], // join attribute value
//                                 val1, v[1], v[2], v[3], v[4],
//                                 v[5], // cartesian_product[0] and [1] store the joined attributes
//                             ]);
//                         }
//                     }
//                 }

//                 // println!("cartesian product {:?}", cartesian_product);

//                 // the order of attributes in cartesian_product: l_orderkey/o_orderkey, c_custkey/o_custkey, l_extendedprice, l_discount, ...
//                 //sort by l_orderkey, o_orderdate, o_shippriority
//                 cartesian_product.sort_by_key(|element| {
//                     element[0].clone() + element[4].clone() + element[5].clone()
//                 });

//                 let mut revenue: Vec<u64> = Vec::new();

//                 for i in 0..cartesian_product.len() {
//                     if i == 0 {
//                         self.config.q_first.enable(&mut region, i)?;
//                         revenue.push(cartesian_product[i][2] * cartesian_product[i][3]);
//                     } else {
//                         self.config.q_sort.enable(&mut region, i)?;
//                         groupby_sort_chip.assign(
//                             &mut region,
//                             i,
//                             &[
//                                 F::from(cartesian_product[i - 1][0]),
//                                 F::from(cartesian_product[i - 1][4]),
//                                 F::from(cartesian_product[i - 1][5]),
//                             ],
//                             &[
//                                 F::from(cartesian_product[i][0]),
//                                 F::from(cartesian_product[i][4]),
//                                 F::from(cartesian_product[i][5]),
//                             ],
//                         )?;

//                         // check if it is the first value
//                         if cartesian_product[i - 1][0] == cartesian_product[i][0]
//                             && cartesian_product[i - 1][4] == cartesian_product[i][4]
//                             && cartesian_product[i - 1][5] == cartesian_product[i][5]
//                         {
//                             self.config.q_first.enable(&mut region, i)?;
//                             revenue.push(cartesian_product[i][2] * cartesian_product[i][3]);
//                         } else {
//                             self.config.q_nonfirst.enable(&mut region, i)?;
//                             revenue.push(
//                                 revenue[i - 1] + cartesian_product[i][2] * cartesian_product[i][3],
//                             );
//                         }
//                     }

//                     region.assign_advice(
//                         || "revenue",
//                         self.config.revenue,
//                         i,
//                         || Value::known(F::from(revenue[i])),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_l_orderkey",
//                         self.config.groupby_l_orderkey,
//                         i,
//                         || Value::known(F::from(cartesian_product[i][0])),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_o_custkey",
//                         self.config.groupby_o_custkey,
//                         i,
//                         || Value::known(F::from(cartesian_product[i][1])),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_l_extendedprice",
//                         self.config.groupby_l_extendedprice,
//                         i,
//                         || Value::known(F::from(cartesian_product[i][2])),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_l_discount",
//                         self.config.groupby_l_discount,
//                         i,
//                         || Value::known(F::from(cartesian_product[i][3])),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_o_orderdate",
//                         self.config.groupby_o_orderdate,
//                         i,
//                         || Value::known(F::from(cartesian_product[i][4])),
//                     )?;

//                     region.assign_advice(
//                         || "groupby_o_shippriority",
//                         self.config.groupby_o_shippriority,
//                         i,
//                         || Value::known(F::from(cartesian_product[i][5])),
//                     )?;
//                 }

//                 // generate revenue_final
//                 println!("product: {:?}", cartesian_product);
//                 let mut revenue_final: Vec<(u64, u64)> = Vec::new(); // by removing intermediate revenue values, i.e. only keep the final revenue of each group
//                 for i in 0..revenue.len() - 1 {
//                     if cartesian_product[i][0] != cartesian_product[i + 1][0]
//                         && cartesian_product[i][4] != cartesian_product[i + 1][4]
//                         && cartesian_product[i][5] != cartesian_product[i + 1][5]
//                     {
//                         revenue_final.push((revenue[i], cartesian_product[i][4]))
//                     }
//                 }
//                 revenue_final.push((
//                     revenue[revenue.len() - 1],
//                     cartesian_product[revenue.len() - 1][4],
//                 ));

//                 // order by revenue desc, o_orderdate;
//                 // let mut revenue_o_orderdate_sorted: Vec<u64> = Vec::new();
//                 revenue_final.sort_by(|a, b| {
//                     // Compare first values in descending order
//                     let cmp_first = b.0.cmp(&a.0);

//                     // If first values are the same, compare second values in ascending order
//                     if cmp_first == std::cmp::Ordering::Equal {
//                         a.1.cmp(&b.1)
//                     } else {
//                         cmp_first
//                     }
//                 });
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
//                 println!("revenue: {:?}", revenue_final);
//                 println!("equal check: {:?}", equal_check);

//                 // assign sorted revenue and orderdate
//                 for i in 0..revenue_final.len() {
//                     region.assign_advice(
//                         || "sorted_revenue",
//                         self.config.sorted_revenue,
//                         i,
//                         || Value::known(F::from(revenue_final[i].0)),
//                     )?;
//                     region.assign_advice(
//                         || "sorted_orderdate",
//                         self.config.sorted_orderdate,
//                         i,
//                         || Value::known(F::from(revenue_final[i].1)),
//                     )?;

//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.equal_check,
//                         i,
//                         || Value::known(equal_check[i]),
//                     )?;

//                     if i != 0 {
//                         self.config.q_sort_final.enable(&mut region, i)?; // selectors can not be dynamic, need to correct
//                         lt_revenue_final_chip.assign(
//                             &mut region,
//                             i,
//                             F::from(revenue_final[i].0),
//                             F::from(revenue_final[i - 1].0),
//                         )?;

//                         lt_orderdate_final_chip.assign(
//                             &mut region,
//                             i,
//                             F::from(revenue_final[i - 1].1),
//                             F::from(revenue_final[i].1),
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
//     l_orderkey: [u64; N],
//     l_extendedprice: [u64; N],
//     l_discount: [u64; N],
//     l_shipdate: [u64; N],

//     o_orderdate: [u64; N],
//     o_shippriority: [u64; N],
//     o_custkey: [u64; N],
//     o_orderkey: [u64; N],

//     c_mktsegment: [u64; N],
//     c_custkey: [u64; N],

//     pub l_condition: u64,
//     pub o_condition: u64,
//     pub c_condition: u64,

//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             l_orderkey: [0; N],
//             l_extendedprice: [0; N],
//             l_discount: [0; N],
//             l_shipdate: [0; N],

//             o_orderdate: [0; N],
//             o_shippriority: [0; N],
//             o_custkey: [0; N],
//             o_orderkey: [0; N],

//             c_mktsegment: [0; N],
//             c_custkey: [0; N],

//             l_condition: 0,
//             o_condition: 0,
//             c_condition: 0,
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
//             self.l_shipdate,
//             self.o_orderdate,
//             self.o_shippriority,
//             self.o_custkey,
//             self.o_orderkey,
//             self.c_mktsegment,
//             self.c_custkey,
//             self.l_condition,
//             self.o_condition,
//             self.c_condition,
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
//         let k = 16;
//         // let mut rng = rand::thread_rng();

//         let mut l_orderkey: [u64; N] = [1; N];
//         l_orderkey[5] = 3;
//         l_orderkey[6] = 3;
//         let mut l_extendedprice: [u64; N] = [1; N];
//         let mut l_discount: [u64; N] = [1; N];
//         let mut l_shipdate: [u64; N] = [1; N];
//         let mut o_orderdate: [u64; N] = [2; N];
//         let mut o_shippriority: [u64; N] = [3; N];
//         let mut o_custkey: [u64; N] = [2; N];
//         o_custkey[5] = 3;
//         o_custkey[6] = 3;

//         let mut o_orderkey: [u64; N] = [3; N];
//         let mut c_mktsegment: [u64; N] = [13; N];
//         c_mktsegment[5] = 11;
//         c_mktsegment[6] = 11;
//         let mut c_custkey: [u64; N] = [3; N];

//         let mut l_condition: u64 = 5;
//         let mut o_condition: u64 = 10;
//         let mut c_condition: u64 = 11;

//         // for i in 0..N {
//         //     l_returnflag[i] = rng.gen_range(1..=100000) as u64;
//         //     l_linestatus[i] = rng.gen_range(1..=100000) as u64;
//         // }

//         // check[0] = false;

//         // let mut l_discount: Vec<u64> = Vec::new();
//         // for i in 0..N {
//         //     l_returnflag[i] = (N - i) as u64;
//         // }

//         // l_returnflag[0] = 3;
//         // l_returnflag[1] = 5;
//         // l_returnflag[2] = 5;
//         // l_returnflag[3] = 1;
//         // l_returnflag[4] = 1;

//         let circuit = MyCircuit::<Fp> {
//             l_orderkey,
//             l_extendedprice,
//             l_discount,
//             l_shipdate,
//             o_orderdate,
//             o_shippriority,
//             o_custkey,
//             o_orderkey,
//             c_mktsegment,
//             c_custkey,
//             l_condition,
//             o_condition,
//             c_condition,
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
