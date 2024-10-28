import { Keypair, PrivKey } from 'maci-domainobjs';
import { ApiPromise } from '@polkadot/api';
import { serializePublicKey, loadVerifyingKeys } from '../serialize';
import { InfimumExtrinsic } from '../extrinsic';
import { PollOutcome, ProofData, VerifyingKeys } from '../interface';
import { User } from './user';

export class Coordinator extends User
{
    public verifyingKey: VerifyingKeys;

    constructor(
        api: ApiPromise,
        keyringURI: string = '//Alice',
        privateKey: string = '//Alice',
        verifyKeyPath: string = '../../circuits/build'
    )
    {
        super(api, keyringURI, privateKey);

        this.verifyingKey = loadVerifyingKeys(verifyKeyPath);
    }

    // Must be called before other methods.
    async registerAsCoordinator()
    {
        const result = await this.sendExtrinsic(
            InfimumExtrinsic.registerAsCoordinator,
            [
                serializePublicKey(this.palletKeypair),
                this.verifyingKey
            ]
        );

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }

    async createPoll(
        signupPeriod: number,
        votingPeriod: number,
        registrationDepth: number,
        interactionDepth: number,
        processSubtreeDepth: number,
        tallySubtreeDepth: number,
        voteOptionTreeDepth: number,
        voteOptions: Array<number>
    )
    {
        const result = await this.sendExtrinsic(
            InfimumExtrinsic.createPoll,
            [
                signupPeriod,
                votingPeriod,
                registrationDepth,
                interactionDepth,
                processSubtreeDepth,
                tallySubtreeDepth,
                voteOptionTreeDepth,
                voteOptions
            ]
        );

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }

    async mergePollState()
    {
        const result = await this.sendExtrinsic(InfimumExtrinsic.mergePollState, []);

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }

    async commitOutcome(
        batches: Array<[ ProofData, Array<number> ]>,
        outcome?: PollOutcome
    )
    {
        const result = await this.sendExtrinsic(
            InfimumExtrinsic.commitOutcome,
            [
                batches,
                outcome
            ]
        );

        if (result.error) throw new Error(result.error);
        return result.depositEvents;
    }

    async nullifyPoll()
    {
        const result = await this.sendExtrinsic(InfimumExtrinsic.nullifyPoll, []);

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }

    async rotateKeys(
        privateKey: string,
        verifyingKey: VerifyingKeys
    )
    {
        const keypair = new Keypair(new PrivKey(privateKey));
        const result = await this.sendExtrinsic(
            InfimumExtrinsic.rotateKeys,
            [
                serializePublicKey(keypair),
                verifyingKey
            ]
        );

        this.verifyingKey = verifyingKey;
        this.palletKeypair = keypair;

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }
}
