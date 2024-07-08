use crate::chips::is_zero::{IsZeroChip, IsZeroConfig};
use crate::chips::is_zero_v1::{IsZeroV1Chip, IsZeroV1Config};
use crate::chips::is_zero_v2::{IsZeroV2Chip, IsZeroV2Config};
use crate::chips::lessthan_or_equal_generic::{
    LtEqGenericChip, LtEqGenericConfig, LtEqGenericInstruction,
};
use crate::chips::lessthan_or_equal_v1::{LtEqVecChip, LtEqVecConfig, LtEqVecInstruction};
use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use halo2_proofs::{halo2curves::ff::PrimeField, plonk::Expression};
use std::{default, marker::PhantomData};

use std::cmp::Ordering;
use std::time::Instant;

const NUM_BYTES: usize = 5;



// #[derive(Default)]
// We should use the selector to skip the row which does not satisfy shipdate values
pub trait Field: PrimeField<Repr = [u8; 32]> {}

impl<F> Field for F where F: PrimeField<Repr = [u8; 32]> {}

#[derive(Clone, Debug)]
pub struct TestCircuitConfig<F: Field + Ord> {
    q_enable: Selector,
    q_accu: Selector,
    q_sort: Vec<Selector>,
    q_perm: Vec<Selector>,

    // lineitem: [Column<Advice>; 7], // l_quanity, l_exten, l_dis, l_tax, l_ret, l_linest, l_ship
    lineitem: Vec<Column<Advice>>,
    groupby: Vec<Column<Advice>>,

    sum_qty: Column<Advice>,
    sum_base_price: Column<Advice>,
    sum_disc_price: Column<Advice>,
    sum_charge: Column<Advice>,
    sum_discount: Column<Advice>,

    right_shipdate: Column<Advice>,

    // groupby: [Column<Advice>; 7],
    check: Column<Advice>,
    equal_check: Column<Advice>,
    count_check: Column<Advice>,

    equal_v2: IsZeroV2Config<F>,
    compare_condition: Vec<LtEqGenericConfig<F, NUM_BYTES>>,
    compare_v1: LtEqVecConfig<F, NUM_BYTES>,
    instance: Column<Instance>,
    instance_test: Column<Advice>,
}

#[derive(Debug, Clone)]
pub struct TestChip<F: Field + Ord> {
    config: TestCircuitConfig<F>,
}

impl<F: Field + Ord> TestChip<F> {
    pub fn construct(config: TestCircuitConfig<F>) -> Self {
        Self { config }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> TestCircuitConfig<F> {
        let instance = meta.instance_column();
        meta.enable_equality(instance);
        let instance_test = meta.advice_column();
        meta.enable_equality(instance_test);

        let q_enable = meta.selector();
        let q_accu = meta.selector();
        let mut q_sort = Vec::new();
        for _ in 0..1 {
            q_sort.push(meta.selector());
        }

        let mut q_perm = Vec::new();
        for _ in 0..2 {
            q_perm.push(meta.complex_selector());
        }

        // let lineitem = [meta.advice_column(); 7];
        // let groupby = [meta.advice_column(); 7];
        let mut lineitem = Vec::new();
        let mut groupby = Vec::new();
        for _ in 0..7 {
            lineitem.push(meta.advice_column());
            groupby.push(meta.advice_column());
        }

        let right_shipdate = meta.advice_column();

        let sum_qty = meta.advice_column();
        let sum_base_price = meta.advice_column();
        let sum_disc_price = meta.advice_column();
        let sum_charge = meta.advice_column();
        let sum_discount = meta.advice_column();

        let check = meta.advice_column();
        let equal_check = meta.advice_column();
        let count_check = meta.advice_column();

        let mut is_zero_vectors = Vec::new();
        for _ in 0..2 {
            is_zero_vectors.push(meta.advice_column());
        }

        // constraints for l_shipdate <= date '1998-12-01' - interval ':1' day (3)

        let compare_v1 = LtEqVecChip::configure(
            meta,
            |meta| meta.query_selector(q_enable),
            |meta| meta.query_advice(lineitem[6], Rotation::cur()),
            |meta| meta.query_advice(right_shipdate, Rotation::cur()),
        );

        meta.create_gate(
            "verifies l_shipdate <= date '1998-12-01' - interval ':1' day (3)",
            |meta| {
                let q_enable = meta.query_selector(q_enable);
                let check = meta.query_advice(check, Rotation::cur());
                vec![q_enable * (compare_v1.is_lt(meta, None) - check)]
            },
        );

        // // l_returnflag[i-1] <= l_returnflag[i]
        let mut compare_condition = Vec::new();
        let config_lt = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[0]),
            |meta| vec![meta.query_advice(groupby[4], Rotation::prev())],
            |meta| vec![meta.query_advice(groupby[4], Rotation::cur())],
        );
        compare_condition.push(config_lt);
        // l_linestatus[i-1] <= l_linestatus[i]
        let config_lt_1 = LtEqGenericChip::configure(
            meta,
            |meta| meta.query_selector(q_sort[0]),
            |meta| vec![meta.query_advice(groupby[5], Rotation::prev())],
            |meta| vec![meta.query_advice(groupby[5], Rotation::cur())],
        );
        compare_condition.push(config_lt_1);

