use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};
use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use crate::chips::less_than::{LtChip, LtConfig, LtInstruction};
use crate::chips::lessthan_or_equal_generic::{
    LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
};
use crate::chips::lessthan_or_equal_v1::{LtEqVecChip, LtEqVecConfig, LtEqVecInstruction};
use std::thread::sleep;
use std::{default, marker::PhantomData};

// use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use itertools::iproduct;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use std::mem;

const NUM_BYTES: usize = 5;
pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field + Ord> {
    q_enable: Vec<Selector>,

    q_sort: Vec<Selector>,
    q_accu: Selector,
    q_join: Vec<Selector>,

    customer: Vec<Column<Advice>>, //  c_custkey, c_nationkey
    orders: Vec<Column<Advice>>,   // o_orderdate, o_custkey, o_orderkey
    lineitem: Vec<Column<Advice>>, // l_order, l_extened, l_dis, l_supp
    supplier: Vec<Column<Advice>>, // s_nationkey, s_supp
    nation: Vec<Column<Advice>>,   // n_nationkey, n_regionkey, n_name
    regions: Vec<Column<Advice>>,  // r_regionkey, r_name

    join_group: Vec<Vec<Column<Advice>>>, // its length is 10
    disjoin_group: Vec<Vec<Column<Advice>>>,

    check: Vec<Column<Advice>>,

    deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey

    dedup_sort: Vec<Column<Advice>>,

    condition: Vec<Column<Advice>>, // conditional value for l, c and o

    groupby: Vec<Column<Advice>>,
    orderby: Vec<Column<Advice>>,
    perm_helper: Vec<Vec<Column<Advice>>>,

    equal_check: Column<Advice>, // check if sorted_revenue[i-1] = sorted_revenue[i]
    revenue: Column<Advice>,

    equal_condition: Vec<IsZeroConfig<F>>,

    compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,

    lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,

    // perm: Vec<PermAnyConfig>,
    instance: Column<Instance>,
    instance_test: Column<Advice>,
}

#[derive(Debug, Clone)]
pub struct TestChip<F: Field + Ord> {
    config: TestCircuitConfig<F>,
}

// conditions for filtering in tables: customer, orders,lineitem
//   c_mktsegment = ':1', o_orderdate < date ':2', and l_shipdate > date ':2'

// Circuits illustration
// | l_orderkey |  l_extendedprice | l_discount | l_shipdate | ...
// ------+-------+------------+------------------------+-------------------------------
//    |     |       |         0              |  0

