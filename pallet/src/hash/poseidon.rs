use sp_std::prelude::*;
use sp_std::vec::Vec;
use sp_std::convert::{TryInto};
use sp_std::result::Result;

use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField, Zero};

pub const HASH_LEN: usize = 32;
pub const MAX_X5_LEN: usize = 13;

#[derive(Debug, PartialEq)]
pub enum PoseidonError
{
    InvalidNumberOfInputs {
        inputs: usize,
        max_limit: usize,
        width: usize,
    },
    EmptyInput,
    InvalidInputLength {
        len: usize,
        modulus_bytes_len: usize,
    },
    BytesToPrimeFieldElement { bytes: Vec<u8> },
    InputLargerThanModulus,
    VecToArray,
    U64ToU8,
    BytesToBigInt,
    InvalidWidthCircom { width: usize, max_limit: usize },
}

/// Parameters for the Poseidon hash algorithm.
pub struct PoseidonParameters<F: PrimeField>
{
    /// Round constants.
    pub ark: Vec<F>,
    /// MDS matrix.
    pub mds: Vec<Vec<F>>,
    /// Number of full rounds (where S-box is applied to all elements of the
    /// state).
    pub full_rounds: usize,
    /// Number of partial rounds (where S-box is applied only to the first
    /// element of the state).
    pub partial_rounds: usize,
    /// Number of prime fields in the state.
    pub width: usize,
    /// Exponential used in S-box to power elements of the state.
    pub alpha: u64,
}

impl<F: PrimeField> PoseidonParameters<F> 
{
    pub fn new(
        ark: Vec<F>,
        mds: Vec<Vec<F>>,
        full_rounds: usize,
        partial_rounds: usize,
        width: usize,
        alpha: u64,
    ) -> Self {
        Self {
            ark,
            mds,
            full_rounds,
            partial_rounds,
            width,
            alpha,
        }
    }
}

/// Trait for hashing inputs that are prime field elements.
pub trait PoseidonHasher<F: PrimeField>
{
    /// Calculates a Poseidon hash for the given input of prime fields and
    /// returns the result as a prime field.
    fn hash(&mut self, inputs: &[F]) -> Result<F, PoseidonError>;
}

/// Trait for hashing inputs that are byte slices.
pub trait PoseidonBytesHasher
{
    /// Calculates a Poseidon hash for the given input of big-endian byte slices
    /// and returns the result as a byte array.
    fn hash_bytes_be(&mut self, inputs: &[&[u8]]) -> Result<[u8; HASH_LEN], PoseidonError>;

    /// Calculates a Poseidon hash for the given input of little-endian byte
    /// slices and returns the result as a byte array.
    fn hash_bytes_le(&mut self, inputs: &[&[u8]]) -> Result<[u8; HASH_LEN], PoseidonError>;
}

/// A stateful sponge performing Poseidon hash computation.
pub struct Poseidon<F: PrimeField>
{
    params: PoseidonParameters<F>,
    domain_tag: F,
    state: Vec<F>,
}

impl<F: PrimeField> Poseidon<F>
{
    /// Returns a new Poseidon hasher based on the given parameters.
    ///
    /// Optionally, a domain tag can be provided. If it is not provided, it
    /// will be set to zero.
    pub fn new(params: PoseidonParameters<F>) -> Self 
    {
        Self::with_domain_tag(params, F::zero())
    }

    fn with_domain_tag(params: PoseidonParameters<F>, domain_tag: F) -> Self 
    {
        let width = params.width;
        Self {
            domain_tag,
            params,
            state: Vec::with_capacity(width),
        }
    }

    #[inline(always)]
    fn apply_ark(&mut self, round: usize) 
    {
        self.state.iter_mut().enumerate().for_each(|(i, a)| {
            let c = self.params.ark[round * self.params.width + i];
            *a += c;
        });
    }

    #[inline(always)]
    fn apply_sbox_full(&mut self) 
    {
        self.state.iter_mut().for_each(|a| {
            *a = a.pow([self.params.alpha]);
        });
    }

    #[inline(always)]
    fn apply_sbox_partial(&mut self) 
    {
        self.state[0] = self.state[0].pow([self.params.alpha]);
    }

    #[inline(always)]
    fn apply_mds(&mut self) 
    {
        let new_state: Vec<F> = (0..self.state.len())
            .map(|i| {
                self.state
                    .iter()
                    .enumerate()
                    .fold(F::zero(), |acc, (j, a)| acc + *a * self.params.mds[i][j])
            })
            .collect();
        self.state = new_state;
    }
}