        meta.create_gate("verifies orderby scenarios", |meta| {
            let q_sort = meta.query_selector(q_sort[0]);
            // vec![
            //     q_sort.clone()
            //             * (config_lt.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
            //             * (equal_v0.expr() * config_lt_1.is_lt(meta, None)
            //                 - Expression::Constant(F::ONE)),
            // ]
            let equal = meta.query_advice(groupby[4], Rotation::prev())
                - meta.query_advice(groupby[4], Rotation::cur());
            vec![
                q_sort.clone()
                        * (config_lt.is_lt(meta, None) - Expression::Constant(F::ONE)) // or
                        * (equal * config_lt_1.is_lt(meta, None)
                            - Expression::Constant(F::ONE)),
            ]
        });

        // check whether (l_returnflag[i-1], l_linestatus[i-1]) = (l_returnflag[i], l_linestatus[i])
        let equal_v2 = IsZeroV2Chip::configure(
            meta,
            |meta| meta.query_selector(q_sort[0]),
            |meta| {
                vec![
                    meta.query_advice(groupby[4], Rotation::prev())
                        - meta.query_advice(groupby[4], Rotation::cur()),
                    meta.query_advice(groupby[5], Rotation::prev())
                        - meta.query_advice(groupby[5], Rotation::cur()),
                ]
            },
            is_zero_vectors, // s_acctbal[i] = s_acctbal[i-1]
        );

        meta.create_gate("count_check", |meta| {
            let q_sort = meta.query_selector(q_sort[0]);
            let prev_count = meta.query_advice(count_check, Rotation::prev());
            let count_check = meta.query_advice(count_check, Rotation::cur());
            let equal_check = meta.query_advice(equal_check, Rotation::cur());
            vec![
                q_sort.clone()
                    * (prev_count * equal_check + Expression::Constant(F::ONE) - count_check),
            ]
        });

