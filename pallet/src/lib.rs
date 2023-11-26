#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_std::vec::Vec;
use frame_support::storage::bounded_vec::BoundedVec;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

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
		CoordinatorRegistered { who: T::AccountId }

	}

	#[pallet::error]
	pub enum Error<T>
	{
		/// Coordinator is already registered.
		CoordinatorAlreadyRegistered,

		/// Coordinator public key is too long.
		CoordinatorPublicKeyTooLong,

		/// Coordinator verification key is too long.
		CoordinatorVerifyKeyTooLong,

	}

	/// Coordinator storage definition.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Coordinator<T: Config> 
	{
		/// ...
		pub public_key: CoordinatorPublicKeyDef<T>,

		/// ...
		pub verify_key: CoordinatorVerifyKeyDef<T>,

		// TODO (rb) poll ids
	}

	/// Map of coordinators to their keys.
	#[pallet::storage]
	pub type Coordinators<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId,
		Coordinator<T>,
		OptionQuery
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

			let vk: CoordinatorVerifyKeyDef<T> = verify_key
				.try_into()
				.map_err(|_| Error::<T>::CoordinatorVerifyKeyTooLong)?;

			// Store the key in the map
			Coordinators::<T>::insert(&sender, Coordinator {
				public_key: pk,
				verify_key: vk
			});

			// Emit a registration event
			Self::deposit_event(Event::CoordinatorRegistered { who: sender });
			
			// Coordinator was successfully registered.
			Ok(())
		}
	}
}
