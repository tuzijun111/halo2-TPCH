# halo2-TPCH
To generate the proofs for SQL queries Q1, Q3, Q5, Q8, Q9 and Q18, please run the following commands at the root of the project.

cargo test --package halo2-experiments --lib -- sql::q1_final_v4::tests::test_1 --exact --nocapture

cargo test --package halo2-experiments --lib -- sql::q3_final_v7::tests::test_1 --exact --nocapture

cargo test --package halo2-experiments --lib -- sql::q5_final_v4::tests::test_1 --exact --nocapture

cargo test --package halo2-experiments --lib -- sql::q8_final_v3::tests::test_1 --exact --nocapture

cargo test --package halo2-experiments --lib -- sql::q9_final_v2::tests::test_1 --exact --nocapture

cargo test --package halo2-experiments --lib -- sql::q18_final_v2::tests::test_1 --exact --nocapture

For different datasets, please choose the correct public parameters.