impl<F: Field + Ord> TestChip<F> {
    pub fn construct(config: TestCircuitConfig<F>) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> TestCircuitConfig<F> {
        let instance = meta.instance_column();
        meta.enable_equality(instance);
        let instance_test = meta.advice_column();
        meta.enable_equality(instance_test);

        let mut q_enable = Vec::new();
        for i_ in 0..3 {
            q_enable.push(meta.selector());
        }

        let mut q_sort = Vec::new();
        for i_ in 0..7 {
            q_sort.push(meta.selector());
        }

        let q_accu = meta.selector();

        let mut q_join = Vec::new();
        for i_ in 0..12 {
            q_join.push(meta.complex_selector());
        }

        let mut customer = Vec::new();
        let mut orders = Vec::new();
        let mut lineitem = Vec::new();
        let mut supplier = Vec::new();
        let mut nation = Vec::new();
        let mut regions = Vec::new();

        for _ in 0..2 {
            customer.push(meta.advice_column());
            supplier.push(meta.advice_column());
            regions.push(meta.advice_column());
        }
        for _ in 0..3 {
            orders.push(meta.advice_column());
            nation.push(meta.advice_column());
        }
        for _ in 0..4 {
            lineitem.push(meta.advice_column());
        }

        let mut join_group = Vec::new();
        let mut disjoin_group = Vec::new();

        for l in [3, 2, 4, 5, 9, 2, 11, 3, 14, 2] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            join_group.push(col.clone());
        }
        for l in [3, 2, 4, 5, 9, 2, 11, 3, 14, 2] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            disjoin_group.push(col.clone());
        }

        let mut perm_helper = Vec::new();
        for l in [3, 2, 4, 2, 3, 2] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            perm_helper.push(col.clone());
        }

        let mut deduplicate = Vec::new();
        let mut dedup_sort = Vec::new();
        let mut condition = Vec::new();

        for _ in 0..6 {
            dedup_sort.push(meta.advice_column());
        }
        for _ in 0..12 {
            deduplicate.push(meta.advice_column());
        }
        for _ in 0..3 {
            condition.push(meta.advice_column());
        }

        // let mut join = Vec::new();
        let mut groupby = Vec::new();
        let mut orderby = Vec::new();
        for _ in 0..3 {
            // join.push(meta.advice_column());
            groupby.push(meta.advice_column());
        }
        for _ in 0..2 {
            orderby.push(meta.advice_column());
        }
        let equal_check = meta.advice_column();
        let revenue = meta.advice_column();

        let mut is_zero_advice_column = Vec::new();
        for _ in 0..2 {
            is_zero_advice_column.push(meta.advice_column());
        }

        let mut check = Vec::new();
        for _ in 0..3 {
            check.push(meta.advice_column());
        }

        // r_name = ':1'
        let mut equal_condition = Vec::new();
        let config = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[0]), // this is the q_enable
            |meta| {
                meta.query_advice(regions[1], Rotation::cur())
                    - meta.query_advice(condition[0], Rotation::cur())
            }, // this is the value
            is_zero_advice_column[0], // this is the advice column that stores value_inv
        );
        equal_condition.push(config.clone());

        meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
            let s = meta.query_selector(q_enable[0]);
            let output = meta.query_advice(check[0], Rotation::cur());
            vec![
                s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
                s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
            ]
        });

        let mut compare_condition = Vec::new();

        // o_orderdate >= date ':2'
        let config = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[1]),
            |meta| vec![meta.query_advice(condition[1], Rotation::cur())],
            |meta| vec![meta.query_advice(orders[0], Rotation::cur())], // we put the left and right value at the first two positions of value_l
        );
        meta.create_gate(
            "verifies o_orderdate >= date ':2'", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[1]);
                let check = meta.query_advice(check[1], Rotation::cur());
                vec![q_enable * (config.clone().is_lt(meta, None) - check)]
            },
        );
        compare_condition.push(config);

        let mut lt_compare_condition = Vec::new();
        // o_orderdate < date ':2' + interval '1' year
        let config = LtChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[1]),
            |meta| meta.query_advice(orders[0], Rotation::cur()),
            |meta| meta.query_advice(condition[2], Rotation::cur()), // we put the left and right value at the first two positions of value_l
        );
        meta.create_gate(
            "verifies o_orderdate < date ':2' + interval '1' year", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[1]);
                let check = meta.query_advice(check[2], Rotation::cur());
                vec![q_enable * (config.clone().is_lt(meta, None) - check)]
            },
        );
        lt_compare_condition.push(config);

        // disjoin sort check
        // dedup check
        let lookup_configs = [
            (0, 0, 1), // (disjoin_group index, column index)
            (1, 1, 0),
            (2, 2, 0),
            (3, 3, 2),
            (4, 6, 9),
            (5, 7, 0),
            (6, 8, 12),
            (7, 9, 0),
        ];

        for (i, j, k) in lookup_configs.iter() {
            meta.lookup_any("dedup check", |meta| {
                let input = meta.query_advice(disjoin_group[*j][*k], Rotation::cur());
                let table = meta.query_advice(deduplicate[*i], Rotation::cur());
                vec![(input, table)]
            });
        }
        // the tuple dedup
        let lookup_configs = [
            (4, 8, 9, 3, 8), // (disjoin_group index, column index)
            (5, 10, 11, 1, 0),
        ];
        for (i, j, p, q, k) in lookup_configs.iter() {
            meta.lookup_any("dedup check", |meta| {
                let input1 = meta.query_advice(disjoin_group[*i][*q], Rotation::cur());
                let input2 = meta.query_advice(disjoin_group[*i][*k], Rotation::cur());
                let table1 = meta.query_advice(deduplicate[*j], Rotation::cur());
                let table2 = meta.query_advice(deduplicate[*p], Rotation::cur());
                vec![(input1, table1), (input2, table2)]
            });
        }

        // two permutation check: join and disjoin
        PermAnyChip::configure(
            meta,
            q_join[0],
            q_join[6],
            orders.clone(),
            perm_helper[0].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[1],
            q_join[7],
            customer.clone(),
            perm_helper[1].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[2],
            q_join[8],
            lineitem.clone(),
            perm_helper[2].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[3],
            q_join[9],
            supplier.clone(),
            perm_helper[3].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[4],
            q_join[10],
            nation.clone(),
            perm_helper[4].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[5],
            q_join[11],
            regions.clone(),
            perm_helper[5].clone(),
        );

        // join sort check
        for i in 0..4 {
            let config = LtChip::configure(
                meta,
                |meta| meta.query_selector(q_sort[i]),
                |meta| meta.query_advice(dedup_sort[i], Rotation::prev()),
                |meta| meta.query_advice(dedup_sort[i], Rotation::cur()), // we put the left and right value at the first two positions of value_l
            );
            lt_compare_condition.push(config.clone());
            meta.create_gate("t[i-1]<t[i]'", |meta| {
                let q_enable = meta.query_selector(q_sort[i]);
                vec![q_enable * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
            });
        }

        // group by
        let config = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[4]),
            |meta| vec![meta.query_advice(groupby[0], Rotation::prev())],
            |meta| vec![meta.query_advice(groupby[0], Rotation::cur())],
        );
        compare_condition.push(config);

        // groupby permutation check

        // sum gate: sum(l_extendedprice * (1 - l_discount)) as revenue, note that revenue column starts by zero and its length is 1 more than others
        meta.create_gate("accumulate constraint", |meta| {
            let q_accu = meta.query_selector(q_accu);
            let prev_revenue = meta.query_advice(revenue.clone(), Rotation::cur());
            let extendedprice = meta.query_advice(groupby[1], Rotation::cur());
            let discount = meta.query_advice(groupby[2], Rotation::cur());
            let sum_revenue = meta.query_advice(revenue, Rotation::next());
            let check = meta.query_advice(equal_check, Rotation::cur());

            vec![
                q_accu.clone()
                    * (check.clone() * prev_revenue
                        + extendedprice.clone()
                            * (Expression::Constant(F::from(1000)) - discount.clone())
                        - sum_revenue),
            ]
        });

        // orderby
        // (1) revenue[i-1] >= revenue[i]
        let config = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[5]), // q_sort[1] should start from index 1
            |meta| vec![meta.query_advice(orderby[1], Rotation::cur())], // revenue
            |meta| vec![meta.query_advice(orderby[1], Rotation::prev())],
        );
        compare_condition.push(config.clone());

        meta.create_gate("verifies orderby scenarios", |meta| {
            let q_sort = meta.query_selector(q_sort[5]);

            vec![q_sort.clone() * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
        });

        TestCircuitConfig {
            q_enable,
            q_sort,
            q_accu,
            q_join,

            customer,
            orders,
            lineitem,
            supplier,
            nation,
            regions,

            join_group,
            disjoin_group,

            check,
            deduplicate,
            dedup_sort,
            condition,

            groupby,
            orderby,
            perm_helper,
            equal_check,

            revenue,

            equal_condition,
            lt_compare_condition,
            compare_condition,
            // perm,
            instance,
            instance_test,
        }
    }

    pub fn assign(
        &self,
        // layouter: &mut impl Layouter<F>,
        layouter: &mut impl Layouter<F>,

        customer: Vec<Vec<u64>>,
        orders: Vec<Vec<u64>>,
        lineitem: Vec<Vec<u64>>,
        supplier: Vec<Vec<u64>>,
        nation: Vec<Vec<u64>>,
        regions: Vec<Vec<u64>>,

        condition: Vec<u64>, // its length is 3
    ) -> Result<AssignedCell<F, F>, Error> {
        let mut equal_chip = Vec::new();
        let mut compare_chip = Vec::new();
        let mut lt_compare_chip = Vec::new();

        for i in 0..self.config.equal_condition.len() {
            let chip = IsZeroChip::construct(self.config.equal_condition[i].clone());
            equal_chip.push(chip);
        }
        for i in 0..self.config.compare_condition.len() {
            let chip = LtEqGenericChip::construct(self.config.compare_condition[i].clone());
            chip.load(layouter)?;
            compare_chip.push(chip);
        }

        for i in 0..self.config.lt_compare_condition.len() {
            let chip = LtChip::construct(self.config.lt_compare_condition[i].clone());
            chip.load(layouter)?;
            lt_compare_chip.push(chip);
        }

        // println!("equal chip: {:?}", equal_chip.len());
        // println!("compare chip: {:?}", compare_chip.len());
        // println!("lt compre chip: {:?}", lt_compare_chip.len());

        // local computation
        let mut r_check = Vec::new(); // t/f
        let mut o1_check = Vec::new(); // t/f
        let mut o2_check = Vec::new(); // 0, 1

        // 0 <= o_orderdate < date_to_timestamp("1998-01-01") - date_to_timestamp("1997-01-01")

        for i in 0..orders.len() {
            if orders[i][0] >= condition[1] {
                o1_check.push(true);
            } else {
                o1_check.push(false);
            }
            if orders[i][0] < condition[2] {
                o2_check.push(true);
            } else {
                o2_check.push(false);
            }
        }
        for i in 0..regions.len() {
            if regions[i][1] == condition[0] {
                r_check.push(F::from(1));
            } else {
                r_check.push(F::from(0));
            }
        }

        // compute values related to the join operation locally

        let mut o_combined: Vec<Vec<_>> = orders
            .clone()
            .into_iter()
            .filter(|row| row[0] >= condition[1] && row[0] < condition[2]) // r_name = ':3'
            .collect();

        println!("Orders: {:?}", o_combined.len());
        let mut r_combined: Vec<Vec<_>> = regions
            .clone()
            .into_iter()
            .filter(|row| row[1] == condition[0]) // r_name = ':3'
            .collect();

        let mut join_value: Vec<Vec<_>> = vec![vec![]; 10];
        let mut disjoin_value: Vec<Vec<_>> = vec![vec![]; 10];

        let mut combined = Vec::new();
        combined.push(o_combined); // 3
        combined.push(customer.clone()); // its length is 2

        combined.push(lineitem.clone()); // 4
        combined.push(supplier.clone()); // 2
        combined.push(nation.clone()); // 3
        combined.push(r_combined); // 2

        // customer: Vec<Column<Advice>>, //  c_custkey, c_nationkey
        // orders: Vec<Column<Advice>>,   // o_orderdate, o_custkey, o_orderkey
        // lineitem: Vec<Column<Advice>>, // l_order, l_extened, l_dis, l_supp
        // supplier: Vec<Column<Advice>>, // s_nationkey, s_supp
        // nation: Vec<Column<Advice>>,   // n_nationkey, n_regionkey, n_name
        // regions: Vec<Column<Advice>>,  // r_regionkey, r_name

        // note that the second table has the primary key to be joined in the following steps
        // orders, customer,     join step 1
        // lineitem, join(orders, customer) i.e. previous join,  join step 2
        // previous join, supplier,  join step 3,   doing l_suppkey = s_suppkey and c_nationkey = s_nationkey together
        // previous join, nation, join step 4
        // previous join, region, join 5

        // finally: lineitem + orders + customer + supplier + nation + region

        // join step 1
        let mut map = HashMap::new();
        for val in &combined[1] {
            map.insert(val[0], val);
        }
        for val in &combined[0] {
            if map.contains_key(&val[1]) {
                join_value[0].push(val.clone()); // join values
            } else {
                disjoin_value[0].push(val); // disjoin values
            }
        }
        map.clear();
        for val in &combined[0] {
            map.insert(val[1], val);
        }
        for val in &combined[1] {
            if map.contains_key(&val[0]) {
                join_value[1].push(val.clone()); // join values
            } else {
                disjoin_value[1].push(val); // disjoin values
            }
        }

        let mut temp_join = combined[0].to_vec();
        let mut to_add = Vec::new();
        for ab in temp_join.iter() {
            for c in combined[1].iter() {
                if ab[1] == c[0] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let temp_join = to_add;
        println!("Join 1: {:?}", temp_join.len());

        // join step 2
        map.clear();
        for val in &temp_join {
            map.insert(val[2], val);
        }
        for val in &combined[2] {
            if map.contains_key(&val[0]) {
                join_value[2].push(val.clone()); // join values
            } else {
                disjoin_value[2].push(val); // disjoin values
            }
        }
        map.clear();
        for val in &combined[2] {
            map.insert(val[0], val);
        }
        for val in &temp_join {
            if map.contains_key(&val[2]) {
                join_value[3].push(val.clone()); // join values
            } else {
                disjoin_value[3].push(val); // disjoin values
            }
        }

        let mut to_add = Vec::new();
        for ab in combined[2].iter() {
            for c in temp_join.iter() {
                if ab[0] == c[2] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let temp_join = to_add;
        println!("Join 2: {:?}", temp_join.len());

        // join step 3
        map.clear();
        let mut map1 = HashMap::new(); // for another key
        for val in &combined[3] {
            map.insert(val[1], val); //s_suppkey
            map1.insert(val[0], val); //s_nationkey
        }
        for val in &temp_join {
            if map.contains_key(&val[3]) && map1.contains_key(&val[8]) {
                join_value[4].push(val.clone()); // join values
            } else {
                disjoin_value[4].push(val); // disjoin values
            }
        }
        map.clear();
        map1.clear();
        for val in &temp_join {
            map.insert(val[3], val);
            map1.insert(val[8], val);
        }
        for val in &combined[3] {
            if map.contains_key(&val[1]) && map.contains_key(&val[0]) {
                join_value[5].push(val.clone()); // join values
            } else {
                disjoin_value[5].push(val); // disjoin values
            }
        }

        let mut to_add = Vec::new();
        for ab in temp_join.iter() {
            for c in combined[3].iter() {
                if ab[3] == c[1] && ab[8] == c[0] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let temp_join = to_add;
        println!("Join 3: {:?}", temp_join.len());

        // join step 4
        map.clear();
        for val in &combined[4] {
            map.insert(val[0], val);
        }
        for val in &temp_join {
            if map.contains_key(&val[9]) {
                join_value[6].push(val.clone()); // join values
            } else {
                disjoin_value[6].push(val); // disjoin values
            }
        }
        map.clear();
        for val in &temp_join {
            map.insert(val[9], val);
        }
        for val in &combined[4] {
            if map.contains_key(&val[0]) {
                join_value[7].push(val.clone()); // join values
            } else {
                disjoin_value[7].push(val); // disjoin values
            }
        }

        let mut to_add = Vec::new();
        for ab in temp_join.iter() {
            for c in combined[4].iter() {
                if ab[9] == c[0] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let temp_join = to_add;
        println!("Join 4: {:?}", temp_join.len());

        // join step 5
        map.clear();
        for val in &combined[5] {
            map.insert(val[0], val);
        }
        for val in &temp_join {
            if map.contains_key(&val[12]) {
                join_value[8].push(val.clone()); // join values
            } else {
                disjoin_value[8].push(val); // disjoin values
            }
        }
        map.clear();
        for val in &temp_join {
            map.insert(val[12], val);
        }
        for val in &combined[5] {
            if map.contains_key(&val[0]) {
                join_value[9].push(val.clone()); // join values
            } else {
                disjoin_value[9].push(val); // disjoin values
            }
        }

        let mut to_add = Vec::new();
        for ab in temp_join.iter() {
            for c in combined[5].iter() {
                if ab[12] == c[0] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }

        let mut cartesian_product = to_add;
        println!("Join 5: {:?}", temp_join.len());

        let index1 = [1, 0, 0, 2, 9, 0, 12, 0];
        let index2 = [(3, 8), (1, 0)];
        let indices1 = [0, 1, 2, 3];
        let indices2 = [6, 7, 8, 9];

        // disjoin vectors
        let mut dis_vectors: Vec<Vec<u64>> = Vec::new();
        for i in 0..4 {
            let mut column: Vec<u64> = disjoin_value[indices1[i]]
                .iter()
                .map(|v| v[index1[i]])
                .collect();
            let unique_column: Vec<u64> = column
                .into_iter()
                .collect::<HashSet<_>>() // This removes duplicates
                .into_iter()
                .collect();
            dis_vectors.push(unique_column);
        }
        for i in 0..4 {
            let mut column: Vec<u64> = disjoin_value[indices2[i]]
                .iter()
                .map(|v| v[index1[i + 4]])
                .collect();
            let unique_column: Vec<u64> = column
                .into_iter()
                .collect::<HashSet<_>>() // This removes duplicates
                .into_iter()
                .collect();
            dis_vectors.push(unique_column);
        }

        let mut dis1: Vec<(u64, u64)> = disjoin_value[4]
            .iter()
            .map(|v| (v[index2[0].0], v[index2[0].1]))
            .collect();
        let dis1_unique: Vec<(u64, u64)> = dis1
            .into_iter()
            .collect::<HashSet<_>>() // This removes duplicates
            .into_iter()
            .collect();
        let mut dis2: Vec<(u64, u64)> = disjoin_value[5]
            .iter()
            .map(|v| (v[index2[1].0], v[index2[1].1]))
            .collect();
        let dis2_unique: Vec<(u64, u64)> = dis2
            .into_iter()
            .collect::<HashSet<_>>() // This removes duplicates
            .into_iter()
            .collect();

        // concatenate two vectors for sorting
        let mut concatenated_vectors = Vec::new();
        for i in (0..dis_vectors.len()).step_by(2) {
            if let (Some(first), Some(second)) = (dis_vectors.get(i), dis_vectors.get(i + 1)) {
                let concatenated = first
                    .iter()
                    .cloned()
                    .chain(second.iter().cloned())
                    .collect::<Vec<u64>>();
                concatenated_vectors.push(concatenated);
            }
        }
        let mut concatenated_tuple = dis1_unique.clone();
        concatenated_tuple.extend(dis2_unique.clone().into_iter());

        for mut element in &mut concatenated_vectors {
            element.sort();
        }

        concatenated_tuple.sort_by_key(|element| element.0 + element.1);

        // group by n_name
        cartesian_product.sort_by_key(|element| element[13]);

        let mut equal_check: Vec<F> = Vec::new();
        if cartesian_product.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
        }

        for row in 1..cartesian_product.len() {
            if cartesian_product[row][13] == cartesian_product[row - 1][13] {
                equal_check.push(F::from(1));
            } else {
                equal_check.push(F::from(0));
            }
        }

        let n = cartesian_product.len() + 1;
        let mut revenue: Vec<F> = vec![F::from(0); n];
        for i in 1..n {
            revenue[i] = revenue[i - 1] * equal_check[i - 1]  // sum(l_extendedprice * (1 - l_discount)) as revenue,
                + F::from(cartesian_product[i - 1][1] * (1000 - cartesian_product[i - 1][2]));
            // cartesian_product[i - 1].push(revenue[i].clone()); // add this value to correct row of cartesian product
        }

        // order by revenue desc,
        let mut grouped_data: Vec<Vec<F>> = Vec::new();
        for row in &cartesian_product {
            // Check if the group (a1 value) already exists
            match grouped_data.iter_mut().find(|g| g[0] == F::from(row[13])) {
                Some(group) => {
                    group[1] += F::from(row[1] * 1000 - row[2]); // Add to the existing sum
                }
                None => {
                    grouped_data.push(vec![F::from(row[13]), F::from(row[1] * (1000 - row[2]))]);
                    // Create a new group
                }
            }
        }

        grouped_data.sort_by_key(|element| Reverse(element[1].clone()));

        layouter.assign_region(
            || "witness",
            |mut region| {
                //assign input values
                for i in 0..customer.len() {
                    self.config.q_join[1].enable(&mut region, i)?;
                    for j in 0..customer[0].len() {
                        region.assign_advice(
                            || "customer",
                            self.config.customer[j],
                            i,
                            || Value::known(F::from(customer[i][j])),
                        )?;
                    }
                }

                for i in 0..orders.len() {
                    self.config.q_enable[1].enable(&mut region, i)?;
                    if o1_check[i] == true && o2_check[i] == true {
                        self.config.q_join[0].enable(&mut region, i)?;
                    }
                    for j in 0..orders[0].len() {
                        region.assign_advice(
                            || "orders",
                            self.config.orders[j],
                            i,
                            || Value::known(F::from(orders[i][j])),
                        )?;
                    }

                    region.assign_advice(
                        || "check1",
                        self.config.check[1],
                        i,
                        || Value::known(F::from(o1_check[i] as u64)),
                    )?;
                    region.assign_advice(
                        || "check2",
                        self.config.check[2],
                        i,
                        || Value::known(F::from(o2_check[i] as u64)),
                    )?;

                    region.assign_advice(
                        || "condition for orders",
                        self.config.condition[1],
                        i,
                        || Value::known(F::from(condition[1])),
                    )?;
                    region.assign_advice(
                        || "condition for orders",
                        self.config.condition[2],
                        i,
                        || Value::known(F::from(condition[2])),
                    )?;
                }
                for i in 0..lineitem.len() {
                    self.config.q_join[2].enable(&mut region, i)?;
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[j],
                            i,
                            || Value::known(F::from(lineitem[i][j])),
                        )?;
                    }
                }

                for i in 0..supplier.len() {
                    self.config.q_join[3].enable(&mut region, i)?;
                    for j in 0..supplier[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.supplier[j],
                            i,
                            || Value::known(F::from(supplier[i][j])),
                        )?;
                    }
                }

                for i in 0..nation.len() {
                    self.config.q_join[4].enable(&mut region, i)?;
                    for j in 0..nation[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.nation[j],
                            i,
                            || Value::known(F::from(nation[i][j])),
                        )?;
                    }
                }

                for i in 0..regions.len() {
                    self.config.q_enable[0].enable(&mut region, i)?;
                    if r_check[i] == F::from(1) {
                        self.config.q_join[5].enable(&mut region, i)?;
                    }

                    for j in 0..regions[0].len() {
                        region.assign_advice(
                            || "r",
                            self.config.regions[j],
                            i,
                            || Value::known(F::from(regions[i][j])),
                        )?;
                    }

                    region.assign_advice(
                        || "check0",
                        self.config.check[0],
                        i,
                        || Value::known(r_check[i]),
                    )?;

                    region.assign_advice(
                        || "condition for customer",
                        self.config.condition[0],
                        i,
                        || Value::known(F::from(condition[0])),
                    )?;

                    equal_chip[0].assign(
                        &mut region,
                        i,
                        Value::known(F::from(regions[i][1]) - F::from(condition[0])),
                    )?; // r_name = ':1'
                }

                // assign join and disjoin values
                for k in 0..join_value.len() {
                    // Assuming self.config has an array or vector of configurations corresponding to k

                    let join_config = &self.config.join_group[k];

                    // Process join_value[k]
                    for i in 0..join_value[k].len() {
                        for j in 0..join_value[k][i].len() {
                            region.assign_advice(
                                || "n",
                                join_config[j],
                                i,
                                || Value::known(F::from(join_value[k][i][j])),
                            )?;
                        }
                    }
                }

                for k in 0..disjoin_value.len() {
                    let disjoin_config = &self.config.disjoin_group[k];

                    for i in 0..disjoin_value[k].len() {
                        for j in 0..disjoin_value[k][i].len() {
                            region.assign_advice(
                                || "n",
                                disjoin_config[j],
                                i,
                                || Value::known(F::from(disjoin_value[k][i][j])),
                            )?;
                        }
                    }
                }

                for k in 0..disjoin_value.len() {
                    let disjoin_config = &self.config.disjoin_group[k];

                    for i in 0..disjoin_value[k].len() {
                        for j in 0..disjoin_value[k][i].len() {
                            region.assign_advice(
                                || "n",
                                disjoin_config[j],
                                i,
                                || Value::known(F::from(disjoin_value[k][i][j])),
                            )?;
                        }
                    }
                }

                // assign perm_helper to merge join_value and disjoin_value for permutation
                for (idx, &k) in [0, 1, 2, 5, 7, 9].iter().enumerate() {
                    let join_config = &self.config.join_group[k];
                    let perm_config = &self.config.perm_helper[idx];
                    // Process join_value[k]
                    for i in 0..join_value[k].len() {
                        for j in 0..join_value[k][0].len() {
                            let cell1 = region
                                .assign_advice(
                                    || "join_config",
                                    join_config[j],
                                    i,
                                    || Value::known(F::from(join_value[k][i][j])),
                                )?
                                .cell();
                            let cell2 = region
                                .assign_advice(
                                    || "perm_config",
                                    perm_config[j],
                                    i,
                                    || Value::known(F::from(join_value[k][i][j])),
                                )?
                                .cell();
                            // region.constrain_equal(cell1, cell2)?; // copy constraints
                        }
                    }

                    let disjoin_config = &self.config.disjoin_group[k];
                    for i in 0..disjoin_value[k].len() {
                        for j in 0..disjoin_value[k][i].len() {
                            let cell1 = region
                                .assign_advice(
                                    || "n",
                                    disjoin_config[j],
                                    i,
                                    || Value::known(F::from(disjoin_value[k][i][j])),
                                )?
                                .cell();

                            let cell2 = region
                                .assign_advice(
                                    || "perm_config",
                                    perm_config[j],
                                    i + join_value[k].len(),
                                    || Value::known(F::from(disjoin_value[k][i][j])),
                                )?
                                .cell();
                            // region.constrain_equal(cell1, cell2)?; // copy constraints
                        }
                    }
                }

                for (idx, &k) in [0, 1, 2, 5, 7, 9].iter().enumerate() {
                    for i in 0..join_value[k].len() + disjoin_value[k].len() {
                        self.config.q_join[idx + 6].enable(&mut region, i)?;
                    }
                }

                // dedup
                for i in 0..dis_vectors.len() {
                    for j in 0..dis_vectors[i].len() {
                        let cell1 = region
                            .assign_advice(
                                || "deduplicated_b2_vec",
                                self.config.deduplicate[i],
                                j,
                                || Value::known(F::from(dis_vectors[i][j])),
                            )?
                            .cell();
                    }
                }
                for i in 0..dis1_unique.len() {
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[8],
                        i,
                        || Value::known(F::from(dis1_unique[i].0)),
                    )?;
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[9],
                        i,
                        || Value::known(F::from(dis1_unique[i].1)),
                    )?;
                }
                for i in 0..dis2_unique.len() {
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[10],
                        i,
                        || Value::known(F::from(dis2_unique[i].0)),
                    )?;
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[11],
                        i,
                        || Value::known(F::from(dis2_unique[i].1)),
                    )?;
                }

                // dedup sort assign
                for i in 0..concatenated_vectors.len() {
                    for j in 0..concatenated_vectors[i].len() {
                        if j > 0 {
                            self.config.q_sort[i].enable(&mut region, j)?;
                        }
                        region.assign_advice(
                            || "deduplicated_tuple",
                            self.config.dedup_sort[i],
                            j,
                            || Value::known(F::from(concatenated_vectors[i][j])),
                        )?;
                    }
                }
                for i in 0..concatenated_tuple.len() {
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.dedup_sort[4],
                        i,
                        || Value::known(F::from(concatenated_tuple[i].0)),
                    )?;
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.dedup_sort[5],
                        i,
                        || Value::known(F::from(concatenated_tuple[i].1)),
                    )?;
                }

                // revenue

                for i in 0..equal_check.len() {
                    self.config.q_accu.enable(&mut region, i)?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.equal_check,
                        i,
                        || Value::known(equal_check[i]),
                    )?;
                }

                for i in 0..n {
                    region.assign_advice(
                        || "revenue",
                        self.config.revenue,
                        i,
                        || Value::known(revenue[i]),
                    )?;
                }

                // assign chips
                for i in 0..orders.len() {
                    compare_chip[0].assign(
                        &mut region,
                        i,
                        &[F::from(condition[1])],
                        &[F::from(orders[i][0])],
                    )?;

                    lt_compare_chip[0].assign(
                        &mut region,
                        i,
                        Value::known(F::from(orders[i][0])),
                        Value::known(F::from(condition[2])),
                    )?;
                }

                for i in 0..cartesian_product.len() {
                    // self.config.q_perm[0].enable(&mut region, i)?; // q_perm[0]
                    for (idx, &j) in [13, 1, 2].iter().enumerate() {
                        region.assign_advice(
                            || "groupby",
                            self.config.groupby[idx],
                            i,
                            || Value::known(F::from(cartesian_product[i][j])),
                        )?;
                    }
                    if i > 0 {
                        self.config.q_sort[4].enable(&mut region, i)?;
                        compare_chip[1].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[F::from(cartesian_product[i - 1][13])],
                            &[F::from(cartesian_product[i][13])],
                        )?;
                    }
                }

                // println!("grouped data 1 {:?}", grouped_data.len());
                for i in 0..grouped_data.len() {
                    if i > 0 {
                        self.config.q_sort[5].enable(&mut region, i)?; // start at the index 1

                        compare_chip[2].assign(
                            // revenue[i-1] > revenue[i]
                            &mut region,
                            i, // assign groupby compare chip
                            &[grouped_data[i][1]],
                            &[grouped_data[i - 1][1]],
                        )?;
                    }
                    for j in 0..2 {
                        region.assign_advice(
                            || "orderby",
                            self.config.orderby[j],
                            i,
                            || Value::known(grouped_data[i][j]),
                        )?;
                    }
                }

                for i in 0..concatenated_vectors.len() {
                    for j in 0..concatenated_vectors[i].len() {
                        if j > 0 {
                            self.config.q_sort[i].enable(&mut region, j)?;
                            lt_compare_chip[i + 1].assign(
                                &mut region,
                                j,
                                Value::known(F::from(concatenated_vectors[i][j - 1])),
                                Value::known(F::from(concatenated_vectors[i][j])),
                            )?;
                        }
                    }
                }

                // instance test

                let out = region.assign_advice(
                    || "orderby",
                    self.config.instance_test,
                    0,
                    || Value::known(F::from(1)),
                )?;
                Ok(out)
            },
        )
    }

    pub fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

