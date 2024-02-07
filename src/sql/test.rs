use crate::chips::less_than::{LtChip, LtConfig, LtInstruction};
use crate::is_zero::{IsZeroChip, IsZeroConfig};
// use eth_types::Field;
use halo2_proofs::halo2curves::ff::PrimeField;
use halo2_proofs::{
    // arithmetic::Field,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Instance, Selector},
    poly::Rotation,
};

const NUM_BYTES: usize = 5;

pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

#[derive(Debug, Clone)]
// We add the is_zero_config to the FunctionConfig as this is the gadget that we'll be using
// The is_zero_config is the configuration for the IsZeroChip and is composed of an advice column and an expression
struct FunctionConfig<F: Field + Ord> {
    selector: Selector,
    a: Column<Advice>,
    b: Column<Advice>,
    a_equals_b: IsZeroConfig<F>,
    output: Column<Advice>,
    instance: Column<Instance>,
    lt: LtConfig<F, NUM_BYTES>,
    check: Column<Advice>,
}

#[derive(Debug, Clone)]
struct FunctionChip<F: Field + Ord> {
    config: FunctionConfig<F>,
}

impl<F: Field + Ord> FunctionChip<F> {
    pub fn construct(config: FunctionConfig<F>) -> Self {
        Self { config }
    }

    // Chip configuration. This is where we define the gates
    pub fn configure(meta: &mut ConstraintSystem<F>) -> FunctionConfig<F> {
        let selector = meta.selector();
        let a = meta.advice_column();
        let b = meta.advice_column();
        let output = meta.advice_column();
        let is_zero_advice_column = meta.advice_column();
        let instance = meta.instance_column();
        let check = meta.advice_column();

        meta.enable_equality(instance);
        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(output);
        meta.enable_equality(check);

        // We set the configuration for our gadget chip here!
        let a_equals_b = IsZeroChip::configure(
            meta,
            |meta| meta.query_selector(selector), // this is the q_enable
            |meta| meta.query_advice(a, Rotation::cur()) - meta.query_advice(b, Rotation::cur()), // this is the value
            is_zero_advice_column, // this is the advice column that stores value_inv
        );

        // We now need to set up our custom gate!
        meta.create_gate("f(a, b) = if a == b {1} else {0}", |meta| {
            let s = meta.query_selector(selector);
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());

            // a  |  b  | s      |a == b | output  |  s * (a == b) * (output - 1) | s * (1 - a == b) * (output - 0)
            // --------------------------------
            // 10 | 10  | 1      | 1     | 1       | 1 * 1 * 0                    | 1 * 0 * 1 = 0
            // 10 | 12  | 1      | 0     | 0       | 1 * 0 * (-1)                 | 1 * 1 * 0 = 0
            let output = meta.query_advice(output, Rotation::cur());

            vec![
                s.clone() * (a_equals_b.expr() * (output.clone() - (a.clone() - b.clone()))), // in this case output == c
                s * (Expression::Constant(F::ONE) - a_equals_b.expr()) * (output - (a + b)), // in this case output == a - b
            ]
            // vec![
            //     s.clone() * (a_equals_b.expr() * (output.clone() - Expression::Constant(F::one()))), // in this case output == c
            //     s * (Expression::Constant(F::one()) - a_equals_b.expr()) * (output - Expression::Constant(F::zero())), // in this case output == a - b
            // ]
        });

        // lt test
        let lt = LtChip::configure(
            meta,
            |meta| meta.query_selector(selector),
            |meta| meta.query_advice(a, Rotation::cur()),
            |meta| meta.query_advice(b, Rotation::cur()), // we put the left and right value at the first two positions of value_l
        );
        meta.create_gate(
            "verifies o_orderdate < date ':2'", // just use less_than for testing here
            |meta| {
                let q_enable = meta.query_selector(selector);
                let check = meta.query_advice(check, Rotation::cur());
                vec![q_enable * (lt.is_lt(meta, None) - check)]
            },
        );

