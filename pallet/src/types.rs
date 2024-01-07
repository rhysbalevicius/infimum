use sp_std::vec;
use frame_support::pallet_prelude::*;
use sp_runtime::traits::SaturatedConversion;
use dusk_bls12_381::BlsScalar;
use dusk_poseidon::sponge;

pub type BlockNumber = u64;
pub type CommitmentData = HashBytes;
pub type CommitmentIndex = u32;
pub type Commitment = (CommitmentIndex, CommitmentData);
pub type HashBytes = [u8; 32];
pub type Outcome = u128;
pub type OutcomeIndex = u32;
pub type PollId = u32;
pub type PollInteractionData = [[u64; 4]; 16]; 
pub type ProofData = [[u64; 4]; 16];
pub type ProofBatches = vec::Vec<(ProofData, CommitmentData)>;
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
#[derive(Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
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

    /// The number of the block in which the poll was created.
    pub created_at: BlockNumber,

    /// The mutable poll state.
    pub state: PollState<T>,

    /// The poll config.
    pub config: PollConfiguration<T>
}

/// ...
pub trait PollProvider<T: crate::Config>
{
    /// ...
    fn register_participant(
        self, 
        public_key: PublicKey, 
        timestamp: u64
    ) -> (u32, Self);
    /// ...
    fn consume_interaction(
        self,
        public_key: PublicKey,
        data: PollInteractionData
    ) -> (u32, Self);
    // /// ...
    // fn prove(
    //     self,
    //     verify_key: VerifyKey<T>,
    //     batches: ProofBatches
    // ) -> Self;
    // /// ...
    // fn verify(
    //     &self,
    //     outcome: Option<u32>
    // ) -> Result<u32, crate::Error<T>>;
    // /// ...
    // fn is_verifiable(&self) -> bool;
    /// ...
    fn is_merged(&self) -> bool;
    /// ...
    fn registration_limit_reached(&self) -> bool;
    /// ...
    fn interaction_limit_reached(&self) -> bool;
    /// ...
    fn is_voting_period(&self) -> bool;
    /// ...
    fn is_registration_period(&self) -> bool;
    /// ...
    fn is_over(&self) -> bool;
    /// ...
    fn is_fulfilled(&self) -> bool;
}

/// ...
impl<T: crate::Config> PollProvider<T> for Poll<T>
{
    /// ...
    fn register_participant(
        mut self, 
        public_key: PublicKey,
        timestamp: u64
    ) -> (u32, Self)
    {
        let arity: usize = self.config.tree_arity.into();
        let data = sponge::hash(&vec![
            BlsScalar(public_key.x),
            BlsScalar(public_key.y),
            BlsScalar::from(timestamp)
        ]);
        self.state.registrations = self.state.registrations.insert(arity, data);
        (self.state.registrations.count, self)
    }

    /// ...
    fn consume_interaction(
        mut self, 
        public_key: PublicKey,
        data: PollInteractionData
    ) -> (u32, Self)
    {
        let arity: usize = self.config.tree_arity.into();
        let mut data = data.map(|x| BlsScalar(x)).to_vec();
        data.push(BlsScalar(public_key.x));
        data.push(BlsScalar(public_key.y));
        
        self.state.interactions = self.state.interactions.insert(arity, sponge::hash(&data));
        (self.state.interactions.count, self)
    }

    // /// ...
    // fn prove(
    //     mut self,
    //     verify_key: VerifyKey<T>,
    //     batches: ProofBatches
    // ) -> Self
    // {
    //     self
    // }

    // /// ...
    // fn verify(
    //     &self,
    //     outcome: Option<u32>
    // ) -> Result<u32, crate::Error<T>>
    // {
    //     Ok(1)
    // }

    // /// ...
    // fn is_verifiable(&self) -> bool
    // {
    //     false
    // }

    /// ...
    fn registration_limit_reached(&self) -> bool
    {
        self.state.registrations.count >= self.config.max_registrations
    }

    /// ...
    fn interaction_limit_reached(&self) -> bool
    {
        self.state.interactions.count >= T::MaxPollInteractions::get()
    }

    /// Returns true iff poll is not None and `now` preceeds the end time of the poll.
    fn is_voting_period(&self) -> bool
    {
        let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
        let voting_period_start = self.created_at + self.config.signup_period;
        let voting_period_end = voting_period_start + self.config.voting_period;
        now >= voting_period_start && now < voting_period_end
    }

    /// Returns true iff poll is currently within the registration period.
	fn is_registration_period(&self) -> bool
	{
		let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
		now >= self.created_at && now < self.created_at + self.config.signup_period
	}

    /// Returns true iff poll has ended.
    fn is_over(&self) -> bool
    {
		let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
		let voting_period_start = self.created_at + self.config.signup_period;
		let voting_period_end = voting_period_start + self.config.voting_period;
		now > voting_period_end
    }

    /// Returns true iff poll outcome has been committed to state.
    fn is_fulfilled(&self) -> bool
    {
        self.state.outcome.is_some()
    }

    fn is_merged(&self) -> bool
    {
        self.state.registrations.root.is_some() && self.state.interactions.root.is_some()
    }
}

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PollState<T: crate::Config>
{
    /// The merkle tree of registration data.
    pub registrations: PollStateTree<T>,

    /// The merkle tree of interaction data.
    pub interactions: PollStateTree<T>,

    /// The current proof commitment.
    pub commitment: Commitment,

    /// The final result of the poll.
    pub outcome: Option<Outcome>,
}

/// ...
impl<T: crate::Config> Default for PollState<T>
{
    fn default() -> PollState<T>
    {
        PollState {
            registrations: PollStateTree::default(),
            interactions: PollStateTree::default(),
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
    /// The number of blocks for which the registration period is active.
    pub signup_period: BlockNumber,

    /// The number of blocks for which the voting period is active.
    pub voting_period: BlockNumber,

    /// The maximum number of participants permitted.
    pub max_registrations: u32,

    /// The possible outcomes of the poll.
    pub vote_options: VoteOptions<T>,

    /// The arity of the state trees.
    pub tree_arity: u8
}

/// ...
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct PollStateTree<T>
{
    /// The depth of the subtree.
    pub depth: u32,

    /// The number of non-nil leaves.
    pub count: u32,

    /// The (depth, hash) pairs of the incrementally merged subtree.
    pub hashes: vec::Vec<(u32, HashBytes)>,

    /// The root of the tree of depth `T::MaxTreeDepth` which contains
    /// the leaves of `hashes` and zeros elsewhere.
    pub root: Option<HashBytes>,

    _marker: PhantomData<T>
}

/// ...
impl<T: crate::Config> Default for PollStateTree<T>
{
    fn default() -> PollStateTree<T>
    {
        PollStateTree {
            depth: 0,
            count: 0,
            hashes: vec::Vec::<(u32, HashBytes)>::new(),
            root: None,
            _marker: PhantomData
        }
    }
}

/// ...
pub trait PartialMerkleStack<T: crate::Config>
{
    /// Inserts a new leaf into the tree.
    fn insert(self, arity: usize, data: BlsScalar) -> Self;
}

/// ...
impl<T: crate::Config> PartialMerkleStack<T> for PollStateTree<T>
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
        arity: usize,
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