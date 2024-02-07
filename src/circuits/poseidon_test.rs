/*
An easy-to-use implementation of the Poseidon Hash in the form of a Halo2 Chip. While the Poseidon Hash function
is already implemented in halo2_gadgets, there is no wrapper chip that makes it easy to use in other circuits.
*/

use super::super::chips::poseidon::{PoseidonChip, PoseidonConfig};
use halo2_gadgets::poseidon::primitives::*;
use halo2_proofs::{circuit::*, plonk::*};
use halo2curves::pasta::Fp;
use std::marker::PhantomData;

struct PoseidonCircuit<
    S: Spec<Fp, WIDTH, RATE>,
    const WIDTH: usize,
    const RATE: usize,
    const L: usize,
> {
    message: [Value<Fp>; L],
    output: Value<Fp>,
    _spec: PhantomData<S>,
}

impl<S: Spec<Fp, WIDTH, RATE>, const WIDTH: usize, const RATE: usize, const L: usize> Circuit<Fp>
    for PoseidonCircuit<S, WIDTH, RATE, L>
{
    type Config = PoseidonConfig<WIDTH, RATE, L>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self {
            message: (0..L)
                .map(|i| Value::unknown())
                .collect::<Vec<Value<Fp>>>()
                .try_into()
                .unwrap(),
            output: Value::unknown(),
            _spec: PhantomData,
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> PoseidonConfig<WIDTH, RATE, L> {
        PoseidonChip::<S, WIDTH, RATE, L>::configure(meta)
    }

    fn synthesize(
        &self,
        config: PoseidonConfig<WIDTH, RATE, L>,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let poseidon_chip = PoseidonChip::<S, WIDTH, RATE, L>::construct(config);
        let message_cells = poseidon_chip
            .load_private_inputs(layouter.namespace(|| "load private inputs"), self.message)?;
        let result = poseidon_chip.hash(layouter.namespace(|| "poseidon chip"), &message_cells)?;
        poseidon_chip.expose_public(layouter.namespace(|| "expose result"), &result, 0)?;
        Ok(())
    }
}

mod tests {
    use std::marker::PhantomData;

    use super::PoseidonCircuit;
    use crate::data::data_processing;
    use chrono::{DateTime, NaiveDate, Utc};
    use halo2_gadgets::poseidon::{
        primitives::{self as poseidon, ConstantLength, P128Pow5T3 as OrchardNullifier, Spec},
        Hash,
    };
    use halo2_proofs::{circuit::Value, dev::MockProver};
    use halo2curves::pasta::Fp;

    #[test]
    fn test() {
        fn string_to_u64(s: &str) -> u64 {
            let mut result = 0;

            for (i, c) in s.chars().enumerate() {
                result += (i as u64 + 1) * (c as u64);
            }

            result
        }
        fn scale_by_1000(x: f64) -> u64 {
            (1000.0 * x) as u64
        }
        fn date_to_timestamp(date_str: &str) -> u64 {
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(date) => {
                    let datetime: DateTime<Utc> =
                        DateTime::<Utc>::from_utc(date.and_hms(0, 0, 0), Utc);
                    datetime.timestamp() as u64
                }
                Err(_) => 0, // Return a default value like 0 in case of an error
            }
        }

        // let lineitem_file_path = "/Users/binbingu/halo2-TPCH/src/data/lineitem.tbl";
        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";
        let mut lineitem: Vec<Vec<Fp>> = Vec::new();

        if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
            // Convert the Vec<Region> to a 2D vector
            lineitem = records
                .iter()
                .map(|record| {
                    vec![
                        Fp::from(record.l_quantity),
                        Fp::from(scale_by_1000(record.l_extendedprice)),
                        Fp::from(scale_by_1000(record.l_discount)),
                        Fp::from(scale_by_1000(record.l_tax)),
                        Fp::from(string_to_u64(&record.l_returnflag)),
                        Fp::from(string_to_u64(&record.l_linestatus)),
                        Fp::from(date_to_timestamp(&record.l_shipdate)),
                        // Fp::from(string_to_u64(&record.l_shipdate)),
                    ]
                })
                .collect();
        }

        let lineitem: Vec<Vec<Fp>> = lineitem.iter().take(10).cloned().collect();
        // println!("{:?}", lineitem);
        // Assuming `L` is the fixed size of your message array
        const L: usize = 2; // Define L based on your circuit requirements

        // Example function to prepare your message from lineitem
        fn prepare_message(lineitem: Vec<Vec<Fp>>) -> [Value<Fp>; L] {
            let mut message: Vec<Value<Fp>> = Vec::new();

            // Implement your selection strategy here
            // For example, flattening the first few elements of lineitem:
            for item in lineitem.iter().flatten().take(L) {
                message.push(Value::known(*item));
            }

            // If the collected items are less than L, pad the message with zeros or another default value
            while message.len() < L {
                message.push(Value::known(Fp::zero())); // Use an appropriate default value
            }

            // Convert Vec<Value<Fp>> to [Value<Fp>; L]
            let message_array: [Value<Fp>; L] =
                message.try_into().expect("Incorrect message length");
            message_array
        }
        // let message = prepare_message(lineitem);
        let mut message = [Fp::from(0); L];

        for (i, item) in lineitem.iter().flatten().take(L).enumerate() {
            // message[i] = *item;
            println!("{:?}", item);
        }
        // let input = 99u64;
        // let message = [lineitem[0][0], lineitem[0][1], lineitem[0][2]];

        let output =
            poseidon::Hash::<_, OrchardNullifier, ConstantLength<L>, 3, 2>::init().hash(message);

        let circuit = PoseidonCircuit::<OrchardNullifier, 3, 2, L> {
            message: message.map(|x| Value::known(x)),
            output: Value::known(output),
            _spec: PhantomData,
        };
        let public_input = vec![output];
        let prover = MockProver::run(10, &circuit, vec![public_input.clone()]).unwrap();
        prover.assert_satisfied();
    }
}
