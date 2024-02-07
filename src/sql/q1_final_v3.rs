// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
// use std::{default, marker::PhantomData};

// // use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
// use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
// use gadgets::lessthan_or_equal_generic::{
//     LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
// };
// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// use std::cmp::Ordering;
// use std::time::Instant;

// const N: usize = 10;
// const NUM_BYTES: usize = 5;

// // #[derive(Default)]
// // We should use the selector to skip the row which does not satisfy shipdate values

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: Selector,
//     q_accu: Selector,
//     q_sort: Selector,
//     q_cond: Selector,

//     // lineitem: [Column<Advice>; 7], // l_quanity, l_exten, l_dis, l_tax, l_ret, l_linest, l_ship
//     lineitem: Vec<Column<Advice>>,
//     groupby: Vec<Column<Advice>>,

//     sum_qty: Column<Advice>,
//     sum_base_price: Column<Advice>,
//     sum_disc_price: Column<Advice>,
//     sum_charge: Column<Advice>,
//     sum_discount: Column<Advice>,

//     right_shipdate: Column<Advice>,

//     // groupby: [Column<Advice>; 7],
//     check: Column<Advice>,
//     equal_check: Column<Advice>,
//     count_check: Column<Advice>,

//     equal_v1: IsZeroConfig<F>,
//     equal_condition: Vec<IsZeroV2Config<F>>,

//     compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
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
//         let q_enable = meta.complex_selector();
//         let q_accu = meta.complex_selector();
//         let q_sort = meta.complex_selector();
//         let q_cond = meta.complex_selector();

//         // let lineitem = [meta.advice_column(); 7];
//         // let groupby = [meta.advice_column(); 7];
//         let mut lineitem = Vec::new();
//         let mut groupby = Vec::new();
//         for _ in 0..7 {
//             lineitem.push(meta.advice_column());
//             groupby.push(meta.advice_column());
//         }

//         let right_shipdate = meta.advice_column();

//         let sum_qty = meta.advice_column();
//         let sum_base_price = meta.advice_column();
//         let sum_disc_price = meta.advice_column();
//         let sum_charge = meta.advice_column();
//         let sum_discount = meta.advice_column();

//         let check = meta.advice_column();
//         let equal_check = meta.advice_column();
//         let count_check = meta.advice_column();

//         let is_zero_advice_column1 = meta.advice_column();

//         let mut is_zero_vectors = Vec::new();
//         for _ in 0..2 {
//             is_zero_vectors.push(meta.advice_column());
//         }

//         let constant = meta.fixed_column();
//         // let instance = meta.instance_column();

//         for i in 0..7 {
//             meta.enable_equality(lineitem[i]);
//             meta.enable_equality(groupby[i]);
//         }

//         meta.enable_equality(right_shipdate);
//         // meta.enable_equality(instance);

//         meta.enable_equality(sum_qty);
//         meta.enable_equality(sum_base_price);
//         meta.enable_equality(sum_disc_price);
//         meta.enable_equality(sum_charge);
//         meta.enable_equality(sum_discount);
//         meta.enable_constant(constant);

//         // constraints for l_shipdate <= date '1998-12-01' - interval ':1' day (3)
//         let mut compare_condition = Vec::new();

//         let config = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| vec![meta.query_advice(lineitem[6], Rotation::cur())],
//             |meta| vec![meta.query_advice(right_shipdate, Rotation::cur())],
//         );
//         compare_condition.push(config.clone());
//         meta.create_gate(
//             "verifies l_shipdate <= date '1998-12-01' - interval ':1' day (3)",
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);
//                 let check = meta.query_advice(check, Rotation::cur());
//                 vec![q_enable * (config.is_lt(meta, None) - check)]
//             },
//         );

//         // permutation check for groupby

//         // groupby and order by l_returnflag, l_linestatus;
//         //

//         let equal_v1 = IsZeroChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| {
//                 meta.query_advice(groupby[4], Rotation::prev())
//                     - meta.query_advice(groupby[4], Rotation::cur())
//             },
//             is_zero_advice_column1,
//         );

//         // l_returnflag[i-1] <= l_returnflag[i]
//         let config_lt = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| vec![meta.query_advice(groupby[4], Rotation::prev())],
//             |meta| vec![meta.query_advice(groupby[4], Rotation::cur())],
//         );
//         compare_condition.push(config_lt);
//         // l_linestatus[i-1] <= l_linestatus[i]
//         let config_lt_1 = LtEqGenericChip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| vec![meta.query_advice(groupby[5], Rotation::prev())],
//             |meta| vec![meta.query_advice(groupby[5], Rotation::cur())],
//         );
//         compare_condition.push(config_lt_1);

