#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec::Vec;
use frame_support::storage::bounded_vec::BoundedVec;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

type PollId = u32;
type PollIds<T> = BoundedVec<PollId, <T as Config>::MaxCoordinatorPolls>;
type CoordinatorPublicKeyDef<T> = BoundedVec<u8, <T as Config>::MaxPublicKeyLength>;
type CoordinatorVerifyKeyDef<T> = BoundedVec<u8, <T as Config>::MaxVerifyKeyLength>;

#[frame_support::pallet]
pub mod pallet 
{
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

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

		/// The maximum length of a coordinator public key.
		#[pallet::constant]
		type MaxPublicKeyLength: Get<u32>;

		/// The maximum length of a coordinator verification key.
		#[pallet::constant]
		type MaxVerifyKeyLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> 
	{
		/// A new coordinator was registered.
		CoordinatorRegistered { who: T::AccountId },
		
		/// A coordinator rotated one of their keys.
		CoordinatorKeyChanged { 
			/// The coordinator.
			who: T::AccountId, 
			/// The new public key, if it was rotated.
			public_key: Option<CoordinatorPublicKeyDef<T>>,
			/// The new verify key, if it was rotated.
			verify_key: Option<CoordinatorVerifyKeyDef<T>>
		},
		
		/// A new poll was created.
		PollCreated { coordinator: T::AccountId /* ... */ },

		Test { t: BlockNumberFor<T> }
	}

	#[pallet::error]
	pub enum Error<T>
	{
		/// Coordinator is already registered.
		CoordinatorAlreadyRegistered,

		/// Coordinator is not registered.
		CoordinatorNotRegistered,
		
		/// Coordinator public key is too long.
		CoordinatorPublicKeyTooLong,
		
		/// Coordinator verification key is too long.
		CoordinatorVerifyKeyTooLong,

	}

	/// The next free poll index, i.e. the number of polls created so far.
	#[pallet::storage]
	#[pallet::getter(fn poll_count)]
	pub type PollCount<T> = StorageValue<_, PollId, ValueQuery>;

	// #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	// pub struct Test 
	// {
	// 	x:u64,
	// 	y:u64
	// }

	/// Poll storage definition.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Poll<T: Config, PollId>
	{
		/// The poll id.
		index: PollId,

		/// The poll creator.
		coordinator: T::AccountId,

		/// The poll creation time.
		created_at: BlockNumberFor<T>,

		/// The poll signup period.
		signup_period: BlockNumberFor<T>,

		/// The poll voting period.
		voting_period: BlockNumberFor<T>,

		// /// The result of the poll.

		// /// Processing data?

		// /// Metadata?

		// /// The options (e.g. fn preimages?).
	}

	/// Map of ids to polls.
	#[pallet::storage]
	pub type Polls<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PollId,
		Poll<T, PollId>
	>;

	/// Coordinator storage definition.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Coordinator<T: Config> 
	{
		/// The coordinators public key.
		pub public_key: CoordinatorPublicKeyDef<T>,

		/// The coordinators verify key.
		pub verify_key: CoordinatorVerifyKeyDef<T>,

		///// The coordinators polls.
		// pub poll_ids: CoordinatorPollIds<T>
		// pub poll_ids: Vec<PollId>
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
	pub type CoordinatorPollIDs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		PollIds<T>
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> 
	{
		/// Register the caller as a coordinator, granting the ability to create polls.
		/// 
		/// The dispatch origin of this call must be _Signed_ and the sender must
		/// have funds to cover the deposit.
		///
		/// - `public_key`: The public key of the coordinator.
		/// - `verify_key`: The verification key of the coordinator.
		///
		/// Emits `CoordinatorRegistered`.
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
		pub fn register_as_coordinator(
			origin: OriginFor<T>,
			public_key: Vec<u8>,
			verify_key: Vec<u8>
		) -> DispatchResult
		{
			// TODO (rb) should we permit the pallet to be configured such that only `sudo` may register coordinators? 

			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;
			
			// A coordinator may only be registered once.
			ensure!(
				!Coordinators::<T>::contains_key(&sender), 
				Error::<T>::CoordinatorAlreadyRegistered
			);

			// Validate the key provided, throw if it fails
			// TODO (rb) verify that the public key is well defined
			// TODO (rb) split out verification logic into helper fn
			
			let pk: CoordinatorPublicKeyDef<T> = public_key
				.try_into()
				.map_err(|_| Error::<T>::CoordinatorPublicKeyTooLong)?;

			let vk: CoordinatorVerifyKeyDef<T> = verify_key
				.try_into()
				.map_err(|_| Error::<T>::CoordinatorVerifyKeyTooLong)?;

			// Store the coordinator keys.
			Coordinators::<T>::insert(&sender, Coordinator {
				public_key: pk,
				verify_key: vk,
				// poll_ids: CoordinatorPollIds::<T>::default()
			});

			// Emit a registration event
			Self::deposit_event(Event::CoordinatorRegistered { who: sender });
			
			// Coordinator was successfully registered.
			Ok(())
		}

	}
}
