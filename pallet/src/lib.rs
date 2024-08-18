#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec;
use sp_runtime::traits::SaturatedConversion;

use ark_bn254::{
    Bn254,
    Fr,
    G1Affine, 
    G2Affine
};
use ark_serialize::{CanonicalDeserialize};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{
    Groth16,
    data_structures::Proof,
    data_structures::VerifyingKey
};

pub mod hash;
pub mod poll;

pub use poll::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// pub mod benchmarking;

#[frame_support::pallet]
pub mod pallet 
{
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum number of polls a given coordinator may create.
		#[pallet::constant]
		type MaxCoordinatorPolls: Get<u32>;

		/// The maximum number of poll outcomes.
		#[pallet::constant]
		type MaxVoteOptions: Get<u32>;

		/// The maximum allowable number of registrations.
		#[pallet::constant]
		type MaxPollRegistrations: Get<u32>;

		/// The maximum allowable number of poll interactions.
		#[pallet::constant]
		type MaxPollInteractions: Get<u32>;

		/// The maximal allowable number of iterations in an extrinsic.
		#[pallet::constant]
		type MaxIterationDepth: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> 
	{
		/// A new coordinator was registered.
		CoordinatorRegistered {
			/// The coordinator.
			who: T::AccountId,
			/// The public key of the coordinator.
			public_key: PublicKey,
			/// The verify key of the coordinator.
			verify_key: VerifyKey
		},

		/// A coordinator rotated one of their keys.
		CoordinatorKeysChanged { 
			/// The coordinator.
			who: T::AccountId, 
			/// The new public key.
			public_key: PublicKey,
			/// The new verify key.
			verify_key: VerifyKey
		},

		/// A participant registered to vote in a poll.
		ParticipantRegistered { 
			/// The index of the poll registered in.
			poll_id: PollId,
			/// The current registration count.
			count: u32,
			/// The block number of the registration.
			block: BlockNumber,
			/// The registrations ephemeral public key.
			public_key: PublicKey
		},

		/// A new poll was created.
		PollCreated {
			/// The poll index.
			poll_id: PollId,
			/// The poll coordinator.
			coordinator: T::AccountId,
			/// The block number the poll signup period ends and voting commences.
			starts_at: BlockNumber,
			/// The block number the voting period commences.
			ends_at: BlockNumber
		},

		/// Poll was interacted with.
		PollInteraction {
			/// The index of the poll interacted with.
			poll_id: PollId,
			/// The current interaction count.
			count: u32,
			/// Ephemeral public key used to encrypt the message.
			public_key: PublicKey,
			/// Interaction data.
			data: PollInteractionData
		},

		/// Poll state was partially processed.
		PollCommitmentUpdated {
			/// The poll index.
			poll_id: PollId,
			/// The new commitment value.
			commitment: Commitment
		},

		/// Poll state tree root was computed. 
		PollStateMerged {
			/// The poll index.
			poll_id: PollId,
			/// The poll registrations tree root.
			registration_root: Option<HashBytes>,
			/// The poll interactions tree root.
			interaction_root: Option<HashBytes>,
		},

		/// Poll result was verified.
		PollOutcome {
			/// The poll index.
			poll_id: PollId,
			/// The outcome of the poll.
			outcome: u128
		},

		/// Empty and expired poll was nullified.
		PollNullified {
			/// The poll index.
			poll_id: PollId
		}
	}

	#[pallet::error]
	pub enum Error<T>
	{
		/// Coordinator is already registered.
		CoordinatorAlreadyRegistered,

		/// Coordinator role not found.
		CoordinatorNotRegistered,

		/// Coordinator poll limit reached.
		CoordinatorPollLimitReached,

		/// Maximum number of participants have registered.
		ParticipantRegistrationLimitReached,

		/// Maximum number of interactions has been reached.
		ParticipantInteractionLimitReached,

		/// Poll config is invalid.
		PollConfigInvalid,

		/// Poll registration period is in progress.
		PollRegistrationInProgress,

		/// Poll registration period has ended.
		PollRegistrationHasEnded,

		/// Poll voting period is in progress.
		PollVotingInProgress,

		/// A poll owned by the same coordinator has not yet ended or is missing a valid outcome.
		PollCurrentlyActive,

		/// Poll has ended and may no longer be interacted with by participants.
		PollVotingHasEnded,

		/// Poll does not exist.
		PollDoesNotExist,

		/// Poll data is empty.
		PollDataEmpty,

		/// Poll outcome was previously committed and verified.
		PollOutcomeAlreadyDetermined,

