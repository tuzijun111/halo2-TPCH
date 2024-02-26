use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use crate::chips::less_than::{LtChip, LtConfig, LtInstruction};
use crate::chips::lessthan_or_equal_generic::{
    LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
};
use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};

use std::{default, marker::PhantomData};

// use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
use crate::chips::permutation_any::{PermAnyChip, PermAnyConfig};
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use itertools::iproduct;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;

use std::mem;

const NUM_BYTES: usize = 5;

pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

// #[derive(Default)]
// We should use the selector to skip the row which does not satisfy shipdate values

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field + Ord> {
    q_sort: Vec<Selector>,
    q_accu: Selector,
    q_perm: Vec<Selector>,

    part: Vec<Column<Advice>>,     // p_partkey, p_type, p_name
    supplier: Vec<Column<Advice>>, // s_suppkey, s_nationkey
    lineitem: Vec<Column<Advice>>, // l_extendedprice, l_discount, l_partkey, l_suppkey, l_orderkey, l_quantity
    partsupp: Vec<Column<Advice>>, // ps_suppkey, ps_partkey, ps_supplycost,
    orders: Vec<Column<Advice>>,   // o_year, o_orderdate, o_orderkey, o_custkey
    nation: Vec<Column<Advice>>,   // n_nationkey, n_regionkey, n_name

    groupby: Vec<Column<Advice>>,

    equal_check: Column<Advice>,
    perm_helper: Vec<Vec<Column<Advice>>>,

    join_group: Vec<Vec<Column<Advice>>>,
    disjoin_group: Vec<Vec<Column<Advice>>>,

    deduplicate: Vec<Column<Advice>>,
    dedup_sort: Vec<Column<Advice>>,
    orderby: Vec<Column<Advice>>,

    sum_profit: Vec<Column<Advice>>,

    compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
    lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,

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
        let mut q_sort = Vec::new();
        for i_ in 0..6 {
            q_sort.push(meta.selector());
        }
        let q_accu = meta.selector();
        let mut q_perm = Vec::new();
        for i_ in 0..12 {
            q_perm.push(meta.complex_selector());
        }

        let mut part = Vec::new();
        let mut supplier = Vec::new();
        let mut lineitem = Vec::new();
        let mut partsupp = Vec::new();
        let mut orders = Vec::new();
        let mut nation = Vec::new();

        for _ in 0..2 {
            supplier.push(meta.advice_column());
        }
        for _ in 0..3 {
            part.push(meta.advice_column());
            partsupp.push(meta.advice_column());
            nation.push(meta.advice_column());
        }
        for _ in 0..4 {
            orders.push(meta.advice_column());
        }
        for _ in 0..6 {
            lineitem.push(meta.advice_column());
        }

        let mut join_group = Vec::new();
        let mut disjoin_group = Vec::new();

        for l in [6, 2, 8, 3, 11, 3, 14, 4, 18, 3] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            join_group.push(col.clone());
        }
        for l in [6, 2, 8, 3, 11, 3, 14, 4, 18, 3] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            disjoin_group.push(col.clone());
        }

        let mut perm_helper = Vec::new();
        for l in [6, 2, 3, 3, 4, 3] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            perm_helper.push(col);
        }

        let equal_check = meta.advice_column();

        let mut deduplicate = Vec::new();
        let mut dedup_sort = Vec::new();

        for _ in 0..4 {
            dedup_sort.push(meta.advice_column());
        }
        for _ in 0..8 {
            deduplicate.push(meta.advice_column());
        }

        let mut groupby = Vec::new();
        for _ in 0..7 {
            groupby.push(meta.advice_column());
        }
        let mut sum_profit = Vec::new();
        for _ in 0..1 {
            sum_profit.push(meta.advice_column());
        }

        let mut orderby = Vec::new();

        for _ in 0..3 {
            orderby.push(meta.advice_column());
        }

        let mut compare_condition = Vec::new();
        let mut lt_compare_condition = Vec::new();

        // permutation check for disjoin and join
        PermAnyChip::configure(
            meta,
            q_perm[0],
            q_perm[6],
            lineitem.clone(),
            perm_helper[0].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[1],
            q_perm[7],
            supplier.clone(),
            perm_helper[1].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[2],
            q_perm[8],
            partsupp.clone(),
            perm_helper[2].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[3],
            q_perm[9],
            part.clone(),
            perm_helper[3].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[4],
            q_perm[10],
            orders.clone(),
            perm_helper[4].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[5],
            q_perm[11],
            nation.clone(),
            perm_helper[5].clone(),
        );

        // disjoin sort check
        // dedup check
        let lookup_configs = [
            (0, 0, 3), // (disjoin_group index, column index)
            // (1, 1, 0),
            (1, 4, 2),
            // (3, 5, 0),
            (2, 6, 4),
            // (5, 7, 2),
            (3, 8, 7),
            // (7, 9, 0),
        ];

        for (i, j, k) in lookup_configs.iter() {
            meta.lookup_any("dedup check1", |meta| {
                let input = meta.query_advice(disjoin_group[*j][*k], Rotation::cur());
                let table = meta.query_advice(deduplicate[*i], Rotation::cur());
                vec![(input, table)]
            });
        }
        // the tuple dedup
        let lookup_configs = [
            (2, 4, 5, 2, 3), // (disjoin_group index, column index)
            (3, 6, 7, 1, 0),
        ];
        for (i, j, p, q, k) in lookup_configs.iter() {
            meta.lookup_any("dedup check2", |meta| {
                let input1 = meta.query_advice(disjoin_group[*i][*q], Rotation::cur());
                let input2 = meta.query_advice(disjoin_group[*i][*k], Rotation::cur());
                let table1 = meta.query_advice(deduplicate[*j], Rotation::cur());
                let table2 = meta.query_advice(deduplicate[*p], Rotation::cur());
                vec![(input1, table1), (input2, table2)]
            });
        }

        // join sort check
        for i in 0..4 {
            let config = LtChip::configure(
                meta,
                |meta| meta.query_selector(q_sort[i]),
                |meta| meta.query_advice(dedup_sort[i], Rotation::prev()),
                |meta| meta.query_advice(dedup_sort[i], Rotation::cur()),
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
            |meta| {
                vec![
                    meta.query_advice(groupby[0], Rotation::prev()),
                    meta.query_advice(groupby[1], Rotation::prev()),
                ]
            },
            |meta| {
                vec![
                    meta.query_advice(groupby[0], Rotation::cur()),
                    meta.query_advice(groupby[1], Rotation::cur()),
                ]
            },
        );
        compare_condition.push(config);
        meta.create_gate("t[i-1]<=t[i]'", |meta| {
            let q_enable = meta.query_selector(q_sort[4]);
            vec![q_enable * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
        });

        // sum gate: sum(l_extendedprice * (1 - l_discount)) as revenue, note that revenue column starts by zero and its length is 1 more than others
        meta.create_gate("accumulate constraint", |meta| {
            let q_accu = meta.query_selector(q_accu);
            let prev_revenue = meta.query_advice(sum_profit[0].clone(), Rotation::cur());
            let extendedprice = meta.query_advice(groupby[2], Rotation::cur());
            let discount = meta.query_advice(groupby[3], Rotation::cur());
            let supplycost = meta.query_advice(groupby[4], Rotation::cur());
            let quantity = meta.query_advice(groupby[5], Rotation::cur());
            let sum_revenue = meta.query_advice(sum_profit[0], Rotation::next());
            let check = meta.query_advice(equal_check, Rotation::cur());

            vec![
                q_accu.clone()
                    * (check.clone() * prev_revenue
                        + (extendedprice.clone()
                            * (Expression::Constant(F::from(1000)) - discount.clone())
                            - supplycost * quantity)
                        - sum_revenue),
            ]
        });
        // orderby
        // nation[i-1] <= nation[i]
        let config1 = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[5]), // q_sort[1] should start from index 1
            |meta| vec![meta.query_advice(orderby[0], Rotation::prev())], // revenue
            |meta| vec![meta.query_advice(orderby[0], Rotation::cur())],
        );
        compare_condition.push(config1.clone());

        // nation[i-1] = nation[i] and o_year[i-1]>= o_year[i]
        let config2 = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[5]), // q_sort[1] should start from index 1
            |meta| vec![meta.query_advice(orderby[1], Rotation::cur())], // revenue
            |meta| vec![meta.query_advice(orderby[1], Rotation::prev())],
        );
        compare_condition.push(config2.clone());

        meta.create_gate("verifies orderby scenarios", |meta| {
            let q_sort = meta.query_selector(q_sort[5]);
            let nation1 = meta.query_advice(orderby[0], Rotation::prev());
            let nation2 = meta.query_advice(orderby[0], Rotation::cur());

            vec![
                q_sort.clone() *
                (config1.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
                        * ((Expression::Constant(F::ONE) - (nation1 - nation2)) * config2.is_lt(meta, None)
                            - Expression::Constant(F::ONE)),
            ]
        });

        TestCircuitConfig {
            q_sort,
            q_accu,
            q_perm,

            part,
            supplier,
            lineitem,
            partsupp,
            orders,
            nation,
            equal_check,
            perm_helper,

            groupby,
            join_group,
            disjoin_group,
            dedup_sort,
            deduplicate,
            orderby,
            sum_profit,
            lt_compare_condition,
            compare_condition,

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
        partsupp: Vec<Vec<u64>>,
        orders: Vec<Vec<u64>>,
        nation: Vec<Vec<u64>>,
    ) -> Result<AssignedCell<F, F>, Error> {
        let mut compare_chip = Vec::new();
        let mut lt_compare_chip = Vec::new();
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

        let mut join_value: Vec<Vec<_>> = vec![Default::default(); 10];
        let mut disjoin_value: Vec<Vec<_>> = vec![Default::default(); 10];

        let mut combined = Vec::new();
        combined.push(lineitem.clone()); // its length is 6
        combined.push(supplier.clone()); // 2
        combined.push(partsupp.clone()); // 3
        combined.push(part.clone()); // 3
        combined.push(orders.clone()); // 4
        combined.push(nation.clone()); // 3
                                       // part: Vec<Column<Advice>>,     // p_partkey, p_type, p_name
                                       // supplier: Vec<Column<Advice>>, // s_suppkey, s_nationkey
                                       // lineitem: Vec<Column<Advice>>, // l_extendedprice, l_discount, l_partkey, l_suppkey, l_orderkey, l_quantity,
                                       // partsupp: Vec<Column<Advice>>, // ps_suppkey, ps_partkey, ps_supplycost,
                                       // orders: Vec<Column<Advice>>,   // o_year, o_orderdate, o_orderkey, o_custkey
                                       // nation: Vec<Column<Advice>>,   // n_nationkey, n_regionkey, n_name

        // s_suppkey = l_suppkey
        // and ps_suppkey = l_suppkey
        // and ps_partkey = l_partkey     note that Primary Key of partsupp is: PS_PARTKEY, PS_SUPPKEY
        // and p_partkey = l_partkey
        // and o_orderkey = l_orderkey
        // and s_nationkey = n_nationkey

        // lineitem + supplier + partsupp + part + orders + nation
        // join 1: (3, 0)
        // join 2: ((2,3), (1,0))
        // the rest

        let join_index = [(2, 0), (4, 2), (7, 0)];

        // join step 1
        let mut map = HashMap::new();
        for val in &combined[1] {
            map.insert(val[0], val);
        }
        for val in &combined[0] {
            if map.contains_key(&val[3]) {
                join_value[0].push(val.clone()); // join values
            } else {
                disjoin_value[0].push(val.clone()); // disjoin values
            }
        }
        map.clear();
        for val in &combined[0] {
            map.insert(val[3], val);
        }
        for val in &combined[1] {
            if map.contains_key(&val[0]) {
                join_value[1].push(val.clone()); // join values
            } else {
                disjoin_value[1].push(val.clone()); // disjoin values
            }
        }

        let mut temp_join = combined[0].to_vec();
        let mut to_add = Vec::new();
        for ab in temp_join.iter() {
            for c in combined[1].iter() {
                if ab[3] == c[0] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let mut temp_join = to_add;
        // join 2
        map.clear();
        let mut map1 = HashMap::new(); // for another key
        for val in &combined[2] {
            map.insert(val[1], val); //s_suppkey
            map1.insert(val[0], val); //s_nationkey
        }
        for val in &temp_join {
            if map.contains_key(&val[2]) && map1.contains_key(&val[3]) {
                join_value[2].push(val.clone()); // join values
            } else {
                disjoin_value[2].push(val.clone()); // disjoin values
            }
        }
        map.clear();
        map1.clear();
        for val in &temp_join {
            map.insert(val[2], val);
            map1.insert(val[3], val);
        }
        for val in &combined[2] {
            if map.contains_key(&val[1]) && map.contains_key(&val[0]) {
                join_value[3].push(val.clone()); // join values
            } else {
                disjoin_value[3].push(val.clone()); // disjoin values
            }
        }

        let mut to_add = Vec::new();
        for ab in temp_join.iter() {
            for c in combined[2].iter() {
                if ab[2] == c[1] && ab[3] == c[0] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let mut temp_join = to_add;

        // join 3-5
        for i in 3..6 {
            let mut map = HashMap::new();
            for val in &combined[i] {
                map.insert(val[join_index[i - 3].1], val);
            }
            for val in &temp_join {
                if map.contains_key(&val[join_index[i - 3].0]) {
                    join_value[(i - 1) * 2].push(val.clone()); // join values
                } else {
                    disjoin_value[(i - 1) * 2].push(val.clone()); // disjoin values
                }
            }
            map.clear();
            for val in &temp_join {
                map.insert(val[join_index[i - 3].0], val);
            }
            for val in &combined[i] {
                if map.contains_key(&val[join_index[i - 3].1]) {
                    join_value[(i - 1) * 2 + 1].push(val.clone()); // join values
                } else {
                    disjoin_value[(i - 1) * 2 + 1].push(val.clone()); // disjoin values
                }
            }

            let mut to_add = Vec::new();
            for ab in &temp_join {
                for c in &combined[i] {
                    if ab[join_index[i - 3].0] == c[join_index[i - 3].1] {
                        let mut joined = ab.to_vec();
                        joined.extend_from_slice(c);
                        to_add.push(joined);
                        break;
                    }
                }
            }
            temp_join = to_add;
        }

        let mut cartesian_product = &temp_join;

        let index1 = [3, 0, 2, 0, 4, 2, 7, 0];
        let index2 = [(2, 3), (1, 0)];
        let indices1 = [0, 1];
        let indices2 = [4, 5, 6, 7, 8, 9];

        // disjoin vectors
        let mut dis_vectors: Vec<Vec<u64>> = Vec::new();
        for i in 0..2 {
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
        for i in 0..6 {
            let mut column: Vec<u64> = disjoin_value[indices2[i]]
                .iter()
                .map(|v| v[index1[i + 2]])
                .collect();
            let unique_column: Vec<u64> = column
                .into_iter()
                .collect::<HashSet<_>>() // This removes duplicates
                .into_iter()
                .collect();
            dis_vectors.push(unique_column);
        }

        let mut dis1: Vec<(u64, u64)> = disjoin_value[2]
            .iter()
            .map(|v| (v[index2[0].0], v[index2[0].1]))
            .collect();
        let dis1_unique: Vec<(u64, u64)> = dis1
            .into_iter()
            .collect::<HashSet<_>>() // This removes duplicates
            .into_iter()
            .collect();
        let mut dis2: Vec<(u64, u64)> = disjoin_value[3]
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
        // join1 is used for permutation cheeck and later operations
        let mut join1: Vec<Vec<F>> = cartesian_product
            .iter()
            .map(|v| {
                let mut new_vec = Vec::new();
                if v.len() >= 1 {
                    new_vec.push(F::from(v[20])); // n_name as nation,
                    new_vec.push(F::from(v[14])); // extract(year from o_orderdate) as o_year,
                    new_vec.push(F::from(v[0])); // l_extendedprice *
                    new_vec.push(F::from(v[1])); // l_discount
                    new_vec.push(F::from(v[10])); // ps_supplycost
                    new_vec.push(F::from(v[5])); // l_quantity
                    new_vec.push(F::from(v[0] * (1000 - v[1])) - F::from(v[10] * v[5]));
                    // l_extendedprice * (1 - l_discount) - ps_supplycost * l_quantity as amount
                }
                new_vec
            })
            .collect();
        // group by nation, o_year
        join1.sort_by_key(|v| v[0] + v[1]);

        //
        let mut equal_check: Vec<F> = Vec::new();

        if join1.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
        }

        for row in 1..join1.len() {
            // self.config.q_sort[6].enable(&mut region, row)?;
            if join1[row][0] == join1[row - 1][0] && join1[row][1] == join1[row - 1][1] {
                // check if o_year[i-1] = o_year[i]
                equal_check.push(F::from(1));
            } else {
                equal_check.push(F::from(0));
            }
        }
        // sum_profits
        let n = join1.len() + 1;
        let mut sum_profit: Vec<F> = vec![F::from(0); n];
        for i in 1..n {
            sum_profit[i] = sum_profit[i - 1] * equal_check[i - 1]  // sum(l_extendedprice * (1 - l_discount)) as revenue,
                + join1[i-1][6];
        }

        // order by
        // nation,
        // o_year desc;
        let mut grouped_data: Vec<Vec<F>> = Vec::new();
        for row in &join1 {
            // Check if the group (a1 value) already exists
            match grouped_data
                .iter_mut()
                .find(|g| g[0] == row[0] && g[1] == row[1])
            {
                Some(group) => {
                    group[2] += row[6]; // Add to the existing sum
                }
                None => {
                    grouped_data.push(vec![row[0], row[1], row[6]]);
                    // Create a new group
                }
            }
        }
        // println!("cartesian {:?}", cartesian_product);
        // println!("grouped data {:?}", grouped_data);

        grouped_data.sort_by(|a, b| match a[0].cmp(&b[0]) {
            Ordering::Equal => b[1].cmp(&a[1]),
            other => other,
        });

        layouter.assign_region(
            || "witness",
            |mut region| {
                //assign input values
                for i in 0..part.len() {
                    self.config.q_perm[3].enable(&mut region, i)?;
                    for j in 0..part[0].len() {
                        region.assign_advice(
                            || "p",
                            self.config.part[j],
                            i,
                            || Value::known(F::from(part[i][j])),
                        )?;
                    }
                }
                for i in 0..supplier.len() {
                    self.config.q_perm[1].enable(&mut region, i)?;
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
                    self.config.q_perm[0].enable(&mut region, i)?;
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[j],
                            i,
                            || Value::known(F::from(lineitem[i][j])),
                        )?;
                    }
                }
                for i in 0..partsupp.len() {
                    self.config.q_perm[2].enable(&mut region, i)?;
                    for j in 0..partsupp[0].len() {
                        region.assign_advice(
                            || "ps",
                            self.config.partsupp[j],
                            i,
                            || Value::known(F::from(partsupp[i][j])),
                        )?;
                    }
                }
                for i in 0..orders.len() {
                    self.config.q_perm[4].enable(&mut region, i)?;
                    for j in 0..orders[0].len() {
                        region.assign_advice(
                            || "o",
                            self.config.orders[j],
                            i,
                            || Value::known(F::from(orders[i][j])),
                        )?;
                    }
                }
                for i in 0..nation.len() {
                    self.config.q_perm[5].enable(&mut region, i)?;
                    for j in 0..nation[0].len() {
                        region.assign_advice(
                            || "n",
                            self.config.nation[j],
                            i,
                            || Value::known(F::from(nation[i][j])),
                        )?;
                    }
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
                for (k, p) in [(0, 0), (1, 1), (2, 3), (3, 5), (4, 7), (5, 9)] {
                    let join_config = &self.config.join_group[p];
                    let perm_config = &self.config.perm_helper[k];

                    for i in 0..join_value[p].len() {
                        for j in 0..join_value[p][0].len() {
                            // println!("{:?}", j);
                            let cell1 = region
                                .assign_advice(
                                    || "join_config",
                                    join_config[j],
                                    i,
                                    || Value::known(F::from(join_value[p][i][j])),
                                )?
                                .cell();
                            let cell2 = region
                                .assign_advice(
                                    || "perm_config",
                                    perm_config[j],
                                    i,
                                    || Value::known(F::from(join_value[p][i][j])),
                                )?
                                .cell();
                            // region.constrain_equal(cell1, cell2)?; // copy constraints
                        }
                    }

                    let disjoin_config = &self.config.disjoin_group[p];
                    for i in 0..disjoin_value[p].len() {
                        for j in 0..disjoin_value[p][i].len() {
                            let cell1 = region
                                .assign_advice(
                                    || "n",
                                    disjoin_config[j],
                                    i,
                                    || Value::known(F::from(disjoin_value[p][i][j])),
                                )?
                                .cell();

                            let cell2 = region
                                .assign_advice(
                                    || "perm_config",
                                    perm_config[j],
                                    i + join_value[p].len(),
                                    || Value::known(F::from(disjoin_value[p][i][j])),
                                )?
                                .cell();
                            // region.constrain_equal(cell1, cell2)?; // copy constraints
                        }
                    }
                }
                for (k, i) in [(0, 0), (1, 1), (2, 3), (3, 5), (4, 7), (5, 9)] {
                    for j in 0..join_value[i].len() + disjoin_value[i].len() {
                        self.config.q_perm[k + 6].enable(&mut region, j)?;
                    }
                }

                // dedup
                for i in [0, 2, 4, 6] {
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
                for i in 0..dis1_unique.len() {
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[4],
                        i,
                        || Value::known(F::from(dis1_unique[i].0)),
                    )?;
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[5],
                        i,
                        || Value::known(F::from(dis1_unique[i].1)),
                    )?;
                }
                for i in 0..dis2_unique.len() {
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[6],
                        i,
                        || Value::known(F::from(dis2_unique[i].0)),
                    )?;
                    region.assign_advice(
                        || "deduplicated_tuple",
                        self.config.deduplicate[7],
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
                            || "dedup sort",
                            self.config.dedup_sort[i],
                            j,
                            || Value::known(F::from(concatenated_vectors[i][j])),
                        )?;
                    }
                }

                for i in 0..join1.len() {
                    if i > 0 {
                        self.config.q_sort[4].enable(&mut region, i)?;
                    }
                    for j in 0..join1[0].len() {
                        region.assign_advice(
                            || "groupby",
                            self.config.groupby[j],
                            i,
                            || Value::known(join1[i][j]),
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
                // sum_profit assign
                for i in 0..n {
                    region.assign_advice(
                        || "sum_profit",
                        self.config.sum_profit[0],
                        i,
                        || Value::known(sum_profit[i]),
                    )?;
                }

                for i in 0..grouped_data.len() {
                    for j in 0..2 {
                        region.assign_advice(
                            || "orderby",
                            self.config.orderby[j],
                            i,
                            || Value::known(grouped_data[i][j]),
                        )?;
                    }
                    region.assign_advice(
                        // because we do not need row[2] of grouped_data
                        || "orderby",
                        self.config.orderby[2],
                        i,
                        || Value::known(grouped_data[i][2]),
                    )?;
                }

                // assign chips
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

                for i in 0..join1.len() {
                    if i > 0 {
                        compare_chip[0].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[join1[i - 1][0], join1[i - 1][1]],
                            &[join1[i][0], join1[i][1]],
                        )?;
                    }
                }

                for i in 0..grouped_data.len() {
                    if i > 0 {
                        self.config.q_sort[5].enable(&mut region, i)?;
                        compare_chip[1].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[grouped_data[i - 1][0]],
                            &[grouped_data[i][0]],
                        )?;
                        compare_chip[2].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[grouped_data[i][1]],
                            &[grouped_data[i - 1][1]],
                        )?;
                    }
                }

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
    partsupp: Vec<Vec<u64>>,
    orders: Vec<Vec<u64>>,
    nation: Vec<Vec<u64>>,

    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            part: Vec::new(),
            supplier: Vec::new(),
            lineitem: Vec::new(),
            partsupp: Vec::new(),
            orders: Vec::new(),
            nation: Vec::new(),

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
            self.partsupp.clone(),
            self.orders.clone(),
            self.nation.clone(),
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
        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem_240K.tbl";
        let orders_file_path = "/home/cc/halo2-TPCH/src/data/orders.tbl";
        let partsupp_file_path = "/home/cc/halo2-TPCH/src/data/partsupp.tbl";
        let nation_file_path = "/home/cc/halo2-TPCH/src/data/nation.tbl";

        let mut part: Vec<Vec<u64>> = Vec::new();
        let mut supplier: Vec<Vec<u64>> = Vec::new();
        let mut lineitem: Vec<Vec<u64>> = Vec::new();
        let mut partsupp: Vec<Vec<u64>> = Vec::new();
        let mut orders: Vec<Vec<u64>> = Vec::new();
        let mut nation: Vec<Vec<u64>> = Vec::new();

        if let Ok(records) = data_processing::part_read_records_from_file(part_file_path) {
            // Convert the Vec<Region> to a 2D vector
            part = records
                .iter()
                .map(|record| {
                    vec![
                        record.p_partkey,
                        string_to_u64(&record.p_type),
                        string_to_u64(&record.p_name),
                    ]
                })
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
                        record.l_quantity,
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::partsupp_read_records_from_file(partsupp_file_path) {
            // Convert the Vec<Region> to a 2D vector
            partsupp = records
                .iter()
                .map(|record| {
                    vec![
                        record.ps_suppkey,
                        record.ps_partkey,
                        scale_by_1000(record.ps_supplycost),
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

        // let part: Vec<Vec<u64>> = part.iter().take(300).cloned().collect();
        // let supplier: Vec<Vec<u64>> = supplier.iter().take(300).cloned().collect();
        // let lineitem: Vec<Vec<u64>> = lineitem.iter().take(300).cloned().collect();
        // let orders: Vec<Vec<u64>> = orders.iter().take(300).cloned().collect();

        let circuit = MyCircuit::<Fp> {
            part,
            supplier,
            lineitem,
            partsupp,
            orders,
            nation,

            _marker: PhantomData,
        };

        let public_input = vec![Fp::from(1)];

        // let test = true;
        let test = false;

        if test {
            let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
            prover.assert_satisfied();
        } else {
            let proof_path = "/home/cc/halo2-TPCH/src/sql/proof_q9_240K";
            generate_and_verify_proof(k, circuit, &public_input, proof_path);
        }
    }
}
