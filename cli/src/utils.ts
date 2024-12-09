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

export const provePollResults = async (poll: Poll, artefactPath: string = 'data') =>
{
    const { processInputs, tallyInputs, outcome } = getPollResults(poll);
    const batches = await prove(processInputs, tallyInputs, artefactPath);
    return {
        batches,
        outcome
    };
};

export const prove = async (
    processInputs: Array<IProcessMessagesCircuitInputs>,
    tallyInputs: Array<ITallyCircuitInputs>,
    artefactPath: string
): Promise<Array<[ProofData, Array<number>]>> =>
{
    const vkProcess = JSON.parse(fs.readFileSync(path.join(__dirname, `${artefactPath}/vk-process.json`)).toString());
    const vkTally = JSON.parse(fs.readFileSync(path.join(__dirname, `${artefactPath}/vk-tally.json`)).toString());

    const processProofs: Array<[ProofData, Array<number> ]> = [];
    for (const input of processInputs)
    {
        const { proof, publicSignals } = await snarkjs.groth16.fullProve(
            input as unknown as snarkjs.CircuitSignals,
            path.join(__dirname, `${artefactPath}/process.wasm`),
            path.join(__dirname, `${artefactPath}/process.zkey`)
        );
        const commitment = bnToBytes(input['newSbCommitment']);
        processProofs.push([ inf.serialize_proof(proof), commitment ]);

        if (!(await snarkjs.groth16.verify(vkProcess, publicSignals, proof)))
            throw new Error('Process circuit verification failed');
    }

    const tallyProofs: Array<[ProofData, Array<number> ]> = [];
    for (const input of tallyInputs)
    {
        const { proof, publicSignals } = await snarkjs.groth16.fullProve(
            input as unknown as snarkjs.CircuitSignals,
            path.join(__dirname, `${artefactPath}/tally.wasm`),
            path.join(__dirname, `${artefactPath}/tally.zkey`)
        );
        const commitment = bnToBytes(input['newTallyCommitment']);
        tallyProofs.push([ inf.serialize_proof(proof), commitment ]);

        if (!(await snarkjs.groth16.verify(vkTally, publicSignals, proof)))
            throw new Error('Tally circuit verification failed'); 
    }

    return [
        ...processProofs,
        ...tallyProofs
    ];
};

export const asHex = (val: BigNumberish): string => `0x${BigInt(val).toString(16)}`;

export const getPollResults = (poll: Poll) =>
{
    const processInputs: Array<IProcessMessagesCircuitInputs> = [];
    for (;;)
    {
        try
        {
            const inputs = poll.processMessages(poll.pollId, false, false);
            processInputs.push(inputs);
        }
        catch (_) { break; }
    }
    
    const tallyInputs: Array<ITallyCircuitInputs> = [];
    for (;;)
    {
        try
        {
            const inputs = poll.tallyVotesNonQv();
            tallyInputs.push(inputs);
        }
        catch (_) { break; }
    }

    const tally = tallyInputs.at(-1)!;
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
        processInputs,
        tallyInputs,
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
