use eth_types::Field;
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
use gadgets::lessthan_or_equal::{LtEqChip, LtEqConfig, LtEqInstruction};
use gadgets::lessthan_or_equal_generic::{
    LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
};

use std::{default, marker::PhantomData};

// use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use itertools::iproduct;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::HashSet;

use std::mem;

const N: usize = 1;
const NUM_BYTES: usize = 3;

// #[derive(Default)]
// We should use the selector to skip the row which does not satisfy shipdate values

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field> {
    q_enable: [Selector; 5],

    lineitem: [Column<Advice>; 2], // l_orderkey, l_quantity
    orders: [Column<Advice>; 4],   // o_orderdate, o_orderkey, o_custkey, o_totalprice
    customer: [Column<Advice>; 2], // c_name, c_custkey,

    condition: [Column<Advice>; 1], // sum(l_quantity) > :1

    check: [Column<Advice>; 1],
    join_column: [Column<Advice>; 5],
    disjoin_column: [Column<Advice>; 5],

    sum_quantity: [Column<Advice>; 2], // for select l_orderkey from lineitem group by l_orderkey having sum(l_quantity) > :1

    // groupby: [Column<Advice>; 2],
    l_condition: LtConfig<F, NUM_BYTES>,
    // groupby_sort: LtEqGenericConfig<F, NUM_BYTES>,
    // revenue_final: LtEqGenericConfig<F, NUM_BYTES>,
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
        let q_enable = [
            meta.complex_selector(),
            meta.complex_selector(),
            meta.complex_selector(),
            meta.complex_selector(),
            meta.complex_selector(),
        ];

        let lineitem = [meta.advice_column(), meta.advice_column()];

        let orders = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];
        let customer = [meta.advice_column(), meta.advice_column()];

        let condition = [meta.advice_column()];
        let check = [meta.advice_column()];
        let sum_quantity = [meta.advice_column(), meta.advice_column()];
        // let groupby = [meta.advice_column(), meta.advice_column()];

        meta.enable_equality(condition[0]);
        meta.enable_equality(check[0]);

        for i in 0..2 {
            meta.enable_equality(lineitem[i]);
            meta.enable_equality(customer[i]);
            meta.enable_equality(sum_quantity[i]);
        }

        for i in 0..4 {
            meta.enable_equality(orders[i]);
        }

        let join_column = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let disjoin_column = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        let l_condition = LtChip::configure(
            meta,
            |meta| meta.query_selector(q_enable[0]),
            |meta| meta.query_advice(condition[0], Rotation::cur()),
            |meta| meta.query_advice(sum_quantity[1], Rotation::cur()), // we put the left and right value at the first two positions of value_l
        );

        // // gate for o_orderdate >= date ':2'
        // meta.create_gate("verifies o_orderdate < date ':2'", |meta| {
        //     let q_enable = meta.query_selector(q_enable);
        //     let check = meta.query_advice(o2_check, Rotation::cur());
        //     vec![q_enable * (lt_o2_condition.is_lt(meta, None) - check)]
        // });

        // // gate for r_name = ':1'
        // meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
        //     let s = meta.query_selector(q_enable);
        //     let output = meta.query_advice(r_check, Rotation::cur());
        //     vec![
        //         s.clone()
        //             * (lt_r_condition.expr() * (output.clone() - Expression::Constant(F::ONE))), // in this case output == 1
        //         s * (Expression::Constant(F::ONE) - lt_r_condition.expr()) * (output), // in this case output == 0
        //     ]
        // });

        // // groupby
        // let groupby_sort = LtEqGenericChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_sort),
        //     |meta| vec![meta.query_advice(groupby_name, Rotation::prev())],
        //     |meta| vec![meta.query_advice(groupby_name, Rotation::cur())],
        // );

        // let revenue_final = LtEqGenericChip::configure(
        //     meta,
        //     |meta| meta.query_selector(q_sort),
        //     |meta| vec![meta.query_advice(sorted_revenue, Rotation::prev())],
        //     |meta| vec![meta.query_advice(sorted_revenue, Rotation::cur())],
        // );

        TestCircuitConfig {
            q_enable,
            lineitem,
            orders,
            customer,
            condition,
            check,
            join_column,
            disjoin_column,
            sum_quantity,
            l_condition,
        }
    }

    pub fn assign(
        &self,
        layouter: &mut impl Layouter<F>,
        lineitem: [[F; N]; 2],
        orders: [[F; N]; 4],
        customer: [[F; N]; 2],

        l_condition: F,
    ) -> Result<(), Error> {
        // Result<AssignedCell<F, F>, Error> {
        let l_cond_chip = LtChip::construct(self.config.l_condition);
        // let groupby_sort_chip = LtEqGenericChip::construct(self.config.groupby_sort.clone());
        // let lt_revenue_final_chip = LtEqGenericChip::construct(self.config.revenue_final.clone());

        l_cond_chip.load(layouter)?;
        // lt_o2_cond_chip.load(layouter)?;

        layouter.assign_region(
            || "witness",
            |mut region| {
                //assign input values
                for i in 0..lineitem.len() {
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[i],
                            i,
                            || Value::known(lineitem[i][j]),
                        )?;
                    }
                }

                for i in 0..orders.len() {
                    for j in 0..orders[0].len() {
                        region.assign_advice(
                            || "o",
                            self.config.orders[i],
                            i,
                            || Value::known(orders[i][j]),
                        )?;
                    }
                }
                for i in 0..customer.len() {
                    for j in 0..customer[0].len() {
                        region.assign_advice(
                            || "n",
                            self.config.customer[i],
                            i,
                            || Value::known(customer[i][j]),
                        )?;
                    }
                }

                // select l_orderkey from lineitem group by l_orderkey having sum(l_quantity) > :1
                let mut l_combined: Vec<Vec<_>> = (0..lineitem[0].len())
                    .map(|i| lineitem.iter().map(|row| row[i]).collect())
                    .collect();

                let o_combined: Vec<Vec<_>> = (0..orders[0].len())
                    .map(|i| orders.iter().map(|row| row[i]).collect())
                    .collect();
                let c_combined: Vec<Vec<_>> = (0..customer[0].len())
                    .map(|i| customer.iter().map(|row| row[i]).collect())
                    .collect();

                l_combined.sort_by_key(|v| v[0]);
                let mut orderkey_sums = Vec::<Vec<_>>::new();
                let mut sum = F::from(0);
                let mut current_key = F::from(0);
                for v in &l_combined {
                    let orderkey = v[0];
                    let quantity = v[1];
                    if orderkey != current_key {
                        if current_key != F::from(0) {
                            orderkey_sums.push(vec![current_key, sum]);
                        }
                        current_key = orderkey;
                        sum = F::from(0);
                    }
                    sum += quantity;
                }
                orderkey_sums.push(vec![current_key, sum]);
                let sum_quantity: Vec<Vec<_>> = orderkey_sums
                    .into_iter()
                    .filter(|v| v[1] > l_condition)
                    .collect();

                for i in 0..sum_quantity.len() {
                    region.assign_advice(
                        || "n",
                        self.config.sum_quantity[0],
                        i,
                        || Value::known(sum_quantity[i][0]),
                    )?;

                    region.assign_advice(
                        || "n",
                        self.config.sum_quantity[1],
                        i,
                        || Value::known(sum_quantity[i][1]),
                    )?;

                    l_cond_chip.assign(
                        &mut region,
                        i,
                        Value::known(l_condition),
                        Value::known(sum_quantity[i][1]),
                    )?;
                }

                //create the values for join and disjoin
                let mut join_value: Vec<Vec<_>> = vec![Default::default(); 4];
                let mut disjoin_value: Vec<Vec<_>> = vec![Default::default(); 4];
                // c_custkey = o_custkey
                // and o_orderkey = l_orderkey

                let mut combined = Vec::new();
                combined.push(sum_quantity); // its length is 2
                combined.push(o_combined); // 4
                combined.push(c_combined); // 2

                // (input1 index, input2 index, join attribute index of input1, join attribute of input2)
                let index = [
                    (0, 1, 0, 1), //   o_orderkey = l_orderkey
                    (1, 2, 2, 1), // c_custkey = o_custkey
                ];

                for i in 0..index.len() {
                    for val in combined[index[i].0].iter() {
                        if let Some(_) = combined[index[i].1]
                            .iter()
                            .find(|v| v[index[i].3] == val[index[i].2])
                        {
                            join_value[i * 2].push(val); // join values
                        } else {
                            disjoin_value[i * 2].push(val); // disjoin values
                        }
                    }
                    for val in combined[index[i].1].iter() {
                        if let Some(_) = combined[index[i].0]
                            .iter()
                            .find(|v| v[index[i].2] == val[index[i].3])
                        {
                            join_value[i * 2 + 1].push(val);
                        } else {
                            disjoin_value[i * 2 + 1].push(val);
                        }
                    }
                }

                // compute final table by applying all joins
                let join_index = [
                    (0, 1, 0, 1),     //   o_orderkey = l_orderkey
                    (1, 2, 2 + 2, 1), // c_custkey = o_custkey
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

                println!(
                    "product: {:?}",
                    cartesian_product,
                    // cartesian_product[0].len()
                );

                // group by c_name, c_custkey, o_orderkey, o_orderdate, o_totalprice
                cartesian_product.sort_by_key(|v| v[2] + v[3] + v[5] + v[6] + v[7]);

                // order by o_totalprice desc, o_orderdate;
                cartesian_product.sort_by(|a, b| match b[5].cmp(&a[5]) {
                    // o_totalprice desc
                    Ordering::Equal => a[2].cmp(&b[2]), //  o_orderdate; in asending order by default
                    other => other,
                });

                // for i in 0..cartesian_product.len() {
                //     let value_21 = cartesian_product[i][21];
                //     let new_value = if value_21 == x {
                //         cartesian_product[i][22]
                //     } else {
                //         cartesian_product[i][0]
                //     };

                //     cartesian_product[i].push(new_value);
                // }

                // for i in 0..cartesian_product.len() {
                //     if i == 0 {
                //         self.config.q_enable[1].enable(&mut region, i)?;
                //         revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
                //         revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
                //     // l_extendedprice * (1 - l_discount)
                //     } else {
                //         self.config.q_sort.enable(&mut region, i)?;
                //         groupby_sort_chip.assign(
                //             &mut region,
                //             i,
                //             &[cartesian_product5[i - 1].10],
                //             &[cartesian_product5[i].10],
                //         )?;

                //         // check if it is the first value
                //         if cartesian_product5[i - 1].10 == cartesian_product5[i].10 {
                //             self.config.q_first.enable(&mut region, i)?;
                //             revenue.push(cartesian_product5[i].1 * cartesian_product5[i].2);
                //         } else {
                //             self.config.q_nonfirst.enable(&mut region, i)?;
                //             revenue.push(
                //                 revenue[i - 1] + cartesian_product5[i].1 * cartesian_product5[i].2,
                //             );
                //         }
                //     }
                //     // assign revenue column
                //     region.assign_advice(
                //         || "revenue",
                //         self.config.revenue,
                //         i,
                //         || Value::known(revenue[i]),
                //     )?;

                //     region.assign_advice(
                //         || "groupby_name",
                //         self.config.groupby_name,
                //         i,
                //         || Value::known(cartesian_product5[i].10),
                //     )?;

                //     region.assign_advice(
                //         || "groupby_l_extendedprice",
                //         self.config.groupby_extendedprice,
                //         i,
                //         || Value::known(cartesian_product5[i].1),
                //     )?;

                //     region.assign_advice(
                //         || "groupby_l_discount",
                //         self.config.groupby_discount,
                //         i,
                //         || Value::known(cartesian_product5[i].2),
                //     )?;
                // }

                // // generate revenue_final
                // // println!("product: {:?}", cartesian_product5);
                // let mut revenue_final = Vec::new(); // by removing intermediate revenue values, i.e. only keep the final revenue of each group
                // if revenue.len() > 0 {
                //     for i in 0..revenue.len() - 1 {
                //         if cartesian_product5[i].10 != cartesian_product5[i + 1].10 {
                //             revenue_final.push((cartesian_product5[i].10, revenue[i]))
                //         }
                //     }

                //     revenue_final.push((
                //         cartesian_product5[revenue.len() - 1].10,
                //         revenue[revenue.len() - 1],
                //     ));
                // }

                // // order by revenue desc

                // revenue_final.sort_by_key(|&(value, _)| Reverse(value));

                // // assign values of equal check for verifying if revenue_final is sorted
                // let mut equal_check: Vec<F> = Vec::new();

                // if revenue_final.len() == 1 {
                //     equal_check.push(F::from(0)); // 0 assigned to the first value in equal_check
                // } else {
                //     equal_check.push(F::from(0));
                //     for i in 1..revenue_final.len() {
                //         if revenue_final[i] == revenue_final[i - 1] {
                //             equal_check.push(F::from(1));
                //         } else {
                //             equal_check.push(F::from(0))
                //         }
                //     }
                // }
                // // println!("revenue: {:?}", revenue_final);
                // // println!("equal check: {:?}", equal_check);

                // // assign sorted revenue and orderdate
                // for i in 0..revenue_final.len() {
                //     region.assign_advice(
                //         || "sorted_revenue",
                //         self.config.sorted_revenue,
                //         i,
                //         || Value::known(revenue_final[i].1),
                //     )?;

                //     region.assign_advice(
                //         || "equal_check",
                //         self.config.equal_check,
                //         i,
                //         || Value::known(equal_check[i]),
                //     )?;

                //     if i != 0 {
                //         self.config.q_sort_final.enable(&mut region, i)?;
                //         lt_revenue_final_chip.assign(
                //             &mut region,
                //             i,
                //             &[revenue_final[i].1],
                //             &[revenue_final[i - 1].1],
                //         )?;
                //     }
                // }

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

struct MyCircuit<F: Copy> {
    lineitem: [[F; N]; 2],
    orders: [[F; N]; 4],
    customer: [[F; N]; 2],

    pub l_condition: F,
    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            lineitem: [Default::default(); 2],
            orders: [Default::default(); 4],
            customer: [Default::default(); 2],

            l_condition: Default::default(),
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
            self.lineitem,
            self.orders,
            self.customer,
            self.l_condition,
        )?;

        // for (i, cell) in out_b_cells.iter().enumerate() {
        //     test_chip.expose_public(&mut layouter, cell.clone(), i)?;
        // }

        // test_chip.expose_public(&mut layouter, out_b_cell, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    use super::N;
    // use rand::Rng;
    // use halo2_proofs::poly::commitment::Params
    use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

    use std::marker::PhantomData;

    #[test]
    fn test_1() {
        let k = 10;
        // let mut rng = rand::thread_rng();

        let lineitem = [[Fp::from(2); N]; 2];
        let orders = [[Fp::from(2); N]; 4];
        let customer = [[Fp::from(2); N]; 2];

        let l_condition = Fp::from(1);

        let circuit = MyCircuit::<Fp> {
            lineitem,
            orders,
            customer,

            l_condition,
            _marker: PhantomData,
        };

        // let prover = MockProver::run(k, &circuit, vec![z.to_vec()]).unwrap();
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }
}
// time cargo test --package halo2-experiments --lib -- sql::q3_final_v1::tests::test_1 --exact --nocapture