impl<F: PrimeField> PoseidonHasher<F> for Poseidon<F> 
{
    fn hash(&mut self, inputs: &[F]) -> Result<F, PoseidonError> 
    {
        if inputs.len() != self.params.width - 1 
        {
            return Err(PoseidonError::InvalidNumberOfInputs {
                inputs: inputs.len(),
                max_limit: self.params.width - 1,
                width: self.params.width,
            });
        }

        self.state.push(self.domain_tag);

        for input in inputs 
        {
            self.state.push(*input);
        }

        let all_rounds = self.params.full_rounds + self.params.partial_rounds;
        let half_rounds = self.params.full_rounds / 2;

        // full rounds + partial rounds
        for round in 0..half_rounds 
        {
            self.apply_ark(round);
            self.apply_sbox_full();
            self.apply_mds();
        }

        for round in half_rounds..half_rounds + self.params.partial_rounds 
        {
            self.apply_ark(round);
            self.apply_sbox_partial();
            self.apply_mds();
        }

        for round in half_rounds + self.params.partial_rounds..all_rounds 
        {
            self.apply_ark(round);
            self.apply_sbox_full();
            self.apply_mds();
        }

        let result = self.state[0];
        self.state.clear();
        Ok(result)
    }
}

impl<F: PrimeField> PoseidonBytesHasher for Poseidon<F> 
{
    fn hash_bytes_be(&mut self, inputs: &[&[u8]]) -> Result<[u8; HASH_LEN], PoseidonError> 
    {
        let inputs: Result<Vec<F>, PoseidonError> = inputs
            .iter()
            .map(|input| {
                validate_bytes_length::<F>(input)?;
                let mut input_reversed = input.to_vec();
                input_reversed.reverse();
                bytes_to_prime_field_element::<F>(&input_reversed)
            })
            .collect();
        let inputs = inputs?;
        let hash = self.hash(&inputs)?;

        let mut bytes = hash.into_bigint().to_bytes_le();
        bytes.reverse(); // Convert to big-endian
        bytes
            .try_into()
            .map_err(|_| PoseidonError::VecToArray)
    }

    fn hash_bytes_le(&mut self, inputs: &[&[u8]]) -> Result<[u8; HASH_LEN], PoseidonError> 
    {
        let inputs: Result<Vec<F>, PoseidonError> = inputs
            .iter()
            .map(|input| {
                validate_bytes_length::<F>(input)?;
                bytes_to_prime_field_element::<F>(input)
            })
            .collect();
        let inputs = inputs?;
        let hash = self.hash(&inputs)?;

        let bytes = hash.into_bigint().to_bytes_le();
        bytes
            .try_into()
            .map_err(|_| PoseidonError::VecToArray)
    }
}

/// Checks whether a slice of bytes is not empty or its length does not exceed
/// the modulus size of the prime field. If it does, an error is returned.
pub fn validate_bytes_length<F>(input: &[u8]) -> Result<(), PoseidonError>
where
    F: PrimeField,
{
    let modulus_bytes_len = ((F::MODULUS_BIT_SIZE + 7) / 8) as usize;

    if input.is_empty() 
    {
        return Err(PoseidonError::EmptyInput);
    }
    if input.len() > modulus_bytes_len 
    {
        return Err(PoseidonError::InvalidInputLength {
            len: input.len(),
            modulus_bytes_len,
        });
    }
    Ok(())
}

pub fn bytes_to_prime_field_element<F>(input: &[u8]) -> Result<F, PoseidonError>
where
    F: PrimeField,
{
    // Ensure the input length matches the modulus size
    let modulus_bytes_len = ((F::MODULUS_BIT_SIZE + 7) / 8) as usize;

    if input.len() != modulus_bytes_len
    {
        return Err(PoseidonError::InvalidInputLength {
            len: input.len(),
            modulus_bytes_len,
        });
    }

    // Use from_le_bytes_mod_order (since we reversed the bytes for big-endian)
    let element = F::from_le_bytes_mod_order(input);

    // Ensure the element is less than the modulus
    if element.into_bigint() >= F::MODULUS
    {
        return Err(PoseidonError::InputLargerThanModulus);
    }

    Ok(element)
}

impl<F: PrimeField> Poseidon<F>
{
    pub fn new_circom(nr_inputs: usize) -> Result<Poseidon<Fr>, PoseidonError>
    {
        Self::with_domain_tag_circom(nr_inputs, Fr::zero())
    }

    pub fn with_domain_tag_circom(
        nr_inputs: usize,
        domain_tag: Fr,
    ) -> Result<Poseidon<Fr>, PoseidonError>
    {
        let width = nr_inputs + 1;
        if width > MAX_X5_LEN {
            return Err(PoseidonError::InvalidWidthCircom {
                width,
                max_limit: MAX_X5_LEN,
            });
        }

        let params = crate::hash::parameters::get_poseidon_parameters::<Fr>(
            width.try_into().map_err(|_| PoseidonError::U64ToU8)?,
        )?;
        Ok(Poseidon::<Fr>::with_domain_tag(params, domain_tag))
    }
}
    