// use eth_types::Field;
// // use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
// use gadgets::lessthan_or_equal::{LtEqChip, LtEqConfig, LtEqInstruction};
// use gadgets::lessthan_or_equal_v1::{LtEqV1Chip, LtEqV1Config, LtEqV1Instruction};
// use std::{default, marker::PhantomData};

// // use crate::chips::is_zero_v1::{IsZeroChip, IsZeroConfig};
// use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
// use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation};

// use std::mem;

// const N: usize = 10;

// // #[derive(Default)]
// // define circuit struct using array of usernames and balances

// #[derive(Clone, Debug)]
// pub struct TestCircuitConfig<F: Field> {
//     q_enable: halo2_proofs::plonk::Selector,
//     q_accu: Selector,
//     q_sort: Selector,
//     q_equal: Selector,

//     l_quantity: Column<Advice>,
//     l_extendedprice: Column<Advice>,
//     l_discount: Column<Advice>,
//     l_tax: Column<Advice>,

//     sum_qty: Column<Advice>,
//     sum_base_price: Column<Advice>,
//     sum_disc_price: Column<Advice>,
//     sum_charge: Column<Advice>,

//     l_returnflag: Column<Advice>,
//     l_linestatus: Column<Advice>,
//     perm_l_returnflag: Column<Advice>,
//     perm_l_linestatus: Column<Advice>,

//     // l_linestatus: Column<Advice>,
//     l_shipdate: Column<Advice>,
//     right_shipdate: Column<Advice>,

//     check: Column<Advice>,
//     check_sort: Column<Advice>,
//     equal_or_not: Column<Advice>,
//     // pub instance: Column<Instance>,
//     lt_right_shipdate: LtConfig<F, 3>,
//     // lt_returnflag_sort: LtEqConfig<F, 3>,
//     lt_returnflag_linestatus: LtEqV1Config<F, 3>,
//     a_equals_b: IsZeroV2Config<F>,
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
//         let q_equal = meta.complex_selector();

//         let l_quantity = meta.advice_column();
//         let l_extendedprice = meta.advice_column();
//         let l_discount = meta.advice_column();
//         let l_tax = meta.advice_column();

//         let l_returnflag = meta.advice_column();
//         let perm_l_returnflag = meta.advice_column();
//         let l_linestatus = meta.advice_column();
//         let perm_l_linestatus = meta.advice_column();

//         let l_shipdate = meta.advice_column();
//         let right_shipdate = meta.advice_column();

//         let sum_qty = meta.advice_column();
//         let sum_base_price = meta.advice_column();
//         let sum_disc_price = meta.advice_column();
//         let sum_charge = meta.advice_column();

//         let check = meta.advice_column();
//         let check_sort = meta.advice_column();
//         let is_zero_advice_column = meta.advice_column();
//         let equal_or_not = meta.advice_column();

//         let constant = meta.fixed_column();
//         // let instance = meta.instance_column();

//         meta.enable_equality(l_quantity);
//         meta.enable_equality(l_extendedprice);
//         meta.enable_equality(l_discount);
//         meta.enable_equality(l_tax);
//         meta.enable_equality(l_returnflag);
//         meta.enable_equality(perm_l_returnflag);
//         meta.enable_equality(l_linestatus);
//         meta.enable_equality(perm_l_linestatus);
//         // meta.enable_equality(is_zero_advice_column);
//         meta.enable_equality(l_shipdate);
//         meta.enable_equality(right_shipdate);
//         // meta.enable_equality(instance);

//         meta.enable_equality(sum_qty);
//         meta.enable_equality(sum_base_price);
//         meta.enable_equality(sum_disc_price);
//         meta.enable_equality(sum_charge);

//         meta.enable_equality(equal_or_not);

//         meta.enable_constant(constant);

//         let lt_right_shipdate = LtChip::configure(
//             meta,
//             |meta| meta.query_selector(q_enable),
//             |meta| meta.query_advice(l_shipdate, Rotation::cur()),
//             |meta| meta.query_advice(right_shipdate, Rotation::cur()), // we put the left and right value at the first two positions of value_l
//         );

//         // let lt_returnflag_sort = LtEqChip::configure(
//         //     meta,
//         //     |meta| meta.query_selector(q_sort),
//         //     |meta| meta.query_advice(perm_l_returnflag, Rotation::prev()),
//         //     |meta| meta.query_advice(perm_l_returnflag, Rotation::cur()),
//         // );