struct MyCircuit<F> {
    customer: Vec<Vec<u64>>,
    orders: Vec<Vec<u64>>,
    lineitem: Vec<Vec<u64>>,
    supplier: Vec<Vec<u64>>,
    nation: Vec<Vec<u64>>,
    regions: Vec<Vec<u64>>,

    pub condition: Vec<u64>,

    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            customer: Vec::new(),
            orders: Vec::new(),
            lineitem: Vec::new(),
            supplier: Vec::new(),
            nation: Vec::new(),
            regions: Vec::new(),

            condition: vec![Default::default(); 3],
            _marker: PhantomData,
        }
    }
}

impl<F: Field + Ord> Circuit<F> for MyCircuit<F> {
    type Config = TestCircuitConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        TestChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let test_chip = TestChip::construct(config);

        let out_cells = test_chip.assign(
            &mut layouter,
            self.customer.clone(),
            self.orders.clone(),
            self.lineitem.clone(),
            self.supplier.clone(),
            self.nation.clone(),
            self.regions.clone(),
            self.condition.clone(),
        )?;

        test_chip.expose_public(&mut layouter, out_cells, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    use crate::data::data_processing;
    use chrono::{DateTime, NaiveDate, Utc};
    use halo2_proofs::dev::MockProver;
    use halo2curves::pasta::{pallas, vesta, EqAffine, Fp};
    use std::marker::PhantomData;

    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        plonk::{
            create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column,
            ConstraintSystem, Error, Instance,
        },
        poly::{
            commitment::{Params, ParamsProver},
            ipa::{
                commitment::{IPACommitmentScheme, ParamsIPA},
                multiopen::ProverIPA,
                strategy::SingleStrategy,
            },
            VerificationStrategy,
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    };
    use rand::rngs::OsRng;
    use std::process;
    use std::time::Instant;
    use std::{fs::File, io::Write, path::Path};

    fn generate_and_verify_proof<C: Circuit<Fp>>(
        k: u32,
        circuit: C,
        public_input: &[Fp], // Adjust the type according to your actual public input type
        proof_path: &str,
    ) {
        // Time to generate parameters
        // let params_time_start = Instant::now();
        // let params: ParamsIPA<vesta::Affine> = ParamsIPA::new(k);
        let params_path = "/home/cc/halo2-TPCH/src/sql/param18";
        // let mut fd = std::fs::File::create(&proof_path).unwrap();
        // params.write(&mut fd).unwrap();
        // println!("Time to generate params {:?}", params_time);

        // read params16
        let mut fd = std::fs::File::open(&params_path).unwrap();
        let params = ParamsIPA::<vesta::Affine>::read(&mut fd).unwrap();

        // Time to generate verification key (vk)
        let params_time_start = Instant::now();
        let vk = keygen_vk(&params, &circuit).expect("keygen_vk should not fail");
        let params_time = params_time_start.elapsed();
        println!("Time to generate vk {:?}", params_time);

        // Time to generate proving key (pk)
        let params_time_start = Instant::now();
        let pk = keygen_pk(&params, vk.clone(), &circuit).expect("keygen_pk should not fail");
        let params_time = params_time_start.elapsed();
        println!("Time to generate pk {:?}", params_time);

        // Proof generation
        let mut rng = OsRng;
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof::<IPACommitmentScheme<_>, ProverIPA<_>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[public_input]], // Adjust as necessary for your public input handling
            &mut rng,
            &mut transcript,
        )
        .expect("proof generation should not fail");
        let proof = transcript.finalize();

