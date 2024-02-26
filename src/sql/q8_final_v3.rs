use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};
use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use crate::chips::less_than::{LtChip, LtConfig, LtInstruction};
use crate::chips::lessthan_or_equal_generic::{
    LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
};
use std::{default, marker::PhantomData};
// use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use std::cmp::Reverse;

use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use std::mem;

const NUM_BYTES: usize = 5;

pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

// #[derive(Default)]
// We should use the selector to skip the row which does not satisfy shipdate values

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field + Ord> {
    q_enable: Vec<Selector>,
    q_sort: Vec<Selector>,
    q_accu: Selector,
    q_join: Vec<Selector>,

    part: Vec<Column<Advice>>,      // p_partkey, p_type
    supplier: Vec<Column<Advice>>,  // s_suppkey, s_nationkey
    lineitem: Vec<Column<Advice>>, // l_extendedprice, l_discount, l_partkey, l_suppkey, l_orderkey,
    orders: Vec<Column<Advice>>, // o_year , o_orderdate, o_orderkey, o_custkey      (o_year needs to be generated as an extra column)
    customer: Vec<Column<Advice>>, // c_custkey, c_nationkey,
    nation_n1: Vec<Column<Advice>>, // n_nationkey, n_regionkey
    nation_n2: Vec<Column<Advice>>, // n_nationkey, n_regionkey, n_name
    regions: Vec<Column<Advice>>, // r_name, r_regionkey

    condition: Vec<Column<Advice>>, // r_name = ':2', o_orderdate between date '1995-01-01' and date '1996-12-31', p_type = ':3'

    check: Vec<Column<Advice>>,

    groupby: Vec<Column<Advice>>,

    join_group: Vec<Vec<Column<Advice>>>,
    disjoin_group: Vec<Vec<Column<Advice>>>,

    deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey

    dedup_sort: Vec<Column<Advice>>,

    sum_col: Vec<Column<Advice>>, // its length is 2, sum(case	when nation = ':1' then volumn	else 0 end), sum(volume), mkt_share is represented with the previous two
    equal_check: Column<Advice>,

    equal_condition: Vec<IsZeroConfig<F>>,
    compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
    lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,
    perm_helper: Vec<Vec<Column<Advice>>>,

    instance: Column<Instance>,
    instance_test: Column<Advice>,
}