        //sum gate
        meta.create_gate("accumulate constraint", |meta| {
            let q_accu = meta.query_selector(q_accu);
            let prev_b_sum_qty = meta.query_advice(sum_qty, Rotation::cur());
            let prev_b_sum_base_price = meta.query_advice(sum_base_price, Rotation::cur());
            let prev_b_sum_disc_price = meta.query_advice(sum_disc_price, Rotation::cur());
            let prev_b_sum_charge = meta.query_advice(sum_charge, Rotation::cur());
            let prev_b_sum_discount = meta.query_advice(sum_discount, Rotation::cur());

            let quantity = meta.query_advice(groupby[0], Rotation::cur());
            let extendedprice = meta.query_advice(groupby[1], Rotation::cur());
            let discount = meta.query_advice(groupby[2], Rotation::cur());
            let tax = meta.query_advice(groupby[3], Rotation::cur());

            let sum_qty = meta.query_advice(sum_qty, Rotation::next());
            let sum_base_price = meta.query_advice(sum_base_price, Rotation::next());
            let sum_disc_price = meta.query_advice(sum_disc_price, Rotation::next());
            let sum_charge = meta.query_advice(sum_charge, Rotation::next());
            let sum_discount = meta.query_advice(sum_discount, Rotation::next());

            let check = meta.query_advice(equal_check, Rotation::cur());

            vec![
                q_accu.clone()
                    * (check.clone() * prev_b_sum_qty.clone() + quantity.clone() - sum_qty.clone()),
                q_accu.clone()
                    * (check.clone() * prev_b_sum_base_price + extendedprice.clone()
                        - sum_base_price),
                q_accu.clone()
                    * (check.clone() * prev_b_sum_disc_price
                        + extendedprice.clone()
                            * (Expression::Constant(F::from(1000)) - discount.clone())
                        - sum_disc_price),
                q_accu.clone()
                    * (check.clone() * prev_b_sum_charge
                        + extendedprice
                            * (Expression::Constant(F::from(1000)) - discount.clone())
                            * (Expression::Constant(F::from(1000)) + tax)
                        - sum_charge),
                q_accu * (check.clone() * prev_b_sum_discount + discount - sum_discount),
            ]
        });

        // groupby permutation
        meta.shuffle("permutation check", |meta| {
            // Inputs
            let q1 = meta.query_selector(q_perm[0]);
            let q2 = meta.query_selector(q_perm[1]);
            let lineitem_queries: Vec<_> = lineitem
                .iter()
                .map(|&idx| meta.query_advice(idx, Rotation::cur()))
                .collect();

            let groupby_queries: Vec<_> = groupby
                .iter()
                .map(|&idx| meta.query_advice(idx, Rotation::cur()))
                .collect();

            let constraints: Vec<_> = lineitem_queries
                .iter()
                .zip(groupby_queries.iter())
                .map(|(input, table)| (q1.clone() * input.clone(), q2.clone() * table.clone()))
                .collect();

            constraints
        });

