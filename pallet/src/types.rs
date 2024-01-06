use sp_std::vec;
use frame_support::pallet_prelude::*;
use super::{VoteOptions, VerifyKey};

pub type PollId = u32;
pub type Timestamp = u64;
pub type Duration = Timestamp;
pub type HashBytes = [u8; 32];
pub type PollInteractionData = [[u64; 4]; 16]; 
pub type ProofData = [[u64; 4]; 16];
pub type CommitmentData = HashBytes;
pub type CommitmentIndex = u32;
pub type Commitment = (CommitmentIndex, CommitmentData);

/// Coordinator storage definition.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Coordinator<T: crate::Config> 
{
    /// The coordinators public key.
    pub public_key: PublicKey,

    /// The coordinators verify key.
    pub verify_key: VerifyKey<T>,

    /// The coordinators most recent poll (may be active).
    pub last_poll: Option<PollId>
}

/// A public key used to facillitate secret sharing between participants and coordinators.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey 
{
    /// A 256-bit x-coordinate of the public key.
    pub x: [u64; 4],

    /// A 256-bit y-coordinate of the public key.
    pub y: [u64; 4]
}

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Poll<T: crate::Config>
{
    /// The poll id.
    pub index: PollId,

    /// The poll creator.
    pub coordinator: T::AccountId,

    /// The poll creation time (in ms).
    pub created_at: Timestamp,

    /// Optional metadata associated to the poll.
    pub metadata: Option<T::Hash>,

    /// The mutable poll state.
    pub state: PollState,

    /// The poll config.
    pub config: PollConfiguration<T>
}

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PollState
{
    /// The number of registered participants. Known only after the poll has been processed.
    pub num_participants: u32,

    /// The number of registered participants. Known only after the poll has been processed.
    pub num_interactions: u32,

    /// The merkle tree of registration data.
    pub registration_tree: PollStateTree,

    /// The merkle tree of interaction data.
    pub interaction_tree: PollStateTree,

    /// The number of valid commitments witnessed.
    pub num_witnessed: u32,

    /// The current proof commitment.
    pub commitment: Commitment,

    /// The final result of the poll.
    pub outcome: Option<u128>,
}

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PollStateTree
{
    /// The depth of the subtree.
    pub subtree_depth: u8,

    /// The hashes of the incrementally merged subtree.
    pub subtree_hashes: vec::Vec<HashBytes>,

    /// The subroot of the tree.
    pub subtree_root: Option<HashBytes>,

    /// The root of the "full"-depth tree containing `subtree_root` and zeros elsewhere.
    pub root: Option<HashBytes>,
}

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PollConfiguration<T: crate::Config>
{
    /// The poll signup duration (in ms).
    pub signup_period: Duration,

    /// The poll voting duration (in ms).
    pub voting_period: Duration,

    /// The maximum number of participants permitted.
    pub max_participants: u32,

    /// The possible outcomes of the poll.
    pub vote_options: VoteOptions<T>,

    /// The size of 
    pub batch_size: u8,

    /// The arity of the state trees.
    pub tree_arity: u8
}

impl Default for PollState 
{
    fn default() -> PollState 
    {
        PollState {
            num_participants: 0,
            num_interactions: 0,
            num_witnessed: 0,
            registration_tree: PollStateTree::default(),
            interaction_tree: PollStateTree::default(),
            commitment: (0, [0; 32]),
            outcome: None
        }
    }
}

impl Default for PollStateTree
{
    fn default() -> PollStateTree
    {
        PollStateTree {
            subtree_depth: 0,
            subtree_root: None,
            subtree_hashes: vec::Vec::<HashBytes>::new(),
            root: None
        }
    }
}
