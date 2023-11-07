use super::super::chips::less_than_v1_test::{LessThanChip, LessThanConfig};

// use ff::Field;
use eth_types::Field;
use halo2_proofs::{circuit::*, plonk::*};

#[derive(Default)]

// define circuit struct using array of usernames and balances
struct MyCircuit<F: Field> {
    pub input: Vec<Value<F>>,
}

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = LessThanConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let input = meta.advice_column();
        let table: Column<Advice> = meta.advice_column();

        LessThanChip::configure(meta, input, table)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // We create a new instance of chip using the config passed as input
        let chip = LessThanChip::<F>::construct(config);

        // assign value to the chip
        let input = self.input.clone();
        let _ = chip.assign(layouter.namespace(|| "init table"), input);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    use eth_types::Field;
    use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::bn256::Fr as Fp};
    #[test]
    fn test_less_than_2() {
        let k = 10;

        // initate value
        let mut value = [Value::known(Fp::from(8)); 12];
        for i in 0..value.len() {
            value[i] = Value::known(Fp::from(i as u64));
        }

        let circuit = MyCircuit::<Fp> {
            input: value.to_vec(),
        };

        // let target = 800;

        // define public inputs looping from target to 0 and adding each value to pub_inputs vector
        // let mut pub_inputs = vec![];
        // for i in 700..target {
        //     pub_inputs.push(Fp::from(i));
        // }

        // should verify as value is less than target
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();

        // // shouldn't verify as value is greater than target
        // let target_2 = 754;

        // let mut pub_inputs_2 = vec![];
        // for i in 0..target_2 {
        //     pub_inputs_2.push(Fp::from(i));
        // }

        // let invalid_prover = MockProver::run(k, &circuit, vec![pub_inputs_2]).unwrap();

        // assert!(invalid_prover.verify().is_err());
    }
}