        FunctionConfig {
            selector,
            a,
            b,
            a_equals_b,
            output,
            instance,
            lt,
            check,
        }
    }

    // execute assignment on a, b, c, output column + is_zero advice column
    pub fn assign(
        &self,
        layouter: &mut impl Layouter<F>,
        a: F,
        b: F,
    ) -> Result<AssignedCell<F, F>, Error> {
        let is_zero_chip = IsZeroChip::construct(self.config.a_equals_b.clone());
        let chip = LtChip::construct(self.config.lt.clone());
        chip.load(layouter)?;

        layouter.assign_region(
            || "f(a, b) = if a == b {1} else {0}",
            |mut region| {
                // let a1 = a;
                // let b1 = b;
                self.config.selector.enable(&mut region, 0)?;
                region.assign_advice(|| "a", self.config.a, 0, || Value::known(a))?;
                region.assign_advice(|| "b", self.config.b, 0, || Value::known(b))?;
                // a.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;

                // remember that the is_zero assign will assign the inverse of the value provided to the advice column
                is_zero_chip.assign(&mut region, 0, Value::known(a - b))?;
                // let output = if a1 == b1 {F::from(1) } else { F::from(0)};
                let output = if a == b { a - b } else { a + b };
                // region.assign_advice(|| "output", self.config.output, 0, || Value::known(output))

                region.assign_advice(
                    || "check",
                    self.config.check,
                    0,
                    || Value::known(F::from(1)),
                )?;

                chip.assign(&mut region, 0, Value::known(a), Value::known(b))?;

                let out_cell = region.assign_advice(
                    || "output",
                    self.config.output,
                    0,
                    || Value::known(output),
                );
                out_cell
                // Ok(())
            },
        )
    }

    pub fn expose_public(
        &self,
        layouter: &mut impl Layouter<F>,
        cell: AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct FunctionCircuit<F> {
    a: F,
    b: F,
}

impl<F: Field + Ord> Circuit<F> for FunctionCircuit<F> {
    type Config = FunctionConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        FunctionChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FunctionChip::construct(config);
        let out_cell = chip.assign(&mut layouter, self.a, self.b)?;
        chip.expose_public(&mut layouter, out_cell, 0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuits::utils::full_prover;
    use halo2_proofs::dev::MockProver;

    use halo2curves::pasta::{pallas, vesta, EqAffine, Fp};

    use bincode;
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        plonk::{
            create_proof, keygen_pk, keygen_vk, verify_proof, Advice, Circuit, Column,
            ConstraintSystem, Error, Instance,
        },
        poly::{
            commitment::{Params, ParamsProver},
            ipa::{
                commitment::{IPACommitmentScheme, ParamsIPA},
                multiopen::ProverIPA,
                strategy::SingleStrategy,
            },
            VerificationStrategy,
        },
        transcript::{
            Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
        },
    };
    use rand::rngs::OsRng;
    use serde::{Deserialize, Serialize};
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;

    #[test]
    fn test_example3() {
        let k = 16;
        let circuit = FunctionCircuit {
            a: Fp::from(2),
            b: Fp::from(4),
        };
        let public_input = vec![Fp::from(6)];

        // let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        // prover.assert_satisfied();
        // full_prover(circuit, k, &public_input);

        let params: ParamsIPA<vesta::Affine> = ParamsIPA::new(k);
        let proof_path = "/home/cc/halo2-TPCH/src/sql/param16";
        let mut fd = std::fs::File::create(&proof_path).unwrap();
        params.write(&mut fd).unwrap();

        // let mut fd = std::fs::File::open(&proof_path).unwrap();
        // let params = ParamsIPA::<vesta::Affine>::read(&mut fd).unwrap();

        // let vk = keygen_vk(&params, &circuit).expect("keygen_vk should not fail");
        // let pk = keygen_pk(&params, vk, &circuit).expect("keygen_pk should not fail");
        // let mut rng = OsRng;

        // let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);

        // let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        // create_proof::<IPACommitmentScheme<_>, ProverIPA<_>, _, _, _, _>(
        //     &params,
        //     &pk,
        //     &[circuit],
        //     &[&[&public_input]],
        //     &mut rng,
        //     &mut transcript,
        // )
        // .expect("proof generation should not fail");
        // let proof = transcript.finalize();

        // let strategy = SingleStrategy::new(&params);
        // let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        // assert!(verify_proof(
        //     &params,
        //     pk.get_vk(),
        //     strategy,
        //     &[&[&public_input]],
        //     &mut transcript
        // )
        // .is_ok());
    }
}
