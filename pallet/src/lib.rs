#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::storage::bounded_vec::BoundedVec;
pub use pallet::*;
use sp_std::vec::Vec;

type CoordinatorPublicKeyDef<T> = BoundedVec<u8, <T as Config>::MaxPublicKeyLength>;

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

		/// The maximum length of a coordinator verification key.
		#[pallet::constant]
		type MaxVerificationKeyLength: Get<u32>;

		/// The maximum length of a coordinator public key.
		#[pallet::constant]
		type MaxPublicKeyLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> 
	{
		/// A new coordinator was registered.
		CoordinatorRegistered { who: T::AccountId } // TODO (rb) coordinator obj

	}

	#[pallet::error]
	pub enum Error<T>
	{
		/// Coordinator is already registered.
		CoordinatorAlreadyRegistered,

		/// Coordinator public key is too long.
		CoordinatorPublicKeyTooLong,

	}

	/// Map of coordinators to their keys.
	#[pallet::storage]
	pub type Coordinators<T: Config> = StorageMap< // TODO (rb) coordinator obj
		_, 
		Blake2_128Concat, T::AccountId,
		CoordinatorPublicKeyDef<T>,
		OptionQuery
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> 
	{
		/// Permits the caller to create polls, and stores their keys.
		#[pallet::call_index(0)]
		#[pallet::weight(0)] // TODO weights
		pub fn register_as_coordinator(
			origin: OriginFor<T>,
			public_key: Vec<u8>
		) -> DispatchResult
		{
			// Check that the extrinsic was signed and get the signer.
			let sender = ensure_signed(origin)?;
			
			// A coordinator may only be registered once.
			ensure!(
				!Coordinators::<T>::contains_key(&sender), 
				Error::<T>::CoordinatorAlreadyRegistered
			);

			// Validate the key provided, throw if it fails
			// TODO (rb) verify that the public key is well defined
			
			let pk: CoordinatorPublicKeyDef<T> = public_key
				.try_into()
				.map_err(|_| Error::<T>::CoordinatorPublicKeyTooLong)?;

			// Store the key in the map
			Coordinators::<T>::insert(&sender, pk);

			// Emit a registration event
			Self::deposit_event(Event::CoordinatorRegistered { who: sender });
			
			// Coordinator was successfully registered.
			Ok(())
		}

		// /// Permits a coordinator to rotate their public,private keypair. Rejected if called during an ongoing poll.
		// #[pallet::weight(0)] // TODO weights
		// pub fn rotate_public_key(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }

		// // Permites a coordinator to rotate their verification key. Rejected if called after signup period.
		// #[pallet::weight(0)] // TODO weights
		// pub fn rotate_verify_key(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }

		// // Permits a user to participate in an upcoming poll. Rejected if called after signup period.
		// #[pallet::weight(0)] // TODO weights
		// pub fn register_as_participant(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }

		// /// Instantiates a new poll object with the caller as the designated coordinator. Emits an event with the poll data.
		// #[pallet::weight(0)] // TODO weights
		// pub fn create_poll(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// A coordinator may only have a single active poll at a given time

		// 	// A poll has the following properties:
		// 	// - id
		// 	// - coordinator
		// 	// - options vector
		// 	// - start/end times
		// 	// - result

		// 	Ok(())
		// }

		// /// Inserts a message into the message tree for future processing by the coordinator. Valid messages include: a vote, 
		// /// and a key rotation. Rejected if sent outside of the timeline specified by the poll config. Participants may secretly
		// /// call this method to override their vote, thereby deincentivizing bribery.
		// #[pallet::weight(0)] // TODO weights
		// pub fn interact_with_poll(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }

		// /// Used by the coordinator to compute roots of message state tree, which is used as a commitment value by the proof 
		// /// verification logic. Rejected if called prior to poll end.
		// #[pallet::weight(0)] // TODO weights
		// pub fn merge_poll_state(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }

		// /// Verifies the proof that the current batch of messages have been correctly processed and, if successful, updates
		// /// the current verification state. Rejected if called prior to the merge of poll state.
		// #[pallet::weight(0)] // TODO weights
		// pub fn commit_processed_messages(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }

		// /// Verifies the proof that the current batch of votes has been correctly tallied and, if successful, updates the 
		// /// current verification state. Rejected if messages have not yet been processed. On verification of the final
		// /// batch the poll result is recorded in storage and an event is emitted containing the result. Rejected if called
		// /// before poll end.
		// #[pallet::weight(0)] // TODO weights
		// pub fn commit_tally_result(
		// 	_origin: OriginFor<T>,
		// 	some_arg: Vec<u8>
		// ) -> DispatchResult
		// {
		// 	// TODO 
		// 	Ok(())
		// }
	}
}
