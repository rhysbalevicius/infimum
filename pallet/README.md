# infimum-pallet

## Interface

### Dispatchable Functions

#### Public

- `register_as_coordinator` - Registers the caller as a coordinator.
- `rotate_keys` - Permits a registered coordinator to rotate their keys. Rejects if called during an active poll.
- `create_poll` - Permits a registered coordinator to create a new poll.
- `merge_poll_state` - Compute the roots of the current poll state trees. This operation must be performed prior to commiting the poll outcome. 
- `commit_outcome` - Permits a coordinator to commit, in batches, proofs that all of the valid participant registrations and poll interactions were included in the computation which decided the winning vote option. 
- `nullify_poll` - Permits a coordinator to mark a poll with a tombstone in the event that it expired and did not record a single interaction.
- `register_as_participant` - Permits a signer to participate in an upcoming poll. Rejected if signup period has elapsed.
- `interact_with_poll` - Permits a signer to interact with an ongoing poll. Rejects if not within the voting period. Valid messages include: a vote, and a key rotation. Participants may secretly call this method (e.g., using a different signer) in order to override their previous vote. 

### Storage Items

- `Polls` - Map between poll id's and polls. Polls contain configuration specific information such as vote options and the current state.
- `Coordinators` - A registry of coordinators.
- `CoordinatorPollIds` - A map of coordinators to the poll ids they manage.

### Events:

- `CoordinatorRegistered` - A new coordinator was registered.
- `CoordinatorKeysChanged` - A coordinator rotated one of their keys.
- `ParticipantRegistered` - A participant registered to vote in a poll.
- `PollCreated` - A new poll was created.
- `PollInteraction` - Poll was interacted with.
- `PollCommitmentUpdated` - Poll state was partially processed.
- `PollStateMerged` - Poll state tree root was computed.
- `PollOutcome` - Poll result was verified.
- `PollNullified` - Empty and expired poll was nullified.

### Errors:

- `CoordinatorAlreadyRegistered` - A coordinator has tried to reregister.
- `CoordinatorNotRegistered` - A signer has called an extrinsic which is designated only for coordinators, such as `create_poll`.
- `CoordinatorPollLimitReached` - A coordinator tries to create a poll, but has already created the maximum allowable number of polls.
- `ParticipantRegistrationLimitReached` - A signer tries to register in a poll, but the maximum allowable number of registrations has already been reached.
- `ParticipantInteractionLimitReached` - A signer tries to interact with a poll, but the maximum allowable number of interactions has already been reached.
- `PollConfigInvalid` - A coordinator has tried to create a poll with an invalid parameterization.
- `PollRegistrationInProgress` - A participant or coordinator has attempted to perform some action which is restricted during poll registration.
- `PollRegistrationHasEnded` - A signer has tried to register for a poll which is no longer in the registration period.
- `PollVotingInProgress` - A coordinator has attempted to perform some action which is restricted during the poll voting period.
- `PollCurrentlyActive` - A poll owned by the same coordinator has not yet ended or is missing a valid outcome.
- `PollVotingHasEnded` - A poll has ended and may no longer be interacted with by participants.
- `PollDoesNotExist` - A bad poll id was supplied to some extrinsic.
- `PollDataEmpty` - A coordinator tried to process the state of a poll without sufficiently many registrations or interactions.
- `PollOutcomeAlreadyDetermined` - A coordinator tried to commit the outcome of a poll which has already been decided.
- `PollStateNotMerged` - A coordinator tried to submit proofs prior to merging the poll state trees.
- `PollMergeFailed` - An attempt to merge on of the state trees failed.
- `PollRegistrationFailed` - An attempt to register in a poll failed.
- `PollInteractionFailed` - An attempt to interact with a poll failed.
- `MalformedKeys` - A bad verification key or public key was supplied by a user.
- `MalformedProof` - A zero-knowledge proof was submitted but failed to pass verification.
- `MalformedInput` - The arguments passed to an extrinsic are insufficient.

## Usage

### Integration

In order to [add the pallet](https://docs.substrate.io/tutorials/build-application-logic/add-a-pallet/) you'll need choose values for the following config:

```rust
use pallet_infimum;

pub trait Config: pallet_infimum::Config
{
    type RuntimeEvent = RuntimeEvent;
    
    /// The maximum number of polls that any individual coordinator may be responsible for.
    type MaxCoordinatorPolls = ConstU32<1028>;

    /// The maximal number of potential outcomes any one poll may have.  
    type MaxVoteOptions = ConstU32<32>;
	
    /// The maximal number of registrations any one poll may have.
    type MaxPollRegistrations = ConstU32<65536>;
    
    /// The maximal number of registrations any one poll may have.
    type MaxPollInteractions = ConstU32<65536>;
}
```

## Dependencies

This pallet currently depends upon the following dependencies:
- [ark-bn254](https://docs.rs/ark-bn254/latest/ark_bn254/)
- [ark-ff](https://docs.rs/ark-ff/latest/ark_ff/)
- [ark-serialize](https://docs.rs/ark-serialize/latest/ark_serialize/)
- [ark-groth16](https://docs.rs/ark-groth16/latest/ark_groth16/)
- [ark-crypto-primitives](https://crates.io/crates/ark-crypto-primitives)
