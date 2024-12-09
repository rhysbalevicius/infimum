use frame_support::pallet_prelude::*;
use sp_std::vec;

/// A zk verification key.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct VerifyKey
{
    pub alpha_g1: vec::Vec<u8>,
    pub beta_g2: vec::Vec<u8>,
    pub gamma_g2: vec::Vec<u8>,
    pub delta_g2: vec::Vec<u8>,
    pub gamma_abc_g1: vec::Vec<vec::Vec<u8>>,
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
