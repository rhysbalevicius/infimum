use frame_support::pallet_prelude::*;
use sp_std::vec;

use crate::poll::{
    CommitmentIndex,
    CommitmentData,
    PollId,
    PublicKey,
    VerifyKey,
    HashBytes
};

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

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Commitment
{
    /// The commitment to the message processing circuit. 
    pub process: (CommitmentIndex, CommitmentData),

    /// The commitment to the tallying circuit.
    pub tally: (CommitmentIndex, CommitmentData),

    /// The expected number of process commitments.
    pub expected_process: CommitmentIndex,

    /// The expected number of tally commitments.
    pub expected_tally: CommitmentIndex
}

/// A serialized groth16 proof.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ProofData
{
    pub pi_a: vec::Vec<u8>,
    pub pi_b: vec::Vec<u8>,
    pub pi_c: vec::Vec<u8>
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PollOutcome
{
    /// The results of the tally per option.
    pub tally_results: vec::Vec<u32>,

    /// The proof of correctness for the results.
    pub tally_result_proofs: vec::Vec<vec::Vec<vec::Vec<HashBytes>>>,

    /// The total number of votes cast represented as a (big-endian) byte array.
    pub total_spent: HashBytes,

    /// The salt for the total votes.
    pub total_spent_salt: HashBytes,

    /// The salt for the tally results.
    pub tally_result_salt: HashBytes,

    /// The salted commitment of the vote tally.
    pub new_results_commitment: HashBytes,

    /// The hash of the spent votes and salt.
    pub spent_votes_hash: HashBytes
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
