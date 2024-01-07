#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec;

use frame_support::traits::UnixTime;
use sp_runtime::traits::SaturatedConversion;

pub mod types;
pub use types::*;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

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

		/// The maximum length of a coordinators verification key.
		#[pallet::constant]
		type MaxVerifyKeyLength: Get<u32>;

		/// The maximum arity for the state trees.
		#[pallet::constant]
		type MaxTreeArity: Get<u8>;

		/// The minimum arity for the state trees.
		#[pallet::constant]
		type MinTreeArity: Get<u8>;

		// /// The maximum state tree depth.
		#[pallet::constant]
		type MaxTreeDepth: Get<u8>;

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
			public_key: PublicKey
		},

		/// A coordinator rotated one of their keys.
		CoordinatorKeyChanged { 
			/// The coordinator.
			who: T::AccountId, 
			/// The new public key, if it was rotated.
			public_key: Option<PublicKey>,
			/// The new verify key, if it was rotated.
			verify_key: Option<VerifyKey<T>>
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
			index: PollId,
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

		/// Poll result was verified.
		PollOutcome {
			/// The poll index.
			index: PollId,
			/// The outcome of the poll.
			outcome: u128
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

		/// Maximum number of interactions have committed.
		ParticipantInteractionLimitReached,

		/// Poll config is invalid.
		PollConfigInvalid,

		/// Poll registration period is in progress.
		PollRegistrationInProgress,

		/// Poll registration period has ended.
		PollRegistrationHasEnded,

		/// Poll voting period is in progress.
		PollVotingInProgress,

		/// Poll has not ended.
		PollHasNotEnded,

		/// Poll has ended and may no longer be interacted with by participants.
		PollVotingHasEnded,

		/// Poll does not exist.
		PollDoesNotExist,

		/// Poll data is empty.
		PollDataEmpty,

		/// Poll must be processed before a new one is created.
		PollMissingOutcome,

		/// Poll trees must be merged before an outcome may be committed.
		PollTreesNotMerged,

		/// Poll outcome was previously committed and verified.
		PollOutcomeAlreadyCommitted,

		/// Poll data is malformed.
		MalformedPollData,

		/// The public key is malformed.
		MalformedPublicKey,

		/// The verify key is malformed.
		MalformedVerifyKey,

		/// A proof was rejected.
		MalformedProof
	}

	/// Map of ids to polls.
	#[pallet::storage]
	pub type Polls<T: Config> = CountedStorageMap<
		_,
		Twox64Concat,
		PollId,
		Poll<T>
	>;

	/// Map of coordinators to their keys.
	#[pallet::storage]
	pub type Coordinators<T: Config> = CountedStorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId,
		Coordinator<T>
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
			verify_key: vec::Vec<u8>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			let verify_key: VerifyKey<T> = verify_key
				.try_into()
				.map_err(|_| Error::<T>::MalformedVerifyKey)?;

			// A coordinator may only be registered once.
			ensure!(
				!Coordinators::<T>::contains_key(&sender), 
				Error::<T>::CoordinatorAlreadyRegistered
			);

			// Store the coordinator keys.
			Coordinators::<T>::insert(&sender, Coordinator {
				last_poll: None,
				public_key: public_key.clone(),
				verify_key
			});

			// Emit a registration event
			Self::deposit_event(Event::CoordinatorRegistered {
				who: sender,
				public_key
			});
			
			// Coordinator was successfully registered.
			Ok(())
		}

		/// Permits a coordinator to rotate their public key.
		/// Rejected if called during the voting period of the coordinators poll.
		///
		/// - `public_key`: The new public key for the coordinator.
		///
		/// Emits `CoordinatorKeyChanged`.
		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		pub fn rotate_public_key(
			origin: OriginFor<T>,
			public_key: PublicKey
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Check if origin is registered as a coordinator.
			let Some(mut coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };

			// Check that the coordinator's most recent poll is not currently in progress, if it exists.
			if let Some(index) = coordinator.last_poll
			{
				ensure!(
					!poll_in_voting_period(Polls::<T>::get(index)),
					Error::<T>::PollVotingInProgress
				);
			}

			// Update and store the coordinators updated public key.
			coordinator.public_key = public_key.clone();
			Coordinators::<T>::insert(&sender, coordinator);
	
			// Emit the key rotation event.
			Self::deposit_event(Event::CoordinatorKeyChanged {
				who: sender,
				public_key: Some(public_key),
				verify_key: None
			});

			Ok(())
		}

		/// Permits a coordinator to rotate their verification key.
		/// Rejected if called during the voting period of the coordinators poll.
		///
		/// - `verify_key`: The new verification key for the coordinator.
		///
		/// Emits `CoordinatorKeyChanged`.
		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		pub fn rotate_verify_key(
			origin: OriginFor<T>,
			verify_key: vec::Vec<u8>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			let verify_key: VerifyKey<T> = verify_key
				.try_into()
				.map_err(|_| Error::<T>::MalformedVerifyKey)?;

			// Check if origin is registered as a coordinator.
			let Some(mut coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };

			// Check that the coordinator's most recent poll is not currently in progress, if it exists.
			if let Some(index) = coordinator.last_poll
			{
				ensure!(
					!poll_in_voting_period(Polls::<T>::get(index)),
					Error::<T>::PollVotingInProgress
				);
			}

			// Update and store the coordinators updated verification key.
			coordinator.verify_key = verify_key.clone();
			Coordinators::<T>::insert(&sender, coordinator);

			// Emit the key rotation event.
			Self::deposit_event(Event::CoordinatorKeyChanged {
				who: sender,
				public_key: None,
				verify_key: Some(verify_key)
			});

			Ok(())
		}

		/// Create a new poll object where the caller is the designated coordinator.
		///
		/// - `signup_period`: The number of blocks for which the registration period is active.
		/// - `voting_period`: The number of blocks for which the voting period is active.
		/// - `max_registrations`: The maximum number of participants permitted.
		/// - `vote_options`: The possible outcomes of the poll.
		/// - `tree_arity`: The arity of the state trees.
		/// - `batch_size`: 
		/// - `metadata`: Optional metadata associated to the poll.
		///
		/// Emits `PollCreated`.
		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 3))]
		pub fn create_poll(
			origin: OriginFor<T>,
			signup_period: BlockNumber,
			voting_period: BlockNumber,
			max_registrations: u32,
			vote_options: vec::Vec<u128>,
			tree_arity: u8
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Validate config parameters.
			ensure!(
				(
					max_registrations <= T::MaxPollRegistrations::get() &&
					tree_arity <= T::MaxTreeArity::get() &&
					tree_arity >= T::MinTreeArity::get()
				),
				Error::<T>::PollConfigInvalid
			);

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
					// Reject if last created poll is on-going.
					ensure!(
						poll_is_over(poll.clone()),
						Error::<T>::PollHasNotEnded 
					);
	
					// Reject if last created poll has not been processed.
					ensure!(
						poll.state.outcome.is_some(),
						Error::<T>::PollMissingOutcome
					);
				}
			}

			// Insert the poll into storage.
			let index = Polls::<T>::count();
			let created_at = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
			Polls::<T>::insert(&index, Poll {
				index,
				created_at,
				coordinator: sender.clone(),
				state: PollState::default(),
				config: PollConfiguration {
					signup_period,
					voting_period,
					max_registrations,
					vote_options,
					tree_arity
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
				index,
				starts_at,
				ends_at
			});

			Ok(())
		}

		// /// Compute the merkle roots of the current poll state tree. This operation must be
		// /// performed prior to commiting the poll outcome. Registration tree may be merged as
		// /// long as the registration period has elapsed, and the interaction tree may be 
		// /// merged as long as the voting period has elapsed.
		// ///
		// /// Emits `PollStateMerged`.
		// #[pallet::call_index(4)]
		// #[pallet::weight(T::DbWeight::get().reads_writes(3, 2))] 
		// pub fn merge_poll_state(
		// 	_origin: OriginFor<T>
		// ) -> DispatchResult
		// {
			// // Check that the extrinsic was signed and get the signer.
			// let sender = ensure_signed(origin)?;
			
			// // Get the coordinators most recent poll.
			// let Some(coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };
			// let Some(index) = coordinator.last_poll else { Err(<Error::<T>>::PollDoesNotExist)? };
			// let Some(mut poll) = Polls::<T>::get(index) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// // Check that the poll is not currently in the registration period.
			// ensure!(
			// 	!poll_in_signup_period(poll.clone()),
			// 	Error::<T>::PollRegistrationInProgress
			// );

			// // We call this extrinsic twice: to merge the registration and interaction data respectively.
			// if poll.state.registration_tree.subtree_root.is_none()
			// {
			// 	// Ensure that there was at least one registration.
			// 	ensure!(
			// 		poll.state.registration_tree.subtree_hashes.len() > 0,
			// 		Error::<T>::PollDataEmpty
			// 	);

			// 	// Compute the root of the tree whose leaves are the hashes of the registration data.
			// 	let (root, subtree_root, subtree_depth) = compute_state_tree::<T>(
			// 		poll.state.registration_tree.subtree_hashes,
			// 		poll.config.tree_arity.into()
			// 	);
			// 	// Update the poll state tree.
			// 	let state_tree = PollStateTree {
			// 		subtree_hashes: vec![],
			// 		subtree_depth: subtree_depth,
			// 		subtree_root: subtree_root,
			// 		root: root
			// 	};
			// 	poll.state.registration_tree = state_tree.clone();
			// 	Polls::<T>::insert(&index, poll);

			// 	// Emit the hash event.
			// 	Self::deposit_event(Event::PollStateMerged {
			// 		index,
			// 		registration_tree: Some(state_tree),
			// 		interaction_tree: None
			// 	});
			// }
			// else if poll.state.interaction_tree.subtree_root.is_none()
			// {
			// 	// Check that the poll is not currenltly in the voting period.
			// 	ensure!(
			// 		!poll_in_voting_period(Some(poll.clone())),
			// 		Error::<T>::PollVotingInProgress
			// 	);

			// 	// Ensure that there was at least one interaction.
			// 	ensure!(
			// 		poll.state.interaction_tree.subtree_hashes.len() > 0,
			// 		Error::<T>::PollDataEmpty
			// 	);

			// 	// Compute the root of the tree whose leaves are the hashes of the interaction data.
			// 	let (root, subtree_root, subtree_depth) = compute_state_tree::<T>(
			// 		poll.state.interaction_tree.subtree_hashes,
			// 		poll.config.tree_arity.into()
			// 	);

			// 	// Update the poll state tree.
			// 	let state_tree = PollStateTree {
			// 		subtree_hashes: vec![],
			// 		subtree_depth: subtree_depth,
			// 		subtree_root: subtree_root,
			// 		root: root
			// 	};
			// 	poll.state.interaction_tree = state_tree.clone();
			// 	Polls::<T>::insert(&index, poll);

			// 	// Emit the hash event.
			// 	Self::deposit_event(Event::PollStateMerged {
			// 		index,
			// 		registration_tree: None,
			// 		interaction_tree: Some(state_tree)
			// 	});
			// }

			// // Poll data has already been merged.
			// else { Err(<Error::<T>>::PollDataEmpty)? }

		// 	Ok(())
		// }

		// /// Verifies the proof that the current batch of messages have been correctly processed and, if successful, updates
		// /// the current verification state. Rejected if called prior to the merge of poll state.
		// /// Verifies the proof that the current batch of votes has been correctly tallied and, if successful, updates the 
		// /// current verification state. Rejected if messages have not yet been processed. On verification of the final
		// /// batch the poll result is recorded in storage and an event is emitted containing the result. Rejected if called
		// /// before poll end.

		/// TODO (M1) write header
		///
		/// - `batches`: ...
		/// - `outcome`: The index of the option in VoteOptions. Include only with the last batch, or after the last batch has been verified.
		/// 
		/// Emits `PollOutcome` once the final batch has been verified.
		// #[pallet::call_index(5)]
		// #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
		// pub fn commit_outcome(
		// 	_origin: OriginFor<T>,
		// 	_batches: vec::Vec<(ProofData, CommitmentData)>,
		// 	_outcome: Option<u32>
		// ) -> DispatchResult
		// {
		// 	// // Check that the extrinsic was signed and get the signer.
		// 	// let sender = ensure_signed(origin)?;

		// 	// // Get the coordinators most recent poll.
		// 	// let Some(coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };
		// 	// let Some(index) = coordinator.last_poll else { Err(<Error::<T>>::PollDoesNotExist)? };
		// 	// let Some(mut poll) = Polls::<T>::get(index) else { Err(<Error::<T>>::PollDoesNotExist)? };

		// 	// ensure!(
		// 	// 	poll.state.outcome.is_none(),
		// 	// 	Error::<T>::PollOutcomeAlreadyCommitted
		// 	// );

		// 	// // Verify each batch of proofs, in order.
		// 	// for (proof, commitment) in batches.iter()
		// 	// {
		// 	// 	ensure!(
		// 	// 		verify_proof(
		// 	// 			poll.clone(),
		// 	// 			coordinator.verify_key.clone(),
		// 	// 			*proof,
		// 	// 			*commitment
		// 	// 		),
		// 	// 		Error::<T>::MalformedProof
		// 	// 	);
		// 	// 	// TODO (M1)
		// 	// 	// poll.state.num_witnessed += 1;
		// 	// 	// poll.state.commitment = *commitment;
		// 	// }

		// 	// // Once the final batch is verified, we verify that the outcome matches the final commitment
		// 	// if let Some(outcome) = verify_outcome(poll.clone(), outcome)
		// 	// {
		// 	// 	poll.state.outcome = Some(outcome);

		// 	// 	Self::deposit_event(Event::PollOutcome { 
		// 	// 		index,
		// 	// 		outcome
		// 	// 	});
		// 	// }

		// 	// // Update the poll state.
		// 	// Polls::<T>::insert(index, poll);

		// 	Ok(())
		// }

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
			let sender = ensure_signed(origin)?;

			// Ensure that the poll exists and get it.
			let Some(mut poll) = Polls::<T>::get(&poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// Check that the poll is still in the signup period.
			ensure!(
				poll_in_signup_period(poll.clone()),
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
			let (count, poll) = poll.register_participant(public_key, block);
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
			let Some(mut poll) = Polls::<T>::get(&poll_id) else { Err(<Error::<T>>::PollDoesNotExist)? };

			// Confirm that the poll is currently within it's voting period.
			ensure!(
				!poll_is_over(poll.clone()),
				Error::<T>::PollVotingHasEnded
			);

			// Check that we've not reached the maximum number of interactions.
			ensure!(
				!poll.interaction_limit_reached(),
				Error::<T>::ParticipantInteractionLimitReached
			);

			// Insert the interaction data into the poll state.
			let (count, poll) = poll.consume_interaction(public_key, data);
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

	/// Returns true iff poll is not None and `now` preceeds the end time of the poll.
	fn poll_in_voting_period<T: Config>(
		poll: Option<Poll<T>>
	) -> bool
	{
		if let Some(p) = poll
		{
			let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
			let voting_period_start = p.created_at + p.config.signup_period;
			let voting_period_end = voting_period_start + p.config.voting_period;
			return now >= voting_period_start && now < voting_period_end;
		}
		false
	}

	/// Returns true iff `now` preceeds the start time of the poll.
	fn poll_in_signup_period<T: Config>(
		poll: Poll<T>
	) -> bool
	{
		let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
		return now < poll.created_at + poll.config.signup_period;
	}

	/// Returns true iff poll has ended.
	fn poll_is_over<T: Config>(
		poll: Poll<T>
	) -> bool
	{
		let now = <frame_system::Pallet<T>>::block_number().saturated_into::<u64>();
		let voting_period_start = poll.created_at + poll.config.signup_period;
		let voting_period_end = voting_period_start + poll.config.voting_period;
		return now < voting_period_end;
	}

	// fn hash_level(
	// 	nodes: vec::Vec<HashBytes>,
	// 	arity: usize
	// ) -> vec::Vec<HashBytes> 
	// {
	// 	let capacity: usize = nodes.len().div_ceil(arity);
	// 	let mut parents = vec::Vec::<HashBytes>::with_capacity(capacity);

	// 	let mut index = 0;
	// 	let mut subtree = vec::Vec::<BlsScalar>::with_capacity(arity);

	// 	// Hash each subtree of nodes respecting the provided tree arity
	// 	for leaf in nodes.iter()
	// 	{
	// 		subtree.push(BlsScalar::from_bytes(leaf).unwrap_or(BlsScalar::zero()));
	// 		index += 1;

	// 		if index % arity == 0
	// 		{
	// 			parents.push(sponge::hash(&subtree[..]).to_bytes());
	// 			subtree.clear();
	// 			index = 0;
	// 		}
	// 	}

	// 	// Fill the last subtree with zeros before hashing, if incomplete
	// 	if index != 0 && parents.len() < capacity
	// 	{
	// 		for _ in index..arity { subtree.push(BlsScalar::zero()); }
	// 		parents.push(sponge::hash(&subtree[..]).to_bytes());
	// 	}

	// 	parents
	// }

	// fn compute_subtree_root(
	// 	leaves: vec::Vec<HashBytes>,
	// 	arity: usize
	// ) -> (Option<HashBytes>, u8)
	// {
	// 	let mut depth: u8 = 0;
	// 	let mut nodes = leaves;
		
	// 	// Performs `ceil(log(leaves.len()))` iterations
	// 	while nodes.len() > 1
	// 	{
	// 		nodes = hash_level(nodes, arity);
	// 		depth += 1;
	// 	}

	// 	(nodes.first().copied(), depth)
	// }

	// fn compute_full_root<T: Config>(
	// 	subtree_root: Option<HashBytes>,
	// 	subtree_depth: u8,
	// 	arity: usize
	// ) -> Option<HashBytes>
	// { 
	// 	let max_depth = T::MaxTreeDepth::get();
	// 	if subtree_depth >= max_depth { return subtree_root }

	// 	let rem_depth = max_depth - subtree_depth;
	// 	let Some(_root) = subtree_root else { return None };
	// 	let mut root = BlsScalar::from_bytes(&_root).unwrap_or(BlsScalar::zero());

	// 	for _ in 0..rem_depth
	// 	{
	// 		let mut subtree = vec![BlsScalar::zero(); arity];
	// 		subtree[0] = root;
	// 		root = sponge::hash(&subtree);
	// 	}

	// 	Some(root.to_bytes())
	// }

	// fn compute_state_tree<T: Config>(
	// 	leaves: vec::Vec<HashBytes>,
	// 	arity: usize
	// ) -> (Option<HashBytes>, Option<HashBytes>, u8)
	// {
	// 	let (subtree_root, subtree_depth) = compute_subtree_root(leaves, arity);
	// 	let root = compute_full_root::<T>(subtree_root, subtree_depth, arity);

	// 	(root, subtree_root, subtree_depth)
	// }

	// ==========================================
	// TODO (M2) 
	// fn verify_proof<T: Config>(
	// 	_poll_data: Poll<T>,
	// 	_verify_key: VerifyKey<T>,
	// 	_proof_data: ProofData,
	// 	_commitment: CommitmentData
	// ) -> bool
	// {
	// 	true
	// }
	// fn verify_outcome<T: Config>(
	// 	poll_data: Poll<T>,
	// 	index: Option<u32>
	// ) -> Option<u128>
	// {
	// 	let Some(index) = index else { return None };
	// 	if (index as usize) < poll_data.config.vote_options.len()
	// 	{
	// 		return Some(poll_data.config.vote_options[index as usize]);
	// 	}

	// 	None
	// }
}
