use frame_support::pallet_prelude::*;
use sp_runtime::traits::SaturatedConversion;
use crate::poll::{
    Poll, 
    PublicKey,
    AmortizedIncrementalMerkleTree, 
    MerkleTreeError
};

pub trait PollProvider<T: crate::Config>: Sized
{
    // fn register_participant(
    //     self, 
    //     public_key: PublicKey, 
    //     timestamp: u64
    // ) -> Result<(u32, Self), MerkleTreeError>;

    // fn consume_interaction(
    //     self,
    //     public_key: PublicKey,
    //     data: PollInteractionData
    // ) -> Result<(u32, Self), MerkleTreeError>;

    fn merge_registrations(self) -> Result<Self, MerkleTreeError>;

    fn merge_interactions(self) -> Result<Self, MerkleTreeError>;
    
    fn registration_limit_reached(&self) -> bool;

    fn interaction_limit_reached(&self) -> bool;

    fn is_voting_period(&self) -> bool;

    fn is_registration_period(&self) -> bool;

    fn is_over(&self) -> bool;

    fn is_fulfilled(&self) -> bool;

    fn is_merged(&self) -> bool;

    fn is_nullified(&self) -> bool;

    fn nullify(self) -> Self;
}

impl<T: crate::Config> PollProvider<T> for Poll<T>
{
    // fn register_participant(
    //     mut self, 
    //     public_key: PublicKey,
    //     timestamp: u64
    // ) -> Result<(u32, Self), MerkleTreeError>;
    // {
    //     // uint256[4] memory plaintext;
    //     // plaintext[0] = _stateLeaf.pubKey.x;
    //     // plaintext[1] = _stateLeaf.pubKey.y;
    //     // plaintext[2] = _stateLeaf.voiceCreditBalance;
    //     // plaintext[3] = _stateLeaf.timestamp;
    //     // ciphertext = hash4(plaintext);

    //     // let data = sponge::hash(&vec![
    //     //     BlsScalar::from_raw(public_key.x),
    //     //     BlsScalar::from_raw(public_key.y),
    //     //     BlsScalar::from(timestamp)
    //     // ]);
    //     self.state.registrations = self.state.registrations.insert(leaf);
    //     (self.state.registrations.count, self)
    // }

    // fn consume_interaction(
    //     mut self, 
    //     public_key: PublicKey,
    //     data: PollInteractionData
    // ) -> Result<(u32, Self), MerkleTreeError>;
    // {
    //     // let arity: usize = self.config.tree_arity.into();
    //     // let mut data = data.map(|x| BlsScalar::from_raw(x)).to_vec();
    //     // data.push(BlsScalar::from_raw(public_key.x));
    //     // data.push(BlsScalar::from_raw(public_key.y));
        
    //     // self.state.interactions = self.state.interactions.insert(arity, sponge::hash(&data));
    //     // (self.state.interactions.count, self)

    //     // uint256[5] memory n;
    //     // n[0] = _message.data[0];
    //     // n[1] = _message.data[1];
    //     // n[2] = _message.data[2];
    //     // n[3] = _message.data[3];
    //     // n[4] = _message.data[4];
    
    //     // uint256[5] memory m;
    //     // m[0] = _message.data[5];
    //     // m[1] = _message.data[6];
    //     // m[2] = _message.data[7];
    //     // m[3] = _message.data[8];
    //     // m[4] = _message.data[9];
    
    //     // msgHash = hash4([hash5(n), hash5(m), _encPubKey.x, _encPubKey.y]);
    // }

    fn merge_registrations(
        mut self
    ) -> Result<Self, MerkleTreeError>
    {
        self.state.registrations = self.state.registrations.merge()?;

        // uint256[3] memory sb;
        // sb[0] = _mergedStateRoot;
        // sb[1] = emptyBallotRoot;
        // sb[2] = uint256(0);
        // currentSbCommitment = hash3(sb);

        Ok(self)
    }

    fn merge_interactions(
        mut self
    ) -> Result<Self, MerkleTreeError>
    {
        self.state.interactions = self.state.interactions.merge()?;
        Ok(self)
    }

    fn registration_limit_reached(&self) -> bool
    {
        self.state.registrations.count >= self.config.max_registrations
    }

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

    /// Returns true iff poll outcome has been committed to state, or the poll is dead.
    fn is_fulfilled(&self) -> bool
    {
        self.state.outcome.is_some() || self.is_nullified()
    }

    fn is_merged(&self) -> bool
    {
        self.state.registrations.root.is_some() && self.state.interactions.root.is_some()
    }

    fn is_nullified(&self) -> bool
    {
        self.state.tombstone
    }

    fn nullify(mut self) -> Self
    {
        self.state.tombstone = true;
        self
    }
}