//         meta.create_gate("verifies orderby scenarios", |meta| {
//             let q_sort = meta.query_selector(q_sort);
//             // let output = meta.query_advice(equal_check[0], Rotation::cur());
//             vec![
//                 q_sort.clone()
//                         * (config_lt.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
//                         * (equal_v1.expr() * config_lt_1.is_lt(meta, None)
//                             - Expression::Constant(F::ONE)),
//             ]
//         });

//         // equal_check
//         let mut equal_condition = Vec::new();
//         let config = IsZeroV2Chip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| {
//                 vec![
//                     meta.query_advice(groupby[4], Rotation::prev())  // p_size
//                     - meta.query_advice(groupby[4], Rotation::cur()),
//                     meta.query_advice(groupby[5], Rotation::prev())  // p_size
//                     - meta.query_advice(groupby[5], Rotation::cur()),
//                 ]
//             },
//             is_zero_vectors, // s_acctbal[i] = s_acctbal[i-1]
//         );
//         equal_condition.push(config.clone());
//         meta.create_gate("equal_check", |meta| {
//             let q_sort = meta.query_selector(q_sort);
//             let check = meta.query_advice(equal_check, Rotation::cur());
//             vec![q_sort.clone() * (equal_v1.expr() - check)]
//         });
//         meta.create_gate("count_check", |meta| {
//             let q_sort = meta.query_selector(q_sort);
//             let prev_count = meta.query_advice(count_check, Rotation::prev());
//             let count_check = meta.query_advice(count_check, Rotation::cur());
//             let equal_check = meta.query_advice(equal_check, Rotation::cur());
//             vec![
//                 q_sort.clone()
//                     * (prev_count * equal_check + Expression::Constant(F::ONE) - count_check),
//             ]
//         });

//         //sum gate
//         meta.create_gate("accumulate constraint", |meta| {
//             let q_accu = meta.query_selector(q_accu);
//             let prev_b_sum_qty = meta.query_advice(sum_qty, Rotation::cur());
//             let prev_b_sum_base_price = meta.query_advice(sum_base_price, Rotation::cur());
//             let prev_b_sum_disc_price = meta.query_advice(sum_disc_price, Rotation::cur());
//             let prev_b_sum_charge = meta.query_advice(sum_charge, Rotation::cur());
//             let prev_b_sum_discount = meta.query_advice(sum_discount, Rotation::cur());

//             let quantity = meta.query_advice(groupby[0], Rotation::cur());
//             let extendedprice = meta.query_advice(groupby[1], Rotation::cur());
//             let discount = meta.query_advice(groupby[2], Rotation::cur());
//             let tax = meta.query_advice(groupby[3], Rotation::cur());

//             let sum_qty = meta.query_advice(sum_qty, Rotation::next());
//             let sum_base_price = meta.query_advice(sum_base_price, Rotation::next());
//             let sum_disc_price = meta.query_advice(sum_disc_price, Rotation::next());
//             let sum_charge = meta.query_advice(sum_charge, Rotation::next());
//             let sum_discount = meta.query_advice(sum_discount, Rotation::next());

//             let check = meta.query_advice(equal_check, Rotation::cur());

//             vec![
//                 q_accu.clone()
//                     * (check.clone() * prev_b_sum_qty.clone() + quantity.clone() - sum_qty.clone()),
//                 q_accu.clone()
//                     * (check.clone() * prev_b_sum_base_price + extendedprice.clone()
//                         - sum_base_price),
//                 q_accu.clone()
//                     * (check.clone() * prev_b_sum_disc_price
//                         + extendedprice.clone()
//                             * (Expression::Constant(F::from(1000)) - discount.clone())
//                         - sum_disc_price),
//                 q_accu.clone()
//                     * (check.clone() * prev_b_sum_charge
//                         + extendedprice
//                             * (Expression::Constant(F::from(1000)) - discount.clone())
//                             * (Expression::Constant(F::from(1000)) + tax)
//                         - sum_charge),
//                 q_accu * (check.clone() * prev_b_sum_discount + discount - sum_discount),
//             ]
//         });

//         // groupby permutation
//         meta.shuffle("permutation check", |meta| {
//             // Inputs
//             let q = meta.query_selector(q_cond);
//             let lineitem_queries: Vec<_> = lineitem
//                 .iter()
//                 .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//                 .collect();

//             let groupby_queries: Vec<_> = groupby
//                 .iter()
//                 .map(|&idx| meta.query_advice(idx, Rotation::cur()))
//                 .collect();

//             let constraints: Vec<_> = lineitem_queries
//                 .iter()
//                 .zip(groupby_queries.iter())
//                 .map(|(input, table)| (q.clone() * input.clone(), table.clone()))
//                 .collect();

//             constraints
//         });