		/// Poll state trees have not yet been merged.
		PollStateNotMerged,

		/// Poll state tree merge operation failed.
		PollMergeFailed { reason: u8 },

		/// Poll registration failed.
		PollRegistrationFailed { reason: u8 },

		/// Poll interaction failed.
		PollInteractionFailed { reason: u8 },

		/// The key(s) provided are malformed.
		MalformedKeys,

		/// A proof was rejected.
		MalformedProof,
	}

	/// Map of ids to polls.
	#[pallet::storage]
	#[pallet::getter(fn polls)]
	pub type Polls<T: Config> = CountedStorageMap<
		_,
		Twox64Concat,
		PollId,
		Poll<T>
	>;

	/// Map of coordinators to their keys.
	#[pallet::storage]
	#[pallet::getter(fn coordinators)]
	pub type Coordinators<T: Config> = CountedStorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId,
		Coordinator
	>;

	/// Map of coordinators to the poll Ids they manage.
	#[pallet::storage]
	#[pallet::getter(fn poll_ids)]
	pub type CoordinatorPollIds<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		vec::Vec<PollId>,
		ValueQuery
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> 
	{
		/// Register the caller as a coordinator, granting the ability to create polls.
		///
		/// - `public_key`: The public key of the coordinator.
		/// - `verify_key`: The verification key of the coordinator.
		///
		/// Emits `CoordinatorRegistered`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn register_as_coordinator(
			origin: OriginFor<T>,
			public_key: PublicKey,
			verify_key: VerifyKey
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Ensure the verification key can be serialized as affine points.
			ensure!(serialize_vkey(verify_key.clone()).is_some(), Error::<T>::MalformedKeys);

			// A coordinator may only be registered once.
			ensure!(
				!Coordinators::<T>::contains_key(&sender), 
				Error::<T>::CoordinatorAlreadyRegistered
			);

			// Store the coordinator keys.
			Coordinators::<T>::insert(&sender, Coordinator {
				last_poll: None,
				public_key,
				verify_key: verify_key.clone()
			});

			// Emit a registration event.
			Self::deposit_event(Event::CoordinatorRegistered {
				who: sender,
				public_key,
				verify_key
			});

			Ok(())
		}

		/// Permits a coordinator to rotate their public and verification keys.
		/// Rejected if an extant poll is ongoing or awaiting processing.
		///
		/// - `public_key`: The new public key for the coordinator.
		/// - `verify_key`: The new verification key for the coordinator.
		///
		/// Emits `CoordinatorKeyChanged`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		pub fn rotate_keys(
			origin: OriginFor<T>,
			public_key: PublicKey,
			verify_key: VerifyKey
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Ensure the verification key can be serialized as affine points.
			ensure!(serialize_vkey(verify_key.clone()).is_some(), Error::<T>::MalformedKeys);

			// Check if origin is registered as a coordinator.
			let Some(mut coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };

			// Ensure that the most recent poll is not currently in progress and is not missing an outcome, if it exists.
			if let Some(index) = coordinator.last_poll
			{
				if let Some(poll) = Polls::<T>::get(index)
				{
					ensure!(
						poll.is_over() && poll.is_fulfilled(),
						Error::<T>::PollCurrentlyActive
					);
				}
			}

			coordinator.public_key = public_key.clone();
			coordinator.verify_key = verify_key.clone();

			// Update and store the coordinators updated key(s).
			Coordinators::<T>::insert(&sender, coordinator);
	
			// Emit the key rotation event.
			Self::deposit_event(Event::CoordinatorKeysChanged {
				who: sender,
				public_key,
				verify_key
			});

