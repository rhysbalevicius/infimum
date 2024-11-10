import inf from 'inf-lib';
import { ApiPromise } from '@polkadot/api';
import { IProcessMessagesCircuitInputs, ITallyCircuitInputs, Poll } from 'maci-core';
import { genTreeCommitment, genTreeProof, hashLeftRight } from 'maci-crypto';
import { BigNumberish } from 'maci-domainobjs/build/ts/types';
import * as snarkjs from 'snarkjs';
import path from 'path';
import fs from 'fs';
import { bnToBytes } from './serialize';
import { ProofData } from './interface';

export const waitForBlock = async (api: ApiPromise, targetBlock: number): Promise<void> => 
{
    return new Promise(async resolve =>
    {
        const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) =>
        {
            const currentBlock = header.number.toNumber();
            if (currentBlock >= targetBlock)
            {
                unsubscribe();
                resolve();
            }
        });
    });
};

export const waitForNumBlocks = async (api: ApiPromise, numBlocks: number): Promise<void> => 
    {
        return new Promise(async resolve =>
        {
            let startBlock: number | undefined;
            const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) =>
            {
                const currentBlock = header.number.toNumber();
                if (startBlock === undefined) startBlock = currentBlock;

                if ((currentBlock - startBlock) >= numBlocks)
                {
                    unsubscribe();
                    resolve();
                }
            });
        });
    };

export const provePollResults = async (poll: Poll) =>
{
    const { process, tally, outcome } = getPollResults(poll);
    const batches = await prove(process, tally);
    return {
        batches,
        outcome
    };
};

export const prove = async (
    processInputs: IProcessMessagesCircuitInputs,
    tallyInputs: ITallyCircuitInputs,
    artefactPath: string = 'data'
): Promise<Array<[ProofData, Array<number>]>> =>
{
    const process = await snarkjs.groth16.fullProve(
        processInputs as unknown as snarkjs.CircuitSignals,
        path.join(__dirname, `${artefactPath}/process.wasm`),
        path.join(__dirname, `${artefactPath}/process.zkey`)
    );
    const processCommitment = bnToBytes(processInputs['newSbCommitment']);

    const tally = await snarkjs.groth16.fullProve(
        tallyInputs as unknown as snarkjs.CircuitSignals,
        path.join(__dirname, `${artefactPath}/tally.wasm`),
        path.join(__dirname, `${artefactPath}/tally.zkey`)
    );
    const tallyCommitment = bnToBytes(tallyInputs['newTallyCommitment']);

    const vkProcess = JSON.parse(fs.readFileSync(path.join(__dirname, `${artefactPath}/vk-process.json`)).toString());
    const vkTally = JSON.parse(fs.readFileSync(path.join(__dirname, `${artefactPath}/vk-tally.json`)).toString());

    if (!(await snarkjs.groth16.verify(vkProcess, process.publicSignals, process.proof)))
        throw new Error('Process circuit verification failed');

    if (!(await snarkjs.groth16.verify(vkTally, tally.publicSignals, tally.proof)))
        throw new Error('Tally circuit verification failed');

    return [
        [ inf.serialize_proof(process.proof) as ProofData, processCommitment ],
        [ inf.serialize_proof(tally.proof) as ProofData, tallyCommitment ]
    ];
};

export const asHex = (val: BigNumberish): string => `0x${BigInt(val).toString(16)}`;

export const getPollResults = (poll: Poll) =>
{
    const process = poll.processMessages(poll.pollId, false, false);
    const tally = poll.tallyVotesNonQv();
    const newResultsCommitment = bnToBytes(
        genTreeCommitment(
            poll.tallyResult,
            BigInt(asHex(tally!.newResultsRootSalt as BigNumberish)),
            poll.treeDepths.voteOptionTreeDepth,
        )
    );
    const spentVotesHash = bnToBytes(
        hashLeftRight(
            poll.totalSpentVoiceCredits,
            BigInt(asHex(tally!.newSpentVoiceCreditSubtotalSalt as BigNumberish)),
        )
    );
    const tallyResults = poll.tallyResult.map(x => Number(x));
    const voteOptionIndices = tallyResults.map((_, i) => i);
    const totalSpentSalt = bnToBytes(tally.newSpentVoiceCreditSubtotalSalt);
    const tallyResultSalt = bnToBytes(tally.newResultsRootSalt);
    const totalSpent = bnToBytes(poll.totalSpentVoiceCredits);
    const tallyResultProofs = voteOptionIndices
        .map(index => genTreeProof(index, poll.tallyResult, Number(poll.treeDepths.voteOptionTreeDepth)))
        .map(a => a.map(b => b.map(c => bnToBytes(c))));

    return {
        process,
        tally,
        outcome: {
            tallyResults,
            tallyResultProofs,
            totalSpent,
            totalSpentSalt,
            tallyResultSalt,
            newResultsCommitment,
            spentVotesHash
        }
    };
};
