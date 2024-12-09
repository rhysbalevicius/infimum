import { ApiPromise } from '@polkadot/api';
import { serializePublicKey } from '../serialize';
import { InfimumExtrinsic } from '../extrinsic';
import { User } from './user';
import { Keypair } from 'maci-domainobjs';

export class Participant extends User
{
    constructor(
        api: ApiPromise,
        keyringURI: string = '//Bob',
        privateKey: string = '//Bob'
    )
    {
        super(api, keyringURI, privateKey);
    }
    
    async registerAsParticipant(pollId: number)
    {
        const result = await this.sendExtrinsic(
            InfimumExtrinsic.registerAsParticipant,
            [
                pollId,
                serializePublicKey(this.palletKeypair)
            ]
        );

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }

    async interactWithPoll(
        pollId: number,
        keypair: Keypair,
        data: Array<Array<number>>
    )
    {
        const result = await this.sendExtrinsic(
            InfimumExtrinsic.interactWithPoll,
            [
                pollId,
                serializePublicKey(keypair),
                data
            ]
        );

        if (result.error) throw new Error(result.error);
        return result.depositEvents.at(0)!.data;
    }
}
