use frame_support::pallet_prelude::*;
use sp_runtime::traits::SaturatedConversion;
use ark_bn254::{Fr};
use ark_ff::{PrimeField, BigInteger};
use crate::hash::{Poseidon, PoseidonHasher};
use crate::poll::{
    Poll, 
    PublicKey,
    PollInteractionData,
    AmortizedIncrementalMerkleTree, 
    MerkleTreeError,
    zeroes::EMPTY_BALLOT_ROOT
};

pub trait PollProvider<T: crate::Config>: Sized
{
    fn register_participant(
        self, 
        public_key: PublicKey, 
        timestamp: u64
    ) -> Result<(u32, Self), MerkleTreeError>;

    fn consume_interaction(
        self,
        public_key: PublicKey,
        data: PollInteractionData
    ) -> Result<(u32, Self), MerkleTreeError>;

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
    fn register_participant(
        mut self, 
        public_key: PublicKey,
        timestamp: u64
    ) -> Result<(u32, Self), MerkleTreeError>
    {
        let Some(mut hasher) = Poseidon::<Fr>::new_circom(4).ok() else { Err(MerkleTreeError::HashFailed)? };

        let mut timestamp_bytes = [0u8; 32];
        timestamp_bytes[24..].copy_from_slice(&timestamp.to_be_bytes());

        let mut credit_bytes = [0u8; 32];
        credit_bytes[31] = 1;

        let inputs: Vec<Fr> = Vec::from([ public_key.x, public_key.y, credit_bytes, timestamp_bytes ])
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();

        let Some(result) = hasher.hash(&inputs).ok() else { Err(MerkleTreeError::HashFailed)? };
        let bytes = result.into_bigint().to_bytes_be();
        let mut leaf = [0u8; 32];
        leaf[..bytes.len()].copy_from_slice(&bytes);

        self.state.registrations = self.state.registrations.insert(leaf)?;

        Ok((self.state.registrations.count, self))
    }

    fn consume_interaction(
        mut self, 
        public_key: PublicKey,
        data: PollInteractionData
    ) -> Result<(u32, Self), MerkleTreeError>
    {
        let Some(mut hash4) = Poseidon::<Fr>::new_circom(4).ok() else { Err(MerkleTreeError::HashFailed)? };
        let Some(mut hash5) = Poseidon::<Fr>::new_circom(5).ok() else { Err(MerkleTreeError::HashFailed)? };

        let left_inputs: Vec<Fr> = Vec::from([ data[0], data[1], data[2], data[3], data[4] ])
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();

        let right_inputs: Vec<Fr> = Vec::from([ data[5], data[6], data[7], data[8], data[9] ])
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();

        let Some(left) = hash5.hash(&left_inputs).ok() else { Err(MerkleTreeError::HashFailed)? };
        let Some(right) = hash5.hash(&right_inputs).ok() else { Err(MerkleTreeError::HashFailed)? };

        let left_bytes = left.into_bigint().to_bytes_be();
        let right_bytes = right.into_bigint().to_bytes_be();

        let inputs: Vec<Fr> = Vec::from([
            left_bytes,
            right_bytes,
            Vec::from(public_key.x),
            Vec::from(public_key.y)
        ])
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();

        let Some(result) = hash4.hash(&inputs).ok() else { Err(MerkleTreeError::HashFailed)? };
        let bytes = result.into_bigint().to_bytes_be();
        let mut leaf = [0u8; 32];
        leaf[..bytes.len()].copy_from_slice(&bytes);

        self.state.interactions = self.state.interactions.insert(leaf)?;

        Ok((self.state.interactions.count, self))
    }

    fn merge_registrations(
        mut self
    ) -> Result<Self, MerkleTreeError>
    {
        self.state.registrations = self.state.registrations.merge()?;

        let Some(root) = self.state.registrations.root else { Err(MerkleTreeError::MergeFailed)? };
        let Some(mut hasher) = Poseidon::<Fr>::new_circom(3).ok() else { Err(MerkleTreeError::HashFailed)? };

        let inputs: Vec<Fr> = Vec::from([ root, EMPTY_BALLOT_ROOT, [0u8;32] ])
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();

        let Some(result) = hasher.hash(&inputs).ok() else { Err(MerkleTreeError::HashFailed)? };
        let bytes = result.into_bigint().to_bytes_be();
        let mut commitment = [0u8; 32];
        commitment[..bytes.len()].copy_from_slice(&bytes);

        self.state.commitment = (0, commitment);

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
