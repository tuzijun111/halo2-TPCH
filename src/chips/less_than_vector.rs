// use eth_types::Field;

use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};

use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, VirtualCells},
    poly::Rotation,
};
use std::cmp;
use std::cmp::Ord;

pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

use super::{
    bool_check,
    util::{expr_from_bytes, pow_of_two},
};

/// Instruction that the Lt chip needs to implement.
pub trait LtVecInstruction<F: Field + Ord> {
    /// Assign the lhs and rhs witnesses to the Lt chip's region.
    fn assign_right_constant(
        &self,
        region: &mut Region<'_, F>,
        // offset: usize,
        // lhs: &[Value<F>],
        // rhs: &[Value<F>],
        lhs: Vec<F>,
        rhs: F,
    ) -> Result<(), Error>;

    fn assign_left_constant(
        &self,
        region: &mut Region<'_, F>,
        // offset: usize,
        // lhs: &[Value<F>],
        // rhs: &[Value<F>],
        lhs: F,
        rhs: Vec<F>,
    ) -> Result<(), Error>;

    fn assign(
        &self,
        region: &mut Region<'_, F>,
        // offset: usize,
        // lhs: &[Value<F>],
        // rhs: &[Value<F>],
        lhs: Vec<F>,
        rhs: Vec<F>,
    ) -> Result<(), Error>;

    /// Load the u8 lookup table.
    fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error>;
}

/// Config for the Lt chip.
#[derive(Clone, Copy, Debug)]
pub struct LtVecConfig<F: Field + Ord, const N_BYTES: usize> {
    /// Denotes the lt outcome. If lhs < rhs then lt == 1, otherwise lt == 0.
    pub lt: Column<Advice>,
    /// Denotes the bytes representation of the difference between lhs and rhs.
    pub diff: [Column<Advice>; N_BYTES],
    /// Denotes the range within which each byte should lie.
    pub u8: Column<Fixed>,
    /// Denotes the range within which both lhs and rhs lie.
    pub range: F,
}

impl<F: Field + Ord, const N_BYTES: usize> LtVecConfig<F, N_BYTES> {
    /// Returns an expression that denotes whether lhs < rhs, or not.
    pub fn is_lt(&self, meta: &mut VirtualCells<F>, rotation: Option<Rotation>) -> Expression<F> {
        meta.query_advice(self.lt, rotation.unwrap_or_else(Rotation::cur))
    }
}

/// Chip that compares lhs < rhs.
#[derive(Clone, Debug)]
pub struct LtVecChip<F: Field + Ord, const N_BYTES: usize> {
    config: LtVecConfig<F, N_BYTES>,
}

impl<F: Field + Ord, const N_BYTES: usize> LtVecChip<F, N_BYTES> {
    /// Configures the Lt chip.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_enable: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
        lhs: impl FnOnce(&mut VirtualCells<F>) -> Expression<F>,
        rhs: impl FnOnce(&mut VirtualCells<F>) -> Expression<F>,
    ) -> LtVecConfig<F, N_BYTES> {
        let lt = meta.advice_column();
        let diff = [(); N_BYTES].map(|_| meta.advice_column());
        let range = pow_of_two(N_BYTES * 8);
        let u8 = meta.fixed_column();

        meta.create_gate("lt gate", |meta| {
            let q_enable = q_enable(meta);
            let lt = meta.query_advice(lt, Rotation::cur());

            let diff_bytes = diff
                .iter()
                .map(|c| meta.query_advice(*c, Rotation::cur()))
                .collect::<Vec<Expression<F>>>();

            let check_a =
                lhs(meta) - rhs(meta) - expr_from_bytes(&diff_bytes) + (lt.clone() * range);

            let check_b = bool_check(lt);

            [check_a, check_b]
                .into_iter()
                .map(move |poly| q_enable.clone() * poly)
        });

        meta.annotate_lookup_any_column(u8, || "LOOKUP_u8");

        diff[0..N_BYTES].iter().for_each(|column| {
            meta.lookup_any("range check for u8", |meta| {
                let u8_cell = meta.query_advice(*column, Rotation::cur());
                let u8_range = meta.query_fixed(u8, Rotation::cur());
                vec![(u8_cell, u8_range)]
            });
        });

        LtVecConfig {
            lt,
            diff,
            range,
            u8,
        }
    }

    /// Constructs a Lt chip given a config.
    pub fn construct(config: LtVecConfig<F, N_BYTES>) -> LtVecChip<F, N_BYTES> {
        LtVecChip { config }
    }
}

