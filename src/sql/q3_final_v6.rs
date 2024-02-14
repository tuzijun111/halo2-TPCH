// use eth_types::Field;

use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};

use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use crate::chips::less_than_vector::{LtVecChip, LtVecConfig, LtVecInstruction};
use crate::chips::lessthan_or_equal_vector::{LtEqVecChip, LtEqVecConfig, LtEqVecInstruction};
// use crate::chips::less_than::{LtChip, LtConfig, LtInstruction};
// use crate::chips::lessthan_or_equal_generic::{
//     LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
// };
use crate::chips::permutation_any::{PermAnyChip, PermAnyConfig};

use std::thread::sleep;
use std::{default, marker::PhantomData};

// use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};

use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem;
use std::process;
use std::time::Instant;

const NUM_BYTES: usize = 5;

pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

// #[derive(Default)]
// We should use the selector to skip the row which does not satisfy shipdate values

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field + Ord> {
    q_enable: Vec<Selector>,
    q_join: Vec<Selector>,
    // q_perm: Vec<Selector>,
    q_sort: Vec<Selector>,
    q_accu: Selector,
    customer: Vec<Column<Advice>>, // c_mkt, c_custkey
    orders: Vec<Column<Advice>>,   // o_orderdate, o_shippri, o_custkey, o_orderkey
    lineitem: Vec<Column<Advice>>, // l_orderkey, l_extened, l_dis, l_ship

    join_group: Vec<Vec<Column<Advice>>>,
    disjoin_group: Vec<Vec<Column<Advice>>>,
    perm_helper: Vec<Vec<Column<Advice>>>, // used for aggregate two groups of columns into one for permutation check
    check: Vec<Column<Advice>>,

    deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey
    deduplicate_helper: Vec<Column<Advice>>, // merging adjacent two groups of columns into one for permutation check
    dedup_sort: Vec<Column<Advice>>,

    condition: Vec<Column<Advice>>, // conditional value for l, c and o

    join1: Vec<Column<Advice>>, // for c_custkey = o_custkey; and especially for the part of o table
    join2: Vec<Column<Advice>>,
    groupby: Vec<Column<Advice>>,
    orderby: Vec<Column<Advice>>,
    // // cartesian: Vec<Column<Advice>>,
    equal_check: Column<Advice>, // check if sorted_revenue[i-1] = sorted_revenue[i]
    revenue: Column<Advice>,
    // lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,
    lt_compare_condition: Vec<LtVecConfig<F, NUM_BYTES>>,
    equal_condition: Vec<IsZeroConfig<F>>,
    compare_condition: Vec<LtEqVecConfig<F, NUM_BYTES>>,
    // perm: Vec<PermAnyConfig>,
    instance: Column<Instance>,
    instance_test: Column<Advice>,
    // instance_test1: Column<Advice>,
    // instance_test2: Column<Advice>,
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
        // let instance_test1 = meta.advice_column();
        // meta.enable_equality(instance_test1);
        // let instance_test2 = meta.advice_column();
        // meta.enable_equality(instance_test2);

        let mut q_enable = Vec::new();
        for i_ in 0..3 {
            q_enable.push(meta.selector());
        }

        let mut q_sort = Vec::new();
        for i_ in 0..4 {
            q_sort.push(meta.selector());
        }

        let mut q_join = Vec::new();
        for i in 0..8 {
            if i < 2 {
                q_join.push(meta.selector());
            } else {
                q_join.push(meta.complex_selector());
            }
        }
        for i_ in 0..5 {
            q_sort.push(meta.selector());
        }

        let q_accu = meta.selector();

        let mut customer = Vec::new();
        let mut orders = Vec::new();
        let mut lineitem = Vec::new();

        for _ in 0..2 {
            customer.push(meta.advice_column());
        }
        for _ in 0..4 {
            orders.push(meta.advice_column());
            lineitem.push(meta.advice_column());
        }
        let mut condition = Vec::new();
        for _ in 0..3 {
            // use meta.equality() for copying check, so we do not need three conditions here
            condition.push(meta.advice_column());
        }

        let mut deduplicate = Vec::new();
        let mut deduplicate_helper = Vec::new();
        let mut dedup_sort = Vec::new();

        for _ in 0..2 {
            dedup_sort.push(meta.advice_column());
            deduplicate_helper.push(meta.advice_column());
        }
        for _ in 0..4 {
            deduplicate.push(meta.advice_column());
        }

        let mut join_group = Vec::new();
        let mut disjoin_group = Vec::new();

        for l in [4, 2, 4, 6] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            join_group.push(col);
        }
        for l in [4, 2, 4, 6] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            disjoin_group.push(col);
        }

        let mut perm_helper = Vec::new();
        for l in [4, 2, 4] {
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            perm_helper.push(col);
        }

        let mut join1 = Vec::new(); // for c table
        for _ in 0..2 {
            join1.push(meta.advice_column());
        }
        let mut join2 = Vec::new(); // for c join o
        for _ in 0..6 {
            join2.push(meta.advice_column());
        }

        let mut groupby = Vec::new();
        let mut orderby = Vec::new();
        for _ in 0..5 {
            groupby.push(meta.advice_column());
        }
        for _ in 0..4 {
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
        // // equality
        // for i in 0..2 {
        //     meta.enable_equality(deduplicate_helper[i]);
        //     meta.enable_equality(dedup_sort[i]);
        // }
        // for i in 0..4 {
        //     // meta.enable_equality(deduplicate[i]);
        //     meta.enable_equality(orderby[i]);
        // }

        // the following two parts are also expensive and should be optimized
        // for vec in join_group.iter().chain(disjoin_group.iter()) {
        //     for &element in vec.iter() {
        //         meta.enable_equality(element);
        //     }
        // }

        // for vec in perm_helper.iter() {
        //     for &element in vec.iter() {
        //         meta.enable_equality(element);
        //     }
        // }

        // meta.enable_equality(equal_check);
        // meta.enable_equality(revenue);

        // // c_mktsegment = ':1'
        let mut equal_condition = Vec::new();
        let config = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[0]), // this is the q_enable
            |meta| {
                meta.query_advice(customer[0], Rotation::cur())
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

        let mut lt_compare_condition = Vec::new();
        // o_orderdate < date ':2'
        let config = LtVecChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[1]),
            |meta| meta.query_advice(orders[0], Rotation::cur()),
            |meta| meta.query_advice(condition[1], Rotation::cur()), // we put the left and right value at the first two positions of value_l
        );

        meta.create_gate(
            "verifies o_orderdate < date ':2'", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[1]);
                let check = meta.query_advice(check[1], Rotation::cur());
                // vec![q_enable * (config.is_lt(meta, None) - check)]
                vec![q_enable.clone() * (config.is_lt(meta, None) - check)]
            },
        );

        lt_compare_condition.push(config);

        // l_shipdate > date ':2'
        let config: LtVecConfig<F, NUM_BYTES> = LtVecChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[2]),
            |meta| meta.query_advice(condition[2], Rotation::cur()),
            |meta| meta.query_advice(lineitem[3], Rotation::cur()), // we put the left and right value at the first two positions of value_l
        );

        meta.create_gate(
            "verifies l_shipdate > date ':2'", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[2]);
                let check = meta.query_advice(check[2], Rotation::cur());
                vec![q_enable * (config.is_lt(meta, None) - check)]
            },
        );
        lt_compare_condition.push(config);

        // // disjoin sort check
        // // dedup check
        // let lookup_configs = [
        //     (0, 2), // (disjoin_group index, column index)
        //     (1, 1),
        //     (2, 0),
        //     (3, 3),
        // ];

        // for (disjoin_index, column_index) in lookup_configs.iter() {
        //     meta.lookup_any("dedup check", |meta| {
        //         let input = meta.query_advice(
        //             disjoin_group[*disjoin_index][*column_index],
        //             Rotation::cur(),
        //         );
        //         let table = meta.query_advice(deduplicate[*disjoin_index], Rotation::cur());
        //         vec![(input, table)]
        //     });
        // }

        // // two permutation check: join and disjoin

        // PermAnyChip::configure(
        //     meta,
        //     q_join[2],
        //     q_join[5],
        //     orders.clone(),
        //     perm_helper[0].clone(),
        // );
        // PermAnyChip::configure(
        //     meta,
        //     q_join[3],
        //     q_join[6],
        //     customer.clone(),
        //     perm_helper[1].clone(),
        // );

        // PermAnyChip::configure(
        //     meta,
        //     q_join[4],
        //     q_join[7],
        //     lineitem.clone(),
        //     perm_helper[2].clone(),
        // );

        // // two dedup permutation check: deduplicate and dedup_sort
        // meta.lookup_any("dedup permtuation check", |meta| {
        //     let input = meta.query_advice(deduplicate_helper[0], Rotation::cur());
        //     let table = meta.query_advice(dedup_sort[0], Rotation::cur());
        //     vec![(input, table)]
        // });
        // meta.lookup_any("dedup permtuation check", |meta| {
        //     let input = meta.query_advice(deduplicate_helper[1], Rotation::cur());
        //     let table = meta.query_advice(dedup_sort[1], Rotation::cur());
        //     vec![(input, table)]
        // });

        // // join1 check
        // meta.create_gate(
        //     "verify join1 values match'", // just use less_than for testing here
        //     |meta| {
        //         let q = meta.query_selector(q_join[0]);
        //         let p1 = meta.query_advice(join_group[0][2], Rotation::cur());
        //         let p2 = meta.query_advice(join1[1], Rotation::cur());
        //         vec![q * (p1 - p2)]
        //     },
        // );

        // // all values of join1 are in join_group[1]
        // meta.lookup_any("check join1", |meta| {
        //     let inputs: Vec<_> = join_group[1] // join1
        //         .iter()
        //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
        //         .collect();

        //     let tables: Vec<_> = join1 //join_group[1]
        //         .iter()
        //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
        //         .collect();

        //     let constraints: Vec<_> = inputs
        //         .iter()
        //         .zip(tables.iter())
        //         .map(|(input, table)| (input.clone(), table.clone()))
        //         .collect();

        //     constraints
        // });

        // // join2 check
        // meta.create_gate(
        //     "verify join2 values match'", // just use less_than for testing here
        //     |meta| {
        //         let q = meta.query_selector(q_join[1]);
        //         let p1 = meta.query_advice(join_group[2][0], Rotation::cur());
        //         let p2 = meta.query_advice(join2[3], Rotation::cur());
        //         vec![q * (p1 - p2)]
        //     },
        // );
        // meta.lookup_any("check join2", |meta| {
        //     let inputs: Vec<_> = join2
        //         .iter()
        //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
        //         .collect();

        //     let tables: Vec<_> = join_group[3]
        //         .iter()
        //         .map(|&idx| meta.query_advice(idx, Rotation::cur()))
        //         .collect();

        //     let constraints: Vec<_> = inputs
        //         .iter()
        //         .zip(tables.iter())
        //         .map(|(input, table)| (input.clone(), table.clone()))
        //         .collect();

        //     constraints
        // });

        // two dedup sort check
        for i in 0..2 {
            let config = LtVecChip::configure(
                meta,
                |meta| meta.query_selector(q_sort[i]),
                |meta| meta.query_advice(dedup_sort[i], Rotation::cur()),
                |meta| meta.query_advice(dedup_sort[i], Rotation::next()), // we put the left and right value at the first two positions of value_l
            );
            lt_compare_condition.push(config.clone());
            meta.create_gate("t[i-1]<t[i]'", |meta| {
                let q_enable = meta.query_selector(q_sort[i]);
                vec![q_enable * (config.is_lt(meta, None) - Expression::Constant(F::ONE))]
            });
        }

        // // join check

        // // group by
        let mut compare_condition = Vec::new();
        // let config = LtEqVecChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_sort[2]),
        //     |meta| {
        //         vec![
        //             meta.query_advice(groupby[0], Rotation::cur()),
        //             meta.query_advice(groupby[1], Rotation::cur()),
        //             meta.query_advice(groupby[2], Rotation::cur()),
        //         ]
        //     },
        //     |meta| {
        //         vec![
        //             meta.query_advice(groupby[0], Rotation::next()),
        //             meta.query_advice(groupby[1], Rotation::next()),
        //             meta.query_advice(groupby[2], Rotation::next()),
        //         ]
        //     },
        // );
        // compare_condition.push(config);

        // // sum gate: sum(l_extendedprice * (1 - l_discount)) as revenue, note that revenue column starts by zero and its length is 1 more than others
        // meta.create_gate("accumulate constraint", |meta| {
        //     let q_accu = meta.query_selector(q_accu);
        //     let prev_revenue = meta.query_advice(revenue.clone(), Rotation::cur());
        //     let extendedprice = meta.query_advice(groupby[3], Rotation::cur());
        //     let discount = meta.query_advice(groupby[4], Rotation::cur());
        //     let sum_revenue = meta.query_advice(revenue, Rotation::next());
        //     let check = meta.query_advice(equal_check, Rotation::cur());

        //     vec![
        //         q_accu.clone()
        //             * (check.clone() * prev_revenue
        //                 + extendedprice.clone()
        //                     * (Expression::Constant(F::from(1000)) - discount.clone())
        //                 - sum_revenue),
        //     ]
        // });

        // // orderby

        // // (1) revenue[i-1] > revenue[i]
        // let config = LtEqVecChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_sort[3]), // q_sort[1] should start from index 1
        //     |meta| vec![meta.query_advice(orderby[3], Rotation::next())], // revenue
        //     |meta| vec![meta.query_advice(orderby[3], Rotation::cur())],
        // );
        // compare_condition.push(config.clone());

        // // revenue[i-1] = revenue[i]
        // let equal_v1 = IsZeroChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_sort[3]),
        //     |meta| {
        //         meta.query_advice(orderby[3], Rotation::cur())
        //             - meta.query_advice(orderby[3], Rotation::next())
        //     },
        //     is_zero_advice_column[1],
        // );
        // equal_condition.push(equal_v1.clone());

        // // o_orderdate[i-1] <= o_orderdate[i]
        // let config_lt = LtEqVecChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_sort[3]), // q_sort[2] should start from index 1
        //     |meta| vec![meta.query_advice(orderby[0], Rotation::cur())],
        //     |meta| vec![meta.query_advice(orderby[0], Rotation::next())],
        // );
        // compare_condition.push(config_lt.clone());

        // meta.create_gate("verifies orderby scenarios", |meta| {
        //     let q_sort = meta.query_selector(q_sort[3]);

        //     vec![
        //         q_sort.clone() *
        //         (config.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
        //                 * (equal_v1.expr() * config_lt.is_lt(meta, None)
        //                     - Expression::Constant(F::ONE)),
        //     ]
        // });

        TestCircuitConfig {
            q_enable,
            q_join,
            // q_dedup,
            // q_perm,
            q_sort,
            q_accu,
            customer,
            orders,
            lineitem,

            join_group,
            disjoin_group,
            check,
            deduplicate,
            deduplicate_helper,
            dedup_sort,

            condition,
            join1,
            join2,
            groupby,
            orderby,
            equal_check,

            revenue,
            lt_compare_condition,
            equal_condition,
            compare_condition,
            perm_helper,
            instance,
            instance_test,
            // instance_test1,
            // instance_test2,
        }
    }

    pub fn assign(
        &self,
        // layouter: &mut impl Layouter<F>,
        layouter: &mut impl Layouter<F>,

        customer: Vec<Vec<u64>>,
        orders: Vec<Vec<u64>>,
        lineitem: Vec<Vec<u64>>,

        condition: [u64; 2],
    ) -> Result<AssignedCell<F, F>, Error> {
        let mut equal_chip = Vec::new();
        let mut compare_chip = Vec::new();
        let mut lt_compare_chip = Vec::new();
        // // let mut perm_chip: Vec<PermAnyChip<_>> = Vec::new();

        for i in 0..self.config.equal_condition.len() {
            let chip = IsZeroChip::construct(self.config.equal_condition[i].clone());
            equal_chip.push(chip);
        }
        for i in 0..self.config.compare_condition.len() {
            let chip = LtEqVecChip::construct(self.config.compare_condition[i].clone());
            chip.load(layouter)?;
            compare_chip.push(chip);
        }

        for i in 0..self.config.lt_compare_condition.len() {
            let chip = LtVecChip::construct(self.config.lt_compare_condition[i].clone());
            chip.load(layouter)?; // lt_compare_chip[0].load(layouter)?; // can we just load u8 range once?
            lt_compare_chip.push(chip);
        }

        // println!("lt compare {:?}", self.config.lt_compare_condition.len());

        // witness local computation, note that the lcoal computation should not be put into layouter.assign_region()
        let start = Instant::now();

        let mut l_check = Vec::new(); // t/f
        let mut o_check = Vec::new(); // t/f
        let mut c_check = Vec::new(); // 0, 1

        for i in 0..customer.len() {
            if customer[i][0] == condition[0] {
                c_check.push(1);
            } else {
                c_check.push(0);
            }
        }
        for i in 0..orders.len() {
            if orders[i][0] < condition[1] {
                o_check.push(true);
            } else {
                o_check.push(false);
            }
        }
        for i in 0..lineitem.len() {
            if lineitem[i][3] > condition[1] {
                l_check.push(true);
            } else {
                l_check.push(false);
            }
        }

        // compute values related to the join operation locally
        // store the attribtues of the tables that will be used in the SQL query in tuples
        // let mut c_combined = customer.clone();
        // let mut o_combined = orders.clone();
        // let mut l_combined = lineitem.clone();

        let duration_block = start.elapsed();
        println!("Time elapsed for block: {:?}", duration_block);

        let c_combined: Vec<Vec<_>> = customer
            .clone()
            .into_iter()
            .filter(|row| row[0] == condition[0]) // r_name = ':3'
            .collect();

        let o_combined: Vec<Vec<_>> = orders
            .clone()
            .into_iter()
            .filter(|row| row[0] < condition[1]) // r_name = ':3'
            .collect();

        let l_combined: Vec<Vec<_>> = lineitem
            .clone()
            .into_iter()
            .filter(|row| row[3] > condition[1]) // r_name = ':3'
            .collect();

        // println!{"Orders: {:?}", o_combined.len()};
        // println!{"Customer: {:?}", c_combined.len()};

        let mut combined = Vec::new();
        combined.push(c_combined); // its length is 2
        combined.push(o_combined); // 4
        combined.push(l_combined); // 4

        let index = [
            (0, 1, 1, 2), //   c_custkey = o_custkey
            (1, 2, 3, 0), //   l_orderkey = o_orderkey
        ];

        // for i in 0..join_value.len(){
        //     println!{"Join Value: {:?}", join_value[i].len()};
        // }

        // compute final table by applying all joins
        let join_index = [
            (0, 1, 1, 2),     //   c_custkey = o_custkey
            (1, 2, 2 + 3, 0), //   l_orderkey = o_orderkey
        ];

        let mut join_value: Vec<Vec<_>> = vec![vec![]; 4];
        let mut disjoin_value: Vec<Vec<_>> = vec![vec![]; 4];

        // join_value [combined[1], combined[0], combined[2], join_1]
        // for val in combined[1].iter() {
        //     if let Some(_) = combined[0]
        //         .iter()
        //         .find(|v| v[index[0].2] == val[index[0].3])
        //     {
        //         join_value[0].push(val.clone()); // join values
        //     } else {
        //         disjoin_value[0].push(val); // disjoin values
        //     }
        // }
        // for val in combined[0].iter() {
        //     if let Some(_) = combined[1]
        //         .iter()
        //         .find(|v| v[index[0].3] == val[index[0].2])
        //     {
        //         join_value[1].push(val.clone()); // join values
        //     } else {
        //         disjoin_value[1].push(val); // disjoin values
        //     }
        // }
        let mut map = HashMap::new();

        // Populate the map with elements from the first vector, using the join key as the map key
        for val in &combined[0] {
            map.insert(val[index[0].2], val);
        }

        for val in &combined[1] {
            // Check if the element exists in the map (thus in combined[0])
            if map.contains_key(&val[index[0].3]) {
                join_value[0].push(val.clone()); // join values
            } else {
                disjoin_value[0].push(val); // disjoin values
            }
        }
        // Reset the map for the reverse operation
        map.clear();
        // Populate the map with elements from the second vector this time
        for val in &combined[1] {
            map.insert(val[index[0].3], val);
        }

        for val in &combined[0] {
            if map.contains_key(&val[index[0].2]) {
                join_value[1].push(val.clone()); // join values
            } else {
                disjoin_value[1].push(val); // disjoin values
            }
        }

        let mut cartesian_product = combined[1].to_vec();
        let mut to_add = Vec::new();

        for ab in cartesian_product.iter() {
            for c in combined[0].iter() {
                if ab[join_index[0].3] == c[join_index[0].2] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        cartesian_product = to_add;

        let mut map = HashMap::new();

        // Populate the map with elements from the first vector, using the join key as the map key
        for val in &cartesian_product {
            map.insert(val[3], val);
        }

        for val in &combined[2] {
            // Check if the element exists in the map (thus in combined[0])
            if map.contains_key(&val[0]) {
                join_value[2].push(val.clone()); // join values
            } else {
                disjoin_value[2].push(val); // disjoin values
            }
        }
        // Reset the map for the reverse operation
        map.clear();
        for val in &combined[2] {
            map.insert(val[0], val);
        }

        for val in &cartesian_product {
            if map.contains_key(&val[3]) {
                join_value[3].push(val.clone()); // join values
            } else {
                disjoin_value[3].push(val); // disjoin values
            }
        }

        // locally compute the second join
        let mut to_add = Vec::new();
        for ab in combined[2].iter() {
            for c in cartesian_product.iter() {
                if ab[join_index[1].3] == c[3] {
                    let mut joined = ab.to_vec();
                    joined.extend_from_slice(c);
                    to_add.push(joined);
                }
            }
        }
        let mut cartesian_product = to_add;

        let mut dis_c_custkey: Vec<u64> = disjoin_value[1].iter().map(|v| v[1]).collect();
        let mut dis_o_custkey: Vec<u64> = disjoin_value[0].iter().map(|v| v[2]).collect();
        let mut dis_o_orderkey: Vec<u64> = disjoin_value[3].iter().map(|v| v[3]).collect();
        let mut dis_l_orderkey: Vec<u64> = disjoin_value[2].iter().map(|v| v[0]).collect();

        // generate deduplicated columns for disjoin values
        // dis_c_custkey.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // dis_c_custkey.dedup();
        dis_o_custkey.sort(); // sort for using dedup()
        dis_o_custkey.dedup();
        // dis_o_orderkey.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // dis_o_orderkey.dedup();
        dis_l_orderkey.sort();
        dis_l_orderkey.dedup();

        // concatenate two vectors for sorting
        let mut new_dedup_1: Vec<u64> = dis_o_custkey
            .clone()
            .into_iter()
            .chain(dis_c_custkey.clone())
            .collect();
        let mut new_dedup_2: Vec<u64> = dis_l_orderkey
            .clone()
            .into_iter()
            .chain(dis_o_orderkey.clone())
            .collect();
        // sort them
        new_dedup_1.sort();
        new_dedup_2.sort();

        // group by
        // l_orderkey,
        // o_orderdate,
        // o_shippriority
        cartesian_product
            .sort_by_key(|element| element[0].clone() + element[7].clone() + element[5].clone());

        let mut equal_check: Vec<F> = Vec::new();
        if cartesian_product.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
        }

        for row in 1..cartesian_product.len() {
            if cartesian_product[row][0] == cartesian_product[row - 1][0]
                && cartesian_product[row][7] == cartesian_product[row - 1][7]
                && cartesian_product[row][5] == cartesian_product[row - 1][5]
            {
                equal_check.push(F::from(1));
            } else {
                equal_check.push(F::from(0));
            }
        }

        let n = cartesian_product.len() + 1;
        let mut revenue: Vec<F> = vec![F::from(0); n];
        for i in 1..n {
            revenue[i] = revenue[i - 1] * equal_check[i - 1]  // sum(l_extendedprice * (1 - l_discount)) as revenue,
                + F::from(cartesian_product[i - 1][1]) * F::from((1000 - cartesian_product[i - 1][2]));
            // cartesian_product[i - 1].push(revenue[i].clone()); // add this value to correct row of cartesian product
        }

        let duration_block = start.elapsed();
        println!("Time elapsed for block: {:?}", duration_block);

        // order by
        // revenue desc,
        // o_orderdate;
        let mut grouped_data: Vec<Vec<u64>> = Vec::new();
        for row in &cartesian_product {
            // Check if the group (a1 value) already exists
            match grouped_data
                .iter_mut()
                .find(|g| g[0] == row[0] && g[1] == row[7] && g[2] == row[5])
            {
                Some(group) => {
                    group[3] += row[1] * (1000 - row[2]); // Add to the existing sum
                }
                None => {
                    grouped_data.push(vec![row[0], row[7], row[5], row[1] * (1000 - row[2])]);
                    // Create a new group
                }
            }
        }
        // println!("cartesian {:?}", cartesian_product);
        // println!("grouped data {:?}", grouped_data);

        grouped_data.sort_by(|a, b| match b[3].cmp(&a[3]) {
            Ordering::Equal => a[1].cmp(&b[1]),
            other => other,
        });

        // local lessthan computation assignment
        let f_new_dedup_1: Vec<F> = new_dedup_1.clone().into_iter().map(F::from).collect();
        let n1 = f_new_dedup_1.len();
        let f_new_dedup_2: Vec<F> = new_dedup_2.clone().into_iter().map(F::from).collect();
        let n2 = f_new_dedup_2.len();

        // lessthan_or_equal

        let n3 = cartesian_product.len();
        let mut cartesian_left = Vec::new();
        let mut cartesian_right = Vec::new();
        for i in 0..n3 {
            if i != 0 {
                let mut new_vec: Vec<F> = Vec::new();
                new_vec.push(From::from(cartesian_product[i][0]));
                new_vec.push(From::from(cartesian_product[i][7]));
                new_vec.push(From::from(cartesian_product[i][5]));
                cartesian_right.push(new_vec);
            }
            if i != n3 - 1 {
                let mut new_vec: Vec<F> = Vec::new();
                new_vec.push(From::from(cartesian_product[i][0]));
                new_vec.push(From::from(cartesian_product[i][7]));
                new_vec.push(From::from(cartesian_product[i][5]));
                cartesian_left.push(new_vec);
            }
        }

        let n4 = grouped_data.len();
        let mut grouped_left_revenue: Vec<Vec<F>> = Vec::new();
        let mut grouped_right_revenue: Vec<Vec<F>> = Vec::new();
        let mut grouped_left_orderdate: Vec<Vec<F>> = Vec::new();
        let mut grouped_right_orderdate: Vec<Vec<F>> = Vec::new();

        for i in 0..n4 {
            if i != 0 {
                let mut new_vec1 = Vec::new();
                new_vec1.push(From::from(grouped_data[i][3].clone()));
                grouped_left_revenue.push(new_vec1);
                let mut new_vec2 = Vec::new();
                new_vec2.push(From::from(grouped_data[i][0].clone()));
                grouped_right_orderdate.push(new_vec2);
            }
            if i != n4 - 1 {
                let mut new_vec1 = Vec::new();
                new_vec1.push(From::from(grouped_data[i][3].clone()));
                grouped_right_revenue.push(new_vec1);
                let mut new_vec2 = Vec::new();
                new_vec2.push(From::from(grouped_data[i][0].clone()));
                grouped_left_orderdate.push(new_vec2);
            }
        }

        layouter.assign_region(
            || "witness",
            |mut region| {
                //assign input values
                // customer
                for i in 0..customer.len() {
                    self.config.q_enable[0].enable(&mut region, i)?;
                    if c_check[i] == 1 {
                        self.config.q_join[3].enable(&mut region, i)?; // used to select the valid rows for permutation
                    }
                    for j in 0..customer[0].len() {
                        region.assign_advice(
                            || "customer",
                            self.config.customer[j],
                            i,
                            || Value::known(F::from(customer[i][j])),
                        )?;
                    }

                    region.assign_advice(
                        || "check0",
                        self.config.check[0],
                        i,
                        || Value::known(F::from(c_check[i])),
                    )?;

                    region.assign_advice(
                        || "condition for customer",
                        self.config.condition[0],
                        i,
                        || Value::known(F::from(condition[0])),
                    )?;
                }
                // orders
                for i in 0..orders.len() {
                    self.config.q_enable[1].enable(&mut region, i)?;
                    // permutation check between orders and join_value[0]
                    if o_check[i] == true {
                        self.config.q_join[2].enable(&mut region, i)?; // used to select the valid rows for permutation
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
                        || Value::known(F::from(o_check[i] as u64)),
                    )?;

                    region.assign_advice(
                        || "condition1",
                        self.config.condition[1],
                        i,
                        || Value::known(F::from(condition[1])),
                    )?;
                }

                // lineitem
                for i in 0..lineitem.len() {
                    self.config.q_enable[2].enable(&mut region, i)?;
                    if l_check[i] == true {
                        self.config.q_join[4].enable(&mut region, i)?; // used to select the valid rows for permutation
                    }
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "lineitem",
                            self.config.lineitem[j],
                            i,
                            || Value::known(F::from(lineitem[i][j])),
                        )?;
                    }

                    region.assign_advice(
                        || "check2",
                        self.config.check[2],
                        i,
                        || Value::known(F::from(l_check[i] as u64)),
                    )?;

                    region.assign_advice(
                        || "condition2",
                        self.config.condition[2],
                        i,
                        || Value::known(F::from(condition[1])),
                    )?;
                }

                for i in 0..join_value[0].len() {
                    self.config.q_join[0].enable(&mut region, i)?; // join1
                    for j in 0..join_value[1].len() {
                        if join_value[0][i][2] == join_value[1][j][1] {
                            for k in 0..join_value[1][0].len() {
                                let value_to_assign = join_value[1][j][k];
                                // println!("b[k]--------{:?}", join_value[1][i][k]);
                                region.assign_advice(
                                    || "generate the first join",
                                    self.config.join1[k],
                                    i,
                                    || Value::known(F::from(value_to_assign)),
                                )?;
                            }
                            break;
                        }
                    }
                }

                //assign join2 before the second join locally
                for i in 0..join_value[2].len() {
                    self.config.q_join[1].enable(&mut region, i)?; // join2
                    for j in 0..join_value[3].len() {
                        if join_value[2][i][0] == join_value[3][j][3] {
                            for k in 0..join_value[3][0].len() {
                                let value_to_assign = join_value[3][j][k];
                                region.assign_advice(
                                    || "generate the second join",
                                    self.config.join2[k],
                                    i,
                                    || Value::known(F::from(value_to_assign)),
                                )?;
                            }
                            break;
                        }
                    }
                }

                // assign join and disjoin values
                for k in 0..join_value.len() {
                    let join_config = &self.config.join_group[k];
                    // Process join_value[k]
                    for i in 0..join_value[k].len() {
                        for j in 0..join_value[k][0].len() {
                            region.assign_advice(
                                || "join_config",
                                join_config[j],
                                i,
                                || Value::known(F::from(join_value[k][i][j].clone())),
                            )?;
                        }
                    }

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
                for k in 0..3 {
                    let join_config = &self.config.join_group[k];
                    let perm_config = &self.config.perm_helper[k];
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

                for i in 0..join_value[0].len() + disjoin_value[0].len() {
                    self.config.q_join[5].enable(&mut region, i)?;
                }

                for i in 0..join_value[1].len() + disjoin_value[1].len() {
                    self.config.q_join[6].enable(&mut region, i)?;
                }
                for i in 0..join_value[2].len() + disjoin_value[2].len() {
                    self.config.q_join[7].enable(&mut region, i)?;
                }

                // println!{"Join Value: {:?}", join_value[0].len()};
                // println!{"Disjoin[1] Value: {:?}", join_value[1].len() + disjoin_value[1].len()};
                // process::exit(0);

                // for i in 0..join_value[0].len() {
                //     self.config.q_join[2].enable(&mut region, i)?;
                //     region.assign_advice(
                //         || "join_config",
                //         self.config.instance_test1,
                //         i,
                //         || Value::known(join_value[0][i][0]),
                //     )?;
                //     region.assign_advice(
                //         || "join_config",
                //         self.config.instance_test2,
                //         i,
                //         || Value::known(join_value[0][i][0]),
                //     )?;
                // }

                for i in 0..dis_o_custkey.len() {
                    let cell1 = region
                        .assign_advice(
                            || "deduplicated_b2_vec",
                            self.config.deduplicate[0],
                            i,
                            || Value::known(F::from(dis_o_custkey[i])),
                        )?
                        .cell();
                    let cell2 = region
                        .assign_advice(
                            || "deduplicate_helper",
                            self.config.deduplicate_helper[0],
                            i,
                            || Value::known(F::from(dis_o_custkey[i])),
                        )?
                        .cell();
                    // region.constrain_equal(cell1, cell2)?;
                }
                for i in 0..dis_c_custkey.len() {
                    let cell1 = region
                        .assign_advice(
                            || "deduplicated_a2_vec",
                            self.config.deduplicate[1],
                            i,
                            || Value::known(F::from(dis_c_custkey[i])),
                        )?
                        .cell();
                    let cell2 = region
                        .assign_advice(
                            || "deduplicate_helper",
                            self.config.deduplicate_helper[0],
                            i + dis_o_custkey.len(),
                            || Value::known(F::from(dis_c_custkey[i])),
                        )?
                        .cell();
                    // region.constrain_equal(cell1, cell2)?;
                }
                for i in 0..dis_l_orderkey.len() {
                    let cell1 = region
                        .assign_advice(
                            || "deduplicated_d2_vec",
                            self.config.deduplicate[2],
                            i,
                            || Value::known(F::from(dis_l_orderkey[i])),
                        )?
                        .cell();
                    let cell2 = region
                        .assign_advice(
                            || "deduplicate_helper",
                            self.config.deduplicate_helper[1],
                            i,
                            || Value::known(F::from(dis_l_orderkey[i])),
                        )?
                        .cell();
                    // region.constrain_equal(cell1, cell2)?;
                }
                for i in 0..dis_o_orderkey.len() {
                    let cell1 = region
                        .assign_advice(
                            || "deduplicated_c2_vec",
                            self.config.deduplicate[3],
                            i,
                            || Value::known(F::from(dis_o_orderkey[i])),
                        )?
                        .cell();
                    let cell2 = region
                        .assign_advice(
                            || "deduplicate_helper",
                            self.config.deduplicate_helper[1],
                            i + dis_l_orderkey.len(),
                            || Value::known(F::from(dis_o_orderkey[i])),
                        )?
                        .cell();
                    // region.constrain_equal(cell1, cell2)?;
                }

                // assign the new dedup
                for i in 0..new_dedup_1.len() {
                    if i < new_dedup_1.len() - 1 {
                        self.config.q_sort[0].enable(&mut region, i)?; // start at index 1
                    }
                    region.assign_advice(
                        || "new_dedup_1",
                        self.config.dedup_sort[0],
                        i,
                        || Value::known(F::from(new_dedup_1[i])),
                    )?;
                }
                // println!("new_dedup_2 {:?}", new_dedup_2.len());
                for i in 0..new_dedup_2.len() {
                    if i < new_dedup_2.len() - 1 {
                        self.config.q_sort[1].enable(&mut region, i)?; // start at index 1
                    }
                    region.assign_advice(
                        || "new_dedup_2",
                        self.config.dedup_sort[1],
                        i,
                        || Value::known(F::from(new_dedup_2[i])),
                    )?;
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
                        || "revenue",
                        self.config.revenue,
                        i,
                        || Value::known(revenue[i]),
                    )?;
                }

                for i in 0..cartesian_product.len() {
                    if i < cartesian_product.len() - 1 {
                        self.config.q_sort[2].enable(&mut region, i)?; // q_sort[2]
                    }
                    for (idx, &j) in [0, 7, 5, 1, 2].iter().enumerate() {
                        region.assign_advice(
                            || "groupby",
                            self.config.groupby[idx],
                            i,
                            || Value::known(F::from(cartesian_product[i][j])),
                        )?;
                    }
                }

                // println!("grouped data 1 {:?}", grouped_data.len());
                for i in 0..grouped_data.len() {
                    if i < grouped_data.len() - 1 {
                        self.config.q_sort[3].enable(&mut region, i)?; // start at the index 1
                    }
                    for j in 0..4 {
                        region.assign_advice(
                            || "orderby",
                            self.config.orderby[j],
                            i,
                            || Value::known(F::from(grouped_data[i][j])),
                        )?;
                    }
                }

                // chip assign
                for i in 0..customer.len() {
                    equal_chip[0].assign(
                        &mut region,
                        i,
                        Value::known(F::from(customer[i][0]) - F::from(condition[0])),
                    )?; // c_mktsegment = ':1'
                }
                // for i in 0..grouped_data.len() {
                //     if i < grouped_data.len() - 1 {
                //         equal_chip[1].assign(
                //             // iszerochip assignment
                //             &mut region,
                //             i,
                //             Value::known(F::from(grouped_data[i][3] - grouped_data[i + 1][3])),
                //         )?; // revenue[i-1] = revenue [i]
                //     }
                // }

                lt_compare_chip[0].assign_right_constant(
                    &mut region,
                    orders
                        .iter()
                        .filter_map(|row| row.get(0)) // Get the first element of the row, if it exists
                        .map(|&element| F::from(element)) // Convert each element to type `F`
                        .collect(),
                    F::from(condition[1]),
                )?;

                lt_compare_chip[1].assign_left_constant(
                    &mut region,
                    F::from(condition[1]),
                    lineitem
                        .iter()
                        .filter_map(|row| row.get(3)) // Get the first element of the row, if it exists
                        .map(|&element| F::from(element)) // Convert each element to type `F`
                        .collect(),
                )?;

                lt_compare_chip[2].assign(
                    &mut region,
                    f_new_dedup_1.clone()[0..n1 - 1].to_vec(),
                    f_new_dedup_1.clone()[1..n1].to_vec(),
                )?;

                lt_compare_chip[3].assign(
                    &mut region,
                    f_new_dedup_2.clone()[0..n2 - 1].to_vec(),
                    f_new_dedup_2.clone()[1..n2].to_vec(),
                )?;

                // compare_chip[0].assign(
                //     &mut region,
                //     cartesian_left.clone(),
                //     cartesian_right.clone(),
                // )?;
                // compare_chip[1].assign(
                //     &mut region,
                //     grouped_left_revenue.clone(),
                //     grouped_right_revenue.clone(),
                // )?;
                // compare_chip[2].assign(
                //     &mut region,
                //     grouped_left_orderdate.clone(),
                //     grouped_right_orderdate.clone(),
                // )?;

                let duration_block = start.elapsed();
                println!("Time elapsed for block: {:?}", duration_block);

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

    pub condition: [u64; 2],

    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            customer: Vec::new(),
            orders: Vec::new(),
            lineitem: Vec::new(),

            condition: [Default::default(); 2],
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
        let params_path = "/home/cc/halo2-TPCH/src/sql/param16";
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
        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";

        let mut customer: Vec<Vec<u64>> = Vec::new();
        let mut orders: Vec<Vec<u64>> = Vec::new();
        let mut lineitem: Vec<Vec<u64>> = Vec::new();

        // if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
        //     // Convert the Vec<Region> to a 2D vector
        //     customer = records
        //         .iter()
        //         .map(|record| {
        //             vec![
        //                 Fp::from(string_to_u64(&record.c_mktsegment)),
        //                 Fp::from(record.c_custkey),
        //             ]
        //         })
        //         .collect();
        // }
        // if let Ok(records) = data_processing::orders_read_records_from_file(orders_file_path) {
        //     // Convert the Vec<Region> to a 2D vector
        //     orders = records
        //         .iter()
        //         .map(|record| {
        //             vec![
        //                 // Fp::from(string_to_u64(&record.o_orderdate)),
        //                 Fp::from(date_to_timestamp(&record.o_orderdate)),
        //                 Fp::from(record.o_shippriority),
        //                 Fp::from(record.o_custkey),
        //                 Fp::from(record.o_orderkey),
        //             ]
        //         })
        //         .collect();
        // }
        // if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
        //     // Convert the Vec<Region> to a 2D vector
        //     lineitem = records
        //         .iter()
        //         .map(|record| {
        //             vec![
        //                 Fp::from(record.l_orderkey),
        //                 Fp::from(scale_by_1000(record.l_extendedprice)),
        //                 Fp::from(scale_by_1000(record.l_discount)),
        //                 // Fp::from(string_to_u64(&record.l_shipdate)),
        //                 Fp::from(date_to_timestamp(&record.l_shipdate)),
        //             ]
        //         })
        //         .collect();
        // }

        if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
            // Convert the Vec<Region> to a 2D vector
            customer = records
                .iter()
                .map(|record| vec![string_to_u64(&record.c_mktsegment), record.c_custkey])
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
                        record.o_shippriority,
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
                        // Fp::from(string_to_u64(&record.l_shipdate)),
                        date_to_timestamp(&record.l_shipdate),
                    ]
                })
                .collect();
        }

        let condition = [string_to_u64("HOUSEHOLD"), date_to_timestamp("1995-03-25")];
        // c_mktsegment = 'HOUSEHOLD'   -> 3367
        // o_orderdate < date '1995-03-25'and l_shipdate > date '1995-03-25'  ->796089600
        //  BUILDING ->   2651;    1996-03-13 -> 2731

        // let customer: Vec<Vec<u64>> = customer.iter().take(100).cloned().collect();
        // let orders: Vec<Vec<u64>> = orders.iter().take(100).cloned().collect();
        // let lineitem: Vec<Vec<u64>> = lineitem.iter().take(1000).cloned().collect();

        let circuit = MyCircuit::<Fp> {
            customer,
            orders,
            lineitem,
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
            let proof_path = "/home/cc/halo2-TPCH/src/sql/proof_q3";
            generate_and_verify_proof(k, circuit, &public_input, proof_path);
        }

        // process::exit(0);
    }
}