//         let lt_returnflag_linestatus = LtEqV1Chip::configure(
//             meta,
//             |meta| meta.query_selector(q_sort),
//             |meta| meta.query_advice(perm_l_returnflag, Rotation::prev()),
//             |meta| meta.query_advice(perm_l_linestatus, Rotation::prev()),
//             |meta| meta.query_advice(perm_l_returnflag, Rotation::cur()),
//             |meta| meta.query_advice(perm_l_linestatus, Rotation::cur()),
//         );

//         let a_equals_b = IsZeroV2Chip::configure(
//             meta,
//             |meta| meta.query_selector(q_equal), // this is the q_enable
//             |meta| {
//                 meta.query_advice(perm_l_returnflag, Rotation::cur())
//                     - meta.query_advice(perm_l_returnflag, Rotation::prev())
//             }, // this is the value
//             is_zero_advice_column,               // this is the advice column that stores value_inv
//         );

//         meta.create_gate(
//             "verifies that `check` current confif = is_lt from LtChip ",
//             |meta| {
//                 let q_enable = meta.query_selector(q_enable);

//                 // This verifies lt(value_l::cur, value_r::cur) is calculated correctly
//                 let check = meta.query_advice(check, Rotation::cur());

//                 vec![q_enable * (lt_right_shipdate.is_lt(meta, None) - check)]
//             },
//         );

//         // check the values in the two columns are sorted (i.e. tuple sorted)
//         meta.create_gate("verifies that t[i-1] <= t[i]", |meta| {
//             let q_sort = meta.query_selector(q_sort);

//             // This verifies lt(value_l::cur, value_r::cur) is calculated correctly
//             let check_sort = meta.query_advice(check_sort, Rotation::cur());

//             vec![q_sort * (lt_returnflag_linestatus.is_lt(meta, None) - check_sort)]
//         });

//         // sum gate
//         meta.create_gate("accumulate constraint", |meta| {
//             let q_accu = meta.query_selector(q_accu);
//             let prev_b_sum_qty = meta.query_advice(sum_qty, Rotation::cur());
//             let prev_b_sum_base_price = meta.query_advice(sum_base_price, Rotation::cur());
//             let prev_b_sum_disc_price = meta.query_advice(sum_disc_price, Rotation::cur());
//             let prev_b_sum_charge = meta.query_advice(sum_charge, Rotation::cur());

//             let quantity = meta.query_advice(l_quantity, Rotation::cur());
//             let extendedprice = meta.query_advice(l_extendedprice, Rotation::cur());
//             let discount = meta.query_advice(l_discount, Rotation::cur());
//             let tax = meta.query_advice(l_tax, Rotation::cur());

//             let sum_qty = meta.query_advice(sum_qty, Rotation::next());
//             let sum_base_price = meta.query_advice(sum_base_price, Rotation::next());
//             let sum_disc_price = meta.query_advice(sum_disc_price, Rotation::next());
//             let sum_charge = meta.query_advice(sum_charge, Rotation::next());

//             let check = meta.query_advice(check, Rotation::cur());

//             vec![
//                 q_accu.clone() * (quantity * check.clone() + prev_b_sum_qty - sum_qty),
//                 q_accu.clone()
//                     * (extendedprice.clone() * check.clone() + prev_b_sum_base_price
//                         - sum_base_price),
//                 q_accu.clone()
//                     * (extendedprice.clone() * discount.clone() * check.clone()
//                         + prev_b_sum_disc_price
//                         - sum_disc_price),
//                 q_accu.clone()
//                     * (extendedprice * discount * tax * check.clone() + prev_b_sum_charge
//                         - sum_charge),
//             ]
//         });

//         meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
//             let s = meta.query_selector(q_equal);
//             // a  |  b  | s      |a == b | output  |  s * (a == b) * (output - 1) | s * (1 - a == b) * (output - 0)
//             // --------------------------------
//             // 10 | 12  | 1      | 0     |  0      | 1 * 0 * -1                   | 1 * 1 * 0 = 0
//             // 10 | 10  | 1      | 1     |  1      | 1 * 1 * 0 (output == 1)      | 1 * 0 * 1 = 0
//             let output = meta.query_advice(equal_or_not, Rotation::cur());
//             vec![
//                 s.clone() * (a_equals_b.expr() * (output.clone() - Expression::Constant(F::one()))), // in this case output == 1
//                 s * (Expression::Constant(F::one()) - a_equals_b.expr()) * (output), // in this case output == 0
//             ]
//         });
//         // let num = meta.num_advice_columns();
//         // println!("Number of columns: {}", num);

