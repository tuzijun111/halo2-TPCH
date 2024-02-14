use std::marker::PhantomData;

// use ff::Field;
use halo2_proofs::arithmetic::Field;
// use halo2_proofs::halo2curves::ff::PrimeField;
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// take an value in the `input` advice column
// the goal is to check whether the value is less than target
// table is the instance column that contains all the values from 0 to (instance-1)
// advice_table gets dynamically filled with the values from table
// The chip checks that the input value is less than the target value
// This gets done by performing a lookup between the input value and the advice_table
// pub trait Field: PrimeField<Repr = [u8; 32]> {}

// impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}
#[derive(Debug, Clone)]

pub struct PermAnyConfig {
    pub q_perm1: Selector,
    pub q_perm2: Selector,
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
        q_perm1: Selector,
        q_perm2: Selector,
        input: Vec<Column<Advice>>,
        table: Vec<Column<Advice>>,
    ) -> PermAnyConfig {
        for col in input.clone() {
            meta.enable_equality(col);
        }

        for col in table.clone() {
            meta.enable_equality(col);
        }

        meta.shuffle("permutation check any", |meta| {
            // Inputs
            let q1 = meta.query_selector(q_perm1);
            let q2 = meta.query_selector(q_perm2);
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
                .map(|(input, table)| (q1.clone() * input.clone(), q2.clone() * table.clone()))
                .collect();

            constraints
        });
        // println!("go here?");

        PermAnyConfig {
            q_perm1,
            q_perm2,
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