//         TestCircuitConfig {
//             q_enable,
//             q_accu,
//             q_sort,
//             lineitem,
//             groupby,
//             right_shipdate,
//             equal_condition,
//             check,
//             equal_check,
//             count_check,
//             sum_qty,
//             compare_condition,
//             q_cond,
//             sum_base_price,
//             sum_disc_price,
//             sum_charge,
//             equal_v1,
//             sum_discount,
//         }
//     }

//     pub fn assign(
//         &self,
//         // layouter: &mut impl Layouter<F>,
//         layouter: &mut impl Layouter<F>,
//         lineitem: Vec<Vec<F>>,
//         right_shipdate: F,
//     ) -> Result<(), Error> {
//         // Result<AssignedCell<F, F>, Error> {
//         let equal_v1_chip = IsZeroChip::construct(self.config.equal_v1.clone());
//         let equal_condition_chip = IsZeroV2Chip::construct(self.config.equal_condition[0].clone());
//         let mut compare_chip = Vec::new();
//         for i in 0..self.config.compare_condition.len() {
//             let chip = LtEqGenericChip::construct(self.config.compare_condition[i].clone());
//             chip.load(layouter)?;
//             compare_chip.push(chip);
//         }
//         // println!("{:?}", self.config.compare_condition.len());

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 let start = Instant::now();

//                 let mut l_check = Vec::new();

//                 for i in 0..lineitem.len() {
//                     self.config.q_enable.enable(&mut region, i)?;
//                     for j in 0..lineitem[0].len() {
//                         region.assign_advice(
//                             || "l",
//                             self.config.lineitem[j],
//                             i,
//                             || Value::known(lineitem[i][j]),
//                         )?;
//                     }

//                     if lineitem[i][6] <= right_shipdate {
//                         l_check.push(true);
//                     } else {
//                         l_check.push(false);
//                     }

//                     region.assign_advice(
//                         || "check",
//                         self.config.check,
//                         i,
//                         || Value::known(F::from(l_check[i] as u64)),
//                     )?;
//                     // only focus on the values after filtering
//                     if l_check[i] == true {
//                         self.config.q_cond.enable(&mut region, i)?;
//                     }

//                     region.assign_advice(
//                         || "right_shipdate value",
//                         self.config.right_shipdate,
//                         i,
//                         || Value::known(right_shipdate),
//                     )?;

//                     compare_chip[0].assign(&mut region, i, &[lineitem[i][6]], &[right_shipdate])?;
//                 }

//                 let duration_block = start.elapsed();
//                 println!("Time elapsed for block: {:?}", duration_block);

//                 let mut l_combined: Vec<Vec<_>> = lineitem
//                     .clone()
//                     .into_iter()
//                     .filter(|row| row[6] <= right_shipdate) // l_shipdate <= date '1998-12-01' - interval ':1' day (3)
//                     .collect();

//                 //     group by
//                 //     l_returnflag,
//                 //     l_linestatus
//                 // order by
//                 //     l_returnflag,
//                 //     l_linestatus;

//                 l_combined.sort_by(|a, b| match a[4].cmp(&b[4]) {
//                     Ordering::Equal => a[5].cmp(&b[5]),
//                     other => other,
//                 });

//                 // println!("l2: {:?}", l_combined.len());

//                 let duration_block = start.elapsed();
//                 println!("Time elapsed for block: {:?}", duration_block);

//                 for i in 0..l_combined.len() {
//                     for j in 0..l_combined[0].len() {
//                         region.assign_advice(
//                             || "",
//                             self.config.groupby[j],
//                             i,
//                             || Value::known(l_combined[i][j]),
//                         )?;
//                     }
//                     if i > 0 {
//                         self.config.q_sort.enable(&mut region, i)?; // groupby sort assignment
//                         compare_chip[1].assign(
//                             &mut region,
//                             i,
//                             &[l_combined[i - 1][4]],
//                             &[l_combined[i][4]],
//                         )?;

//                         compare_chip[2].assign(
//                             &mut region,
//                             i,
//                             &[l_combined[i - 1][5]],
//                             &[l_combined[i][5]],
//                         )?;

//                         equal_v1_chip.assign(
//                             &mut region,
//                             i,
//                             Value::known(l_combined[i - 1][4] - l_combined[i][4]),
//                         )?;
//                         equal_condition_chip.assign(
//                             &mut region,
//                             i,
//                             (
//                                 Value::known(l_combined[i - 1][4] - l_combined[i][4]),
//                                 Value::known(l_combined[i - 1][5] - l_combined[i][5]),
//                             ),
//                         )?;
//                     }
//                 }
//                 // equal check between config.groupby and config.lineitem
//                 // region.constrain_equal(left, right);

