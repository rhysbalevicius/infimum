#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate uint;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

pub mod common;
pub mod deserialization;
pub mod verify;

// pub mod weights;
// pub use weights::*;

use frame_support::storage::bounded_vec::BoundedVec;
pub use pallet::*;
use sp_std::vec::Vec;

// type PublicInputsDef<T> = BoundedVec<u8, <T as Config>::MaxPublicInputsLength>;
// type ProofDef<T> = BoundedVec<u8, <T as Config>::MaxProofLength>;
// type VerificationKeyDef<T> = BoundedVec<u8, <T as Config>::MaxVerificationKeyLength>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	// use crate::{
	// 	common::prepare_verification_key,
	// 	deserialization::{deserialize_public_inputs, Proof, VKey},
	// 	verify::{
	// 		prepare_public_inputs, verify, G1UncompressedBytes, G2UncompressedBytes, GProof,
	// 		VerificationKey, SUPPORTED_CURVE, SUPPORTED_PROTOCOL,
	// 	},
	// };
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		// type WeightInfo: WeightInfo;

		// #[pallet::constant]
		// type MaxPublicInputsLength: Get<u32>;

		// /// The maximum length of the proof.
		// #[pallet::constant]
		// type MaxProofLength: Get<u32>;

		// /// The maximum length of the verification key.
		// #[pallet::constant]
		// type MaxVerificationKeyLength: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// VerificationSetupCompleted,
		// VerificationProofSet,
		// VerificationSuccess { who: T::AccountId },
		// VerificationFailed,
	}

	#[pallet::error]
	pub enum Error<T> {
		// /// Public inputs mismatch
		// PublicInputsMismatch,
		// /// Public inputs vector is to long.
		// TooLongPublicInputs,
		// /// The verification key is to long.
		// TooLongVerificationKey,
		// /// The proof is too long.
		// TooLongProof,
		// /// The proof is too short.
		// ProofIsEmpty,
		// /// Verification key, not set.
		// VerificationKeyIsNotSet,
		// /// Malformed key
		// MalformedVerificationKey,
		// /// Malformed proof
		// MalformedProof,
		// /// Malformed public inputs
		// MalformedPublicInputs,
		// /// Curve is not supported
		// NotSupportedCurve,
		// /// Protocol is not supported
		// NotSupportedProtocol,
		// /// There was error during proof verification
		// ProofVerificationError,
		// /// Proof creation error
		// ProofCreationError,
		// /// Verification Key creation error
		// VerificationKeyCreationError,
	}

	/// Storing a public input.
	// #[pallet::storage]
	// pub type PublicInputStorage<T: Config> = StorageValue<_, PublicInputsDef<T>, ValueQuery>;

	/// Storing a proof.
	// #[pallet::storage]
	// pub type ProofStorage<T: Config> = StorageValue<_, ProofDef<T>, ValueQuery>;

	/// Storing a verification key.
	// #[pallet::storage]
	// pub type VerificationKeyStorage<T: Config> = StorageValue<_, VerificationKeyDef<T>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Permits the caller to create polls, and stores their keys.
		#[pallet::weight(0)] // TODO weights
		pub fn register_as_coordinator(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		/// Permits a coordinator to rotate their public,private keypair. Rejected if called during an ongoing poll.
		#[pallet::weight(0)] // TODO weights
		pub fn rotate_public_key(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		// Permites a coordinator to rotate their verification key. Rejected if called after signup period.
		#[pallet::weight(0)] // TODO weights
		pub fn rotate_verify_key(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		// Permits a user to participate in an upcoming poll. Rejected if called after signup period.
		#[pallet::weight(0)] // TODO weights
		pub fn register_as_participant(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		/// Instantiates a new poll object with the caller as the designated coordinator. Emits an event with the poll data.
		#[pallet::weight(0)] // TODO weights
		pub fn create_poll(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		/// Inserts a message into the message tree for future processing by the coordinator. Valid messages include: a vote, 
		/// and a key rotation. Rejected if sent outside of the timeline specified by the poll config. Participants may secretly
		/// call this method to override their vote, thereby deincentivizing bribery.
		#[pallet::weight(0)] // TODO weights
		pub fn interact_with_poll(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		/// Used by the coordinator to compute roots of message state tree, which is used as a commitment value by the proof 
		/// verification logic. Rejected if called prior to poll end.
		#[pallet::weight(0)] // TODO weights
		pub fn merge_poll_state(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		/// Verifies the proof that the current batch of messages have been correctly processed and, if successful, updates
		/// the current verification state. Rejected if called prior to the merge of poll state.
		#[pallet::weight(0)] // TODO weights
		pub fn commit_processed_messages(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}

		/// Verifies the proof that the current batch of votes has been correctly tallied and, if successful, updates the 
		/// current verification state. Rejected if messages have not yet been processed. On verification of the final
		/// batch the poll result is recorded in storage and an event is emitted containing the result. Rejected if called
		/// before poll end.
		#[pallet::weight(0)] // TODO weights
		pub fn commit_tally_result(
			_origin: OriginFor<T>,
			some_arg: Vec<u8>
		) -> DispatchResult
		{
			// TODO 
			Ok(())
		}
	}

	// 
}

// ====================================================================================================
// zk snark tutorial methods

// /// Store a verification key.
		// #[pallet::weight(<T as Config>::WeightInfo::setup_verification_benchmark(vec_vk.len()))]
		// #[pallet::weight(0)]
		// pub fn setup_verification(
		// 	_origin: OriginFor<T>,
		// 	pub_input: Vec<u8>,
		// 	vec_vk: Vec<u8>,
		// ) -> DispatchResult {
		// 	let inputs = store_public_inputs::<T>(pub_input)?;
		// 	let vk = store_verification_key::<T>(vec_vk)?;
		// 	ensure!(vk.public_inputs_len == inputs.len() as u8, Error::<T>::PublicInputsMismatch);
		// 	Self::deposit_event(Event::<T>::VerificationSetupCompleted);
		// 	Ok(())
		// }

		// /// Verify a proof.
		// #[pallet::weight(<T as Config>::WeightInfo::verify_benchmark(vec_proof.len()))]
		// #[pallet::weight(0)]
		// pub fn verify(origin: OriginFor<T>, vec_proof: Vec<u8>) -> DispatchResult {
		// 	let proof = store_proof::<T>(vec_proof)?;
		// 	let vk = get_verification_key::<T>()?;
		// 	let inputs = get_public_inputs::<T>()?;
		// 	let sender = ensure_signed(origin)?;
		// 	Self::deposit_event(Event::<T>::VerificationProofSet);

		// 	match verify(vk, proof, prepare_public_inputs(inputs)) {
		// 		Ok(true) => {
		// 			Self::deposit_event(Event::<T>::VerificationSuccess { who: sender });
		// 			Ok(())
		// 		},
		// 		Ok(false) => {
		// 			Self::deposit_event(Event::<T>::VerificationFailed);
		// 			Ok(())
		// 		},
		// 		Err(_) => Err(Error::<T>::ProofVerificationError.into()),
		// 	}
		// }



// fn get_public_inputs<T: Config>() -> Result<Vec<u64>, sp_runtime::DispatchError> {
	// 	let public_inputs = PublicInputStorage::<T>::get();
	// 	let deserialized_public_inputs = deserialize_public_inputs(public_inputs.as_slice())
	// 		.map_err(|_| Error::<T>::MalformedPublicInputs)?;
	// 	Ok(deserialized_public_inputs)
	// }

	// fn store_public_inputs<T: Config>(
	// 	pub_input: Vec<u8>,
	// ) -> Result<Vec<u64>, sp_runtime::DispatchError> {
	// 	let public_inputs: PublicInputsDef<T> =
	// 		pub_input.try_into().map_err(|_| Error::<T>::TooLongPublicInputs)?;
	// 	let deserialized_public_inputs = deserialize_public_inputs(public_inputs.as_slice())
	// 		.map_err(|_| Error::<T>::MalformedPublicInputs)?;
	// 	PublicInputStorage::<T>::put(public_inputs);
	// 	Ok(deserialized_public_inputs)
	// }

	// fn get_verification_key<T: Config>() -> Result<VerificationKey, sp_runtime::DispatchError> {
	// 	let vk = VerificationKeyStorage::<T>::get();

	// 	ensure!(!vk.is_empty(), Error::<T>::VerificationKeyIsNotSet);
	// 	let deserialized_vk = VKey::from_json_u8_slice(vk.as_slice())
	// 		.map_err(|_| Error::<T>::MalformedVerificationKey)?;
	// 	let vk = prepare_verification_key(deserialized_vk)
	// 		.map_err(|_| Error::<T>::VerificationKeyCreationError)?;
	// 	Ok(vk)
	// }

	// fn store_verification_key<T: Config>(
	// 	vec_vk: Vec<u8>,
	// ) -> Result<VKey, sp_runtime::DispatchError> {
	// 	let vk: VerificationKeyDef<T> =
	// 		vec_vk.try_into().map_err(|_| Error::<T>::TooLongVerificationKey)?;
	// 	let deserialized_vk = VKey::from_json_u8_slice(vk.as_slice())
	// 		.map_err(|_| Error::<T>::MalformedVerificationKey)?;
	// 	ensure!(deserialized_vk.curve == SUPPORTED_CURVE.as_bytes(), Error::<T>::NotSupportedCurve);
	// 	ensure!(
	// 		deserialized_vk.protocol == SUPPORTED_PROTOCOL.as_bytes(),
	// 		Error::<T>::NotSupportedProtocol
	// 	);

	// 	VerificationKeyStorage::<T>::put(vk);
	// 	Ok(deserialized_vk)
	// }

	// fn store_proof<T: Config>(vec_proof: Vec<u8>) -> Result<GProof, sp_runtime::DispatchError> {
	// 	ensure!(!vec_proof.is_empty(), Error::<T>::ProofIsEmpty);
	// 	let proof: ProofDef<T> = vec_proof.try_into().map_err(|_| Error::<T>::TooLongProof)?;
	// 	let deserialized_proof =
	// 		Proof::from_json_u8_slice(proof.as_slice()).map_err(|_| Error::<T>::MalformedProof)?;
	// 	ensure!(
	// 		deserialized_proof.curve == SUPPORTED_CURVE.as_bytes(),
	// 		Error::<T>::NotSupportedCurve
	// 	);
	// 	ensure!(
	// 		deserialized_proof.protocol == SUPPORTED_PROTOCOL.as_bytes(),
	// 		Error::<T>::NotSupportedProtocol
	// 	);

	// 	ProofStorage::<T>::put(proof);

	// 	let proof = GProof::from_uncompressed(
	// 		&G1UncompressedBytes::new(deserialized_proof.a[0], deserialized_proof.a[1]),
	// 		&G2UncompressedBytes::new(
	// 			deserialized_proof.b[0][0],
	// 			deserialized_proof.b[0][1],
	// 			deserialized_proof.b[1][0],
	// 			deserialized_proof.b[1][1],
	// 		),
	// 		&G1UncompressedBytes::new(deserialized_proof.c[0], deserialized_proof.c[1]),
	// 	)
	// 	.map_err(|_| Error::<T>::ProofCreationError)?;

	// 	Ok(proof)
	// }

// ====================================================================================================
// Nicks pallet

// #![cfg_attr(not(feature = "std"), no_std)]

// // Re-export pallet items so that they can be accessed from the crate namespace.
// pub use pallet::*;

// #[frame_support::pallet]
// pub mod pallet 
// {
//     use frame_support::pallet_prelude::*;
//     use frame_system::pallet_prelude::*;

//     #[pallet::pallet]
//     pub struct Pallet<T>(_);

//     #[pallet::config]
//     pub trait Config: frame_system::Config 
//     {
//         // Because this pallet emits events, it depends on the runtime's definition of an event.
//         type RuntimeEvent : From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
//     }

//     // Pallets use events to inform users when important changes are made.
//     // Event documentation should end with an array that provides descriptive names for paramaters.
//     #[pallet::event]
//     #[pallet::generate_deposit(pub(super) fn deposit_event)]
//     pub enum Event<T: Config> 
//     {
//         // Event emitted when a claim has been created.
//         ClaimCreated { who: T::AccountId, claim: T::Hash },
    
//         // Event emitted when a claim is revoked by the owner.
//         ClaimRevoked { who: T::AccountId, claim: T::Hash }
//     }

//     #[pallet::error]
//     pub enum Error<T> 
//     {
//         // The claim already exists.
//         AlreadyClaimed,
//         // The claim does not exist, so it cannot be revoked.
//         NoSuchClaim,
//         // The claim is owned by another account, so caller can't revoke it.
//         NotClaimOwner,
//     }

//     #[pallet::storage]
//     pub(super) type Claims<T: Config> = 
//         // StorageMap<_, Blake2_128Concat, T::Hash, (T::AccountId, T::BlockNumber)>;
//         StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId>;

//     #[pallet::call]
//     impl<T: Config> Pallet<T> 
//     {
//         #[pallet::weight(0)] // TODO: benchmark + weights generation
//         #[pallet::call_index(1)]
//         pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult 
//         {
//             // Check that the extrinsic was signed and get the signer.
//             // This function will return an error if the extrinsic is not signed.
//             let sender = ensure_signed(origin)?;

//             // Verify that the specified claim has not already been stored. 
//             ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

//             // Get the block number from the FRAME System pallet. 
//             let current_block = <frame_system::Pallet<T>>::block_number();

//             // Store the claim with the sender and the block number.
//             // Claims::<T>::insert(&claim, (&sender, current_block));
//             Claims::<T>::insert(&claim, &sender);

//             // Emit an event that the claim was created.
//             Self::deposit_event(Event::ClaimCreated { who : sender, claim });

//             Ok(())
//         }

//         // #[pallet:weight(0)]
//         // #[pallet::call_index(2)]
//         // pub fn revoke_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult
//         // {
//         //     // TODO
//         // }
//     }
// }