        TestCircuitConfig {
            q_enable,
            q_accu,
            q_sort,
            q_perm,
            lineitem,
            groupby,
            right_shipdate,

            equal_v2,
            check,
            equal_check,
            count_check,
            sum_qty,
            compare_condition,
            compare_v1,
            sum_base_price,
            sum_disc_price,
            sum_charge,

            sum_discount,
            instance,
            instance_test,
        }
    }

    pub fn assign(
        &self,
        // layouter: &mut impl Layouter<F>,
        layouter: &mut impl Layouter<F>,
        lineitem: Vec<Vec<F>>,
        right_shipdate: F,
    ) -> Result<AssignedCell<F, F>, Error> {
        let equal_v2_chip = IsZeroV2Chip::construct(self.config.equal_v2.clone());
        let mut compare_chip = Vec::new();
        for i in 0..self.config.compare_condition.len() {
            let chip = LtEqGenericChip::construct(self.config.compare_condition[i].clone());
            chip.load(layouter)?;
            compare_chip.push(chip);
        }
        let compare_v1_chip = LtEqVecChip::construct(self.config.compare_v1.clone());
        compare_v1_chip.load(layouter)?;

        let start = Instant::now();
        let mut l_check = Vec::new();
        for i in 0..lineitem.len() {
            if lineitem[i][6] <= right_shipdate {
                l_check.push(true);
            } else {
                l_check.push(false);
            }
        }

        let duration_block = start.elapsed();
        println!("Time elapsed for block: {:?}", duration_block);

        let mut l_combined: Vec<Vec<_>> = lineitem
            .clone()
            .into_iter()
            .filter(|row| row[6] <= right_shipdate) // l_shipdate <= date '1998-12-01' - interval ':1' day (3)
            .collect();

        println! {"l: {:?}", l_combined.len()};

        //     group by
        //     l_returnflag,
        //     l_linestatus
        // order by
        //     l_returnflag,
        //     l_linestatus;

        l_combined.sort_by(|a, b| match a[4].cmp(&b[4]) {
            Ordering::Equal => a[5].cmp(&b[5]),
            other => other,
        });

        // println!("l2: {:?}", l_combined.len());

        let duration_block = start.elapsed();
        println!("Time elapsed for block: {:?}", duration_block);

        // equal check between config.groupby and config.lineitem
        // region.constrain_equal(left, right);

        // 0 represents this row is the first one of a group, and 1 otherwise
        let mut equal_check: Vec<F> = Vec::new();
        let mut count_check: Vec<F> = Vec::new();
        if l_combined.len() > 0 {
            equal_check.push(F::from(0)); // add the the first one must be 0
            count_check.push(F::from(1)); // add the count of the first dataitem as 1
        }

        for row in 1..l_combined.len() {
            if l_combined[row][4] == l_combined[row - 1][4] {
                equal_check.push(F::from(1));
                let count_value = *count_check.last().unwrap() + F::from(1);
                count_check.push(count_value);
            } else {
                equal_check.push(F::from(0));
                count_check.push(F::from(1));
            }
        }

        // let mut equal_v1_assign = Vec::new();

        // for i in 0..l_combined.len() - 1 {
        //     equal_v1_assign.push(l_combined[i][4] - l_combined[i + 1][4]);
        // }

        let n = l_combined.len() + 1;
        let mut sum_qty: Vec<F> = vec![F::from(0); n];
        let mut sum_base: Vec<F> = vec![F::from(0); n];
        let mut sum_disc: Vec<F> = vec![F::from(0); n];
        let mut sum_charge: Vec<F> = vec![F::from(0); n];
        let mut sum_discount: Vec<F> = vec![F::from(0); n];

        for i in 1..n {
            sum_qty[i] = sum_qty[i - 1] * equal_check[i - 1] + l_combined[i - 1][0];
            sum_base[i] = sum_base[i - 1] * equal_check[i - 1] + l_combined[i - 1][1];
            sum_disc[i] = sum_disc[i - 1] * equal_check[i - 1]
                + l_combined[i - 1][1] * (F::from(1000) - l_combined[i - 1][2]);
            sum_charge[i] = sum_charge[i - 1] * equal_check[i - 1]
                + l_combined[i - 1][1]
                    * (F::from(1000) - l_combined[i - 1][2])
                    * (F::from(1000) + l_combined[i - 1][3]);
            sum_discount[i] = sum_discount[i - 1] * equal_check[i - 1] + l_combined[i - 1][2];
        }

        let duration_block = start.elapsed();
        println!("Time elapsed for block: {:?}", duration_block);

        layouter.assign_region(
            || "witness",
            |mut region| {
                for i in 0..lineitem.len() {
                    self.config.q_enable.enable(&mut region, i)?;
                    for j in 0..lineitem[0].len() {
                        region.assign_advice(
                            || "l",
                            self.config.lineitem[j],
                            i,
                            || Value::known(lineitem[i][j]),
                        )?;
                    }

                    region.assign_advice(
                        || "check",
                        self.config.check,
                        i,
                        || Value::known(F::from(l_check[i] as u64)),
                    )?;
                    // only focus on the values after filtering
                    if l_check[i] == true {
                        self.config.q_perm[0].enable(&mut region, i)?;
                    }

                    region.assign_advice(
                        || "right_shipdate value",
                        self.config.right_shipdate,
                        i,
                        || Value::known(right_shipdate),
                    )?;
                }

                for i in 0..l_combined.len() {
                    for j in 0..l_combined[0].len() {
                        region.assign_advice(
                            || "",
                            self.config.groupby[j],
                            i,
                            || Value::known(l_combined[i][j]),
                        )?;
                    }

                    if i > 0 {
                        self.config.q_sort[0].enable(&mut region, i)?; // groupby sort assignment

                        compare_chip[0].assign(
                            &mut region,
                            i,
                            &[l_combined[i - 1][4]],
                            &[l_combined[i][4]],
                        )?;

                        compare_chip[1].assign(
                            &mut region,
                            i,
                            &[l_combined[i - 1][5]],
                            &[l_combined[i][5]],
                        )?;

                        equal_v2_chip.assign(
                            &mut region,
                            i,
                            (
                                Value::known(l_combined[i - 1][4] - l_combined[i][4]),
                                Value::known(l_combined[i - 1][5] - l_combined[i][5]),
                            ),
                        )?;
                    }
                }

                for i in 0..equal_check.len() {
                    self.config.q_accu.enable(&mut region, i)?;
                    self.config.q_perm[1].enable(&mut region, i)?;

                    region.assign_advice(
                        || "equal_check",
                        self.config.equal_check,
                        i,
                        || Value::known(equal_check[i]),
                    )?;
                    region.assign_advice(
                        || "count_check",
                        self.config.count_check,
                        i,
                        || Value::known(count_check[i]),
                    )?;
                }

                for i in 0..n {
                    region.assign_advice(
                        || "equal_check",
                        self.config.sum_qty,
                        i,
                        || Value::known(sum_qty[i]),
                    )?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.sum_base_price,
                        i,
                        || Value::known(sum_base[i]),
                    )?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.sum_disc_price,
                        i,
                        || Value::known(sum_disc[i]),
                    )?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.sum_charge,
                        i,
                        || Value::known(sum_charge[i]),
                    )?;
                    region.assign_advice(
                        || "equal_check",
                        self.config.sum_discount,
                        i,
                        || Value::known(sum_discount[i]),
                    )?;

                    // assign chip
                }

                compare_v1_chip.assign_right_constant(
                    &mut region,
                    lineitem
                        .iter()
                        .filter_map(|row| row.get(6)) // Get the first element of the row, if it exists
                        .map(|&element| element) // Convert each element to type `F`
                        .collect(),
                    right_shipdate,
                )?;

                let out = region.assign_advice(
                    || "orderby",
                    self.config.instance_test,
                    0,
                    || Value::known(F::from(1)),
                )?;
                Ok(out)
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