//         TestCircuitConfig {
//             q_enable,
//             q_accu,
//             q_sort,
//             l_quantity,
//             l_linestatus,
//             perm_l_linestatus,
//             l_shipdate,
//             right_shipdate,
//             // instance,
//             check,
//             check_sort,
//             sum_qty,
//             l_returnflag,
//             lt_right_shipdate,
//             lt_returnflag_linestatus,
//             perm_l_returnflag,
//             a_equals_b,
//             equal_or_not,
//             q_equal,
//             l_extendedprice,
//             l_discount,
//             sum_base_price,
//             sum_disc_price,
//             sum_charge,
//             l_tax,
//         }
//     }

//     pub fn assign(
//         &self,
//         // layouter: &mut impl Layouter<F>,
//         layouter: &mut impl Layouter<F>,
//         l_quantity: [u64; N],
//         l_extendedprice: [u64; N],
//         l_discount: [u64; N],
//         l_tax: [u64; N],
//         l_returnflag: [u64; N],
//         l_linestatus: [u64; N],
//         l_shipdate: [u64; N],
//         right_shipdate: u64,
//         check: [bool; N],
//         check_sort: [bool; N],
//     ) -> Result<Vec<AssignedCell<F, F>>, Error> {
//         // Result<AssignedCell<F, F>, Error> {
//         let chip1 = LtChip::construct(self.config.lt_right_shipdate);
//         let chip2 = LtEqV1Chip::construct(self.config.lt_returnflag_linestatus);
//         let is_zero_chip = IsZeroV2Chip::construct(self.config.a_equals_b.clone());

//         chip1.load(layouter)?;
//         chip2.load(layouter)?;

//         let mut equal_or_not_values: Vec<AssignedCell<F, F>> = Vec::with_capacity(N);

//         layouter.assign_region(
//             || "witness",
//             |mut region| {
//                 let t_cell = region.assign_advice_from_constant(
//                     || "first perm_l_returnflag check",
//                     self.config.equal_or_not,
//                     0,
//                     F::zero(),
//                 )?;
//                 equal_or_not_values.push(t_cell);

//                 for i in 0..N {
//                     self.config.q_enable.enable(&mut region, i)?;
//                     region.assign_advice(
//                         || "l_quantity value",
//                         self.config.l_quantity,
//                         i,
//                         || Value::known(F::from(l_quantity[i])),
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
//                         || "l_tax value",
//                         self.config.l_tax,
//                         i,
//                         || Value::known(F::from(l_tax[i])),
//                     )?;

//                     region.assign_advice(
//                         || "l_returnflag",
//                         self.config.l_returnflag,
//                         i,
//                         || Value::known(F::from(l_returnflag[i])),
//                     )?;

//                     region.assign_advice(
//                         || "l_returnflag",
//                         self.config.l_linestatus,
//                         i,
//                         || Value::known(F::from(l_linestatus[i])),
//                     )?;

//                     let mut perm_returnflag_linestatus: [(u64, u64); N] = [(0, 0); N];
//                     for k in 0..N {
//                         perm_returnflag_linestatus[k] = (l_returnflag[k], l_linestatus[k]);
//                     }
//                     perm_returnflag_linestatus.sort_by(
//                         |&(a_returnflag, a_linestatus), &(b_returnflag, b_linestatus)| {
//                             let sum_a = a_returnflag + a_linestatus;
//                             let sum_b = b_returnflag + b_linestatus;

//                             // Compare by the sum
//                             sum_a.cmp(&sum_b)
//                         },
//                     );

//                     // let mut perm_returnflag: [u64; N] = l_returnflag;
//                     // perm_returnflag.sort();
//                     region.assign_advice(
//                         || "perm_l_returnflag",
//                         self.config.perm_l_returnflag,
//                         i,
//                         || Value::known(F::from(perm_returnflag_linestatus[i].0)),
//                     )?;

//                     region.assign_advice(
//                         || "perm_l_linestatus",
//                         self.config.perm_l_linestatus,
//                         i,
//                         || Value::known(F::from(perm_returnflag_linestatus[i].1)),
//                     )?;

//                     region.assign_advice(
//                         || "l_shipdate value",
//                         self.config.l_shipdate,
//                         i,
//                         || Value::known(F::from(l_shipdate[i])),
//                     )?;

