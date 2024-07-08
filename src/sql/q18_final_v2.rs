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
    q_enable: Vec<Selector>,
    q_sort: Vec<Selector>,
    q_accu: Vec<Selector>,
    q_perm: Vec<Selector>,

    lineitem: Vec<Column<Advice>>,     // l_orderkey, l_quantity
    orders: Vec<Column<Advice>>,       // o_orderdate, o_orderkey, o_custkey, o_totalprice
    customer: Vec<Column<Advice>>,     // c_name, c_custkey,
    new_lineitem: Vec<Column<Advice>>, // l_orderkey, l_quantity, sum(l_quantity)

    groupby: Vec<Column<Advice>>,

    join_group: Vec<Vec<Column<Advice>>>,
    disjoin_group: Vec<Vec<Column<Advice>>>,

    deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey

    dedup_sort: Vec<Column<Advice>>,
    perm_helper: Vec<Vec<Column<Advice>>>,
    orderby: Vec<Column<Advice>>,

    sum_quantity: Vec<Column<Advice>>, // for select l_orderkey from lineitem group by l_orderkey having sum(l_quantity) > :1

    equal_check: Vec<Column<Advice>>,

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

        let mut q_enable = Vec::new();
        for i_ in 0..5 {
            q_enable.push(meta.selector());
        }
        let mut q_sort = Vec::new();
        for i_ in 0..8 {
            q_sort.push(meta.selector());
        }
        let mut q_perm = Vec::new();
        for i_ in 0..6 {
            q_perm.push(meta.complex_selector());
        }
        let mut q_accu = Vec::new();
        for i_ in 0..2 {
            q_accu.push(meta.selector());
        }

        let mut customer = Vec::new();
        let mut orders = Vec::new();
        let mut lineitem = Vec::new();
        let mut new_lineitem = Vec::new();

        for _ in 0..2 {
            customer.push(meta.advice_column());
            lineitem.push(meta.advice_column());
        }
        for _ in 0..3 {
            new_lineitem.push(meta.advice_column());
        }
        for _ in 0..4 {
            orders.push(meta.advice_column());
        }

        let mut join_group = Vec::new();
        let mut disjoin_group = Vec::new();

        for l in [3, 4, 7, 2] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            join_group.push(col.clone());
        }
        for l in [3, 4, 7, 2] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            disjoin_group.push(col.clone());
        }

        let mut perm_helper = Vec::new();
        for l in [3, 4, 2] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            perm_helper.push(col);
        }

        let mut equal_check = Vec::new();
        for _ in 0..2 {
            equal_check.push(meta.advice_column());
        }

        let mut deduplicate = Vec::new();
        let mut dedup_sort = Vec::new();

        for _ in 0..2 {
            dedup_sort.push(meta.advice_column());
        }
        for _ in 0..2 {
            deduplicate.push(meta.advice_column());
        }

        let mut groupby = Vec::new();
        for _ in 0..6 {
            groupby.push(meta.advice_column());
        }
        let mut sum_quantity = Vec::new();
        for _ in 0..2 {
            sum_quantity.push(meta.advice_column());
        }

        let mut orderby = Vec::new();

        for _ in 0..6 {
            orderby.push(meta.advice_column());
        }

        // permutation check for disjoin and join
        PermAnyChip::configure(
            meta,
            q_perm[0],
            q_perm[3],
            new_lineitem.clone(),
            perm_helper[0].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[1],
            q_perm[4],
            orders.clone(),
            perm_helper[1].clone(),
        );
        PermAnyChip::configure(
            meta,
            q_perm[2],
            q_perm[5],
            customer.clone(),
            perm_helper[2].clone(),
        );

        // dedup check
        let lookup_configs = [
            (0, 0, 0), // (disjoin_group index, column index)
            (1, 2, 5),
        ];

        for (i, j, k) in lookup_configs.iter() {
            meta.lookup_any("dedup check", |meta| {
                let input = meta.query_advice(disjoin_group[*j][*k], Rotation::cur());
                let table = meta.query_advice(deduplicate[*i], Rotation::cur());
                vec![(input, table)]
            });
        }

        let mut lt_compare_condition = Vec::new();
        // join sort check
        for i in 0..2 {
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

        let mut compare_condition = Vec::new();
        // group by
        let config = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[2]),
            |meta| {
                vec![
                    meta.query_advice(groupby[0], Rotation::prev()),
                    meta.query_advice(groupby[1], Rotation::prev()),
                    meta.query_advice(groupby[2], Rotation::prev()),
                    meta.query_advice(groupby[3], Rotation::prev()),
                    meta.query_advice(groupby[4], Rotation::prev()),
                ]
            },
            |meta| {
                vec![
                    meta.query_advice(groupby[0], Rotation::cur()),
                    meta.query_advice(groupby[1], Rotation::cur()),
                    meta.query_advice(groupby[2], Rotation::cur()),
                    meta.query_advice(groupby[3], Rotation::cur()),
                    meta.query_advice(groupby[4], Rotation::cur()),
                ]
            },
        );
        compare_condition.push(config.clone());
        meta.create_gate("t[i-1]<=t[i]'", |meta| {
            let q_enable = meta.query_selector(q_sort[2]);
            vec![q_enable * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
        });

        // sum gate: the first sum(l_quantity)
        meta.create_gate("accumulate constraint 1", |meta| {
            let q_accu = meta.query_selector(q_accu[0]);
            let prev_revenue = meta.query_advice(sum_quantity[0].clone(), Rotation::cur());

            let quantity = meta.query_advice(lineitem[1], Rotation::cur());
            let sum_revenue = meta.query_advice(sum_quantity[0], Rotation::next());
            let check = meta.query_advice(equal_check[0], Rotation::cur());

            vec![q_accu.clone() * (check.clone() * prev_revenue + quantity - sum_revenue)]
        });

        // sum gate: the second sum(l_quantity)
        meta.create_gate("accumulate constraint 2", |meta| {
            let q_accu = meta.query_selector(q_accu[1]);
            let prev_revenue = meta.query_advice(sum_quantity[1].clone(), Rotation::cur());

            let quantity = meta.query_advice(groupby[5], Rotation::cur());
            let sum_revenue = meta.query_advice(sum_quantity[1], Rotation::next());
            let check = meta.query_advice(equal_check[1], Rotation::cur());

            vec![q_accu.clone() * (check.clone() * prev_revenue + quantity - sum_revenue)]
        });

        // orderby
        // o_totalprice desc,
        let config1 = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[3]), // q_sort[1] should start from index 1
            |meta| vec![meta.query_advice(orderby[4], Rotation::cur())], // revenue
            |meta| vec![meta.query_advice(orderby[4], Rotation::prev())],
        );
        compare_condition.push(config1.clone());

        // o_orderdate;
        let config2 = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[3]), // q_sort[1] should start from index 1
            |meta| vec![meta.query_advice(orderby[3], Rotation::prev())], // revenue
            |meta| vec![meta.query_advice(orderby[3], Rotation::cur())],
        );
        compare_condition.push(config2.clone());

        meta.create_gate("verifies orderby scenarios", |meta| {
            let q_sort = meta.query_selector(q_sort[3]);
            let totalprice1 = meta.query_advice(orderby[0], Rotation::prev());
            let totalprice2 = meta.query_advice(orderby[0], Rotation::cur());

            vec![
                q_sort.clone() *
                (config1.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
                        * ((Expression::Constant(F::ONE) - (totalprice1 - totalprice2)) * config2.is_lt(meta, None)
                            - Expression::Constant(F::ONE)),
            ]
        });

        TestCircuitConfig {
            q_enable,
            q_sort,
            q_accu,
            q_perm,
            lineitem,
            new_lineitem,
            orders,
            customer,
            dedup_sort,
            deduplicate,
            disjoin_group,
            join_group,
            equal_check,
            // condition,
            perm_helper,
            sum_quantity,
            compare_condition,
            lt_compare_condition,

            groupby,
            orderby,

            instance,
            instance_test,
        }
    }

    pub fn assign(
        &self,
        layouter: &mut impl Layouter<F>,
        customer: Vec<Vec<u64>>,
        orders: Vec<Vec<u64>>,
        lineitem: Vec<Vec<u64>>,
        // condition: u64,
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
        // select l_orderkey from lineitem group by l_orderkey having sum(l_quantity) > :1
        let mut l_combined = lineitem.clone();
        l_combined.sort_by_key(|v| v[0]);
        let mut orderkey_groupby_check: Vec<u64> = Vec::new();

        if l_combined.len() > 0 {
            orderkey_groupby_check.push(0); // add the the first one must be 0
        }

        for row in 1..l_combined.len() {
            if l_combined[row][0] == l_combined[row - 1][0] {
                orderkey_groupby_check.push(1);
            } else {
                orderkey_groupby_check.push(0);
            }
        }
        // sum
        let orderkey_size = l_combined.len() + 1;
        let mut sum_quantity: Vec<u64> = vec![0; orderkey_size];
        for i in 1..orderkey_size {
            sum_quantity[i] =
                sum_quantity[i - 1] * orderkey_groupby_check[i - 1] + l_combined[i - 1][1];
        }

        // add a new colum sum(l_quantity) to l_combined
        if l_combined.len() == 1 {
            l_combined[0].push(sum_quantity[1]);
        } else {
            for i in 1..l_combined.len() {
                if orderkey_groupby_check[i] == 0 {
                    l_combined[i - 1].push(sum_quantity[i]);
                }
                if i == l_combined.len() - 1 {
                    if orderkey_groupby_check[i] == 0 {
                        l_combined[i].push(sum_quantity[i + 1]);
                    }
                }
            }
        }

        let mut combined = Vec::new();
        combined.push(l_combined.clone()); // its length is 2
        combined.push(orders.clone()); // 4
        combined.push(customer.clone()); // 2
                                         //     l_orderkey, l_quantity (sum_l_quantity was created as a new column)
                                         //     o_orderdate, o_orderkey, o_custkey, o_totalprice
                                         //     c_name, c_custkey,
                                         //  l_orderkey = o_orderkey
                                         //  and c_custkey = o_custkey

        // lineitem + orders + customer

        let mut join_value: Vec<Vec<_>> = vec![vec![]; 4];
        let mut disjoin_value: Vec<Vec<_>> = vec![vec![]; 4];

        let join_index = [(0, 1), (5, 1)];
        let mut temp_join = combined[0].clone();
        for i in 1..3 {
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
        let mut cartesian_product = temp_join.clone();

        let index1 = [0, 1, 5, 1];
        let indices = [0, 1, 2, 3];
        // dedup
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

        // group by
        // c_name,
        // c_custkey,
        // o_orderkey,
        // o_orderdate,
        // o_totalprice
        // cartesian_product.sort_by_key(|v| v[7] + v[8] + v[4] + v[3] + v[6]);
        let mut join1: Vec<Vec<F>> = cartesian_product
            .iter()
            .map(|v| {
                let mut new_vec = Vec::new();
                if v.len() >= 1 {
                    new_vec.push(F::from(v[7])); // c_name,
                    new_vec.push(F::from(v[8])); // c_custkey,
                    new_vec.push(F::from(v[4])); // o_orderkey,
                    new_vec.push(F::from(v[3])); // o_orderdate,
                    new_vec.push(F::from(v[6])); // o_totalprice
                    new_vec.push(F::from(v[2])); // sum(l_quantity)
                }
                new_vec
            })
            .collect();
        // group by
        join1.sort_by_key(|v| v[0] + v[1] + v[2] + v[3] + v[4]);

        // the second accumulation
        let mut equal_check: Vec<F> = Vec::new();

        if join1.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
        }

        for row in 1..join1.len() {
            // self.config.q_sort[6].enable(&mut region, row)?;
            if join1[row][0] == join1[row - 1][0]
                && join1[row][1] == join1[row - 1][1]
                && join1[row][2] == join1[row - 1][2]
                && join1[row][3] == join1[row - 1][3]
                && join1[row][4] == join1[row - 1][4]
            {
                equal_check.push(F::from(1));
            } else {
                equal_check.push(F::from(0));
            }
        }
        // the second sum
        let n = join1.len() + 1;
        let mut sum_profit: Vec<F> = vec![F::from(0); n];
        for i in 1..n {
            sum_profit[i] = sum_profit[i - 1] * equal_check[i - 1]  // sum(l_extendedprice * (1 - l_discount)) as revenue,
                + join1[i-1][5];
        }

        // order by
        // o_totalprice desc,
        // o_orderdate;
        let mut grouped_data: Vec<Vec<u64>> = Vec::new();
        for row in &cartesian_product {
            // Check if the group (a1 value) already exists
            match grouped_data.iter_mut().find(|g| {
                g[0] == row[7]
                    && g[1] == row[8]
                    && g[2] == row[4]
                    && g[3] == row[3]
                    && g[4] == row[6]
            }) {
                Some(group) => {
                    group[5] += row[2]; // Add to the existing sum, row[2] is the new column, we should not use row[1]
                }
                None => {
                    grouped_data.push(vec![row[7], row[8], row[4], row[3], row[6], row[2]]);
                    // Create a new group
                }
            }
        }

        grouped_data.sort_by(|a, b| match b[4].cmp(&a[4]) {
            Ordering::Equal => a[3].cmp(&b[3]),
            other => other,
        });

        layouter.assign_region(
            || "witness",
            |mut region| {
                //assign input values
                for i in 0..lineitem.len() {
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[j],
                            i,
                            || Value::known(F::from(lineitem[i][j])),
                        )?;
                    }
                }
                for i in 0..l_combined.len() {
                    self.config.q_perm[0].enable(&mut region, i)?;
                    for j in 0..l_combined[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.new_lineitem[j],
                            i,
                            || Value::known(F::from(lineitem[i][j])),
                        )?;
                    }
                }

                for i in 0..orders.len() {
                    self.config.q_perm[1].enable(&mut region, i)?;
                    for j in 0..orders[0].len() {
                        region.assign_advice(
                            || "o",
                            self.config.orders[j],
                            i,
                            || Value::known(F::from(orders[i][j])),
                        )?;
                    }
                }
                for i in 0..customer.len() {
                    self.config.q_perm[2].enable(&mut region, i)?;
                    for j in 0..customer[0].len() {
                        region.assign_advice(
                            || "n",
                            self.config.customer[j],
                            i,
                            || Value::known(F::from(customer[i][j])),
                        )?;
                    }
                }

                // assign join and disjoin values
                for k in 0..join_value.len() {
                    let join_config = &self.config.join_group[k];
                    for i in 0..join_value[k].len() {
                        for j in 0..join_value[k][0].len() {
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

                // dedup
                for i in [0, 2] {
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

                // assign perm_helper to merge join_value and disjoin_value for permutation
                for (k, p) in [(0, 0), (1, 1), (2, 3)] {
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
                for (k, i) in [(0, 0), (1, 1), (2, 3)] {
                    for j in 0..join_value[i].len() + disjoin_value[i].len() {
                        self.config.q_perm[k + 3].enable(&mut region, j)?;
                    }
                }

                // dedup
                for i in [0, 2] {
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

                for i in 0..orderkey_groupby_check.len() {
                    self.config.q_accu[0].enable(&mut region, i)?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.equal_check[0],
                        i,
                        || Value::known(F::from(orderkey_groupby_check[i])),
                    )?;
                }
                for i in 0..equal_check.len() {
                    self.config.q_accu[1].enable(&mut region, i)?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.equal_check[1],
                        i,
                        || Value::known(equal_check[i]),
                    )?;
                }
                // groupby assign

                for i in 0..join1.len() {
                    if i > 0 {
                        self.config.q_sort[2].enable(&mut region, i)?;
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

                // sum_quantity assign
                for i in 0..orderkey_size {
                    region.assign_advice(
                        || "sum_profit",
                        self.config.sum_quantity[0],
                        i,
                        || Value::known(F::from(sum_quantity[i])),
                    )?;
                }
                // second sum assign
                for i in 0..n {
                    region.assign_advice(
                        || "sum_profit",
                        self.config.sum_quantity[1],
                        i,
                        || Value::known(sum_profit[i]),
                    )?;
                }

                for i in 0..grouped_data.len() {
                    for j in 0..grouped_data[0].len() {
                        region.assign_advice(
                            || "orderby",
                            self.config.orderby[j],
                            i,
                            || Value::known(F::from(grouped_data[i][j])),
                        )?;
                    }
                }
                //assign chips
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
                            &[
                                join1[i - 1][0],
                                join1[i - 1][1],
                                join1[i - 1][2],
                                join1[i - 1][3],
                                join1[i - 1][4],
                            ],
                            &[
                                join1[i][0],
                                join1[i][1],
                                join1[i][2],
                                join1[i][3],
                                join1[i][4],
                            ],
                        )?;
                    }
                }

                for i in 0..grouped_data.len() {
                    if i > 0 {
                        self.config.q_sort[3].enable(&mut region, i)?;
                        compare_chip[1].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[F::from(grouped_data[i][4])],
                            &[F::from(grouped_data[i - 1][4])],
                        )?;
                        compare_chip[2].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[F::from(grouped_data[i - 1][3])],
                            &[F::from(grouped_data[i][3])],
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

struct MyCircuit<F> {
    customer: Vec<Vec<u64>>,
    orders: Vec<Vec<u64>>,
    lineitem: Vec<Vec<u64>>,

    // pub condition: u64,
    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            customer: Vec::new(),
            orders: Vec::new(),
            lineitem: Vec::new(),
            // condition: Default::default(),
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
            // self.condition.clone(),
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
    // use halo2_proofs::{dev::MockProver};
    use halo2_proofs::dev::MockProver;
    use std::marker::PhantomData;

    use crate::circuits::utils::full_prover;

    use halo2curves::pasta::{pallas, vesta, EqAffine, Fp};

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

        let customer_file_path = "/home/cc/halo2-TPCH/src/data/customer.tbl";
        let orders_file_path = "/home/cc/halo2-TPCH/src/data/orders.tbl";
        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem_240K.tbl";

        let mut customer: Vec<Vec<u64>> = Vec::new();
        let mut orders: Vec<Vec<u64>> = Vec::new();
        let mut lineitem: Vec<Vec<u64>> = Vec::new();

        if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
            // Convert the Vec<Region> to a 2D vector
            customer = records
                .iter()
                .map(|record| vec![string_to_u64(&record.c_name), record.c_custkey])
                .collect();
        }
        if let Ok(records) = data_processing::orders_read_records_from_file(orders_file_path) {
            // Convert the Vec<Region> to a 2D vector
            orders = records
                .iter()
                .map(|record| {
                    vec![
                        // Fp::from(string_to_u64(&record.o_orderdate)),
                        date_to_timestamp(&record.o_orderdate),
                        record.o_orderkey,
                        record.o_custkey,
                        scale_by_1000(record.o_totalprice),
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
            // Convert the Vec<Region> to a 2D vector
            lineitem = records
                .iter()
                .map(|record| vec![record.l_orderkey, record.l_quantity])
                .collect();
        }

        // let customer: Vec<Vec<u64>> = customer.iter().take(1000).cloned().collect();
        // let orders: Vec<Vec<u64>> = orders.iter().take(1000).cloned().collect();
        // let lineitem: Vec<Vec<u64>> = lineitem.iter().take(1000).cloned().collect();

        let circuit = MyCircuit::<Fp> {
            customer,
            orders,
            lineitem,

            _marker: PhantomData,
        };

        let public_input = vec![Fp::from(1)];

        // let test = true;
        let test = false;

        if test {
            let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
            prover.assert_satisfied();
        } else {
            let proof_path = "/home/cc/halo2-TPCH/src/sql/proof_q18_240K";
            generate_and_verify_proof(k, circuit, &public_input, proof_path);
        }

        // process::exit(0);
    }
}
