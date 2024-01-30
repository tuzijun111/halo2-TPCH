use eth_types::Field;
use gadgets::util::or;
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};
use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
use gadgets::lessthan_or_equal::{LtEqChip, LtEqConfig, LtEqInstruction};
use gadgets::lessthan_or_equal_generic::{
    LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
};

use std::thread::sleep;
use std::{default, marker::PhantomData};

// use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use itertools::iproduct;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use std::mem;

const N: usize = 10;
const NUM_BYTES: usize = 3;

// #[derive(Default)]
// We should use the selector to skip the row which does not satisfy shipdate values

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field> {
    q_enable: Vec<Selector>,
    q_join: Vec<Selector>,
    q_dedup: Vec<Selector>,
    q_perm: Vec<Selector>,

    q_sort: Vec<Selector>,
    q_accu: Selector,

    customer: Vec<Column<Advice>>, //  c_custkey, c_nationkey
    orders: Vec<Column<Advice>>,   // o_orderdate, o_custkey, o_orderkey
    lineitem: Vec<Column<Advice>>, // l_order, l_extened, l_dis, l_supp
    supplier: Vec<Column<Advice>>, // s_nationkey, s_supp
    nation: Vec<Column<Advice>>,   // n_nationkey, n_regionkey, n_name
    regions: Vec<Column<Advice>>,  // r_regionkey, r_name

    join_group: Vec<Vec<Column<Advice>>>, // its length is 12
    disjoin_group: Vec<Vec<Column<Advice>>>,
    // join_and_disjoin2: Vec<Vec<Column<Advice>>>,
    // join_and_disjoin3: Vec<Vec<Column<Advice>>>,
    // join_and_disjoin4: Vec<Vec<Column<Advice>>>,
    // join_and_disjoin5: Vec<Vec<Column<Advice>>>,
    // join_and_disjoin6: Vec<Vec<Column<Advice>>>,

    check: Vec<Column<Advice>>,

    deduplicate: Vec<Column<Advice>>, // deduplicate disjoint values of l_orderkey

    dedup_sort: Vec<Column<Advice>>,

    condition: Vec<Column<Advice>>, // conditional value for l, c and o

    join: Vec<Column<Advice>>,
    groupby: Vec<Column<Advice>>,
    orderby: Vec<Column<Advice>>,

    equal_check: Column<Advice>, // check if sorted_revenue[i-1] = sorted_revenue[i]
    revenue: Column<Advice>,

    lt_compare_condition: Vec<LtConfig<F, NUM_BYTES>>,

    equal_condition: Vec<IsZeroConfig<F>>,
    compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
    perm: Vec<PermAnyConfig>,
}

#[derive(Debug, Clone)]
pub struct TestChip<F: Field> {
    config: TestCircuitConfig<F>,
}

// conditions for filtering in tables: customer, orders,lineitem
//   c_mktsegment = ':1', o_orderdate < date ':2', and l_shipdate > date ':2'

// Circuits illustration
// | l_orderkey |  l_extendedprice | l_discount | l_shipdate | ...
// ------+-------+------------+------------------------+-------------------------------
//    |     |       |         0              |  0

