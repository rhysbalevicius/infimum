# Infimum

[![Build](https://github.com/rhysbalevicius/infimum/actions/workflows/ci.yml/badge.svg)](https://github.com/rhysbalevicius/infimum/actions/workflows/ci.yml)

The Infimum project is intended to provide a private, receipt-free, and verifiable voting apparatus which disincentivizes collusion between participants. Note that the current scope of this project is intended to serve as a proof of concept and should *not* be used in production.

## Overview

In it's current state the pallet provides a basic mechanism for facillitating the "lifecycle" of polls, and has been designed with the intention of supporting the future goals of the project.

## Terminology

- **Poll:** A proposal to enact some outcome.
- **Participant:** A signer who has registered in a poll, and whose interactions with that poll are deemed valid.
- **Coordinator:** A trusted party who may create and manage polls. Coordinators are responsible for verifiably proving the outcome of their polls.
- **Interaction:** A vote, or a nullifier, which is submitted by a participant for a specific poll which they have registered in.

## Goals

The long term goals of the project are aimed towards providing a customizable voting system with the following properties:
- Collusion resistant: Schemes which promote collusion should be deincentivized.
- Private: The preference of an individual participant should be kept secret.
- Receipt freeness: Participants should not be able to prove what they voted for.
- Verifiable: Votes should not be able to be censored, nor cast fraudulently.

Additionally the project is ultimately directed towards being non-opinionated and generic, such that it can be integrated with other voting apparatus, and support different backends.

As this project is still in it's infancy, all of the above have not yet been realized. More detailed items can be found (and proposed!) in the issue tracker.

## Interface

### Dispatchable Functions

#### Public

- `register_as_coordinator` - Registers the caller as a coordinator.
- `rotate_keys` - Permits a registered coordinator to rotate their keys. Rejects if called during an active poll.
- `create_poll` - Permits a registered coordinator to create a new poll.
- `commit_outcome` - Permits a coordinator to commit, in batches, proofs that all of the valid participant registrations and poll interactions were included in the computation which decided the winning vote option. 
- `nullify_poll` - Permits a coordinator to mark a poll with a tombstone in the event that it expired and did not record a single interaction.
- `register_as_participant` - Permits a signer to participate in an upcoming poll. Rejected if signup period has elapsed.
- `interact_with_poll` - Permits a signer to interact with an ongoing poll. Rejects if not within the voting period. Valid messages include: a vote, and a key rotation. Participants may secretly call this method (e.g., using a different signer) in order to override their previous vote. 

### Storage Items

- `Polls` - Map between poll id's and polls. Polls contain configuration specific information such as vote options and the current state.
- `Coordinators` - A registry of coordinators.

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
- `MalformedVerifyKey` - A bad verification key was supplied to some extrinsic.
- `MalformedProof` - A zero-knowledge proof was submitted but failed to pass verification.

## Usage

### Basic usage guide

#### Requirements:
- You will need to have the latest version of [docker](https://www.docker.com/) installed on your machine.

First locally spin up a substrate node, the frontend, and then navigate to http://localhost:8000 where you can begin to play around with the pallet:

```sh
docker compose --profile dev up
```

Both of the frontend and node code we're running are templates. Refer to their codebases here:
- [substrate-node-template](https://github.com/substrate-developer-hub/substrate-node-template)
- [substrate-front-end-template](https://github.com/substrate-developer-hub/substrate-front-end-template)

You're now ready to try out some example scenarios. First, let's try to register as a coordinator. On the frontend, scroll down to the "pallet interactor" section. Next select the Infimum pallet and then the `registerAsCoordinator` extrinsic. This accepts two parameters, try out the following values: 
- For `PublicKey`: try using `{"x":[0,0,0,0], "y":[0,0,0,0]}`. 
- For `VerifyKey`: try using `0x0`.

Note that it's acceptable to pass in dummy variables here. While this is largely due to the fact that the backend to the Pallet is under development for an upcoming milestone. In general it will be up to the runtime developer to implement a custom provider with the responsiblility of guarding registrations (both of coordinators, and participants).

Now create a poll. You can do this by selecting the `createPoll` extrinsic which requires 4 parameters:
- `signupPeriod`: Specifies the number of blocks participants may vote within the poll, e.g. you can try `25`
- `votingPeriod`: Specifies the number of blocks participants may vote within the poll, e.g. you can try `25`
- `maxRegistrations`: Specifies how many participants are allowed to register in the poll, e.g. you can try `256`
- `voteOptions`: This is vec of `u128` representing the potential voting possibilities. An option's identity is encoded by the index, and the value may optionally be used to encode some additional metadata. For example, try `0,1,2,3`.

After successfully submitting the extrinsic, take note of the `poll_id` field emitted in the `PollCreated` event. This will be needed by participants to register in and interact with the poll. By default, the poll id's start at 0 and sequentially increment by 1.

Next sign up a participant with `registerAsParticipant`. In the nav bar at the top of the webapp you can select a new account to sign the transaction, however there is inherently nothing stopping a coordinator from also registerring in the poll. Again, guards against registration will largely be configured by the runtime developer.

We expect that individuals can only register up to `signupPeriod` blocks after the one the poll was created and entered into storage. So depending on what value you picked in the previous step and how long it takes you to complete this one this call may have succeeded or failed. Next try playing around with some of the other extrinsics.

Note that in it's current state this pallet is largely a shell and so naturally leaves a lot to be desired. Only in future milestones will we observe some impression of the [stated goals](#goals) begin to fully materialize.


### Running unit tests

You can run the tests locally if you have [`cargo`](https://doc.rust-lang.org/cargo/index.html) installed.
```sh
cd /path/to/infimum

# Confirmed to be working using `cargo 1.77.0-nightly (1ae631085 2024-01-17)`
cargo +nightly test --manifest-path=./pallet/Cargo.toml
```

Alternatively, the unit tests can also be run with docker. 

```sh
cd /path/to/infimum

# Confirmed to be working using `Docker version 24.0.2, build cb74dfc`
docker compose --profile test up
```


### Integration

In order to [add the pallet](https://docs.substrate.io/tutorials/build-application-logic/add-a-pallet/) you'll need choose values for the following config:

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

## Dependencies

This pallet currently depends upon the following dependencies:
- [Poseidon252](https://github.com/dusk-network/Poseidon252/)
- [bls12_381](https://github.com/dusk-network/bls12_381/)


## References

- This work is based off of the Ethereum project [MACI](https://github.com/privacy-scaling-explorations/maci), which itself was originally proposed by [Vitalik Buterin](https://ethresear.ch/t/minimal-anti-collusion-infrastructure/5413). 
- [W3F Proposal](https://github.com/w3f/Grants-Program/blob/master/applications/infimum.md)