struct MyCircuit<F> {
    pub lineitem: Vec<Vec<F>>,
    pub right_shipdate: F,

    _marker: PhantomData<F>,
}

impl<F: Copy + Default> Default for MyCircuit<F> {
    fn default() -> Self {
        Self {
            lineitem: Vec::new(),
            right_shipdate: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<F: Field + Ord> Circuit<F> for MyCircuit<F> {
    type Config = TestCircuitConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        TestChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let test_chip = TestChip::construct(config);

        let out_b_cells =
            test_chip.assign(&mut layouter, self.lineitem.clone(), self.right_shipdate)?;

        // for (i, cell) in out_b_cells.iter().enumerate() {
        //     test_chip.expose_public(&mut layouter, cell.clone(), i)?;
        // }

        // test_chip.expose_public(&mut layouter, out_b_cell, 0)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MyCircuit;
    // use rand::Rng;
    // use halo2_proofs::poly::commitment::Params
    use crate::data::data_processing;
    use chrono::{DateTime, NaiveDate, Utc};
    use halo2_proofs::dev::MockProver;
    use std::marker::PhantomData;

    use halo2curves::pasta::{pallas, vesta, EqAffine, Fp};

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
    use std::process;
    use std::time::Instant;
    use std::{fs::File, io::Write, path::Path};

    fn generate_and_verify_proof<C: Circuit<Fp>>(
        k: u32,
        circuit: C,
        public_input: &[Fp], // Adjust the type according to your actual public input type
        proof_path: &str,
    ) {
        // Time to generate parameters
        // let params_time_start = Instant::now();
        // let params: ParamsIPA<vesta::Affine> = ParamsIPA::new(k);
        let params_path = "/home/cc/halo2-TPCH/src/sql/param16";
        // let mut fd = std::fs::File::create(&params_path).unwrap();
        // params.write(&mut fd).unwrap();
        // println!("Time to generate params {:?}", params_time_start.elapsed());

        // read params
        let mut fd = std::fs::File::open(&params_path).unwrap();
        let params = ParamsIPA::<vesta::Affine>::read(&mut fd).unwrap();

        // Time to generate verification key (vk)
        let params_time_start = Instant::now();
        let vk = keygen_vk(&params, &circuit).expect("keygen_vk should not fail");
        let params_time = params_time_start.elapsed();
        println!("Time to generate vk {:?}", params_time);

        // Time to generate proving key (pk)
        let params_time_start = Instant::now();
        let pk = keygen_pk(&params, vk.clone(), &circuit).expect("keygen_pk should not fail");
        let params_time = params_time_start.elapsed();
        println!("Time to generate pk {:?}", params_time);

        // Proof generation
        let mut rng = OsRng;
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof::<IPACommitmentScheme<_>, ProverIPA<_>, _, _, _, _>(
            &params,
            &pk,
            &[circuit],
            &[&[public_input]], // Adjust as necessary for your public input handling
            &mut rng,
            &mut transcript,
        )
        .expect("proof generation should not fail");
        let proof = transcript.finalize();

        // Write proof to file
        File::create(Path::new(proof_path))
            .expect("Failed to create proof file")
            .write_all(&proof)
            .expect("Failed to write proof");
        println!("Proof written to: {}", proof_path);

        // Proof verification
        let strategy = SingleStrategy::new(&params);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
        assert!(
            verify_proof(
                &params,
                pk.get_vk(),
                strategy,
                &[&[public_input]], // Adjust as necessary
                &mut transcript
            )
            .is_ok(),
            "Proof verification failed"
        );
    }

    #[test]
    fn test_1() {
        let k = 16;

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
        let mut lineitem: Vec<Vec<Fp>> = Vec::new();

        let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem.tbl";

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

        // let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem_120K.tbl";
        // if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
        //     // Convert the Vec<Region> to a 2D vector
        //     lineitem = records
        //         .iter()
        //         .map(|record| {
        //             vec![
        //                 Fp::from(record.l_quantity),
        //                 Fp::from(scale_by_1000(record.l_extendedprice)),
        //                 Fp::from(scale_by_1000(record.l_discount)),
        //                 Fp::from(scale_by_1000(record.l_tax)),
        //                 Fp::from(string_to_u64(&record.l_returnflag)),
        //                 Fp::from(string_to_u64(&record.l_linestatus)),
        //                 Fp::from(date_to_timestamp(&record.l_shipdate)),
        //                 // Fp::from(string_to_u64(&record.l_shipdate)),
        //             ]
        //         })
        //         .collect();
        // }

        // let lineitem_file_path = "/home/cc/halo2-TPCH/src/data/lineitem_240K.tbl";
        // if let Ok(records) = data_processing::lineitem_read_records_from_file(lineitem_file_path) {
        //     // Convert the Vec<Region> to a 2D vector
        //     lineitem = records
        //         .iter()
        //         .map(|record| {
        //             vec![
        //                 Fp::from(record.l_quantity),
        //                 Fp::from(scale_by_1000(record.l_extendedprice)),
        //                 Fp::from(scale_by_1000(record.l_discount)),
        //                 Fp::from(scale_by_1000(record.l_tax)),
        //                 Fp::from(string_to_u64(&record.l_returnflag)),
        //                 Fp::from(string_to_u64(&record.l_linestatus)),
        //                 Fp::from(date_to_timestamp(&record.l_shipdate)),
        //                 // Fp::from(string_to_u64(&record.l_shipdate)),
        //             ]
        //         })
        //         .collect();
        // }

        let right_shipdate = Fp::from(902102400);
        // l_shipdate <= date '1998-08-03'
        // let right_shipdate = Fp::from(2730);

        // let lineitem: Vec<Vec<Fp>> = lineitem.iter().take(10).cloned().collect();

        let circuit = MyCircuit::<Fp> {
            lineitem,

            right_shipdate,

            _marker: PhantomData,
        };

        let public_input = vec![Fp::from(1)];

        // let test = true;
        let test = false;

        if test {
            let prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
            prover.assert_satisfied();
        } else {
            let proof_path = "/home/cc/halo2-TPCH/src/sql/proof_q1";
            generate_and_verify_proof(k, circuit, &public_input, proof_path);
        }
    }
}
