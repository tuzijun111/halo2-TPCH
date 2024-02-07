// use std::marker::PhantomData;

// // use ff::Field;
// use eth_types::Field;
// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// // take an value in the `input` advice column
// // the goal is to check whether the value is less than target
// // table is the instance column that contains all the values from 0 to (instance-1)
// // advice_table gets dynamically filled with the values from table
// // The chip checks that the input value is less than the target value
// // This gets done by performing a lookup between the input value and the advice_table

// #[derive(Debug, Clone)]
// pub struct LessThanConfig {
//     input: Column<Advice>,
//     input2: Column<Advice>,
//     table: Column<Advice>,
//     table2: Column<Advice>,
//     q_cond: Selector,
//     // advice_table: Column<Advice>,
// }

// #[derive(Debug, Clone)]
// pub struct LessThanChip<F: Field> {
//     config: LessThanConfig,
//     _marker: PhantomData<F>,
// }

// impl<F: Field> LessThanChip<F> {
//     pub fn construct(config: LessThanConfig) -> Self {
//         Self {
//             config,
//             _marker: PhantomData,
//         }
//     }

//     pub fn configure(
//         meta: &mut ConstraintSystem<F>,
//         input: Column<Advice>,
//         input2: Column<Advice>,
//         table: Column<Advice>,
//         table2: Column<Advice>,
//     ) -> LessThanConfig {
//         // let advice_table = meta.advice_column();
//         // let advice_table2 = meta.advice_column();
//         let q_cond = meta.complex_selector();
//         meta.enable_equality(table);
//         // meta.enable_equality(advice_table);
//         // meta.annotate_lookup_any_column(advice_table, || "Adv-table");

//         // Dynamic lookup check
//         // TO DO: does it mean that we looking up input inside advice_table?
//         // let x = meta.lookup_any("dynamic lookup check", |meta| {
//         //     let input = meta.query_advice(input, Rotation::cur());
//         //     let advice_table = meta.query_advice(table, Rotation::cur());

//         //     vec![(input, advice_table)]
//         // });

//         // regular use case
//         // let x = meta.shuffle("permutation check", |meta| {
//         //     let input = meta.query_advice(input, Rotation::cur());
//         //     let advice_table = meta.query_advice(table, Rotation::cur());

//         //     vec![(input, advice_table)]
//         // });

//         // select part of the input column for permutation check
//         meta.shuffle("permutation check", |meta| {
//             let q = meta.query_selector(q_cond); //used to control
//             let input = meta.query_advice(input, Rotation::cur());
//             let input2 = meta.query_advice(input2, Rotation::cur());
//             let advice_table = meta.query_advice(table, Rotation::cur());
//             let advice_table2 = meta.query_advice(table2, Rotation::cur());

//             vec![
//                 (q.clone() * input, advice_table),
//                 (q * input2, advice_table2),
//             ]
//         });

//         // println!("the value of meta.lookup_any {:?}", x);

//         LessThanConfig {
//             input,
//             input2,
//             table,
//             table2,
//             q_cond,
//             // advice_table,
//         }
//     }

//     pub fn assign(
//         &self,
//         mut layouter: impl Layouter<F>,
//         input: Vec<Value<F>>,
//         input2: Vec<Value<F>>,
//         table: Vec<Value<F>>,
//         table2: Vec<Value<F>>,
//         table4: Vec<Value<F>>,
//         table6: Vec<Value<F>>,
//     ) -> Result<(), Error> {
//         layouter.assign_region(
//             || "less than assignment",
//             |mut region| {
//                 for i in 0..4 {
//                     if i < 2 {
//                         region.assign_advice(|| "input", self.config.table, i, || table[i])?;
//                         region.assign_advice(|| "input", self.config.table2, i, || table2[i])?;
//                     } else {
//                         region.assign_advice(|| "input", self.config.table, i, || table4[i - 2])?;
//                         region.assign_advice(
//                             || "input",
//                             self.config.table2,
//                             i,
//                             || table6[i - 2],
//                         )?;
//                     }
//                 }

//                 for i in 0..8 {
//                     if i % 2 == 0 {
//                         self.config.q_cond.enable(&mut region, i)?;
//                     }
//                     region.assign_advice(|| "input", self.config.input, i, || input[i])?;
//                     region.assign_advice(|| "input", self.config.input2, i, || input2[i])?;
//                 }

//                 Ok(())
//             },
//         )
//     }
// }