//                     region.assign_advice(
//                         || "right_shipdate value",
//                         self.config.right_shipdate,
//                         i,
//                         || Value::known(F::from(right_shipdate)),
//                     )?;

//                     region.assign_advice(
//                         || "check",
//                         self.config.check,
//                         i,
//                         || Value::known(F::from(check[i] as u64)),
//                     )?;

//                     region.assign_advice(
//                         || "check_sort",
//                         self.config.check_sort,
//                         i,
//                         || Value::known(F::from(check_sort[i] as u64)),
//                     )?;

//                     // let scalar_value = v.into_bits();
//                     chip1.assign(
//                         &mut region,
//                         i,
//                         F::from(l_shipdate[i]),
//                         F::from(right_shipdate),
//                     )?;

//                     if i != 0 {
//                         self.config.q_sort.enable(&mut region, i)?;
//                         self.config.q_equal.enable(&mut region, i)?;
//                         chip2.assign(
//                             &mut region,
//                             i,
//                             F::from(perm_returnflag_linestatus[i - 1].0),
//                             F::from(perm_returnflag_linestatus[i - 1].1),
//                             F::from(perm_returnflag_linestatus[i].0),
//                             F::from(perm_returnflag_linestatus[i].1),
//                         )?;
//                         // assign the values to the equal_or_not column
//                         // let a_cell = region.assign_advice(
//                         //     || "a",
//                         //     self.config.perm_l_returnflag,
//                         //     i - 1,
//                         //     || Value::known(F::from(perm_returnflag[i - 1])),
//                         // )?;
//                         // let b_cell = region.assign_advice(
//                         //     || "b",
//                         //     self.config.perm_l_returnflag,
//                         //     i,
//                         //     || Value::known(F::from(perm_returnflag[i])),
//                         // )?;

//                         is_zero_chip.assign(
//                             &mut region,
//                             i,
//                             // TO DO: make perm_returnflag_linestatus[i].0 - perm_returnflag_linestatus[i-1].0
//                             // and perm_returnflag_linestatus[i].1 - perm_returnflag_linestatus[i-1].1 hold at the same time
//                             Value::known(F::from(
//                                 perm_returnflag_linestatus[i].0
//                                     - perm_returnflag_linestatus[i - 1].0
//                                     + perm_returnflag_linestatus[i].1
//                                     - perm_returnflag_linestatus[i - 1].1,
//                             )),
//                         )?;

//                         let output = if ((perm_returnflag_linestatus[i].0
//                             == perm_returnflag_linestatus[i - 1].0)
//                             && (perm_returnflag_linestatus[i].1
//                                 == perm_returnflag_linestatus[i - 1].1))
//                         {
//                             F::from(1)
//                         } else {
//                             F::from(0)
//                         };
//                         let out_cell = region.assign_advice(
//                             || "output",
//                             self.config.equal_or_not,
//                             i,
//                             || Value::known(output),
//                         )?;
//                         equal_or_not_values.push(out_cell);
//                     }
//                 }
//                 // there is no previous value for the first value of returnflag, so we need to set it to 0, i.e. it is not equal to its prev()

//                 let mut prev_sum_qty = region.assign_advice_from_constant(
//                     || "first sum_qty accu",
//                     self.config.sum_qty,
//                     0,
//                     F::zero(),
//                 )?;

//                 let mut prev_sum_base = region.assign_advice_from_constant(
//                     || "first sum_base accu",
//                     self.config.sum_base_price,
//                     0,
//                     F::zero(),
//                 )?;

//                 let mut prev_sum_disc = region.assign_advice_from_constant(
//                     || "first sum_disc accu",
//                     self.config.sum_disc_price,
//                     0,
//                     F::zero(),
//                 )?;

//                 let mut prev_sum_charge = region.assign_advice_from_constant(
//                     || "first sum_charge accu",
//                     self.config.sum_charge,
//                     0,
//                     F::zero(),
//                 )?;

//                 // let mut prev_b = b0_cell.clone();
//                 for row in 1..N + 1 {
//                     // enable hash selector
//                     // if row != N {
//                     //     config.q_accu.enable(&mut region, row)?;
//                     // }
//                     self.config.q_accu.enable(&mut region, row - 1)?;