impl<F: Field + Ord, const N_BYTES: usize> LtVecInstruction<F> for LtVecChip<F, N_BYTES> {
    fn assign_right_constant(
        &self,
        region: &mut Region<'_, F>,
        // offset: usize,
        // lhs: &[Value<F>],
        // rhs: &[Value<F>],
        lhs: Vec<F>,
        rhs: F,
    ) -> Result<(), Error> {
        let config = self.config();

        for i in 0..lhs.len() {
            let temp = lhs[i] < rhs;

            region.assign_advice(
                || "lt chip: lt",
                config.lt,
                i,
                || Value::known(F::from(temp as u64)),
            )?;
            let diff = (lhs[i] - rhs) + (if temp { config.range } else { F::ZERO });
            let diff_bytes = diff.to_repr();
            let diff_bytes = diff_bytes.as_ref();
            for (idx, diff_column) in config.diff.iter().enumerate() {
                region.assign_advice(
                    || format!("lt chip: diff byte {}", idx),
                    *diff_column,
                    i,
                    || Value::known(F::from(diff_bytes[idx] as u64)),
                )?;
            }
        }

        Ok(())
    }

    fn assign_left_constant(
        &self,
        region: &mut Region<'_, F>,
        // offset: usize,
        // lhs: &[Value<F>],
        // rhs: &[Value<F>],
        lhs: F,
        rhs: Vec<F>,
    ) -> Result<(), Error> {
        let config = self.config();

        for i in 0..rhs.len() {
            let temp = lhs < rhs[i];

            region.assign_advice(
                || "lt chip: lt",
                config.lt,
                i,
                || Value::known(F::from(temp as u64)),
            )?;
            let diff = (lhs - rhs[i]) + (if temp { config.range } else { F::ZERO });
            let diff_bytes = diff.to_repr();
            let diff_bytes = diff_bytes.as_ref();
            for (idx, diff_column) in config.diff.iter().enumerate() {
                region.assign_advice(
                    || format!("lt chip: diff byte {}", idx),
                    *diff_column,
                    i,
                    || Value::known(F::from(diff_bytes[idx] as u64)),
                )?;
            }
        }

        Ok(())
    }

    fn assign(&self, region: &mut Region<'_, F>, lhs: Vec<F>, rhs: Vec<F>) -> Result<(), Error> {
        let config = self.config();

        for i in 0..lhs.len() {
            let temp = lhs[i] < rhs[i];

            region.assign_advice(
                || "lt chip: lt",
                config.lt,
                i,
                || Value::known(F::from(temp as u64)),
            )?;
            let diff = (lhs[i] - rhs[i]) + (if temp { config.range } else { F::ZERO });
            let diff_bytes = diff.to_repr();
            let diff_bytes = diff_bytes.as_ref();
            for (idx, diff_column) in config.diff.iter().enumerate() {
                region.assign_advice(
                    || format!("lt chip: diff byte {}", idx),
                    *diff_column,
                    i,
                    || Value::known(F::from(diff_bytes[idx] as u64)),
                )?;
            }
        }

        Ok(())
    }

    fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        const RANGE: usize = 256;

        layouter.assign_region(
            || "load u8 range check table",
            |mut region| {
                for i in 0..RANGE {
                    region.assign_fixed(
                        || "assign cell in fixed column",
                        self.config.u8,
                        i,
                        || Value::known(F::from(i as u64)),
                    )?;
                }
                Ok(())
            },
        )
    }
}

impl<F: Field + Ord, const N_BYTES: usize> Chip<F> for LtVecChip<F, N_BYTES> {
    type Config = LtVecConfig<F, N_BYTES>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}
