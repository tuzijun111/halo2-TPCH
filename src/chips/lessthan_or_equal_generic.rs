//! Lt chip can be used to compare LT for two expressions LHS and RHS.
use halo2_proofs::{
    circuit::{Chip, Layouter, Region, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, VirtualCells},
    poly::Rotation,
};
use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};
use std::cmp::Ord;

pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

use super::{
    bool_check,
    util::{expr_from_bytes, pow_of_two},
};

/// Instruction that the Lt chip needs to implement.
pub trait LtEqGenericInstruction<F: Field + Ord> {
    /// Assign the lhs and rhs witnesses to the Lt chip's region.
    fn assign(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        lhs: &[F],
        rhs: &[F],
    ) -> Result<(), Error>;

    /// Load the u8 lookup table.
    fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error>;
}

/// Config for the Lt chip.
#[derive(Clone, Copy, Debug)]
pub struct LtEqGenericConfig<F: Field + Ord, const N_BYTES: usize> {
    /// Denotes the lt outcome. If lhs < rhs then lt == 1, otherwise lt == 0.
    pub lt: Column<Advice>,
    /// Denotes the bytes representation of the difference between lhs and rhs.
    pub diff: [Column<Advice>; N_BYTES],
    /// Denotes the range within which each byte should lie.
    pub u8: Column<Fixed>,
    /// Denotes the range within which both lhs and rhs lie.
    pub range: F,
}

impl<F: Field + Ord, const N_BYTES: usize> LtEqGenericConfig<F, N_BYTES> {
    /// Returns an expression that denotes whether lhs < rhs, or not.
    pub fn is_lt(&self, meta: &mut VirtualCells<F>, rotation: Option<Rotation>) -> Expression<F> {
        meta.query_advice(self.lt, rotation.unwrap_or_else(Rotation::cur))
    }
}

/// Chip that compares lhs < rhs.
#[derive(Clone, Debug)]
pub struct LtEqGenericChip<F: Field + Ord, const N_BYTES: usize> {
    config: LtEqGenericConfig<F, N_BYTES>,
}

impl<F: Field + Ord, const N_BYTES: usize> LtEqGenericChip<F, N_BYTES> {
    /// Configures the Lt chip.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_enable: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
        lhs: impl FnOnce(&mut VirtualCells<F>) -> Vec<Expression<F>>,
        rhs: impl FnOnce(&mut VirtualCells<F>) -> Vec<Expression<F>>,
    ) -> LtEqGenericConfig<F, N_BYTES> {
        let lt = meta.advice_column();
        let diff = [(); N_BYTES].map(|_| meta.advice_column());
        let range = pow_of_two(N_BYTES * 8);
        let u8 = meta.fixed_column();

        meta.create_gate("lt generic gate", |meta| {
            let q_enable = q_enable(meta);
            let lt = meta.query_advice(lt, Rotation::cur());

            let diff_bytes = diff
                .iter()
                .map(|c| meta.query_advice(*c, Rotation::cur()))
                .collect::<Vec<Expression<F>>>();
            let one_expression = Expression::Constant(F::from(1));

            // // concatenate two Expression::F values:
            // let shift_amount = std::cmp::min(N_BYTES * 8, 63);
            // let con1 = lhs(meta) * F::from(1u64 << shift_amount) + rhs(meta);
            // let con2 = lhs1(meta) * F::from(1u64 << shift_amount) + rhs1(meta);
            let lhs_vals = lhs(meta);
            let rhs_vals = rhs(meta);
            let mut lhs_sum = Expression::Constant(F::ZERO);
            let mut rhs_sum = Expression::Constant(F::ZERO);

            for expr in &lhs_vals {
                lhs_sum = lhs_sum + expr.clone();
            }
            for expr in &rhs_vals {
                rhs_sum = rhs_sum + expr.clone();
            }
            let check_a = lhs_sum - (rhs_sum + one_expression) - expr_from_bytes(&diff_bytes)
                + (lt.clone() * range);

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

        LtEqGenericConfig {
            lt,
            diff,
            range,
            u8,
        }
    }

    /// Constructs a Lt chip given a config.
    pub fn construct(config: LtEqGenericConfig<F, N_BYTES>) -> LtEqGenericChip<F, N_BYTES> {
        LtEqGenericChip { config }
    }
}

impl<F: Field + Ord, const N_BYTES: usize> LtEqGenericInstruction<F>
    for LtEqGenericChip<F, N_BYTES>
{
    fn assign(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        lhs: &[F],
        rhs: &[F],
    ) -> Result<(), Error> {
        let config = self.config();

        // let shift_amount = std::cmp::min(N_BYTES * 8, 63);
        // let con1 = lhs * F::from(1u64 << shift_amount) + rhs;
        // let con2 = lhs1 * F::from(1u64 << shift_amount) + rhs1;
        let mut lhs_sum = F::ZERO;
        let mut rhs_sum = F::ZERO;

        for expr in lhs {
            lhs_sum = lhs_sum + expr.clone();
        }
        for expr in rhs {
            rhs_sum = rhs_sum + expr.clone();
        }

        let lt = lhs_sum < (rhs_sum + F::ONE);

        // println!("Anything wrong here? {:?} {:?}", con1, con2);

        region.assign_advice(
            || "lt chip: lt",
            config.lt,
            offset,
            || Value::known(F::from(lt as u64)),
        )?;

        let diff = (lhs_sum - (rhs_sum + F::ONE)) + (if lt { config.range } else { F::ZERO });
        let diff_bytes = diff.to_repr();
        let diff_bytes = diff_bytes.as_ref();
        for (idx, diff_column) in config.diff.iter().enumerate() {
            region.assign_advice(
                || format!("lt chip: diff byte {}", idx),
                *diff_column,
                offset,
                || Value::known(F::from(diff_bytes[idx] as u64)),
            )?;
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

impl<F: Field + Ord, const N_BYTES: usize> Chip<F> for LtEqGenericChip<F, N_BYTES> {
    type Config = LtEqGenericConfig<F, N_BYTES>;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}