impl<F: Field> TestChip<F> {
    pub fn construct(config: TestCircuitConfig<F>) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> TestCircuitConfig<F> {
        let mut q_enable = Vec::new();
        for i_ in 0..3 {
            q_enable.push(meta.complex_selector());
        }

        let mut q_sort = Vec::new();
        for i_ in 0..8 {
            q_sort.push(meta.complex_selector());
        }

        let mut q_join = Vec::new();
        for i_ in 0..1 {
            q_join.push(meta.complex_selector());
        }
        let mut q_dedup = Vec::new();
        for i_ in 0..2 {
            q_dedup.push(meta.complex_selector());
        }
        let mut q_perm = Vec::new();
        for i_ in 0..1 {
            q_perm.push(meta.complex_selector());
        }

        let q_accu = meta.complex_selector();

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
     
        for l in [2,3, 3,4, 4,2, 2,3, 2,3, 3,2] {  
            let mut col = Vec::new();
            for _ in 0..l {
                col.push(meta.advice_column());
            }
            join_group.push(col.clone());
            disjoin_group.push(col.clone());
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

        let mut join = Vec::new();
        let mut groupby = Vec::new();
        let mut orderby = Vec::new();
        for _ in 0..3 {
            join.push(meta.advice_column());
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
        // equality
        // For 'customer', 'supplier', 'regions', 'dedup_sort', 'is_zero_advice_column'
        for vec in [&customer, &supplier, &regions, &dedup_sort, &is_zero_advice_column] {
            for &element in vec.iter() {
                meta.enable_equality(element);
            }
        }

        // For 'orders', 'nation', 'condition', 'join', 'groupby', 'check'
        for vec in [&orders, &nation, &condition, &join, &groupby, &check] {
            for &element in vec.iter() {
                meta.enable_equality(element);
            }
        }

        // For 'lineitem'
        for &element in lineitem.iter() {
            meta.enable_equality(element);
        }

        // For 'deduplicate'
        for &element in deduplicate.iter() {
            meta.enable_equality(element);
        }

        // For 'orderby'
        for &element in orderby.iter() {
            meta.enable_equality(element);
        }

        // For 'equal_check' and 'revenue'
        meta.enable_equality(equal_check);
        meta.enable_equality(revenue);

        // For 'join_group' and 'disjoin_group' (assuming you want to apply it to these as well)
        for vec in join_group.iter().chain(disjoin_group.iter()) {
            for &element in vec.iter() {
                meta.enable_equality(element);
            }
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

        // o_orderdate >= date ':2'
        let mut compare_condition = Vec::new();
        let config: LtEqGenericConfig<F, NUM_BYTES> = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[1]),
            |meta| {
                vec![
                    meta.query_advice(condition[1], Rotation::cur())        
                ]
            },
            |meta| {
                vec![
                    meta.query_advice(orders[0], Rotation::cur())           
                ]
            },
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
            |meta| meta.query_selector(q_enable[2]),
            |meta| meta.query_advice(orders[0], Rotation::cur()),
            |meta| meta.query_advice(condition[2], Rotation::cur()), // we put the left and right value at the first two positions of value_l
        );
        meta.create_gate(
            "verifies o_orderdate < date ':2' + interval '1' year", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(q_enable[2]);
                let check = meta.query_advice(check[2], Rotation::cur());
                vec![q_enable * (config.clone().is_lt(meta, None) - check)]
            },
        );
        lt_compare_condition.push(config);

        // join sort check
        for i in 0..6 {
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
            |meta| meta.query_selector(q_sort[6]),
            |meta| {
                vec![
                    meta.query_advice(groupby[0], Rotation::prev()),
                   
                ]
            },
            |meta| {
                vec![
                    meta.query_advice(groupby[0], Rotation::cur()),
                 
                ]
            },
        );
        compare_condition.push(config);

        // groupby permutation check
        let mut perm = Vec::new();
        let config = PermAnyChip::configure(meta, q_perm[0], join.clone(), groupby.clone());
        perm.push(config);

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
        // (1) revenue[i-1] > revenue[i]
        let config = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[7]), // q_sort[1] should start from index 1
            |meta| vec![meta.query_advice(orderby[1], Rotation::cur())], // revenue
            |meta| vec![meta.query_advice(orderby[1], Rotation::prev())],
        );
        compare_condition.push(config.clone());

        meta.create_gate("verifies orderby scenarios", |meta| {
            let q_sort = meta.query_selector(q_sort[7]);

            vec![
                q_sort.clone() *
                (config.is_lt(meta, None) - Expression::Constant(F::ONE))                  
            ]
        });
        



        TestCircuitConfig {
            q_enable,
            q_join,
            q_dedup,
            q_perm,
            q_sort,
            q_accu,

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
            join,
            groupby,
            orderby,
            equal_check,

            revenue,
            lt_compare_condition,
            equal_condition,
            compare_condition,
            perm,
        }
    }

    pub fn assign(
        &self,
        // layouter: &mut impl Layouter<F>,
        layouter: &mut impl Layouter<F>,

        customer: Vec<Vec<F>>,
        orders: Vec<Vec<F>>,
        lineitem: Vec<Vec<F>>,
        supplier: Vec<Vec<F>>,
        nation: Vec<Vec<F>>,
        regions: Vec<Vec<F>>,

        condition: [F; 3],
    ) -> Result<(), Error> {
        let mut equal_chip = Vec::new();
        let mut compare_chip = Vec::new();
        let mut lt_compare_chip = Vec::new();
        let mut perm_chip = Vec::new();

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

        println!("equal chip: {:?}", equal_chip.len());
        println!("compare chip: {:?}", compare_chip.len());
        println!("lt compre chip: {:?}", lt_compare_chip.len());

        for i in 0..self.config.perm.len() {
            let config = PermAnyChip::construct(self.config.perm[i].clone());
            perm_chip.push(config);
        }

        layouter.assign_region(
            || "witness",
            |mut region| {
                let mut r_check = Vec::new(); // t/f
                let mut o1_check = Vec::new(); // t/f
                let mut o2_check = Vec::new(); // 0, 1

                //assign input values
                for i in 0..customer.len() {
                    for j in 0..customer[0].len() {
                        region.assign_advice(
                            || "customer",
                            self.config.customer[j],
                            i,
                            || Value::known(customer[i][j]),
                        )?;
                    }
                }
   
                for i in 0..orders.len() {
                    self.config.q_enable[1].enable(&mut region, i)?; 
                    self.config.q_enable[2].enable(&mut region, i)?;
                    for j in 0..orders[0].len() {
                        region.assign_advice(
                            || "orders",
                            self.config.orders[j],
                            i,
                            || Value::known(orders[i][j]),
                        )?;
                    }
                    if orders[i][0] >= condition[1] {
                        o1_check.push(true);
                    } else {
                        o1_check.push(false);
                    }
                    if orders[i][0] < condition[0] {
                        o2_check.push(true);
                    } else {
                        o2_check.push(false);
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
                        || Value::known(condition[1]),
                    )?;
                    region.assign_advice(
                        || "condition for orders",
                        self.config.condition[2],
                        i,
                        || Value::known(condition[2]),
                    )?;

                    compare_chip[0].assign(
                        &mut region,
                        i, 
                        &[condition[1]],
                        &[orders[i][0]],
                    )?;
                    lt_compare_chip[0].assign(
                        &mut region,
                        i,
                        Value::known(orders[i][0]),
                        Value::known(condition[2]),
                    )?;
                }
                for i in 0..lineitem.len() {
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[j],
                            i,
                            || Value::known(lineitem[i][j]),
                        )?;
                    }
                }

                for i in 0..supplier.len() {
                    for j in 0..supplier[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.supplier[j],
                            i,
                            || Value::known(supplier[i][j]),
                        )?;
                    }
                }

                for i in 0..nation.len() {
                    for j in 0..nation[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.nation[j],
                            i,
                            || Value::known(nation[i][j]),
                        )?;
                    }
                }
                
                for i in 0..regions.len() {
                    self.config.q_enable[0].enable(&mut region, i)?;
                    for j in 0..regions[0].len() {
                        region.assign_advice(
                            || "customer",
                            self.config.regions[j],
                            i,
                            || Value::known(regions[i][j]),
                        )?;
                    }
                    if regions[i][1] == condition[0] {
                        r_check.push(F::from(1));
                    } else {
                        r_check.push(F::from(0));
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
                        || Value::known(condition[0]),
                    )?;

                    equal_chip[0].assign(
                        &mut region,
                        i,
                        Value::known(regions[i][1] - condition[0]),
                    )?; // r_name = ':1'
                }

           

                // compute values related to the join operation locally
                // store the attribtues of the tables that will be used in the SQL query in tuples
                let mut c_combined = customer.clone();
                let mut o_combined = orders.clone();
                let mut l_combined = lineitem.clone();
                let mut s_combined = supplier.clone();
                let mut n_combined = nation.clone();
                let mut r_combined = regions.clone();

                let o_combined: Vec<Vec<_>> = c_combined
                    .clone()
                    .into_iter()
                    .filter(|row| row[0] >= condition[1] && row[0] < condition[2]) // r_name = ':3'
                    .collect();

                let r_combined: Vec<Vec<_>> = o_combined
                    .clone()
                    .into_iter()
                    .filter(|row| row[1] == condition[0]) // r_name = ':3'
                    .collect();

              

                let mut join_value: Vec<Vec<_>> = vec![vec![]; 12];
                let mut disjoin_value: Vec<Vec<_>> = vec![vec![]; 12];

                let mut combined = Vec::new();
                combined.push(c_combined.clone()); // its length is 2
                combined.push(o_combined.clone()); // 3
                combined.push(l_combined.clone()); // 4
                combined.push(s_combined.clone()); // 2
                combined.push(n_combined.clone()); // 3
                combined.push(r_combined.clone()); // 2

                let index = [
                    (0, 1, 0, 1), //   c_custkey = o_custkey
                    (1, 2, 2, 0), //   l_orderkey = o_orderkey
                    (2, 3, 3, 1), // l_suppkey = s_suppkey
                    (0, 4, 1, 0), // c_nationkey = s_nationkey
                    (3, 4, 0, 0), // s_nationkey = n_nationkey
                    (4, 5, 1, 0), // n_regionkey = r_regionkey
                ];

                for i in 0..index.len() {
                    for val in combined[index[i].0].iter() {
                        if let Some(_) = combined[index[i].1]
                            .iter()
                            .find(|v| v[index[i].3] == val[index[i].2])
                        {
                            join_value[i * 2].push(val.clone()); // join values
                        } else {
                            disjoin_value[i * 2].push(val); // disjoin values
                        }
                    }
                    for val in combined[index[i].1].iter() {
                        if let Some(_) = combined[index[i].0]
                            .iter()
                            .find(|v| v[index[i].2] == val[index[i].3])
                        {
                            join_value[i * 2 + 1].push(val.clone());
                        } else {
                            disjoin_value[i * 2 + 1].push(val);
                        }
                    }
                }
                // assign join and disjoin values
                for k in 0..join_value.len() {
                    // Assuming self.config has an array or vector of configurations corresponding to k
                        
                    let join_config = &self.config.join_group[k]; // Adjust this based on your actual config structure
                    
                    // Process join_value[k]
                    for i in 0..join_value[k].len() {
                        for j in 0..join_value[k][i].len() {
                            region.assign_advice(
                                || "n",
                                join_config[j],
                                i,
                                || Value::known(join_value[k][i][j]),
                            )?;
                        }
                    }        
                }

                for k in 0..disjoin_value.len() {
                    // Assuming self.config has an array or vector of configurations corresponding to k
                    if k > 0 {
                        let disjoin_config = &self.config.disjoin_group[k]; // Adjust this as well   
                        // Process disjoin_value[k]
                        for i in 0..disjoin_value[k].len() {
                            for j in 0..disjoin_value[k][i].len() {
                                region.assign_advice(
                                    || "n",
                                    disjoin_config[j],
                                    i,
                                    || Value::known(disjoin_value[k][i][j]),
                                )?;
                            }          
                        }
                    }
                }

                
                

                // compute final table by applying all joins
                let join_index = [
                    (0, 1, 0, 1), //   c_custkey = o_custkey
                    (1, 2, 2+2, 0), //   l_orderkey = o_orderkey
                    (2, 3, 2+3+3, 1), // l_suppkey = s_suppkey
                    (0, 4, 1, 0), // c_nationkey = s_nationkey
                    (3, 4, 2+3+4+0, 0), // s_nationkey = n_nationkey
                    (4, 5, 2+3+4+2+1, 0), // n_regionkey = r_regionkey
                ];
                fn join_vectors<F>(
                    vectors: &[Vec<Vec<F>>],
                    join_index: &[(usize, usize, usize, usize)],
                ) -> Vec<Vec<F>>
                where
                    F: Clone + PartialEq,
                {
                    let mut join_result = vectors[0].to_vec();
                    for i in 1..vectors.len() {
                        let mut next_join = Vec::new();
                        for ab in join_result.iter() {
                            for c in vectors[i].iter() {
                                if ab[join_index[i - 1].2] == c[join_index[i - 1].3] {
                                    let mut joined = ab.to_vec();
                                    joined.extend_from_slice(c);
                                    next_join.push(joined);
                                }
                            }
                        }
                        join_result = next_join;
                    }

                    join_result
                }

                let mut cartesian_product = join_vectors(&combined, &join_index);

                // assign join: cartesian_product
                for i in 0..cartesian_product.len() {  // n_name, l_ex, l_dis
                    for (idx, &j) in [13, 6, 7].iter().enumerate() {
                        region.assign_advice(
                            || "groupby",
                            self.config.join[idx],
                            i,
                            || Value::known(cartesian_product[i][j]),
                        )?;
                    }
                }
                let input = cartesian_product.clone(); // for permanychip inputs

                let mut dis_c_custkey: Vec<F> = disjoin_value[0].iter().map(|v| v[0]).collect();
                let mut dis_o_custkey: Vec<F> = disjoin_value[1].iter().map(|v| v[1]).collect();
                let mut dis_o_orderkey: Vec<F> = disjoin_value[2].iter().map(|v| v[2]).collect();
                let mut dis_l_orderkey: Vec<F> = disjoin_value[3].iter().map(|v| v[0]).collect();
                let mut dis_l_suppkey: Vec<F> = disjoin_value[4].iter().map(|v| v[3]).collect();
                let mut dis_s_suppkey: Vec<F> = disjoin_value[5].iter().map(|v| v[1]).collect();
                let mut dis_c_nationkey: Vec<F> = disjoin_value[6].iter().map(|v| v[1]).collect();
                let mut dis_s_nationkey: Vec<F> = disjoin_value[7].iter().map(|v| v[0]).collect();
                let mut dis_s1_nationkey: Vec<F> = disjoin_value[8].iter().map(|v| v[0]).collect();
                let mut dis_n_nationkey: Vec<F> = disjoin_value[9].iter().map(|v| v[0]).collect();
                let mut dis_n_regionkey: Vec<F> = disjoin_value[10].iter().map(|v| v[1]).collect();
                let mut dis_r_regionkey: Vec<F> = disjoin_value[11].iter().map(|v| v[0]).collect();

                // generate deduplicated columns for disjoin values
                let mut dis_vectors = vec![
                    &mut dis_c_custkey,
                    &mut dis_o_custkey,
                    &mut dis_o_orderkey,
                    &mut dis_l_orderkey,
                    &mut dis_l_suppkey,
                    &mut dis_s_suppkey,
                    &mut dis_c_nationkey,
                    &mut dis_s_nationkey,
                    &mut dis_s1_nationkey,
                    &mut dis_n_nationkey,
                    &mut dis_n_regionkey,
                    &mut dis_r_regionkey,
                ];
                for dis_vector in dis_vectors.iter_mut() {
                    dis_vector.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    dis_vector.dedup();
                }
            
                
                
                let deduplicate_indices = vec![0,1,2,3,4,5,6,7,8,9,10,11];

                for (index, dis_vector) in dis_vectors.iter().enumerate() {
                    for (i, value) in dis_vector.iter().enumerate() {
                        region.assign_advice(
                            || "deduplicated",
                            self.config.deduplicate[deduplicate_indices[index]],
                            i,
                            || Value::known(*value),
                        )?;
                    }
                }
                
                // concatenate two vectors for sorting
                let mut concatenated_vectors = Vec::new();
                for i in (0..dis_vectors.len()).step_by(2) {
                    if let (Some(first), Some(second)) = (dis_vectors.get(i), dis_vectors.get(i + 1)) {
                        let concatenated = first.iter().cloned().chain(second.iter().cloned()).collect::<Vec<F>>();
                        concatenated_vectors.push(concatenated);
                    }
                }
                                
                for mut element in &mut concatenated_vectors{
                    element.sort();
                }

                // assign the new dedup
                for k in 0..concatenated_vectors.len(){
                    for i in 0..concatenated_vectors[k].len() {
                        if i > 0 {
                            self.config.q_sort[k].enable(&mut region, i)?; // start at index 1

                            lt_compare_chip[1+k].assign(
                                // dedup_sort[][i-1] < dedup_sort[][i]
                                &mut region,
                                i,
                                Value::known(concatenated_vectors[k][i - 1]),
                                Value::known(concatenated_vectors[k][i]),
                            )?;
                        }
                        region.assign_advice(
                            || "new_dedup",
                            self.config.dedup_sort[k],
                            i,
                            || Value::known(concatenated_vectors[k][i]),
                        )?;
                    }
                }

                let join: Vec<Vec<F>> = cartesian_product
                    .iter()
                    .map(|v| {
                        let mut new_vec = Vec::new();
                        if v.len() >= 1 {
                            new_vec.push(v[13]);
                            new_vec.push(v[6]);
                            new_vec.push(v[7]);                     
                        }
                        new_vec
                    })
                    .collect();

                // group by n_name
                cartesian_product.sort_by_key(|element| { element[13].clone() });

                let groupby: Vec<Vec<F>> = cartesian_product
                    .iter()
                    .map(|v| {
                        let mut new_vec = Vec::new();
                        if v.len() >= 1 {      
                            new_vec.push(v[13]);
                            new_vec.push(v[6]);
                            new_vec.push(v[7]);                
                        }
                        new_vec
                    })
                    .collect();

                // region.constrain_equal(left, right)

                let mut equal_check: Vec<F> = Vec::new();
                if cartesian_product.len() > 0 {
                    equal_check.push(F::from(0)); // add the the first one must be 0
                }

                for row in 1..cartesian_product.len() {
                    self.config.q_sort[6].enable(&mut region, row)?; // q_sort[2]
                    if cartesian_product[row][13] == cartesian_product[row - 1][13]
                    {
                        equal_check.push(F::from(1));
                    } else {
                        equal_check.push(F::from(0));
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

                let n = cartesian_product.len() + 1;
                let mut revenue: Vec<F> = vec![F::from(0); n];
                for i in 1..n {
                    revenue[i] = revenue[i - 1] * equal_check[i - 1]  // sum(l_extendedprice * (1 - l_discount)) as revenue,
                        + cartesian_product[i - 1][7] * (F::from(1000) - cartesian_product[i - 1][8]);
                    // cartesian_product[i - 1].push(revenue[i].clone()); // add this value to correct row of cartesian product
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
                    self.config.q_perm[0].enable(&mut region, i)?; // q_perm[0]
                    for (idx, &j) in [13, 6, 7].iter().enumerate() {
                        region.assign_advice(
                            || "groupby",
                            self.config.groupby[idx],
                            i,
                            || Value::known(cartesian_product[i][j]),
                        )?;
                    }
                    if i > 0 {
                        compare_chip[1].assign(
                            &mut region,
                            i, // assign groupby compare chip
                            &[cartesian_product[i - 1][13]],
                            &[cartesian_product[i][13]],
                        )?;
                    }
                }

                let _ = perm_chip[0].assign1(&mut region, join, groupby); // permutation between join and groupby

                // order by revenue desc,            
                let mut grouped_data: Vec<Vec<F>> = Vec::new();
                for row in &cartesian_product {
                    // Check if the group (a1 value) already exists
                    match grouped_data
                        .iter_mut()
                        .find(|g| g[0] == row[13])
                    {
                        Some(group) => {
                            group[1] += row[6] * (F::from(1000) - row[7]); // Add to the existing sum
                        }
                        None => {
                            grouped_data.push(vec![
                                row[13],
                                row[6] * (F::from(1000) - row[7]),
                            ]); // Create a new group
                        }
                    }
                }
                // println!("cartesian {:?}", cartesian_product);
                // println!("grouped data {:?}", grouped_data);

                grouped_data.sort_by_key(|element| { element[1].clone() });
                    
            
                // println!("grouped data 1 {:?}", grouped_data.len());
                for i in 0..grouped_data.len() {
                    if i > 0 {
                        self.config.q_sort[7].enable(&mut region, i)?; // start at the index 1
                        
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

                Ok(())
            },
        )
    }

    // pub fn expose_public(
    //     &self,
    //     layouter: &mut impl Layouter<F>,
    //     cell: AssignedCell<F, F>,
    //     row: usize,
    // ) -> Result<(), Error> {
    //     layouter.constrain_instance(cell.cell(), self.config.instance, row)
    // }
}