        // Write proof to file
        File::create(Path::new(proof_path))
            .expect("Failed to create proof file")
            .write_all(&proof)
            .expect("Failed to write proof");
        println!("Proof written to: {}", proof_path);

        // Proof verification
        let strategy = SingleStrategy::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        assert!(
            verify_proof(
                &params,
                pk.get_vk(),
                strategy,
                &[&[public_input]], // Adjust as necessary
                &mut transcript
            )
            .is_ok(),
            "Proof verification failed"
        );
    }

    #[test]
    fn test_1() {
        let k = 16;

        fn string_to_u64(s: &str) -> u64 {
            let mut result = 0;

            for (i, c) in s.chars().enumerate() {
                result += (i as u64 + 1) * (c as u64);
            }

            result
        }
        fn scale_by_1000(x: f64) -> u64 {
            (1000.0 * x) as u64
        }
        fn date_to_timestamp(date_str: &str) -> u64 {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(date) => {
                    let datetime: DateTime<Utc> =
                        DateTime::<Utc>::from_utc(date.and_hms(0, 0, 0), Utc);
                    datetime.timestamp() as u64
                }
                Err(_) => 0, // Return a default value like 0 in case of an error
            }
        }

        // let customer_file_path = "/Users/binbingu/halo2-TPCH/src/data/customer.tbl";
        // let orders_file_path = "/Users/binbingu/halo2-TPCH/src/data/orders.tbl";
        // let lineitem_file_path = "/Users/binbingu/halo2-TPCH/src/data/lineitem.tbl";
        // let supplier_file_path = "/Users/binbingu/halo2-TPCH/src/data/supplier.tbl";
        // let nation_file_path = "/Users/binbingu/halo2-TPCH/src/data/nation.tbl";
        // let region_file_path = "/Users/binbingu/halo2-TPCH/src/data/region.csv";

        let customer_file_path = "/home/cc/halo2-TPCH/src/data/customer.tbl";
        let orders_file_path = "/home/cc/halo2-TPCH/src/data/orders.tbl";
        // let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";
        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem_240K.tbl";
        let supplier_file_path = "/home/cc/halo2-TPCH/src/data/supplier.tbl";
        let nation_file_path = "/home/cc/halo2-TPCH/src/data/nation.tbl";
        let region_file_path = "/home/cc/halo2-TPCH/src/data/region.cvs";

        let mut customer: Vec<Vec<u64>> = Vec::new();
        let mut orders: Vec<Vec<u64>> = Vec::new();
        let mut lineitem: Vec<Vec<u64>> = Vec::new();
        let mut supplier: Vec<Vec<u64>> = Vec::new();
        let mut nation: Vec<Vec<u64>> = Vec::new();
        let mut regions: Vec<Vec<u64>> = Vec::new();

        if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
            // Convert the Vec<Region> to a 2D vector
            customer = records
                .iter()
                .map(|record| vec![record.c_custkey, record.c_nationkey])
                .collect();
        }
        if let Ok(records) = data_processing::orders_read_records_from_file(orders_file_path) {
            // Convert the Vec<Region> to a 2D vector
            orders = records
                .iter()
                .map(|record| {
                    vec![
                        date_to_timestamp(&record.o_orderdate),
                        record.o_custkey,
                        record.o_orderkey,
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
            // Convert the Vec<Region> to a 2D vector
            lineitem = records
                .iter()
                .map(|record| {
                    vec![
                        record.l_orderkey,
                        scale_by_1000(record.l_extendedprice),
                        scale_by_1000(record.l_discount),
                        record.l_suppkey,
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::supplier_read_records_from_file(supplier_file_path) {
            // Convert the Vec<Region> to a 2D vector
            supplier = records
                .iter()
                .map(|record| vec![record.s_nationkey, record.s_suppkey])
                .collect();
        }
        if let Ok(records) = data_processing::nation_read_records_from_file(nation_file_path) {
            // Convert the Vec<Region> to a 2D vector
            nation = records
                .iter()
                .map(|record| {
                    vec![
                        record.n_nationkey,
                        record.n_regionkey,
                        string_to_u64(&record.n_name),
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::region_read_records_from_csv(region_file_path) {
            // Convert the Vec<Region> to a 2D vector
            regions = records
                .iter()
                .map(|record| vec![record.r_regionkey, string_to_u64(&record.r_name)])
                .collect();
        }

        let condition = vec![
            string_to_u64("EUROPE"),
            date_to_timestamp("1997-01-01"),
            date_to_timestamp("1998-01-01"),
        ];
        // r_name = ':1', "EUROPE" ->1615
        // 1997-01-01 -> 852076800
        // 1998-01-01 -> 883612800

        let circuit = MyCircuit::<Fp> {
            customer,
            orders,
            lineitem,
            supplier,
            nation,
            regions,
            condition,
            _marker: PhantomData,
        };

        let public_input = vec![Fp::from(1)];

        // let test = true;
        let test = false;

        if test {
            let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
            prover.assert_satisfied();
        } else {
            let proof_path = "/home/cc/halo2-TPCH/src/sql/proof_q5_240K";
            generate_and_verify_proof(k, circuit, &public_input, proof_path);
        }
    }
}