//                 // 0 represents this row is the first one of a group, and 1 otherwise
//                 let mut equal_check: Vec<F> = Vec::new();
//                 let mut count_check: Vec<F> = Vec::new();
//                 if l_combined.len() > 0 {
//                     equal_check.push(F::from(0)); // add the the first one must be 0
//                     count_check.push(F::from(1)); // add the count of the first dataitem as 1
//                 }

//                 for row in 1..l_combined.len() {
//                     if l_combined[row][4] == l_combined[row - 1][4] {
//                         equal_check.push(F::from(1));
//                         let count_value = *count_check.last().unwrap() + F::from(1);
//                         count_check.push(count_value);
//                     } else {
//                         equal_check.push(F::from(0));
//                         count_check.push(F::from(1));
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
//                     region.assign_advice(
//                         || "count_check",
//                         self.config.count_check,
//                         i,
//                         || Value::known(count_check[i]),
//                     )?;
//                 }

//                 let n = l_combined.len() + 1;
//                 let mut sum_qty: Vec<F> = vec![F::from(0); n];
//                 let mut sum_base: Vec<F> = vec![F::from(0); n];
//                 let mut sum_disc: Vec<F> = vec![F::from(0); n];
//                 let mut sum_charge: Vec<F> = vec![F::from(0); n];
//                 let mut sum_discount: Vec<F> = vec![F::from(0); n];

//                 for i in 1..n {
//                     sum_qty[i] = sum_qty[i - 1] * equal_check[i - 1] + l_combined[i - 1][0];
//                     sum_base[i] = sum_base[i - 1] * equal_check[i - 1] + l_combined[i - 1][1];
//                     sum_disc[i] = sum_disc[i - 1] * equal_check[i - 1]
//                         + l_combined[i - 1][1] * (F::from(1000) - l_combined[i - 1][2]);
//                     sum_charge[i] = sum_charge[i - 1] * equal_check[i - 1]
//                         + l_combined[i - 1][1]
//                             * (F::from(1000) - l_combined[i - 1][2])
//                             * (F::from(1000) + l_combined[i - 1][3]);
//                     sum_discount[i] =
//                         sum_discount[i - 1] * equal_check[i - 1] + l_combined[i - 1][2];
//                 }

//                 for i in 0..n {
//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.sum_qty,
//                         i,
//                         || Value::known(sum_qty[i]),
//                     )?;
//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.sum_base_price,
//                         i,
//                         || Value::known(sum_base[i]),
//                     )?;
//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.sum_disc_price,
//                         i,
//                         || Value::known(sum_disc[i]),
//                     )?;
//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.sum_charge,
//                         i,
//                         || Value::known(sum_charge[i]),
//                     )?;
//                     region.assign_advice(
//                         || "equal_check",
//                         self.config.sum_discount,
//                         i,
//                         || Value::known(sum_discount[i]),
//                     )?;
//                 }
//                 let duration_block = start.elapsed();
//                 println!("Time elapsed for block: {:?}", duration_block);

//                 Ok(())
//                 // Ok(mem::replace(&mut equal_or_not_values, Vec::new()))
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
//     pub lineitem: Vec<Vec<F>>,
//     pub right_shipdate: F,

//     _marker: PhantomData<F>,
// }

// impl<F: Copy + Default> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             lineitem: Vec::new(),
//             right_shipdate: Default::default(),
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

//         let out_b_cells =
//             test_chip.assign(&mut layouter, self.lineitem.clone(), self.right_shipdate)?;

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

//         // let lineitem_file_path = "/Users/binbingu/halo2-TPCH/src/data/lineitem.tbl";
//         let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";
//         let mut lineitem: Vec<Vec<Fp>> = Vec::new();

//         if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
//             // Convert the Vec<Region> to a 2D vector
//             lineitem = records
//                 .iter()
//                 .map(|record| {
//                     vec![
//                         Fp::from(record.l_quantity),
//                         Fp::from(scale_by_1000(record.l_extendedprice)),
//                         Fp::from(scale_by_1000(record.l_discount)),
//                         Fp::from(scale_by_1000(record.l_tax)),
//                         Fp::from(string_to_u64(&record.l_returnflag)),
//                         Fp::from(string_to_u64(&record.l_linestatus)),
//                         Fp::from(date_to_timestamp(&record.l_shipdate)),
//                         // Fp::from(string_to_u64(&record.l_shipdate)),
//                     ]
//                 })
//                 .collect();
//         }
//         // println!("lineitem length: {:?}", lineitem[0]);

//         let right_shipdate = Fp::from(902102400);
//         // l_shipdate <= date '1998-08-03'
//         // let right_shipdate = Fp::from(2730);

//         let lineitem: Vec<Vec<Fp>> = lineitem.iter().take(10000).cloned().collect();

//         let circuit = MyCircuit::<Fp> {
//             lineitem,

//             right_shipdate,

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
