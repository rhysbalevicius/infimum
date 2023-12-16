#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec;
use frame_support::storage::bounded_vec::BoundedVec;
use frame_support::traits::UnixTime;
use dusk_bls12_381::BlsScalar;
use dusk_poseidon::sponge;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

type PollId = u32;
type Timestamp = u64;
type Duration = Timestamp;
type PoseidonHashBytes = [u8; 32];
type PollInteractionData = [[u64; 4]; 16]; 
type ProofData = [[u64; 4]; 16];
type CommitmentData = PoseidonHashBytes;
type VerifyKey<T> = BoundedVec<u8, <T as Config>::MaxVerifyKeyLength>;
type VoteOptions<T> = BoundedVec<u128, <T as Config>::MaxVoteOptions>;

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

		/// Permit access to the current "timestamp" represented in milliseconds.
		type TimeProvider: UnixTime;

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
			/// The timestamp of the registration.
			timestamp: Timestamp,
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
			starts_at: Timestamp,
			/// The block number the voting period commences.
			ends_at: Timestamp,
			/// Optional metadata associated with poll.
			metadata: Option<T::Hash>
		},

		/// Poll state has been merged.
		PollStateMerged {
			/// The poll index.
			index: PollId,
			/// The poll registration tree.
			registration_tree: Option<PollStateTree>,
			/// The poll interaction tree.
			interaction_tree: Option<PollStateTree>
		},

		/// Poll was interacted with.
		PollInteraction {
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

		/// Participant is already registered in the poll.
		ParticipantAlreadyRegistered,

		/// Poll config is invalid.
		PollConfigInvalid,

		/// Poll registration period is in progress.
		PollRegistrationInProgress,

		/// Poll registration period has ended.
		PollRegistrationHasEnded,

		/// Poll voting period is in progress.
		PollVotingInProgress,

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

	/// Public keys committed by both coordinators and participants.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct PublicKey 
	{
		/// A 256-bit x-coordinate of the public key.
		x: [u64; 4],

		/// A 256-bit y-coordinate of the public key.
		y: [u64; 4]
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct PollState
	{
		/// The number of registered participants. Known only after the poll has been processed.
		num_participants: u32,

		/// The number of registered participants. Known only after the poll has been processed.
		num_interactions: u32,

		/// The merkle tree of registration data.
		registration_tree: PollStateTree,

		/// The merkle tree of interaction data.
		interaction_tree: PollStateTree,

		/// The number of valid commitments witnessed.
		num_witnessed: u32,

		/// The current proof commitment.
		commitment: (u32, [u8; 32]),

		/// The final result of the poll.
		outcome: Option<u128>,
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct PollStateTree
	{
		/// The depth of the subtree.
		subtree_depth: u8,

		// TODO (M1) replace `PollRegistrationHashes` and `PollInteractionHashes` storage with single BoundedVec here
		// subtree_hashes: vec::Vec<[u8; 32]>,

		/// The subroot of the tree.
		subtree_root: Option<PoseidonHashBytes>,

		/// The root of the "full"-depth tree containing `subtree_root` and zeros elsewhere.
		root: Option<PoseidonHashBytes>,
	}

	impl Default for PollStateTree
	{
		fn default() -> PollStateTree
		{
			PollStateTree {
				subtree_depth: 0,
				subtree_root: None,
				root: None
			}
		}
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

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PollConfiguration<T: Config>
	{
		/// The poll signup duration (in ms).
		signup_period: Duration,

		/// The poll voting duration (in ms).
		voting_period: Duration,

		/// The maximum number of participants permitted.
		max_participants: u32,

		/// The possible outcomes of the poll.
		vote_options: VoteOptions<T>,

		/// The size of 
		batch_size: u8,

		/// The arity of the state trees.
		tree_arity: u8
	}

	/// Poll storage definition.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Poll<T: Config, PollId>
	{
		/// The poll id.
		index: PollId,

		/// The poll creator.
		coordinator: T::AccountId,

		/// The poll creation time (in ms).
		created_at: Timestamp,

		/// Optional metadata associated to the poll.
		metadata: Option<T::Hash>,

		/// The mutable poll state.
		state: PollState,

		/// The poll config.
		config: PollConfiguration<T>
	}

	/// Map of ids to polls.
	#[pallet::storage]
	pub type Polls<T: Config> = CountedStorageMap<
		_,
		Twox64Concat,
		PollId,
		Poll<T, PollId>
	>;

	/// Ephemeral map of poll id to the hashes of registration data.
	#[pallet::storage]
	pub type PollRegistrationHashes<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PollId,
		vec::Vec<PoseidonHashBytes>,
		ValueQuery
	>;

	/// Ephemeral map of poll id to the hashes of voting data.
	#[pallet::storage]
	pub type PollInteractionHashes<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PollId,
		vec::Vec<PoseidonHashBytes>,
		ValueQuery
	>;

	/// Map of account ids to the polls they've registered for.
	#[pallet::storage]
	pub type PollsParticipatingIn<T: Config> = StorageMap<
		_, 
		Twox64Concat,
		T::AccountId,
		vec::Vec<PollId>,
		ValueQuery
	>;

	/// Coordinator storage definition.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Coordinator<T: Config> 
	{
		/// The coordinators public key.
		pub public_key: PublicKey,

		/// The coordinators verify key.
		pub verify_key: VerifyKey<T>
	}

	/// Map of coordinators to their keys.
	#[pallet::storage]
	pub type Coordinators<T: Config> = CountedStorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId,
		Coordinator<T>
	>;

	/// Map of coordinators to the poll IDs they manage.
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
		#[pallet::weight(T::DbWeight::get().reads_writes(3, 1))]
		pub fn rotate_public_key(
			origin: OriginFor<T>,
			public_key: PublicKey
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Check if origin is registered as a coordinator.
			let Some(coordinator) = Coordinators::<T>::get(&sender) else { Err(<Error::<T>>::CoordinatorNotRegistered)? };

			// Check that a poll is not currently in progress, if it exists.
			let coord_poll_ids = Self::poll_ids(&sender);
			let last_poll_index = coord_poll_ids.last();
			if let Some(index) = last_poll_index
			{
				ensure!(
					!poll_in_voting_period(Polls::<T>::get(index)),
					Error::<T>::PollVotingInProgress
				);
			}

			// Store the coordinators updated public key.
			Coordinators::<T>::insert(&sender, Coordinator {
				public_key: public_key.clone(),
				verify_key: coordinator.verify_key
			});
	
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
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 1))]
		pub fn rotate_verify_key(
			origin: OriginFor<T>,
			verify_key: Vec<u8>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Check if origin is registered as a coordinator.
			ensure!(
				Coordinators::<T>::contains_key(&sender), 
				Error::<T>::CoordinatorNotRegistered
			);

			// Check that a poll is not currently in progress.
			let coord_poll_ids = Self::poll_ids(&sender);
			let last_poll_index = coord_poll_ids.last();
			if let Some(index) = last_poll_index
			{
				ensure!(
					!poll_in_signup(Polls::<T>::get(index)),
					Error::<T>::PollOngoing
				);
			}

			// TODO (rb) Validate the key provided, throw if it fails
			let vk: CoordinatorVerifyKeyDef<T> = verify_key
				.try_into()
				.map_err(|_| Error::<T>::CoordinatorVerifyKeyTooLong)?;

			if let Some(coordinator) = Coordinators::<T>::get(&sender)
			{
				// Store the coordinators updated public key.
				Coordinators::<T>::insert(&sender, Coordinator {
					public_key: coordinator.public_key,
					verify_key: vk.clone()
				});
			} 

			// Emit the key rotation event.
			Self::deposit_event(Event::CoordinatorKeyChanged {
				who: sender,
				public_key: None,
				verify_key: Some(vk)
			});

			Ok(())
		}

		/// Permits a user to participate in an upcoming poll. Rejected if signup period has elapsed.
		///
		///	- `poll_id`: The id of the poll.
		///
		/// Emits `ParticipantRegistered`.
		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 1))]
		pub fn register_as_participant(
			origin: OriginFor<T>,
			poll_id: PollId
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;

			// Coordinators are not permitted to participate in polls.
			ensure!(
				!Coordinators::<T>::contains_key(&sender), 
				Error::<T>::CoordinatorNotRegistered
			);

			// Check that the poll exists.
			ensure!(
				Polls::<T>::contains_key(&poll_id),
				Error::<T>::PollDoesNotExist
			);

			// Check that the signer has not already registered to vote.
			let participants = PollParticipants::<T>::get(&poll_id);
			ensure!(
				!participants.contains(&sender),
				Error::<T>::ParticipantAlreadyRegistered
			);

			// Check that the maximum number of sign-ups has not been reached.
			let poll = Polls::<T>::get(&poll_id);
			if let Some(ref poll) = poll
			{
				ensure!(
					participants.len() < (poll.max_participants as usize),
					Error::<T>::ParticipantLimitReached
				);
			}

			// Check that the poll has not yet started.
			ensure!(
				poll_in_signup(poll),
				Error::<T>::PollOngoing
			);

			PollParticipants::<T>::append(&poll_id, &sender);

			Self::deposit_event(Event::ParticipantRegistered { 
				who: sender, 
				poll_id: poll_id
			});

			Ok(())
		}

		/// Create a new poll object where the caller is the designated coordinator.
		///
		/// - `signup_period`: Specifies the number of blocks that callers may register as a participant to vote in the poll.
		/// - `voting_period`: Specifies the number of blocks (following the signup period) that registered participants may vote for.
		///
		/// Emits `PollCreated`.
		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
		pub fn create_poll(
			origin: OriginFor<T>,
			signup_period: BlockNumberFor<T>,
			voting_period: BlockNumberFor<T>,
			max_participants: u32
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let coordinator = ensure_signed(origin)?;

			// Check if origin is registered as a coordinator
			ensure!(
				Coordinators::<T>::contains_key(&coordinator), 
				Error::<T>::CoordinatorNotRegistered
			);

			let coord_poll_ids = Self::poll_ids(&coordinator);

			// A coordinator may have at most `MaxCoordinatorPolls` polls, skipped if zero.
			let max_polls = T::MaxCoordinatorPolls::get() as usize;
			ensure!(
				max_polls == 0 || coord_poll_ids.len() < max_polls,
				Error::<T>::CoordinatorMayNotCreatePolls
			);

			// A coordinator may only have a single active poll at a given time.
			let last_poll_index = coord_poll_ids.last();
			if let Some(index) = last_poll_index
			{
				ensure!(
					!poll_is_ongoing(Polls::<T>::get(index)),
					Error::<T>::PollOngoing
				);
			}

			let index = Polls::<T>::count();
			let created_at = <frame_system::Pallet<T>>::block_number();
			Polls::<T>::insert(&index, Poll {
				coordinator: coordinator.clone(),
				index,
				created_at,
				signup_period,
				voting_period,
				max_participants
			});

			CoordinatorPollIDs::<T>::append(&coordinator, index);

			let starts_at = created_at + signup_period;
			let ends_at = starts_at + voting_period;
			Self::deposit_event(Event::PollCreated { 
				index,
				coordinator,
				starts_at,
				ends_at
			});

			Ok(())
		}
	}

	/// Returns true iff poll is not None and `now` preceeds the end time of the poll.
	fn poll_is_ongoing<T: Config>(
		poll: Option<Poll<T, PollId>>
	) -> bool
	{
		if let Some(p) = poll
		{
			let now = <frame_system::Pallet<T>>::block_number();
			return now < (p.created_at + p.voting_period + p.signup_period);
		}
		false
	}

	/// Returns true iff poll is not None and `now` preceeds the start time of the poll.
	fn poll_in_signup<T: Config>(
		poll: Option<Poll<T, PollId>>
	) -> bool
	{
		if let Some(p) = poll
		{
			let now = <frame_system::Pallet<T>>::block_number();
			return now < (p.created_at + p.signup_period);
		}
		false
	}
}
