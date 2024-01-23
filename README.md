# Infimum

The Infimum pallet provides a private, receipt-free, and verifiable voting apparatus which disincentivizes collusion between participants. Note that the current scope of this project is intended to serve as a proof of concept and should *not* be used in production.

## Overview

In it's current state the pallet provides a basic mechanism for facillitating the "lifecycle" of polls, and has been designed with the intention of supporting the future goals of the project.

## Terminology

- **Poll:** A proposal to enact some outcome, encoded as an integer.
- **Participant:** A signer who has registered in an may participate in a poll.
- **Coordinator:** A trusted party who may create polls, and must facillitate processing the outcome.
- **Interaction:** A vote, or a nulifier, submitted by a participant against a specific poll.

## Goals

The long term goals of the project are aimed towards providing a fully-fledged voting system which has the following properties:
- Collusion resistant: Schemes which promote collusion should be deincentivized.
- Private: The preference of an individual participant should be kept secret.
- Receipt freeness: Participants should not be able to prove what they voted for.
- Verifiable: Votes should not be able to be censored, nor cast fraudulently 

Additionally the project is ultimately directed towards being non-opinionated and generic, such that it can be integrated with other voting apparatus, or support different cryptographic backends.

As this project is still in it's infancy, all of the above have not yet been realized. More detailed items can be found (and proposed!) in the issue tracker.

## Interface

### Dispatchable Functions

#### Public

- `register_as_coordinator` - Registers the caller as a coordinator.
- `create_poll` - Permits a registered coordinator to create a new poll.
- `commit_outcome` - Permits a coordinator to commit, in batches, proofs that all of the valid participant registrations and poll interactions were included in the computation which decided the winning vote option. 
- `nullify_poll` - Permits a coordinator to mark a poll with a tombstone in the event that it expired and did not record a single interaction.
- `register_as_participant` - Permits a signer to participate in an upcoming poll. Rejected if signup period has elapsed.
- `interact_with_poll` - Permits a signer to interact with an ongoing poll. Rejects if not within the voting period. Valid messages include: a vote, and a key rotation. Participants may secretly call this method (read: using a different signer) in order to override their previous vote. 

### Storage Items

- `Polls` - Map between poll id's and polls. Polls contain configuration specific information such as vote options and the current state.
- `Coordinators` - A registry of coordinators.

### Events:

- `CoordinatorRegistered` - A new coordinator was registered.
- `CoordinatorKeyChanged` - A coordinator rotated one of their keys.
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
- `MalformedVerifyKey` - A bad verification key was supplied to some extrinsic.
- `MalformedProof` - A zero-knowledge proof was submitted but failed to pass verification.

## Usage

### Integration

```rust
use pallet_infimum;

pub trait Config: pallet_infimum::Config
{
    type RuntimeEvent = RuntimeEvent;
    
    /// The maximum number of polls that any individual coordinator may be responsible for.
    type MaxCoordinatorPolls = ConstU32<1028>;
    
    /// The maximal length of a verification key.
    type MaxVerifyKeyLength = ConstU32<4079>;

    /// The maximal number of potential outcomes any one poll may have.  
    type MaxVoteOptions = ConstU32<32>;
	
    /// The maximal number of registrations any one poll may have.
    type MaxPollRegistrations = ConstU32<65536>;
    
    /// The maximal number of registrations any one poll may have.
    type MaxPollInteractions = ConstU32<65536>;

    /// As an optimization technique, registrations and interactions perform some of the work
    /// of the coordinator ahead of time. This constant permits runtime developers more fine-
    /// grained control over how much of that work is completed -- however, the tradeoff is 
    /// that additional storage space is consumed.
    type MaxIterationDepth = ConstU32<256>;
}
```

### Running tests

Unit tests can be run with docker:
```sh
docker compose --profile test up
```

Alternatively, spin up a node locally and then navigate to http://localhost:8000 where you may play around with the pallet using the frontend template:
```sh
docker compose --profile dev up
```


## Dependencies

This pallet currently depends upon the following dependencies:
- [Poseidon252](https://github.com/dusk-network/Poseidon252/)
- [bls12_381](https://github.com/dusk-network/bls12_381/)


## References

- This work is based off of the Ethereum project [MACI](https://github.com/privacy-scaling-explorations/maci), which itself was originally proposed by [Vitalik Buterin](https://ethresear.ch/t/minimal-anti-collusion-infrastructure/5413). 
- [W3F Proposal](https://github.com/w3f/Grants-Program/blob/master/applications/infimum.md)

