import { Keyring } from '@polkadot/api';

const _keyring = new Keyring({ type: 'sr25519' });
export type NetworkKeyring = ReturnType<typeof _keyring.addFromUri>;

export interface PublicKey
{
    x: Array<number>;
    y: Array<number>;
}

export interface VerifyingKey
{
    alpha_g1: Array<number>;
    beta_g2: Array<number>;
    gamma_g2: Array<number>;
    delta_g2: Array<number>;
    gamma_abc_g1: Array<Array<number>>;
}

export interface VerifyingKeys
{
    process: VerifyingKey;
    tally: VerifyingKey;
}

export interface ProofData
{
    pi_a: Array<number>;
    pi_b: Array<number>;
    pi_c: Array<number>;
}

export interface PollOutcome
{
    tallyResults: number[],
    tallyResultProofs: number[][][][],
    totalSpent: number[],
    totalSpentSalt: number[],
    tallyResultSalt: number[],
    newResultsCommitment: number[],
    spentVotesHash: number[]
}
