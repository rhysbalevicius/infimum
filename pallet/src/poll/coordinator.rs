use frame_support::pallet_prelude::*;

use crate::poll::{PollId};

/// A zk verification key.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct VerifyKey
{
    pub alpha_g1: Vec<u8>,
    pub beta_g2: Vec<u8>,
    pub gamma_g2: Vec<u8>,
    pub delta_g2: Vec<u8>,
    pub gamma_abc_g1: Vec<Vec<u8>>,
}

/// A serialized groth16 proof.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ProofData
{
    pub pi_a: Vec<u8>,
    pub pi_b: Vec<u8>,
    pub pi_c: Vec<u8>
}

/// Coordinator storage definition.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Coordinator
{
    /// The coordinators public key.
    pub public_key: PublicKey,

    /// The coordinators verify key.
    pub verify_key: VerifyKey,

    /// The coordinators most recent poll (may be active).
    pub last_poll: Option<PollId>
}

/// A public key used to facillitate secret sharing between participants and coordinators.
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey 
{
    /// A 256-bit x-coordinate of the public key.
    pub x: [u8; 32],

    /// A 256-bit y-coordinate of the public key.
    pub y: [u8; 32]
}
