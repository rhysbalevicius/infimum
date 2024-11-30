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
- To run the tests on your local machine, you will need to have cargo installed.
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustup install nightly-2023-08-31
rustup override set nightly-2023-08-31
rustup default nightly-2023-08-31
rustup target add wasm32-unknown-unknown --toolchain nightly-2023-08-31
```

### Running unit tests

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

## References

- This work is based off of the Ethereum project [MACI](https://github.com/privacy-scaling-explorations/maci), which itself was originally proposed by [Vitalik Buterin](https://ethresear.ch/t/minimal-anti-collusion-infrastructure/5413). 
- [W3F Proposal](https://github.com/w3f/Grants-Program/blob/master/applications/infimum.md)

