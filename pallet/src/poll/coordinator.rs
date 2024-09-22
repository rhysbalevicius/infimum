use frame_support::pallet_prelude::*;
use sp_std::vec;

use crate::poll::{PollId, PublicKey, VerifyKey};

/// A serialized groth16 proof.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ProofData
{
    pub pi_a: vec::Vec<u8>,
    pub pi_b: vec::Vec<u8>,
    pub pi_c: vec::Vec<u8>
}

/// A pair of verification keys for message processing and tally verification circuits.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct VerifyingKeys
{
    /// The verifying key for the message processing circuit.
    pub process: VerifyKey,

    /// The verifying key for the tally circuit.
    pub tally: VerifyKey
}

/// Coordinator storage definition.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Coordinator
{
    /// The coordinators public key.
    pub public_key: PublicKey,

    /// The coordinators verify key.
    pub verify_key: VerifyingKeys,

    /// The coordinators most recent poll (may be active).
    pub last_poll: Option<PollId>
}
