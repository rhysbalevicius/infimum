use sp_std::vec;
use frame_support::pallet_prelude::*;
use dusk_bls12_381::BlsScalar;
use dusk_poseidon::sponge;

pub type PollId = u32;
pub type Timestamp = u64;
pub type Duration = Timestamp;
pub type HashBytes = [u8; 32];
pub type PollInteractionData = [[u64; 4]; 16]; 
pub type ProofData = [[u64; 4]; 16];
pub type CommitmentData = HashBytes;
pub type CommitmentIndex = u32;
pub type Commitment = (CommitmentIndex, CommitmentData);
pub type VerifyKey<T> = BoundedVec<u8, <T as crate::Config>::MaxVerifyKeyLength>;
pub type VoteOptions<T> = BoundedVec<u128, <T as crate::Config>::MaxVoteOptions>;

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

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PollStateTree
{
    /// The arity of the tree.
    pub arity: u8,

    /// The depth of the subtree.
    pub depth: u32,

    /// The number of non-nil leaves.
    pub count: u32,

    /// The (depth, hash) pairs of the incrementally merged subtree.
    pub hashes: vec::Vec<(u32, HashBytes)>,

    /// The root of the tree of depth `T::MaxTreeDepth` which contains
    /// the leaves of `hashes` and zeros elsewhere.
    pub root: Option<HashBytes>
}

impl Default for PollStateTree
{
    fn default() -> PollStateTree
    {
        PollStateTree {
            arity: 2,
            depth: 0,
            count: 0,
            hashes: vec::Vec::<(u32, HashBytes)>::new(),
            root: None
        }
    }
}

trait PartialMerkleStack<T: crate::Config>
{
    /// Inserts a new leaf into the tree.
    fn insert(self, data: BlsScalar) -> Self; // TODO `data` should be generic 

    
}

impl<T: crate::Config> PartialMerkleStack<T> for PollStateTree
{
    /// Consume a new leaf and produce the resultant partial merkle tree.
    /// NB This function trades off extrinsic weight for storage space. 
    ///    You can tune the proportion to which we make this trade-off by
    ///	   configuring the constant `MaxIterationDepth`.
    ///
    /// -`data`: A new right-most leaf to insert into the tree.
    ///
    fn insert(
        mut self,
        data: BlsScalar
    ) -> Self
    {
        // These elements look like: (depth, hash)
        let mut hashes: vec::Vec<(u32, BlsScalar)> = self.hashes
            .iter()
            .map(|(depth, hash_bytes)| (*depth, BlsScalar::from_bytes(hash_bytes).unwrap_or(BlsScalar::zero())))
            .collect();
        
        hashes.push((1, data));

        // Hash `arity` hashes of equivalent depth until either the depth is exhausted,
        // or there are insufficiently many right-most hashes of equal depth. 
        let arity: usize = self.arity.into();
        let mut depth: u32 = 1;
        loop
        {
            // Guard against the maximal hash depth that can be reached from any individual `insert` operation
            if depth > T::MaxIterationDepth::get() { break; }

            let size = hashes.len();
            if size < arity { break; }

            // Find the index of the first item with a different depth
            let Some(index) = hashes.iter().rposition(|(d,_)| *d != depth) else { break };
            if index + arity != size - 1 { break };

            let subtree: vec::Vec<BlsScalar> = hashes
                .split_off(size.saturating_sub(arity))
                .iter()
                .map(|(_d,h)| *h)
                .collect();

            depth += 1;
            hashes.push((depth, sponge::hash(&subtree)));
        }

        if let Some(hash) = hashes.first() { self.depth = hash.0; }
        self.hashes = hashes.iter().map(|(d, h)| (*d, h.to_bytes())).collect();
        self.count += 1;

        self
    }

    
}