			Ok(())
		}

		/// Create a new poll object where the caller is the designated coordinator.
		///
		/// - `signup_period`: The number of blocks for which the registration period is active.
		/// - `voting_period`: The number of blocks for which the voting period is active.
		/// - `max_registrations`: The maximum number of participants permitted.
		/// - `vote_options`: The possible outcomes of the poll.
		///
		/// Emits `PollCreated`.
		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 3))]
		pub fn create_poll(
			origin: OriginFor<T>,
			signup_period: BlockNumber,
			voting_period: BlockNumber,
			max_registrations: u32,
			process_subtree_depth: u32,
			vote_options: vec::Vec<u128>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Validate config parameters.
			let created_at = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
			ensure!(
				max_registrations <= T::MaxPollRegistrations::get(),
				Error::<T>::PollConfigInvalid
			);

			ensure!(vote_options.len() > 1, Error::<T>::PollConfigInvalid);
			let vote_options: VoteOptions<T> = vote_options
				.try_into()
				.map_err(|_| Error::<T>::PollConfigInvalid)?;

			// Check that sender is registered as a coordinator.
			let Some(mut coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };

			let coord_poll_ids = Self::poll_ids(&sender);

			// A coordinator may have at most `MaxCoordinatorPolls` polls, skipped if zero.
			let max_polls = T::MaxCoordinatorPolls::get() as usize;
			ensure!(
				coord_poll_ids.len() < max_polls,
				Error::<T>::CoordinatorPollLimitReached
			);

			// A coordinator may only have a single active poll at a given time.
			if let Some(index) = coord_poll_ids.last()
			{
				if let Some(poll) = Polls::<T>::get(index)
				{
					// Reject if last created poll is on-going, or has yet to be processed.
					ensure!(
						poll.is_over() && poll.is_fulfilled(),
						Error::<T>::PollCurrentlyActive 
					);
				}
			}

			// Insert the poll into storage.
			let index = Polls::<T>::count();
			Polls::<T>::insert(&index, Poll {
				index,
				created_at,
				coordinator: sender.clone(),
				state: PollState::default(),
				config: PollConfiguration {
					signup_period,
					voting_period,
					max_registrations,
					process_subtree_depth,
					vote_options
				}
			});

			coordinator.last_poll = Some(index);
			Coordinators::<T>::insert(&sender, coordinator);
			CoordinatorPollIds::<T>::append(&sender, index);

			// Emit the creation event.
			let starts_at = created_at + signup_period;
			let ends_at = starts_at + voting_period;
			Self::deposit_event(Event::PollCreated { 
				coordinator: sender,
				poll_id: index,
				starts_at,
				ends_at
			});

			Ok(())
		}

		/// Compute the roots of the current poll state trees. This operation must be performed prior to commiting the poll outcome. 
		/// Registration tree may be merged as long as the registration period has elapsed, and the interaction tree may be merged 
		/// as long as the voting period has elapsed. NB Coordinator's are required to call this extrinsic twice: once to merge the 
		/// registration state tree, and once to merge the interaction state tree.
		///
		/// Emits `PollStateMerged`.
		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))] 
		pub fn merge_poll_state(
			origin: OriginFor<T>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;
			
			// Get the coordinators most recent poll.
			let Some(coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };
			let Some(poll_id) = coordinator.last_poll else { Err(<Error::<T>>::PollDoesNotExist)? };
			let Some(poll) = Polls::<T>::get(poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// Check that the poll is not currently in the registration period.
			ensure!(
				!poll.is_registration_period(),
				Error::<T>::PollRegistrationInProgress
			);

			if poll.state.registrations.root.is_none()
			{
				// Ensure that there was at least one registration.
				ensure!(
					poll.state.registrations.hashes.len() > 0,
					Error::<T>::PollDataEmpty
				);

				// Compute the root of the registration tree and save it.
				let poll = poll
					.merge_registrations()
					.map_err(|error| Error::<T>::PollMergeFailed { reason: error.into() })?;

				Polls::<T>::insert(&poll_id, poll.clone());

				// Emit the hash event.
				Self::deposit_event(Event::PollStateMerged {
					poll_id,
					registration_root: poll.state.registrations.root,
					interaction_root: None
				});
			}

			else if poll.state.interactions.root.is_none()
			{
				// Check that the poll is not currenltly in the voting period.
				ensure!(
					poll.is_over(),
					Error::<T>::PollVotingInProgress
				);

				// Ensure that there was at least one interaction.
				ensure!(
					poll.state.interactions.hashes.len() > 0,
					Error::<T>::PollDataEmpty
				);

				// Compute the root of the interaction tree and save it.
				let poll = poll
					.merge_interactions()
					.map_err(|error| Error::<T>::PollMergeFailed { reason: error.into() })?;

				Polls::<T>::insert(&poll_id, poll.clone());

				// Emit the hash event.
				Self::deposit_event(Event::PollStateMerged {
					poll_id,
					registration_root: None,
					interaction_root: poll.state.interactions.root
				});
			}

			// Poll data has already been merged.
			else { Err(<Error::<T>>::PollDataEmpty)? }

			Ok(())
		}

		/// Permits the coordinator to commit, in batches, proofs that all of the valid participant registrations and poll interactions 
		/// were included in the computation which decided the winning vote option. Each individual proof carries a commitment value 
		/// which is utilized to chain all of the proofs together, and in effect, to validate the final result.
		///
		/// Calls to this extrinsic are rejected if the poll has not ended, or if the root of the state trees have not yet been computed.
		///
		/// - `batches`: The ordered proofs alongside 
		/// - `outcome`: The index of the option voted for (from the `VoteOptions` vec in the poll configuration). This parameter
		///				 should only be included only with the last batch, or in a separate call after the final batch has been verified.
		/// 
		/// Emits `PollOutcome` once the outcome been verified, and `PollCommitmentUpdated` to reflect the updated commitment.
		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		pub fn commit_outcome(
			origin: OriginFor<T>,
			batches: ProofBatches,
			outcome: Option<OutcomeIndex>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Get the coordinators most recent poll.
			let Some(coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };
			let Some(poll_id) = coordinator.last_poll else { Err(<Error::<T>>::PollDoesNotExist)? };
			let Some(mut poll) = Polls::<T>::get(poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// Check that the state trees have been merged 
			ensure!(poll.is_merged(), Error::<T>::PollStateNotMerged);

			//Check that the outcome has not already been committed.
			ensure!(!poll.is_fulfilled(), Error::<T>::PollOutcomeAlreadyDetermined);

			let (mut index, mut cur_commitment) = poll.state.commitment;

			// Verify each batch of proofs, in order.
			for (proof, new_commitment) in batches.iter()
			{
				ensure!(
					verify_proof(
						coordinator.verify_key.clone(),
						poll.clone().get_proof_public_inputs(
							index,
							coordinator.public_key.clone(),
							cur_commitment,
							*new_commitment
						),
						proof.clone()
					),
					Error::<T>::MalformedProof
				);

				index += 1;
				cur_commitment = *new_commitment;
				poll.state.commitment = (index, cur_commitment);
			}

			// Once the final batch is verified, check that the outcome matches the final commitment.
			if let Some(outcome) = verify_outcome(poll.clone(), outcome)
			{
				poll.state.outcome = Some(outcome);

				Self::deposit_event(Event::PollOutcome { 
					poll_id,
					outcome
				});
			}
			else if batches.len() > 0
			{
				Self::deposit_event(Event::PollCommitmentUpdated {
					poll_id,
					commitment: (index, cur_commitment)
				})
			}
			else { Err(<Error::<T>>::MalformedProof)? }

			// Update the poll state.
			Polls::<T>::insert(poll_id, poll);

			Ok(())
		}

		/// Permits the coordinator to nullify a poll which expired without recording a single interaction.
		///
		/// Calls to this extrinsic are rejected if the poll has not ended, or there was at least one interaction.
		/// 
		/// Emits `PollNullified`.
		#[pallet::call_index(5)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		pub fn nullify_poll(
			origin: OriginFor<T>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Get the coordinators most recent poll.
			let Some(coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };
			let Some(poll_id) = coordinator.last_poll else { Err(<Error::<T>>::PollDoesNotExist)? };
			let Some(poll) = Polls::<T>::get(poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			ensure!(
				(!poll.is_registration_period() && poll.state.registrations.count == 0) || 
				(poll.is_over() && poll.state.interactions.count == 0),
				Error::<T>::PollCurrentlyActive
			);

			// Mark the poll as dead.
			Polls::<T>::insert(poll_id, poll.nullify());

			Ok(())
		}

		/// Permits a signer to participate in an upcoming poll. Rejected if signup period has elapsed.
		///
		///	- `poll_id`: The id of the poll.
		/// - `public_key`: The ephemeral public key of the registrant.
		///
		/// Emits `ParticipantRegistered`.
		#[pallet::call_index(6)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn register_as_participant(
			origin: OriginFor<T>,
			poll_id: PollId,
			public_key: PublicKey
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			ensure_signed(origin)?;

			// Ensure that the poll exists and get it.
			let Some(poll) = Polls::<T>::get(&poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// Check that the poll is still in the signup period.
			ensure!(
				poll.is_registration_period(),
				Error::<T>::PollRegistrationHasEnded
			);

			// Check that the maximum number of sign-ups has not been reached.
			ensure!(
				!poll.registration_limit_reached(),
				Error::<T>::ParticipantRegistrationLimitReached
			);

			// Record the hash of the registration data.
			let block = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
			
			// Insert the registration data into the poll state.
			let (count, poll) = poll
				.register_participant(public_key, block)
				.map_err(|error| Error::<T>::PollRegistrationFailed { reason: error.into() })?;

			Polls::<T>::insert(
				&poll_id, 
				poll
			);

			// Emit the registration data for future processing by the coordinator.
			Self::deposit_event(Event::ParticipantRegistered { 
				poll_id,
				count,
				public_key,
				block
			});

			Ok(())
		}

		/// Permits a signer to interact with an ongoing poll. Rejects if not within the voting period. 
		/// Valid messages include: a vote, and a key rotation. Participants may secretly call this 
		/// method (read: using a different signer) in order to override their previous vote. 
		///
		/// - `poll_id`: The index of the poll in storage.
		/// - `public_key`: The current ephemeral public key of the registrant. May be different than 
		///					the one used for registration.
		/// - `data`: The encrypted interaction data.
		///
		/// Emits `PollInteraction`.
		#[pallet::call_index(7)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn interact_with_poll(
			origin: OriginFor<T>,
			poll_id: PollId,
			public_key: PublicKey,
			data: PollInteractionData
		) -> DispatchResult
		{
			// Ensure that the extrinsic was signed.
			ensure_signed(origin)?;

			// Ensure that the poll exists and get it.
			let Some(poll) = Polls::<T>::get(&poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// Confirm that the poll is currently within it's voting period.
			ensure!(!poll.is_registration_period(), Error::<T>::PollRegistrationInProgress);
			ensure!(!poll.is_over(), Error::<T>::PollVotingHasEnded);

			// Check that we've not reached the maximum number of interactions.
			ensure!(
				!poll.interaction_limit_reached(),
				Error::<T>::ParticipantInteractionLimitReached
			);

			// Insert the interaction data into the poll state.
			let (count, poll) = poll
				.consume_interaction(public_key, data)
				.map_err(|error| Error::<T>::PollInteractionFailed { reason: error.into() })?;

			Polls::<T>::insert(
				&poll_id, 
				poll
			);

			// Emit the interaction data for future processing by the coordinator.
			Self::deposit_event(Event::PollInteraction {
				poll_id,
				count,
				public_key,
				data
			});

			Ok(())
		}
	}

	fn serialize_vkey(
		vkey: VerifyKey
	) -> Option<VerifyingKey::<Bn254>>
	{
		let Some(alpha_g1) = G1Affine::deserialize_uncompressed(&*vkey.alpha_g1).ok() else { return None; };
		let Some(beta_g2) = G2Affine::deserialize_uncompressed(&*vkey.beta_g2).ok() else { return None; };
		let Some(gamma_g2) = G2Affine::deserialize_uncompressed(&*vkey.gamma_g2).ok() else { return None; };
		let Some(delta_g2) = G2Affine::deserialize_uncompressed(&*vkey.delta_g2).ok() else { return None; };
		let gamma_abc_g1 = match vkey.gamma_abc_g1
			.iter()
			.map(|g| G1Affine::deserialize_uncompressed(g.as_slice()))
			.collect::<Result<vec::Vec<G1Affine>, _>>()
		{
			Ok(value) => value,
			Err(_) => return None
		};

		Some(VerifyingKey::<Bn254> { alpha_g1, beta_g2, gamma_g2, delta_g2, gamma_abc_g1 })
	}

	fn serialize_proof(
		proof_data: ProofData
	) -> Option<Proof::<Bn254>>
	{
	    let Some(a) = G1Affine::deserialize_uncompressed(&*proof_data.pi_a).ok() else { return None; };
	    let Some(b) = G2Affine::deserialize_uncompressed(&*proof_data.pi_b).ok() else { return None; };
	    let Some(c) = G1Affine::deserialize_uncompressed(&*proof_data.pi_c).ok() else { return None; };

		Some(Proof::<Bn254> { a, b, c })
	}

	fn verify_proof(
		verify_key: VerifyKey,
		public_inputs: vec::Vec<Fr>,
		proof_data: ProofData
	) -> bool
	{
		let Some(vk) = serialize_vkey(verify_key) else { return false; };
		let Some(pvk) = Groth16::<Bn254>::process_vk(&vk).ok() else { return false; };
		let Some(proof) = serialize_proof(proof_data) else { return false; };
		let Some(result) = Groth16::<Bn254>::verify_with_processed_vk(&pvk, &public_inputs, &proof).ok() else { return false; };

		result
	}

	fn verify_outcome<T: Config>(
		poll_data: Poll<T>,
		index: Option<OutcomeIndex>
	) -> Option<Outcome>
	{
		let Some(index) = index else { return None };
		if (index as usize) < poll_data.config.vote_options.len()
		{
			return Some(poll_data.config.vote_options[index as usize]);
		}

		None
	}
}
