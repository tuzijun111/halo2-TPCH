// use eth_types::Field;
use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};

use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};

// pub trait Field: PrimeField<Repr = [u8; 32]> {}

// impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

#[derive(Clone, Debug)]
pub struct IsZeroV1Config<F: PrimeField> {
    pub value_inv: Column<Advice>,
    pub is_zero_expr: Expression<F>,
}

impl<F: PrimeField> IsZeroV1Config<F> {
    pub fn expr(&self) -> Expression<F> {
        self.is_zero_expr.clone()
    }
}

pub struct IsZeroV1Chip<F: PrimeField> {
    config: IsZeroV1Config<F>,
}

impl<F: PrimeField> IsZeroV1Chip<F> {
    pub fn construct(config: IsZeroV1Config<F>) -> Self {
        IsZeroV1Chip { config }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_enable: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
        value: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
        value_inv: Column<Advice>,
    ) -> IsZeroV1Config<F> {
        let mut is_zero_expr = Expression::Constant(F::ZERO);

        meta.create_gate("is_zero", |meta| {
            //
            // valid | value |  value_inv |  1 - value * value_inv | value * (1 - value* value_inv)
            // ------+-------+------------+------------------------+-------------------------------
            //  yes  |   x   |    1/x     |         0              |  0
            //  no   |   x   |    0       |         1              |  x
            //  yes  |   0   |    0       |         1              |  0
            //  yes  |   0   |    y       |         1              |  0
            //
            let value = value(meta);
            let q_enable = q_enable(meta);
            let value_inv = meta.query_advice(value_inv, Rotation::cur());

            is_zero_expr = Expression::Constant(F::ONE) - value.clone() * value_inv;
            vec![q_enable * value * is_zero_expr.clone()]
        });

        IsZeroV1Config {
            value_inv,
            is_zero_expr,
        }
    }

    pub fn assign(&self, region: &mut Region<'_, F>, value: Vec<F>) -> Result<(), Error> {
        for (idx, v) in value.iter().enumerate() {
            let value_inv = v.invert().unwrap_or(F::ZERO);

            region.assign_advice(
                || "value inv",
                self.config.value_inv,
                idx,
                || Value::known(value_inv),
            )?;
        }

        Ok(())
    }
}
