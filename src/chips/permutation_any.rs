use std::marker::PhantomData;

// use ff::Field;
use eth_types::Field;
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// take an value in the `input` advice column
// the goal is to check whether the value is less than target
// table is the instance column that contains all the values from 0 to (instance-1)
// advice_table gets dynamically filled with the values from table
// The chip checks that the input value is less than the target value
// This gets done by performing a lookup between the input value and the advice_table

#[derive(Debug, Clone)]
// pub struct PermAnyConfig<'a> {
//     input: &'a [Column<Advice>],
//     table: &'a [Column<Advice>],
//     q_cond: Selector,
//     // advice_table: Column<Advice>,
// }
pub struct PermAnyConfig {
    pub q_perm: Selector,
    input: Vec<Column<Advice>>,
    table: Vec<Column<Advice>>,
    // advice_table: Column<Advice>,
}

#[derive(Debug, Clone)]
pub struct PermAnyChip<F>
where
    F: Field,
{
    config: PermAnyConfig,
    _marker: PhantomData<F>,
}

impl<'a, F: Field> PermAnyChip<F> {
    pub fn construct(config: PermAnyConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_perm: Selector,
        input: Vec<Column<Advice>>,
        table: Vec<Column<Advice>>,
    ) -> PermAnyConfig {
        for col in input.clone() {
            meta.enable_equality(col);
        }

        for col in table.clone() {
            meta.enable_equality(col);
        }

        // meta.shuffle("permutation check", |meta| {
        //     let q = meta.query_selector(q_enable.clone());

        //     let k = input.len();
        //     let mut vectors = Vec::new();
        //     for i in 0..k {
        //         let input = meta.query_advice(input[i], Rotation::cur());
        //         let table = meta.query_advice(table[i], Rotation::cur());
        //         vectors.push((q.clone() * input, table));
        //     }
        //     vectors
        // });
        meta.shuffle("permutation check", |meta| {
            // Inputs
            let q = meta.query_selector(q_perm);
            let inputs: Vec<_> = input
                .iter()
                .map(|&idx| meta.query_advice(idx, Rotation::cur()))
                .collect();

            let tables: Vec<_> = table
                .iter()
                .map(|&idx| meta.query_advice(idx, Rotation::cur()))
                .collect();

            let constraints: Vec<_> = inputs
                .iter()
                .zip(tables.iter())
                .map(|(input, table)| (q.clone() * input.clone(), table.clone()))
                .collect();

            constraints
        });
        // println!("go here?");

        PermAnyConfig {
            q_perm,
            input,
            table,
        }
    }

    pub fn assign1(
        // regular one
        &self,
        region: &mut Region<'_, F>,
        input: Vec<Vec<F>>,
        table: Vec<Vec<F>>,
    ) -> Result<(), Error> {
        for i in 0..input.len() {
            for j in 0..input[0].len() {
                region.assign_advice(
                    || "input1",
                    self.config.input[j],
                    i,
                    || Value::known(input[i][j]),
                )?;
            }
        }

        for i in 0..table.len() {
            for j in 0..table[0].len() {
                region.assign_advice(
                    || "table",
                    self.config.table[j],
                    i,
                    || Value::known(table[i][j]),
                )?;
            }
        }

        Ok(())
    }

    pub fn assign2(
        // for two input columns and one table column
        &self,
        region: &mut Region<'_, F>,
        input1: Vec<Vec<F>>,
        input2: Vec<Vec<F>>,
        table: Vec<Vec<F>>,
    ) -> Result<(), Error> {
        for i in 0..input1.len() {
            for j in 0..input1[0].len() {
                region.assign_advice(
                    || "input1",
                    self.config.input[j],
                    i,
                    || Value::known(input1[i][j]),
                )?;
            }
        }

        for i in 0..input2.len() {
            for j in 0..input2[0].len() {
                region.assign_advice(
                    || "input2",
                    self.config.input[j],
                    i + input1.len(),
                    || Value::known(input2[i][j]),
                )?;
            }
        }

        for i in 0..table.len() {
            for j in 0..input1[0].len() {
                region.assign_advice(
                    || "table",
                    self.config.table[j],
                    i,
                    || Value::known(table[i][j]),
                )?;
            }
        }

        Ok(())
    }
}
