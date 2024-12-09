use ark_bn254::Fr;
use ark_ff::{
    BigInteger,
    One,
    PrimeField,
    Zero
};
use crate::hash::{
    Poseidon,
    PoseidonError,
    PoseidonHasher,
    PoseidonBytesHasher
};

/// Check the hash of `1` as a prime field element.
#[test]
fn fr_one()
{
    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();

    let expected = [
        0, 122, 243, 70, 226, 211, 4, 39, 158, 121, 224, 169, 243, 2, 63, 119, 18, 148, 167, 138,
        203, 112, 231, 63, 144, 175, 226, 124, 173, 64, 30, 129,
    ];

    let input = Fr::from_be_bytes_mod_order(&[1u8]);
    let hash = hasher.hash(&[input, input]).unwrap();

    assert_eq!(hash.into_bigint().to_bytes_be(), expected);

    let input = Fr::from_be_bytes_mod_order(&[0u8, 1u8]);
    let hash = hasher.hash(&[input, input]).unwrap();

    assert_eq!(hash.into_bigint().to_bytes_be(), expected);

    let input = Fr::from_be_bytes_mod_order(&[0u8, 0u8, 1u8]);
    let hash = hasher.hash(&[input, input]).unwrap();

    assert_eq!(hash.into_bigint().to_bytes_be(), expected);
}

/// Checks the hash of byte slices consistng of ones and twos.
#[test]
fn bytes_ones_twos()
{
    let input1 = Fr::from_be_bytes_mod_order(&[1u8; 32]);
    let input2 = Fr::from_be_bytes_mod_order(&[2u8; 32]);
    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
    let hash = hasher.hash(&[input1, input2]).unwrap();
    assert_eq!(
        hash.into_bigint().to_bytes_be(),
        [
            13, 84, 225, 147, 143, 138, 140, 28, 125, 235, 94, 3, 85, 242, 99, 25, 32, 123, 132,
            254, 156, 162, 206, 27, 38, 231, 53, 200, 41, 130, 25, 144
        ]
    );

    let hash = hasher.hash_bytes_be(&[&[1u8; 32], &[2u8; 32]]).unwrap();
    assert_eq!(
        hash,
        [
            13, 84, 225, 147, 143, 138, 140, 28, 125, 235, 94, 3, 85, 242, 99, 25, 32, 123, 132,
            254, 156, 162, 206, 27, 38, 231, 53, 200, 41, 130, 25, 144
        ]
    );

    let hash = hasher.hash_bytes_le(&[&[1u8; 32], &[2u8; 32]]).unwrap();
    assert_eq!(
        hash,
        [
            144, 25, 130, 41, 200, 53, 231, 38, 27, 206, 162, 156, 254, 132, 123, 32, 25, 99, 242,
            85, 3, 94, 235, 125, 28, 140, 138, 143, 147, 225, 84, 13
        ]
    )
}

/// Checks the hash of bytes slices consisting of ones and twos, with a custom domain tag.
#[test]
fn with_domain_tag()
{
    let input1 = Fr::from_be_bytes_mod_order(&[1u8; 32]);
    let input2 = Fr::from_be_bytes_mod_order(&[2u8; 32]);
    let mut hasher = Poseidon::<Fr>::with_domain_tag_circom(2, Fr::zero()).unwrap();
    let hash = hasher.hash(&[input1, input2]).unwrap();

    let expected_tag_zero = [
        13, 84, 225, 147, 143, 138, 140, 28, 125, 235, 94, 3, 85, 242, 99, 25, 32, 123, 132, 254,
        156, 162, 206, 27, 38, 231, 53, 200, 41, 130, 25, 144,
    ];

    assert_eq!(hash.into_bigint().to_bytes_be(), expected_tag_zero);

    let mut hasher = Poseidon::<Fr>::with_domain_tag_circom(2, Fr::one()).unwrap();
    let hash = hasher.hash(&[input1, input2]).unwrap();

    assert_ne!(hash.into_bigint().to_bytes_be(), expected_tag_zero);
}

/// Check the hash of one and two.
#[test]
fn fr_one_two()
{
    let input1 = Fr::from_be_bytes_mod_order(&[1]);
    let input2 = Fr::from_be_bytes_mod_order(&[2]);

    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
    let hash = hasher.hash(&[input1, input2]).unwrap();

    assert_eq!(
        hash.into_bigint().to_bytes_le(),
        [
            154, 24, 23, 68, 122, 96, 25, 158, 81, 69, 50, 116, 242, 23, 54, 42, 207, 233, 98, 150,
            107, 76, 246, 61, 65, 144, 214, 231, 245, 192, 92, 17
        ]
    );
}

#[test]
fn random_input()
{
    let input1 = Fr::from_be_bytes_mod_order(&[ 93, 202, 70, 122, 46, 238, 242, 161, 142, 171, 237, 131, 78, 254, 47, 96, 170, 173, 24, 112, 8, 112, 73, 123, 248, 7, 9, 75, 55, 214, 196, 114 ]);
    let input2 = Fr::from_be_bytes_mod_order(&[ 131, 162, 129, 115, 20, 245, 254, 5, 200, 101, 156, 226, 102, 57, 207, 152, 105, 122, 29, 235, 131, 196, 247, 239, 5, 252, 253, 181, 251, 93, 114, 219 ]);

    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
    let hash = hasher.hash(&[input1, input2]).unwrap();
    assert_eq!(
        hash.into_bigint().to_bytes_le(),
        [ 64, 118, 212, 28, 127, 187, 234, 52, 44, 113, 111, 106, 189, 79, 8, 95, 185, 37, 62, 152, 72, 127, 150, 110, 238, 135, 124, 47, 20, 139, 115, 36 ]
    )
}