//                     // let mut perm_returnflag: [u64; N] = l_returnflag;
//                     // perm_returnflag.sort();
//                     let sum_qty_cell: AssignedCell<F, F> = region.assign_advice(
//                         || "sum_qty",
//                         self.config.sum_qty,
//                         row,
//                         || {
//                             prev_sum_qty.value().copied()
//                                 + Value::known(F::from(
//                                     l_quantity[row - 1] * (check[row - 1] as u64),
//                                 ))
//                         },
//                         // || {
//                         //     prev_b.value().copied()
//                         //         + Value::known(F::from(
//                         //             perm_returnflag[row - 1] * (check[row - 1] as u64),
//                         //         ))
//                         // },
//                     )?;

//                     let sum_base_cell: AssignedCell<F, F> = region.assign_advice(
//                         || "sum_base",
//                         self.config.sum_base_price,
//                         row,
//                         || {
//                             prev_sum_base.value().copied()
//                                 + Value::known(F::from(
//                                     l_extendedprice[row - 1] * (check[row - 1] as u64),
//                                 ))
//                         },
//                     )?;

//                     let sum_disc_cell: AssignedCell<F, F> = region.assign_advice(
//                         || "sum_disc",
//                         self.config.sum_disc_price,
//                         row,
//                         || {
//                             prev_sum_disc.value().copied()
//                                 + Value::known(F::from(
//                                     l_extendedprice[row - 1]
//                                         * l_discount[row - 1]
//                                         * (check[row - 1] as u64),
//                                 ))
//                         },
//                     )?;

//                     let sum_charge_cell: AssignedCell<F, F> = region.assign_advice(
//                         || "sum_charge",
//                         self.config.sum_charge,
//                         row,
//                         || {
//                             prev_sum_charge.value().copied()
//                                 + Value::known(F::from(
//                                     l_extendedprice[row - 1]
//                                         * l_discount[row - 1]
//                                         * l_tax[row - 1]
//                                         * (check[row - 1] as u64),
//                                 ))
//                         },
//                     )?;

//                     prev_sum_qty = sum_qty_cell;
//                     prev_sum_base = sum_base_cell;
//                     prev_sum_disc = sum_disc_cell;
//                     prev_sum_charge = sum_charge_cell;
//                 }
//                 // Ok(prev_b)
//                 Ok(mem::replace(&mut equal_or_not_values, Vec::new()))
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
//     pub l_quantity: [u64; N],
//     pub l_extendedprice: [u64; N],
//     pub l_discount: [u64; N],
//     pub l_tax: [u64; N],

//     pub l_returnflag: [u64; N],
//     pub l_linestatus: [u64; N],
//     pub l_shipdate: [u64; N],
//     pub right_shipdate: u64,
//     pub check: [bool; N],
//     pub check_sort: [bool; N],

//     _marker: PhantomData<F>,
// }

// impl<F> Default for MyCircuit<F> {
//     fn default() -> Self {
//         Self {
//             l_quantity: [0; N],
//             l_extendedprice: [0; N],
//             l_discount: [0; N],
//             l_tax: [0; N],
//             l_returnflag: [0; N],
//             l_linestatus: [0; N],
//             l_shipdate: [0; N],
//             right_shipdate: 0,
//             check: [false; N],
//             check_sort: [false; N],
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
//             self.l_quantity,
//             self.l_extendedprice,
//             self.l_discount,
//             self.l_tax,
//             self.l_returnflag,
//             self.l_linestatus,
//             self.l_shipdate,
//             self.right_shipdate,
//             self.check,
//             self.check_sort,
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
//     // use halo2_proofs::poly::commitment::Params
//     use halo2_proofs::{dev::MockProver, halo2curves::bn256::Fr as Fp};

//     use std::marker::PhantomData;

//     #[test]
//     fn test_1() {
//         let k = 12;

//         let mut l_quantity: [u64; N] = [1; N];
//         let mut l_extendedprice: [u64; N] = [1; N];
//         let mut l_discount: [u64; N] = [1; N];
//         let mut l_tax: [u64; N] = [1; N];
//         let mut l_returnflag: [u64; N] = [2; N];
//         let mut l_linestatus: [u64; N] = [3; N];
//         let mut l_shipdate: [u64; N] = [3; N];
//         // l_shipdate[0] = 11;
//         let mut right_shipdate: u64 = 100000;
//         let mut check: [bool; N] = [true; N];
//         let mut check_sort: [bool; N] = [true; N];
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
//             l_quantity,
//             l_extendedprice,
//             l_discount,
//             l_tax,
//             l_returnflag,
//             l_linestatus,
//             l_shipdate,
//             right_shipdate,
//             check,
//             check_sort,
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
