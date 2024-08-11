use frame_support::pallet_prelude::*;

use crate::poll::{BlockNumber, VoteOptions};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PollConfiguration<T: crate::Config>
{
    /// The number of blocks for which the registration period is active.
    pub signup_period: BlockNumber,

    /// The number of blocks for which the voting period is active.
    pub voting_period: BlockNumber,

    /// The maximum number of participants permitted.
    pub max_registrations: u32,

    /// The subtree depth to process per commitment.
    pub process_subtree_depth: u32,

    /// The possible outcomes of the poll.
    pub vote_options: VoteOptions<T>,
}
