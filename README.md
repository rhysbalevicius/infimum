# Infimum

[![Build](https://github.com/rhysbalevicius/infimum/actions/workflows/ci.yml/badge.svg)](https://github.com/rhysbalevicius/infimum/actions/workflows/ci.yml)

## Overview

The Infimum project is intended to provide a private, receipt-free, and verifiable voting apparatus which disincentivizes collusion between participants. Note that the current scope of this project is intended to serve as a proof of concept and should *not* be used in production.

## Terminology

- **Poll:** A proposal to enact some outcome.
- **Participant:** A signer who has registered in a poll, and whose interactions with that poll are deemed valid.
- **Coordinator:** A trusted party who may create and manage polls. Coordinators are responsible for verifiably proving the outcome of their polls.
- **Interaction:** A vote, or a nullifier, which is submitted by a participant for a specific poll which they have registered in.

## Usage

### Basic usage guide

#### Requirements:
- If you wish to run the tests with docker, you will need to have the [latest version](https://www.docker.com/) installed on your machine.
- To run the tests on your local machine, you will need to have cargo installed. In particular:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustup install nightly-2023-08-31
rustup override set nightly-2023-08-31
rustup default nightly-2023-08-31
rustup target add wasm32-unknown-unknown --toolchain nightly-2023-08-31
```

### Unit tests

You can run the tests locally if you have [`cargo`](https://doc.rust-lang.org/cargo/index.html) installed.
```sh
cd /path/to/infimum

cargo +nightly test --manifest-path=./pallet/Cargo.toml
```

Alternatively, the unit tests can also be run with docker. 

```sh
cd /path/to/infimum

# Confirmed to be working using `Docker version 24.0.2, build cb74dfc`
docker compose --profile test up
```

### Interacting with the pallet

The CLI for interacting with this pallet is currently under active development. Due to the complexity and highly sensitive nature of the types of the extrinsic parameters other interfaces, such as the [frontend-template](https://github.com/rhysbalevicius/substrate-front-end-template), are highly difficult to use in order to interact with the pallet.  

However in general the interaction flow is:
- A user registers as a coordinator using the `register_as_coordinator` extrinsic.
- A coordinator creates a poll using the `create_poll` extrinsic.
- Participants register for the poll during the registration period using the `register_as_participant` extrinsic.
- Participants interact with the poll during the interaction period using the `interact_with_poll` extrinsic.
- After the poll ends, the coordinator is then permitted:
    - Merges the state trees using the `merge_poll_state` extrinsic. This calculates various public signals on chain which are later used to verify the zero-knowledge proofs.
    - Generates proofs for the poll results offchain. The basic flow for compiling and generating proofs can be found in [circuits](https://github.com/rhysbalevicius/infimum/tree/main/circuits).
    - These proofs are then commited to the pallet storage using the `commit_outcome` extrinsic.
    - After every proof has been successfully submitted a `PollOutcome` event is deposited with the `outcome_index` field set.

An end-to-end example of this flow can currently be found [here](https://github.com/rhysbalevicius/infimum/tree/main/cli/__tests__/e2e.test.ts). In order to run this example, you will first need to locally spin up a substrate node with the Infimum pallet. For your convenience, you can do so with: 
```sh
cd /path/to/infimum

# Download the zkeys
curl --output ./cli/__tests__/data/process.zkey https://cdn.rhys.tech/infimum/process.zkey 
curl --output ./cli/__tests__/data/tally.zkey https://cdn.rhys.tech/infimum/tally.zkey

# Start the node
docker-compose start runtime-node

# Then open a new tab and navigate to the `cli` directory
cd cli

# Install the dependencies
npm i

# Run the end-to-end test
npm run test
```

## References

- This work is based off of the Ethereum project [MACI](https://github.com/privacy-scaling-explorations/maci), which itself was originally proposed by [Vitalik Buterin](https://ethresear.ch/t/minimal-anti-collusion-infrastructure/5413). 
- The zero-knowledge circuits have been forked from [maci-circuits](https://github.com/privacy-scaling-explorations/maci/tree/dev/packages/circuits)
- The poseidon hasher is a fork of [light-poseidon](https://github.com/Lightprotocol/light-poseidon) for `no_std` compatibility
- [W3F Proposal](https://github.com/w3f/Grants-Program/blob/master/applications/infimum.md)

