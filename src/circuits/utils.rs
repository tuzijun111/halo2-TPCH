use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr as Fp, G1Affine},
    plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, Circuit},
    poly::{
        commitment::{Params, ParamsProver},
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use rand::rngs::OsRng;
use std::time::Instant;
use std::{fs::File, io::Write, path::Path};

pub fn full_prover<C: Circuit<Fp>>(circuit: C, k: u32, public_input: &[Fp], proof_path: &str) {
    let params_time_start = Instant::now();
    let params = ParamsKZG::<Bn256>::setup(k, OsRng);
    let params_path = "/home/cc/halo2-TPCH/src/sql/kzg_param18";
    let mut fd = std::fs::File::create(&params_path).unwrap();
    params.write(&mut fd).unwrap();
    let params_time = params_time_start.elapsed();

    // let mut fd = std::fs::File::open(&params_path).unwrap();
    // let params = ParamsKZG::<Bn256>::read(&mut fd).unwrap();

    // let vk_time_start = Instant::now();
    // let vk = keygen_vk(&params, &circuit).unwrap();
    // let vk_time = vk_time_start.elapsed();

    // let pk_time_start = Instant::now();
    // let pk = keygen_pk(&params, vk, &circuit).unwrap();
    // let pk_time = pk_time_start.elapsed();

    // let proof_time_start = Instant::now();
    // let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    // create_proof::<
    //     KZGCommitmentScheme<Bn256>,
    //     ProverSHPLONK<'_, Bn256>,
    //     Challenge255<G1Affine>,
    //     _,
    //     Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
    //     _,
    // >(
    //     &params,
    //     &pk,
    //     &[circuit],
    //     &[&[public_input]],
    //     OsRng,
    //     &mut transcript,
    // )
    // .expect("prover should not fail");
    // let proof = transcript.finalize();
    // // Write proof to file
    // File::create(Path::new(proof_path))
    //     .expect("Failed to create proof file")
    //     .write_all(&proof)
    //     .expect("Failed to write proof");

    // let proof_time = proof_time_start.elapsed();

    // let verifier_params = params.verifier_params();
    // let verify_time_start = Instant::now();
    // let strategy = SingleStrategy::new(&params);
    // let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);
    // assert!(verify_proof::<
    //     KZGCommitmentScheme<Bn256>,
    //     VerifierSHPLONK<'_, Bn256>,
    //     Challenge255<G1Affine>,
    //     Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
    //     SingleStrategy<'_, Bn256>,
    // >(
    //     verifier_params,
    //     pk.get_vk(),
    //     strategy,
    //     &[&[public_input]],
    //     &mut transcript
    // )
    // .is_ok());
    // let verify_time = verify_time_start.elapsed();

    println!("Time to generate params {:?}", params_time);
    // println!("Time to generate vk {:?}", vk_time);
    // println!("Time to generate pk {:?}", pk_time);
    // println!("Prover Time {:?}", proof_time);
    // println!("Verifier Time {:?}", verify_time);
}