#[derive(Debug, Clone)]
pub struct TestChip<F: Field + Ord> {
    config: TestCircuitConfig<F>,
}

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
        for i_ in 0..5 {
            q_enable.push(meta.selector());
        }
        let mut q_sort = Vec::new();
        for i_ in 0..8 {
            q_sort.push(meta.selector());
        }
        let mut q_perm = Vec::new();
        for i_ in 0..1 {
            q_perm.push(meta.selector());
        }
        let q_accu = meta.selector();
        let mut q_join = Vec::new();
        for i_ in 0..16 {
            q_join.push(meta.complex_selector());
        }

        let mut part = Vec::new();
        let mut supplier = Vec::new();
        let mut lineitem = Vec::new();
        let mut orders = Vec::new();
        let mut customer = Vec::new();
        let mut nation_n1 = Vec::new();
        let mut nation_n2 = Vec::new();
        let mut regions = Vec::new();

        for _ in 0..2 {
            part.push(meta.advice_column());
            supplier.push(meta.advice_column());
            customer.push(meta.advice_column());
            nation_n1.push(meta.advice_column());
            regions.push(meta.advice_column());
        }

        for _ in 0..3 {
            nation_n2.push(meta.advice_column());
        }
        for _ in 0..4 {
            orders.push(meta.advice_column());
        }
        for _ in 0..5 {
            lineitem.push(meta.advice_column());
        }

        let mut condition = Vec::new();
        let mut check = Vec::new();
        for _ in 0..5 {
            condition.push(meta.advice_column());
            check.push(meta.advice_column());
        }

        let mut join_group = Vec::new();
        let mut disjoin_group = Vec::new();

        for l in [5, 2, 7, 2, 9, 4, 13, 2, 15, 2, 17, 2, 19, 3] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            join_group.push(col.clone());
        }
        for l in [5, 2, 7, 2, 9, 4, 13, 2, 15, 2, 17, 2, 19, 3] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            disjoin_group.push(col.clone());
        }

        let mut deduplicate = Vec::new();
        let mut dedup_sort = Vec::new();

        for _ in 0..7 {
            dedup_sort.push(meta.advice_column());
        }
        for _ in 0..14 {
            deduplicate.push(meta.advice_column());
        }

        let mut join = Vec::new();
        let mut groupby = Vec::new();
        for _ in 0..3 {
            join.push(meta.advice_column());
            groupby.push(meta.advice_column());
        }
        let mut sum_col = Vec::new();
        for _ in 0..2 {
            sum_col.push(meta.advice_column());
        }
        let equal_check = meta.advice_column();
        let mut is_zero_advice_column = Vec::new();
        for _ in 0..2 {
            is_zero_advice_column.push(meta.advice_column());
        }

        let mut perm_helper = Vec::new();
        for l in [5, 2, 2, 4, 2, 2, 2, 3] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            perm_helper.push(col.clone());
        }

        // let vectors = [
        //     &part,
        //     &supplier,
        //     &lineitem,
        //     &orders,
        //     &customer,
        //     &nation_n1,
        //     &nation_n2,
        //     &regions,
        //     &condition,
        //     &check,
        //     &deduplicate,
        //     &dedup_sort,
        //     &join,
        //     &sum_col,
        // ];

        // for &element in vectors[3] {
        //     meta.enable_equality(element);
        // }

        // // Applying meta.enable_equality() to each element of each vector
        // for vec in vectors {
        //     for element in vec.iter() {
        //         meta.enable_equality(*element);
        //     }
        // }
        // // For 'join_group' and 'disjoin_group' (assuming you want to apply it to these as well)
        // for vec in join_group.iter().chain(disjoin_group.iter()) {
        //     for &element in vec.iter() {
        //         meta.enable_equality(element);
        //     }
        // }

        // // Apply to `equal_check` separately since it's not in a vector
        // meta.enable_equality(equal_check);

        // r_name = ':1'
        let mut equal_condition = Vec::new();
        let config = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[0]), // this is the q_enable
            |meta| {
                meta.query_advice(regions[0], Rotation::cur())
                    - meta.query_advice(condition[0], Rotation::cur())
            }, // this is the value
            is_zero_advice_column[0], // this is the advice column that stores value_inv
        );
        equal_condition.push(config.clone());

        meta.create_gate("(1) f(a, b) = if a == b {1} else {0}", |meta| {
            let s = meta.query_selector(q_enable[0]);
            let output = meta.query_advice(check[0], Rotation::cur());
            vec![
                s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
                s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
            ]
        });
        // o_orderdate between date '1995-01-01' and date '1996-12-31'
        let mut compare_condition = Vec::new();
        let config: LtEqGenericConfig<F, NUM_BYTES> = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[1]),
            |meta| vec![meta.query_advice(condition[1], Rotation::cur())],
            |meta| vec![meta.query_advice(orders[1], Rotation::cur())],
        );
        meta.create_gate(
            "verifies 1995-01-01 <= o_orderdate", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[1]);
                let check = meta.query_advice(check[1], Rotation::cur());
                vec![q_enable * (config.clone().is_lt(meta, None) - check)]
            },
        );
        compare_condition.push(config);
        let config: LtEqGenericConfig<F, NUM_BYTES> = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[2]),
            |meta| vec![meta.query_advice(orders[1], Rotation::cur())],
            |meta| vec![meta.query_advice(condition[2], Rotation::cur())],
        );
        meta.create_gate(
            "verifies  o_orderdate <= 1996-12-31", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[2]);
                let check = meta.query_advice(check[2], Rotation::cur());
                vec![q_enable * (config.clone().is_lt(meta, None) - check)]
            },
        );
        compare_condition.push(config);
        // p_type = ':3'
        let config = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[3]), // this is the q_enable
            |meta| {
                meta.query_advice(part[1], Rotation::cur())
                    - meta.query_advice(condition[3], Rotation::cur())
            }, // this is the value
            is_zero_advice_column[1], // this is the advice column that stores value_inv
        );
        equal_condition.push(config.clone());

        meta.create_gate("(2) f(a, b) = if a == b {1} else {0}", |meta| {
            let s = meta.query_selector(q_enable[3]);
            let output = meta.query_advice(check[3], Rotation::cur());
            vec![
                s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
                s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
            ]
        });
        // nation = ':1', should be checked on the cartesian product i.e. the final join table rather than nation_n2 table
        // let config = IsZeroChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_enable[4]), // this is the q_enable
        //     |meta| {
        //         meta.query_advice(nation_n2[2], Rotation::cur()) //nation_n2
        //             - meta.query_advice(condition[4], Rotation::cur())
        //     }, // this is the value
        //     is_zero_advice_column[0], // this is the advice column that stores value_inv
        // );
        // equal_condition.push(config.clone());

        // meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
        //     let s = meta.query_selector(q_enable[4]);
        //     let output = meta.query_advice(check[4], Rotation::cur());
        //     vec![
        //         s.clone() * (config.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
        //         s * (Expression::Constant(F::ONE) - config.expr()) * (output), // in this case output == 0
        //     ]
        // });

        // dedup check
        let lookup_configs = [
            (0, 0, 2), // (disjoin_group index, column index)
            (1, 2, 3),
            (2, 4, 4),
            (3, 6, 12),
            (4, 8, 14),
            (5, 10, 16),
            (6, 12, 8),
        ];

        for (i, j, k) in lookup_configs.iter() {
            meta.lookup_any("dedup check", |meta| {
                let input = meta.query_advice(disjoin_group[*j][*k], Rotation::cur());
                let table = meta.query_advice(deduplicate[*i], Rotation::cur());
                vec![(input, table)]
            });
        }

        // two permutation check: join and disjoin
        PermAnyChip::configure(
            meta,
            q_join[0],
            q_join[8],
            lineitem.clone(),
            perm_helper[0].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[1],
            q_join[9],
            part.clone(),
            perm_helper[1].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[2],
            q_join[10],
            supplier.clone(),
            perm_helper[2].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[3],
            q_join[11],
            orders.clone(),
            perm_helper[3].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[4],
            q_join[12],
            customer.clone(),
            perm_helper[4].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[5],
            q_join[13],
            nation_n1.clone(),
            perm_helper[5].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[6],
            q_join[14],
            regions.clone(),
            perm_helper[6].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_join[7],
            q_join[15],
            nation_n2.clone(),
            perm_helper[7].clone(),
        );

        // join sort check
        let mut lt_compare_condition = Vec::new();
        for i in 0..7 {
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

        // group by o_year
        let config = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[7]),
            |meta| vec![meta.query_advice(groupby[0], Rotation::prev())],
            |meta| vec![meta.query_advice(groupby[0], Rotation::cur())],
        );
        compare_condition.push(config);

        // sum gate: sum(l_extendedprice * (1 - l_discount)) as revenue, note that revenue column starts by zero and its length is 1 more than others
        meta.create_gate("accumulate constraint", |meta| {
            let q_accu = meta.query_selector(q_accu);
            let prev_nation_volumn = meta.query_advice(sum_col[0].clone(), Rotation::cur());
            let prev_volumn = meta.query_advice(sum_col[1].clone(), Rotation::cur());
            let extendedprice = meta.query_advice(groupby[1], Rotation::cur());
            let discount = meta.query_advice(groupby[2], Rotation::cur());
            let sum_nation_volumn = meta.query_advice(sum_col[0], Rotation::next());
            let sum_volumn = meta.query_advice(sum_col[1], Rotation::next());
            let equal_check = meta.query_advice(equal_check, Rotation::cur());
            let n_check = meta.query_advice(check[4], Rotation::cur());
            let volumn_value =
                extendedprice.clone() * (Expression::Constant(F::from(1000)) - discount.clone());
            vec![
                q_accu.clone()
                    * (equal_check.clone() * prev_nation_volumn + volumn_value.clone()
                        - sum_nation_volumn),
                q_accu * (equal_check * prev_volumn + volumn_value * n_check - sum_volumn),
            ]
        });

        TestCircuitConfig {
            q_enable,
            q_accu,
            q_sort,
            q_join,
            part,
            supplier,
            lineitem,
            orders,
            customer,
            nation_n1,
            nation_n2,
            regions,
            condition,
            check,
            groupby,
            perm_helper,

            join_group,
            disjoin_group,
            dedup_sort,
            deduplicate,
            sum_col,
            equal_check,
            equal_condition,
            compare_condition,
            lt_compare_condition,

            instance,
            instance_test,
        }
    }

    pub fn assign(
        &self,
        layouter: &mut impl Layouter<F>,
        part: Vec<Vec<u64>>,
        supplier: Vec<Vec<u64>>,
        lineitem: Vec<Vec<u64>>,
        orders: Vec<Vec<u64>>,
        customer: Vec<Vec<u64>>,
        nation_n1: Vec<Vec<u64>>,
        nation_n2: Vec<Vec<u64>>,
        regions: Vec<Vec<u64>>, // i.e. region, since halo2 has region keyword, we use different word here

        condition: Vec<u64>, // its last value is "nation = ':1' then volume"
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

        let mut o1_check = Vec::new();
        let mut o2_check = Vec::new();
        let mut r_check = Vec::new();
        let mut p_check = Vec::new();

        for i in 0..part.len() {
            if part[i][1] == condition[3] {
                // p_type = ':3'
                p_check.push(F::from(1));
            } else {
                p_check.push(F::from(0));
            }
        }

        for i in 0..orders.len() {
            if condition[1] <= orders[i][1] {
                o1_check.push(true);
            } else {
                o1_check.push(false);
            }
            if orders[i][1] <= condition[2] {
                o2_check.push(true);
            } else {
                o2_check.push(false);
            }
        }

        for i in 0..regions.len() {
            if regions[i][0] == condition[0] {
                // r_name = ':2'
                r_check.push(F::from(1));
            } else {
                r_check.push(F::from(0));
            }
        }

        let p_combined: Vec<Vec<_>> = part
            .clone() // due to p_type = ':3'
            .into_iter()
            .filter(|row| row[1] == condition[3])
            .collect();

        let o_combined: Vec<Vec<_>> = orders
            .clone() // due to and o_orderdate between date '1995-01-01' and date '1996-12-31'
            .into_iter()
            .filter(|row| condition[1] <= row[1] && row[1] <= condition[2])
            .collect();

        let r_combined: Vec<Vec<_>> = regions
            .clone() // due to p_type = ':3'
            .into_iter()
            .filter(|row| row[0] == condition[0])
            .collect();

        let mut join_value: Vec<Vec<_>> = vec![Default::default(); 14];
        let mut disjoin_value: Vec<Vec<_>> = vec![Default::default(); 14];
        // p_partkey = l_partkey, s_suppkey = l_suppkey, l_orderkey = o_orderkey, o_custkey = c_custkey,
        // c_nationkey = n1.n_nationkey, n1.n_regionkey = r_regionkey, s_nationkey = n2.n_nationkey
        let mut combined = Vec::new();
        combined.push(lineitem.clone()); // its length is 5
        combined.push(p_combined); // 2
        combined.push(supplier.clone()); // 2
        combined.push(o_combined); // 4
        combined.push(customer.clone()); // 2
        combined.push(nation_n1.clone()); // 2
        combined.push(r_combined); // 2
        combined.push(nation_n2.clone()); // 3

        // part: Vec<Column<Advice>>,      // p_partkey, p_type
        // supplier: Vec<Column<Advice>>,  // s_suppkey, s_nationkey
        // lineitem: Vec<Column<Advice>>, // l_extendedprice, l_discount, l_partkey, l_suppkey, l_orderkey,
        // orders: Vec<Column<Advice>>, // o_year , o_orderdate, o_orderkey, o_custkey      (o_year needs to be generated as an extra column)
        // customer: Vec<Column<Advice>>, // c_custkey, c_nationkey,
        // nation_n1: Vec<Column<Advice>>, // n_nationkey, n_regionkey
        // nation_n2: Vec<Column<Advice>>, // n_nationkey, n_regionkey, n_name
        // regions: Vec<Column<Advice>>, // r_name, r_regionkey

        // condition: Vec<Column<Advice>>, // r_name = ':2', o_orderdate between date '1995-01-01' and date '1996-12-31', p_type = ':3'

        // lineitem + part + supplier + orders + customer + nation_n1 + region + nation_n2

        let join_index = [(2, 0), (3, 0), (4, 2), (12, 0), (14, 0), (16, 1), (8, 0)];
        let mut temp_join = combined[0].clone();
        for i in 1..8 {
            let mut map = HashMap::new();
            for val in &combined[i] {
                map.insert(val[join_index[i - 1].1], val);
            }
            for val in &temp_join {
                if map.contains_key(&val[join_index[i - 1].0]) {
                    join_value[(i - 1) * 2].push(val.clone()); // join values
                } else {
                    disjoin_value[(i - 1) * 2].push(val.clone()); // disjoin values
                }
            }
            map.clear();
            for val in &temp_join {
                map.insert(val[join_index[i - 1].0], val);
            }
            for val in &combined[i] {
                if map.contains_key(&val[join_index[i - 1].1]) {
                    join_value[(i - 1) * 2 + 1].push(val.clone()); // join values
                } else {
                    disjoin_value[(i - 1) * 2 + 1].push(val.clone()); // disjoin values
                }
            }

            let mut to_add = Vec::new();
            for ab in &temp_join {
                for c in &combined[i] {
                    if ab[join_index[i - 1].0] == c[join_index[i - 1].1] {
                        let mut joined = ab.to_vec();
                        joined.extend_from_slice(c);
                        to_add.push(joined);
                        break;
                    }
                }
            }
            temp_join = to_add;
        }
        // After loop, temp_join contains the final join results
        let mut cartesian_product = &temp_join;

        let index1 = [2, 0, 3, 0, 4, 2, 12, 0, 14, 0, 16, 1, 8, 0];
        let indices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        let mut dis_vectors: Vec<Vec<u64>> = Vec::new();
        for i in 0..indices.len() {
            let mut column: Vec<u64> = disjoin_value[indices[i]]
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
        for mut element in &mut concatenated_vectors {
            element.sort();
        }

        // select
        //         extract(year from o_orderdate) as o_year,
        //         l_extendedprice * (1 - l_discount) as volume,
        //         n2.n_name as nation

        // join1 is used for permutation cheeck and later operations
        let mut join1: Vec<Vec<F>> = cartesian_product
            .iter()
            .map(|v| {
                let mut new_vec = Vec::new();
                if v.len() >= 1 {
                    new_vec.push(F::from(v[9])); // extract(year from o_orderdate) as o_year,
                    new_vec.push(F::from(v[0] * (1000 - v[1]))); // l_extendedprice * (1 - l_discount) as volume,
                    new_vec.push(F::from(v[21])); // n2.n_name as nation
                }
                new_vec
            })
            .collect();
        // group by o_year
        join1.sort_by_key(|v| v[0]);

        let mut equal_check: Vec<F> = Vec::new();

        if join1.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
        }

        for row in 1..join1.len() {
            // self.config.q_sort[6].enable(&mut region, row)?;
            if join1[row][0] == join1[row - 1][0] {
                // check if o_year[i-1] = o_year[i]
                equal_check.push(F::from(1));
            } else {
                equal_check.push(F::from(0));
            }
        }

        // volumn
        let n = join1.len();
        let mut nation_volumn: Vec<F> = vec![F::from(0); n]; // when nation = ':1' then
        let mut volumn: Vec<F> = vec![F::from(0); n]; // sum(volumn)
        for i in 1..n {
            nation_volumn[i] =
                nation_volumn[i - 1] * equal_check[i - 1] + join1[i - 1][1] * join1[i][3];
            volumn[i] = volumn[i - 1] * equal_check[i - 1] + join1[i - 1][1];
        }

        // add a new column "nation = '1' " into join1
        for vec in &mut join1 {
            if vec[2] == F::from(condition[4]) {
                vec.push(F::from(1));
            } else {
                vec.push(F::from(0));
            }
        }

        let mut equal_check: Vec<F> = Vec::new();

        if join1.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
        }

        for row in 1..join1.len() {
            // self.config.q_sort[6].enable(&mut region, row)?;
            if join1[row][0] == join1[row - 1][0] {
                // check if o_year[i-1] = o_year[i]
                equal_check.push(F::from(1));
            } else {
                equal_check.push(F::from(0));
            }
        }

        // volumn
        let n = join1.len();
        let mut nation_volumn: Vec<F> = vec![F::from(0); n]; // when nation = ':1' then
        let mut volumn: Vec<F> = vec![F::from(0); n]; // sum(volumn)
        for i in 1..n {
            nation_volumn[i] =
                nation_volumn[i - 1] * equal_check[i - 1] + join1[i - 1][1] * join1[i][3];
            volumn[i] = volumn[i - 1] * equal_check[i - 1] + join1[i - 1][1];
        }

        layouter.assign_region(
            || "witness",
            |mut region| {
                for i in 0..part.len() {
                    self.config.q_enable[3].enable(&mut region, i)?;
                    if p_check[i] == F::from(1) {
                        self.config.q_join[1].enable(&mut region, i)?;
                    }

                    for j in 0..part[0].len() {
                        region.assign_advice(
                            || "p",
                            self.config.part[j],
                            i,
                            || Value::known(F::from(part[i][j])),
                        )?;
                    }

                    region.assign_advice(
                        || "check0",
                        self.config.check[3],
                        i,
                        || Value::known(p_check[i]),
                    )?;

                    region.assign_advice(
                        || "condition for part",
                        self.config.condition[3],
                        i,
                        || Value::known(F::from(condition[3])),
                    )?;
                }
                for i in 0..supplier.len() {
                    self.config.q_join[2].enable(&mut region, i)?;
                    for j in 0..supplier[0].len() {
                        region.assign_advice(
                            || "s",
                            self.config.supplier[j],
                            i,
                            || Value::known(F::from(supplier[i][j])),
                        )?;
                    }
                }
                for i in 0..lineitem.len() {
                    self.config.q_join[0].enable(&mut region, i)?;
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[j],
                            i,
                            || Value::known(F::from(lineitem[i][j])),
                        )?;
                    }
                }
                for i in 0..orders.len() {
                    self.config.q_enable[1].enable(&mut region, i)?;
                    self.config.q_enable[2].enable(&mut region, i)?;
                    if o1_check[i] == true && o2_check[i] == true {
                        self.config.q_join[3].enable(&mut region, i)?;
                    }

                    for j in 0..orders[0].len() {
                        region.assign_advice(
                            || "o",
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
                        || "check1",
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
                for i in 0..customer.len() {
                    self.config.q_join[4].enable(&mut region, i)?;
                    for j in 0..customer[0].len() {
                        region.assign_advice(
                            || "c",
                            self.config.customer[j],
                            i,
                            || Value::known(F::from(customer[i][j])),
                        )?;
                    }
                }
                for i in 0..nation_n1.len() {
                    self.config.q_join[5].enable(&mut region, i)?;
                    for j in 0..nation_n1[0].len() {
                        region.assign_advice(
                            || "n",
                            self.config.nation_n1[j],
                            i,
                            || Value::known(F::from(nation_n1[i][j])),
                        )?;
                    }
                }
                for i in 0..nation_n2.len() {
                    self.config.q_join[7].enable(&mut region, i)?;
                    for j in 0..nation_n2[0].len() {
                        region.assign_advice(
                            || "n",
                            self.config.nation_n2[j],
                            i,
                            || Value::known(F::from(nation_n2[i][j])),
                        )?;
                    }
                }
                for i in 0..regions.len() {
                    if r_check[i] == F::from(1) {
                        self.config.q_join[6].enable(&mut region, i)?;
                    }

                    self.config.q_enable[0].enable(&mut region, i)?;
                    for j in 0..regions[0].len() {
                        region.assign_advice(
                            || "region",
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
                }

                // assign join and disjoin values
                for k in 0..join_value.len() {
                    let join_config = &self.config.join_group[k];
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
                    if k > 0 {
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
                }

                // assign perm_helper to merge join_value and disjoin_value for permutation
                for (idx, &k) in [0, 1, 3, 5, 7, 9, 11, 13].iter().enumerate() {
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

                for (idx, &k) in [0, 1, 3, 5, 7, 9, 11, 13].iter().enumerate() {
                    for i in 0..join_value[k].len() + disjoin_value[k].len() {
                        self.config.q_join[idx + 8].enable(&mut region, i)?;
                    }
                }

                // dedup
                for i in [0, 2, 4, 6, 8, 10, 12] {
                    for j in 0..dis_vectors[i].len() {
                        let cell1 = region
                            .assign_advice(
                                || "deduplicated_b2_vec",
                                self.config.deduplicate[i / 2],
                                j,
                                || Value::known(F::from(dis_vectors[i][j])),
                            )?
                            .cell();
                    }
                }

                // dedup sort assign
                for i in 0..concatenated_vectors.len() {
                    for j in 0..concatenated_vectors[i].len() {
                        if j > 0 {
                            self.config.q_sort[i].enable(&mut region, j)?;
                        }
                        region.assign_advice(
                            || "dedup sort",
                            self.config.dedup_sort[i],
                            j,
                            || Value::known(F::from(concatenated_vectors[i][j])),
                        )?;
                    }
                }

                // // assign join:
                // for i in 0..join1.len() {
                //     for j in 0..join1[0].len() {
                //         region.assign_advice(
                //             || "join",
                //             self.config.join[j],
                //             i,
                //             || Value::known(join1[i][j]),
                //         )?;
                //     }
                // }

                for i in 0..join1.len() {
                    for j in 0..join1[0].len() {
                        region.assign_advice(
                            || "groupby",
                            self.config.groupby[j],
                            i,
                            || Value::known(join1[i][j]),
                        )?;
                    }
                }

                for i in 0..join1.len() {
                    // if join1[i][2] == condition[4] {
                    //     n_check.push(F::from(1));
                    // } else {
                    //     n_check.push(F::from(0));
                    // }
                    region.assign_advice(
                        || "check4",
                        self.config.check[4],
                        i,
                        || Value::known(join1[i][3]), // nation = ':1'
                    )?;

                    region.assign_advice(
                        || "condition for nation",
                        self.config.condition[4],
                        i,
                        || Value::known(F::from(condition[4])),
                    )?;

                    equal_chip[2].assign(
                        &mut region,
                        i,
                        Value::known(join1[i][2] - F::from(condition[4])),
                    )?; //p_type = ':3'
                    if i > 0 {
                        self.config.q_sort[7].enable(&mut region, i)?;
                        compare_chip[2].assign(
                            &mut region,
                            i,
                            &[join1[i - 1][0]],
                            &[join1[i][1]],
                        )?;
                    }
                }

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
                        || "nation_volumn",
                        self.config.sum_col[0],
                        i,
                        || Value::known(nation_volumn[i]),
                    )?;
                    region.assign_advice(
                        || "volumn",
                        self.config.sum_col[1],
                        i,
                        || Value::known(volumn[i]),
                    )?;
                }
                // assign chips
                for i in 0..orders.len() {
                    compare_chip[0].assign(
                        &mut region,
                        i,
                        &[F::from(condition[1])],
                        &[F::from(orders[i][1])],
                    )?;
                    compare_chip[1].assign(
                        &mut region,
                        i,
                        &[F::from(orders[i][1])],
                        &[F::from(condition[2])],
                    )?;
                }
                for i in 0..regions.len() {
                    equal_chip[0].assign(
                        &mut region,
                        i,
                        Value::known(F::from(regions[i][0]) - F::from(condition[0])),
                    )?; // r_name = ':1'
                }
                for i in 0..part.len() {
                    equal_chip[1].assign(
                        &mut region,
                        i,
                        Value::known(F::from(part[i][1]) - F::from(condition[3])),
                    )?; //p_type = ':3'
                }
                for i in 0..concatenated_vectors.len() {
                    for j in 0..concatenated_vectors[i].len() {
                        if j > 0 {
                            lt_compare_chip[i].assign(
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

struct MyCircuit<F: Copy> {
    part: Vec<Vec<u64>>,
    supplier: Vec<Vec<u64>>,
    lineitem: Vec<Vec<u64>>,
    orders: Vec<Vec<u64>>,
    customer: Vec<Vec<u64>>,
    nation_n1: Vec<Vec<u64>>,
    nation_n2: Vec<Vec<u64>>,
    regions: Vec<Vec<u64>>,

    pub condition: Vec<u64>,
    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            part: Vec::new(),
            supplier: Vec::new(),
            lineitem: Vec::new(),
            orders: Vec::new(),
            customer: Vec::new(),
            nation_n1: Vec::new(),
            nation_n2: Vec::new(),
            regions: Vec::new(),

            condition: vec![Default::default(); 4],

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
            self.part.clone(),
            self.supplier.clone(),
            self.lineitem.clone(),
            self.orders.clone(),
            self.customer.clone(),
            self.nation_n1.clone(),
            self.nation_n2.clone(),
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

    // use rand::Rng;
    // use halo2_proofs::poly::commitment::Params
    use crate::data::data_processing;
    use chrono::{DateTime, NaiveDate, Utc};
    use halo2_proofs::dev::MockProver;
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
    use halo2curves::pasta::{pallas, vesta, EqAffine, Fp};
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

        let part_file_path = "/home/cc/halo2-TPCH/src/data/part.tbl";
        let supplier_file_path = "/home/cc/halo2-TPCH/src/data/supplier.tbl";
        // let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";
        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem_240K.tbl";
        let orders_file_path = "/home/cc/halo2-TPCH/src/data/orders.tbl";
        let customer_file_path = "/home/cc/halo2-TPCH/src/data/customer.tbl";
        let nation_n1_file_path = "/home/cc/halo2-TPCH/src/data/nation.tbl";
        let nation_n2_file_path = "/home/cc/halo2-TPCH/src/data/nation.tbl";
        let region_file_path = "/home/cc/halo2-TPCH/src/data/region.cvs";

        let mut part: Vec<Vec<u64>> = Vec::new();
        let mut supplier: Vec<Vec<u64>> = Vec::new();
        let mut lineitem: Vec<Vec<u64>> = Vec::new();
        let mut orders: Vec<Vec<u64>> = Vec::new();
        let mut customer: Vec<Vec<u64>> = Vec::new();
        let mut nation_n1: Vec<Vec<u64>> = Vec::new();
        let mut nation_n2: Vec<Vec<u64>> = Vec::new();
        let mut regions: Vec<Vec<u64>> = Vec::new();

        if let Ok(records) = data_processing::part_read_records_from_file(part_file_path) {
            // Convert the Vec<Region> to a 2D vector
            part = records
                .iter()
                .map(|record| vec![record.p_partkey, string_to_u64(&record.p_type)])
                .collect();
        }
        if let Ok(records) = data_processing::supplier_read_records_from_file(supplier_file_path) {
            // Convert the Vec<Region> to a 2D vector
            supplier = records
                .iter()
                .map(|record| vec![record.s_suppkey, record.s_nationkey])
                .collect();
        }
        if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
            // Convert the Vec<Region> to a 2D vector
            lineitem = records
                .iter()
                .map(|record| {
                    vec![
                        scale_by_1000(record.l_extendedprice),
                        scale_by_1000(record.l_discount),
                        record.l_partkey,
                        record.l_suppkey,
                        record.l_orderkey,
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::orders_read_records_from_file(orders_file_path) {
            // Convert the Vec<Region> to a 2D vector
            orders = records
                .iter()
                .map(|record| {
                    vec![
                        record.o_orderdate[..4].parse::<u64>().unwrap(), // o_year
                        date_to_timestamp(&record.o_orderdate),
                        record.o_orderkey,
                        record.o_custkey,
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
            // Convert the Vec<Region> to a 2D vector
            customer = records
                .iter()
                .map(|record| vec![record.c_custkey, record.c_nationkey])
                .collect();
        }

        if let Ok(records) = data_processing::nation_read_records_from_file(nation_n1_file_path) {
            // Convert the Vec<Region> to a 2D vector
            nation_n1 = records
                .iter()
                .map(|record| vec![record.n_nationkey, record.n_regionkey])
                .collect();
        }
        if let Ok(records) = data_processing::nation_read_records_from_file(nation_n2_file_path) {
            // Convert the Vec<Region> to a 2D vector
            nation_n2 = records
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
            string_to_u64("MIDDLE EAST"),
            date_to_timestamp("1995-01-01"),
            date_to_timestamp("1996-12-31"),
            string_to_u64("PROMO BRUSHED COPPER"),
            string_to_u64("EGYPT"),
        ];

        // r_name = "MIDDLE EAST";
        // 1995-01-01 -> 788918400
        // 1996-12-31 -> 851990400
        // p_type = "PROMO BRUSHED COPPER"
        // nation = "EGYPT"

        // 1991-01-01 -> 662688000 just for testing

        // let part: Vec<Vec<u64>> = part.iter().take(300).cloned().collect();
        // let supplier: Vec<Vec<u64>> = supplier.iter().take(300).cloned().collect();
        // let lineitem: Vec<Vec<u64>> = lineitem.iter().take(300).cloned().collect();
        // let orders: Vec<Vec<u64>> = orders.iter().take(300).cloned().collect();
        // let customer: Vec<Vec<u64>> = customer.iter().take(300).cloned().collect();

        // let nation_n1: Vec<Vec<u64>> = nation.iter().take(3).cloned().collect();
        // let nation_n1: Vec<Vec<u64>> = nation.iter().take(3).cloned().collect();
        // let region: Vec<Vec<u64>> = region.iter().take(3).cloned().collect();

        let circuit = MyCircuit::<Fp> {
            part,
            supplier,
            lineitem,
            orders,
            customer,
            nation_n1,
            nation_n2,
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
            let proof_path = "/home/cc/halo2-TPCH/src/sql/proof_q8_240K";
            generate_and_verify_proof(k, circuit, &public_input, proof_path);
        }
    }
}