struct MyCircuit<F> {
    customer: Vec<Vec<F>>,
    orders: Vec<Vec<F>>,
    lineitem: Vec<Vec<F>>,
    supplier: Vec<Vec<F>>,
    nation: Vec<Vec<F>>,
    region: Vec<Vec<F>>,

    pub condition: [F; 3],

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
            region: Vec::new(),

            condition: [Default::default(); 3],
            _marker: PhantomData,
        }
    }
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
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

        let out_b_cells = test_chip.assign(
            &mut layouter,
            self.customer.clone(),
            self.orders.clone(),
            self.lineitem.clone(),
            self.supplier.clone(),
            self.nation.clone(),
            self.region.clone(),
            self.condition.clone(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    use super::N;
    // use rand::Rng;
    // use halo2_proofs::poly::commitment::Params
    use crate::data::data_processing;
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};
    use std::marker::PhantomData;

    #[test]
    fn test_1() {
        let k = 10;

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

        let customer_file_path = "/Users/binbingu/halo2-TPCH/src/data/customer.tbl";
        let orders_file_path = "/Users/binbingu/halo2-TPCH/src/data/orders.tbl";
        let lineitem_file_path = "/Users/binbingu/halo2-TPCH/src/data/lineitem.tbl";
        let supplier_file_path = "/Users/binbingu/halo2-TPCH/src/data/supplier.tbl";
        let nation_file_path = "/Users/binbingu/halo2-TPCH/src/data/nation.tbl";
        let region_file_path = "/Users/binbingu/halo2-TPCH/src/data/region.csv";
        let mut customer: Vec<Vec<Fp>> = Vec::new();
        let mut orders: Vec<Vec<Fp>> = Vec::new();
        let mut lineitem: Vec<Vec<Fp>> = Vec::new();
        let mut supplier: Vec<Vec<Fp>> = Vec::new();
        let mut nation: Vec<Vec<Fp>> = Vec::new();
        let mut region: Vec<Vec<Fp>> = Vec::new();

        if let Ok(records) = data_processing::customer_read_records_from_file(customer_file_path) {
            // Convert the Vec<Region> to a 2D vector
            customer = records
                .iter()
                .map(|record| {
                    vec![
                        
                        Fp::from(record.c_custkey),
                        Fp::from(string_to_u64(&record.c_mktsegment)),
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
                        Fp::from(string_to_u64(&record.o_orderdate)),
                        
                        Fp::from(record.o_custkey),
                        Fp::from(record.o_orderkey),
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
                        Fp::from(record.l_orderkey),
                        Fp::from(scale_by_1000(record.l_extendedprice)),
                        Fp::from(scale_by_1000(record.l_discount)),
                        Fp::from(record.l_suppkey),
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::supplier_read_records_from_file(supplier_file_path) {
            // Convert the Vec<Region> to a 2D vector
            supplier = records
                .iter()
                .map(|record| {
                    vec![
                        Fp::from(record.s_nationkey),
                        Fp::from(record.s_suppkey),
                    
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
                        Fp::from(record.n_nationkey),
                        Fp::from(record.n_regionkey),
                        Fp::from(string_to_u64(&record.n_name)),
                    ]
                })
                .collect();
        }
        if let Ok(records) = data_processing::region_read_records_from_csv(region_file_path) {
            // Convert the Vec<Region> to a 2D vector
            region = records
                .iter()
                .map(|record| {
                    vec![
                        Fp::from(record.r_regionkey),
                       
                        Fp::from(string_to_u64(&record.r_name)),
                    ]
                })
                .collect();
        }

        let condition = [Fp::from(1), Fp::from(10), Fp::from(10)];
        // let condition = [Fp::from(1), Fp::from(2731)];
        //  BUILDING ->   2651;    1996-03-13 -> 2731

        let customer: Vec<Vec<Fp>> = customer.iter().take(3).cloned().collect();
        let orders: Vec<Vec<Fp>> = orders.iter().take(3).cloned().collect();
        let lineitem: Vec<Vec<Fp>> = lineitem.iter().take(3).cloned().collect();
        let supplier: Vec<Vec<Fp>> = supplier.iter().take(3).cloned().collect();
        let nation: Vec<Vec<Fp>> = nation.iter().take(3).cloned().collect();
        let region: Vec<Vec<Fp>> = region.iter().take(3).cloned().collect();

        // let customer: Vec<Vec<Fp>> = vec![
        //     vec![Fp::from(1), Fp::from(2)],
        //     vec![Fp::from(1), Fp::from(3)],
        //     vec![Fp::from(1), Fp::from(4)],
        // ];
        // let orders: Vec<Vec<Fp>> = vec![
        //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(2)],
        //     vec![Fp::from(1), Fp::from(2), Fp::from(2), Fp::from(4)],
        //     vec![Fp::from(1), Fp::from(2), Fp::from(6), Fp::from(2)],
        // ];
        // let lineitem: Vec<Vec<Fp>> = vec![
        //     vec![Fp::from(4), Fp::from(2), Fp::from(1), Fp::from(11)],
        //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(12)],
        //     vec![Fp::from(1), Fp::from(2), Fp::from(1), Fp::from(13)],
        // ];

        let circuit = MyCircuit::<Fp> {
            customer,
            orders,
            lineitem,
            supplier,
            nation,
            region,
            condition,
            _marker: PhantomData,
        };

        // let z = [Fp::from(1 * (N as u64))];
        // let z = [
        //     Fp::from(0),
        //     Fp::from(1),
        //     Fp::from(0),
        //     Fp::from(0),
        //     Fp::from(1),
        // ];

        // let prover = MockProver::run(k, &circuit, vec![z.to_vec()]).unwrap();
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
