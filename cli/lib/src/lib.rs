use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use num_bigint::BigUint;
use num_traits::Num;
use std::str::FromStr;
use ark_bn254::{
    // Bn254,
    // Fr,
    Fq, 
    Fq2, 
    G1Affine, 
    G1Projective, 
    G2Affine, 
    G2Projective
};
use ark_ff::{
    BigInteger256, 
    // PrimeField
};
use ark_serialize::{
    CanonicalSerialize, 
    // CanonicalDeserialize
};
// use ark_crypto_primitives::snark::SNARK;
// use ark_groth16::{
//     Groth16,
//     data_structures::Proof,
//     data_structures::VerifyingKey
// };

#[derive(Serialize, Deserialize)]
pub struct BytesJs
{
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct BigNumberJs
{
    pub value: String,
}

#[wasm_bindgen]
pub fn bytes_be_to_bn(input_js: JsValue) -> Result<JsValue, JsError>
{
    let input: BytesJs = serde_wasm_bindgen::from_value(input_js).unwrap();
    let output = BigNumberJs {
        value: BigUint::from_bytes_be(&input.value).to_string()
    };

    Ok(serde_wasm_bindgen::to_value(&output).unwrap())
}

#[wasm_bindgen]
pub fn bn_to_bytes_be(input_js: JsValue) -> Result<JsValue, JsError>
{
    let input: BigNumberJs = serde_wasm_bindgen::from_value(input_js).unwrap();
    let output = BytesJs {
        value: BigUint::from_str_radix(&input.value, 10).unwrap().to_bytes_be()
    };

    Ok(serde_wasm_bindgen::to_value(&output).unwrap())
}

#[derive(Serialize, Deserialize)]
pub struct VerifyingKeyBigNumber
{
    pub vk_alpha_1: [String; 3],
    pub vk_beta_2: [[String; 2]; 3],
    pub vk_gamma_2: [[String; 2]; 3],
    pub vk_delta_2: [[String; 2]; 3],
    pub ic: Vec<[String; 3]>,
}

#[derive(Serialize, Deserialize)]
pub struct VerifyingKeyByteVector
{
    pub alpha_g1: Vec<u8>,
    pub beta_g2: Vec<u8>,
    pub gamma_g2: Vec<u8>,
    pub delta_g2: Vec<u8>,
    pub gamma_abc_g1: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub struct ProofBigNumber
{
    pub pi_a: [String; 3],
    pub pi_b: [[String; 2]; 3],
    pub pi_c: [String; 3],
}

#[derive(Serialize, Deserialize)]
pub struct ProofByteVector
{
    pub pi_a: Vec<u8>,
    pub pi_b: Vec<u8>,
    pub pi_c: Vec<u8>,
}

fn serialize_point_to_bytes<T: CanonicalSerialize>(point: &T) -> Vec<u8>
{
    let mut bytes = Vec::new();
    point.serialize_uncompressed(&mut bytes).expect("Serialization failed");
    bytes
}

fn fq_from_str(s: &str) -> Fq
{
    BigInteger256::try_from(BigUint::from_str(s).unwrap())
        .unwrap()
        .into()
}

fn g1_bn_to_bytes(els: [String; 3]) -> Vec<u8>
{
    let g1 = G1Affine::from(G1Projective::new(
        fq_from_str(&els[0]),
        fq_from_str(&els[1]),
        fq_from_str(&els[2]),
    ));

    serialize_point_to_bytes(&g1)
}

fn g1_bn_vec_to_bytes(els: Vec<[String; 3]>) -> Vec<Vec<u8>> 
{
    els.into_iter()
        .map(g1_bn_to_bytes)
        .collect()
}

fn g2_bn_to_bytes(els: [[String; 2]; 3]) -> Vec<u8>
{
    let x = Fq2::new(fq_from_str(&els[0][0]), fq_from_str(&els[0][1]));
    let y = Fq2::new(fq_from_str(&els[1][0]), fq_from_str(&els[1][1]));
    let z = Fq2::new(fq_from_str(&els[2][0]), fq_from_str(&els[2][1]));
    let g2 = G2Affine::from(G2Projective::new(x, y, z));

    serialize_point_to_bytes(&g2)
}

#[wasm_bindgen]
pub fn serialize_vkey(
    vkey_js: JsValue
) -> Result<JsValue, JsError>
{
    let vkey_bn: VerifyingKeyBigNumber = serde_wasm_bindgen::from_value(vkey_js).unwrap();

    let vkey = VerifyingKeyByteVector {
        alpha_g1: g1_bn_to_bytes(vkey_bn.vk_alpha_1),
        beta_g2: g2_bn_to_bytes(vkey_bn.vk_beta_2),
        gamma_g2: g2_bn_to_bytes(vkey_bn.vk_gamma_2),
        delta_g2: g2_bn_to_bytes(vkey_bn.vk_delta_2),
        gamma_abc_g1: g1_bn_vec_to_bytes(vkey_bn.ic)
    };

    Ok(serde_wasm_bindgen::to_value(&vkey).unwrap())
}

#[wasm_bindgen]
pub fn serialize_proof(
    proof_js: JsValue
) -> Result<JsValue, JsError>
{
    let proof_bn: ProofBigNumber = serde_wasm_bindgen::from_value(proof_js).unwrap();

    let proof = ProofByteVector {
        pi_a: g1_bn_to_bytes(proof_bn.pi_a),
        pi_b: g2_bn_to_bytes(proof_bn.pi_b),
        pi_c: g1_bn_to_bytes(proof_bn.pi_c),
    };

    Ok(serde_wasm_bindgen::to_value(&proof).unwrap())
}

// #[derive(Serialize, Deserialize)]
// pub struct ImageByteVector
// {
//     pub hash: String
// }

// #[wasm_bindgen]
// pub fn verify_proof(
//     pf_js: JsValue,
//     vkey_js: JsValue,
//     image_js: JsValue
// ) -> Result<bool, JsError>//Result<JsValue, JsError>
// {
//     let vkey: VerifyingKeyByteVector = serde_wasm_bindgen::from_value(vkey_js).unwrap();
//     let pf: ProofByteVector = serde_wasm_bindgen::from_value(pf_js).unwrap();
//     let img: ImageByteVector = serde_wasm_bindgen::from_value(image_js).unwrap();

//     let a = G1Affine::deserialize_uncompressed(&*pf.pi_a).unwrap();
//     let b = G2Affine::deserialize_uncompressed(&*pf.pi_b).unwrap();
//     let c = G1Affine::deserialize_uncompressed(&*pf.pi_c).unwrap();

//     let alpha_g1 = G1Affine::deserialize_uncompressed(&*vkey.alpha_g1).unwrap();
//     let beta_g2 = G2Affine::deserialize_uncompressed(&*vkey.beta_g2).unwrap();
//     let gamma_g2 = G2Affine::deserialize_uncompressed(&*vkey.gamma_g2).unwrap();
//     let delta_g2 = G2Affine::deserialize_uncompressed(&*vkey.delta_g2).unwrap();
//     let gamma_abc_g1 = vkey.gamma_abc_g1
//         .iter()
//         .map(|g| G1Affine::deserialize_uncompressed(g.as_slice()))
//         .collect::<Result<_, _>>()
//         .unwrap();

//     let proof = Proof::<Bn254> { a, b, c };
//     let verify_key = VerifyingKey::<Bn254> { alpha_g1, beta_g2, gamma_g2, delta_g2, gamma_abc_g1 };
//     let pvk = Groth16::<Bn254>::process_vk(&verify_key).unwrap();

//     // let inputs: Vec<Fr> = img.inputs
//     //     .iter()
//     //     .map(|g| Fr::deserialize_uncompressed(g.as_slice()))
//     //     .collect::<Result<_, _>>()
//     //     .unwrap();

//     let bi_image = BigUint::from_str_radix(&img.hash, 10).unwrap();
//     let image = Fr::from_le_bytes_mod_order(&bi_image.to_bytes_le());

//     Ok(Groth16::<Bn254>::verify_with_processed_vk(&pvk, &[image], &proof).unwrap())
// }
