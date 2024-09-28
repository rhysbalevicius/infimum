use frame_support::pallet_prelude::*;
use sp_std::vec;
use crate::poll::{PollConfiguration, PollState, ProofData};
use crate::hash::poseidon::{HASH_LEN};

pub type BlockNumber = u64;
pub type CommitmentIndex = u32;
pub type CommitmentData = HashBytes;
pub type HashBytes = [u8; HASH_LEN];
pub type Outcome = u128;
pub type OutcomeIndex = u32;
pub type PollId = u32;
pub type PollInteractionData = [[u8; 32]; 10]; 
pub type ProofBatches = vec::Vec<(ProofData, CommitmentData)>;
pub type VoteOptions<T> = BoundedVec<u128, <T as crate::Config>::MaxVoteOptions>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Poll<T: crate::Config>
{
    /// The poll id.
    pub index: PollId,

    /// The poll creator.
    pub coordinator: T::AccountId,

    /// The number of the block in which the poll was created.
    pub created_at: BlockNumber,

    /// The mutable poll state.
    pub state: PollState,

    /// The poll config.
    pub config: PollConfiguration<T>
}