/// Checks whether providing an empty input results in an error.
#[test]
fn empty_input() 
{
    let empty: &[u8] = &[];
    let non_empty = &[1u8; 32];

    // All inputs empty.
    for nr_inputs in 1..12 
    {
        let mut hasher = Poseidon::<Fr>::new_circom(nr_inputs).unwrap();

        let mut inputs = Vec::with_capacity(nr_inputs);
        for _ in 0..nr_inputs 
        {
            inputs.push(empty);
        }

        let hash = hasher.hash_bytes_be(inputs.as_slice());
        assert_eq!(hash, Err(PoseidonError::EmptyInput));

        let hash = hasher.hash_bytes_le(inputs.as_slice());
        assert_eq!(hash, Err(PoseidonError::EmptyInput));
    }

    // One empty input.
    for nr_inputs in 1..12 
    {
        let mut hasher = Poseidon::<Fr>::new_circom(nr_inputs).unwrap();

        let mut inputs = Vec::with_capacity(nr_inputs);
        for _ in 0..(nr_inputs - 1) 
        {
            inputs.push(non_empty.as_slice());
        }
        inputs.push(empty);

        let hash = hasher.hash_bytes_be(inputs.as_slice());
        assert_eq!(hash, Err(PoseidonError::EmptyInput));

        let hash = hasher.hash_bytes_le(inputs.as_slice());
        assert_eq!(hash, Err(PoseidonError::EmptyInput));
    }
}

// Test cases were created with circomlibjs poseidon([1, ...]) for 1 to 16 inputs
const CIRCOMLIBJS_TEST_CASES: [[u8; 32]; 12] = [
    [
        41, 23, 97, 0, 234, 169, 98, 189, 193, 254, 108, 101, 77, 106, 60, 19, 14, 150, 164, 209,
        22, 139, 51, 132, 139, 137, 125, 197, 2, 130, 1, 51,
    ],
    [
        0, 122, 243, 70, 226, 211, 4, 39, 158, 121, 224, 169, 243, 2, 63, 119, 18, 148, 167, 138,
        203, 112, 231, 63, 144, 175, 226, 124, 173, 64, 30, 129,
    ],
    [
        2, 192, 6, 110, 16, 167, 42, 189, 43, 51, 195, 178, 20, 203, 62, 129, 188, 177, 182, 227,
        9, 97, 205, 35, 194, 2, 177, 134, 115, 191, 37, 67,
    ],
    [
        8, 44, 156, 55, 10, 13, 36, 244, 65, 111, 188, 65, 74, 55, 104, 31, 120, 68, 45, 39, 216,
        99, 133, 153, 28, 23, 214, 252, 12, 75, 125, 113,
    ],
    [
        16, 56, 150, 5, 174, 104, 141, 79, 20, 219, 133, 49, 34, 196, 125, 102, 168, 3, 199, 43,
        65, 88, 156, 177, 191, 134, 135, 65, 178, 6, 185, 187,
    ],
    [
        42, 115, 246, 121, 50, 140, 62, 171, 114, 74, 163, 229, 189, 191, 80, 179, 144, 53, 215,
        114, 159, 19, 91, 151, 9, 137, 15, 133, 197, 220, 94, 118,
    ],
    [
        34, 118, 49, 10, 167, 243, 52, 58, 40, 66, 20, 19, 157, 157, 169, 89, 190, 42, 49, 178,
        199, 8, 165, 248, 25, 84, 178, 101, 229, 58, 48, 184,
    ],
    [
        23, 126, 20, 83, 196, 70, 225, 176, 125, 43, 66, 51, 66, 81, 71, 9, 92, 79, 202, 187, 35,
        61, 35, 11, 109, 70, 162, 20, 217, 91, 40, 132,
    ],
    [
        14, 143, 238, 47, 228, 157, 163, 15, 222, 235, 72, 196, 46, 187, 68, 204, 110, 231, 5, 95,
        97, 251, 202, 94, 49, 59, 138, 95, 202, 131, 76, 71,
    ],
    [
        46, 196, 198, 94, 99, 120, 171, 140, 115, 48, 133, 79, 74, 112, 119, 193, 255, 146, 96,
        228, 72, 133, 196, 184, 29, 209, 49, 173, 58, 134, 205, 150,
    ],
    [
        0, 113, 61, 65, 236, 166, 53, 241, 23, 212, 236, 188, 235, 95, 58, 102, 220, 65, 66, 235,
        112, 181, 103, 101, 188, 53, 143, 27, 236, 64, 187, 155,
    ],
    [
        20, 57, 11, 224, 186, 239, 36, 155, 212, 124, 101, 221, 172, 101, 194, 229, 46, 133, 19,
        192, 129, 193, 205, 114, 201, 128, 6, 9, 142, 154, 143, 190,
    ],
];

/// Check compatibility with circomlibjs.
#[test]
fn circomlibjs_compat_1_to_12_inputs()
{
    let mut inputs = Vec::new();
    let value = [vec![0u8; 31], vec![1u8]].concat();
    for i in 1..13 
    {
        inputs.push(value.as_slice());
        let mut hasher = Poseidon::<Fr>::new_circom(i).unwrap();
        let hash = hasher.hash_bytes_be(&inputs[..]).unwrap();
        assert_eq!(hash, CIRCOMLIBJS_TEST_CASES[i - 1]);
    }
    let mut inputs = Vec::new();
    let value = [vec![0u8; 31], vec![2u8]].concat();
    for i in 1..13 
    {
        inputs.push(value.as_slice());
        let mut hasher = Poseidon::<Fr>::new_circom(i).unwrap();
        let hash = hasher.hash_bytes_be(&inputs[..]).unwrap();
        assert!(hash != CIRCOMLIBJS_TEST_CASES[i - 1]);
    }
}
