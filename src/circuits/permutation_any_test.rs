use super::super::chips::permutation_any::{PermAnyChip, PermAnyConfig};

// use ff::Field;
use eth_types::Field;
// use halo2_proofs::{circuit::Value, halo2curves::bn256::Fr as Fp};
use halo2_proofs::{circuit::*, plonk::*};

// // define circuit struct using array of usernames and balances
// trait CircuitWithN {
//     const N: usize;
// }

#[derive(Default)]
struct MyCircuit<F: Field> {
    pub input1: Vec<Vec<F>>,
    pub input2: Vec<Vec<F>>,
    pub table: Vec<Vec<F>>,
}

// impl<F: Field> CircuitWithN for MyCircuit<'_, F> {
//     const N: usize = 2;
// }

impl<F: Field> Circuit<F> for MyCircuit<F> {
    type Config = PermAnyConfig;

    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        const N: usize = 2;
        let q_enable = meta.complex_selector();

        // // Create arrays instead of vectors
        let mut input_columns = Vec::new();
        let mut table_columns = Vec::new();
        for i in 0..N {
            input_columns.push(meta.advice_column());
            table_columns.push(meta.advice_column());
        }
        // let mut input_columns = [meta.advice_column(); 2];
        // let mut table_columns = [meta.advice_column(); 2];

        PermAnyChip::configure(meta, q_enable, input_columns, table_columns)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // We create a new instance of chip using the config passed as input
        // const N: usize = 2;

        let chip = PermAnyChip::<F>::construct(config.clone());

        // assign value to the chip
        let input1 = self.input1.clone();
        let input2 = self.input2.clone();
        let table = self.table.clone();

        layouter.assign_region(
            || "witness",
            |mut region| {
                for i in 0..2 {
                    if i >= 1 {
                        config.q_perm.enable(&mut region, i)?;
                    }
                }

                chip.assign1(&mut region, input2.clone(), table.clone());
                Ok(())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    use eth_types::Field;
    use halo2_proofs::{circuit::Value, dev::MockProver, halo2curves::bn256::Fr as Fp};
    #[test]
    fn test_perm() {
        let k = 10;

        // initate value
        let input1: Vec<Vec<Fp>> = vec![vec![Fp::from(1), Fp::from(2)]];
        let input2: Vec<Vec<Fp>> = vec![
            vec![Fp::from(3), Fp::from(4)],
            vec![Fp::from(5), Fp::from(6)],
        ];
        let table: Vec<Vec<Fp>> = vec![
            // vec![Fp::from(1), Fp::from(2)],
            // vec![Fp::from(3), Fp::from(4)],
            vec![Fp::from(5), Fp::from(6)],
        ];

        let circuit = MyCircuit::<Fp> {
            input1,
            input2,
            table,
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
