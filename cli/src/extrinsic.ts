import { ApiPromise } from '@polkadot/api';
import { AddressOrPair } from '@polkadot/api/types';
import { DispatchError } from '@polkadot/types/interfaces';
import {
    PollOutcome,
    ProofData,
    PublicKey,
    VerifyingKeys
} from './interface';

const INFIMUM_PALLET_NAME = 'infimum';

export enum InfimumExtrinsic
{
    registerAsCoordinator = 'registerAsCoordinator',
    registerAsParticipant = 'registerAsParticipant',
    interactWithPoll = 'interactWithPoll',
    createPoll = 'createPoll',
    mergePollState = 'mergePollState',
    commitOutcome = 'commitOutcome',
    nullifyPoll = 'nullifyPoll',
    rotateKeys = 'rotateKeys',
}

interface InfimumExtrinsicArgs
{
    [InfimumExtrinsic.registerAsCoordinator]: [
        // public_key: PublicKey
        PublicKey,
        // verify_key: VerifyingKeys
        VerifyingKeys
    ];
    [InfimumExtrinsic.registerAsParticipant]: [
        // poll_id: PollId
        number,
        // public_key: PublicKey
        PublicKey
    ];
    [InfimumExtrinsic.interactWithPoll]: [
        // poll_id: PollId
        number,
        // public_key: PublicKey
        PublicKey,
        // data: PollInteractionData
        Array<Array<number>>
    ];
    [InfimumExtrinsic.createPoll]: [
        // signup_period: BlockNumber
        number, 
        // voting_period: BlockNumber
        number, 
        // registration_depth: u8
        number, 
        // interaction_depth: u8
        number, 
        // process_subtree_depth: u8
        number, 
        // tally_subtree_depth: u8
        number, 
        // vote_option_tree_depth: u8
        number, 
        // vote_options: vec::Vec<u128>
        Array<number> 
    ];
    [InfimumExtrinsic.mergePollState]: [];
    [InfimumExtrinsic.commitOutcome]: [
        // batches: ProofBatches
        Array<[ ProofData, Array<number> ]>,
        // outcome: Option<PollOutcome>
        PollOutcome | undefined
    ];
    [InfimumExtrinsic.nullifyPoll]: [];
    [InfimumExtrinsic.rotateKeys]: [
        // public_key: PublicKey
        PublicKey,
        // verify_key: VerifyingKeys
        VerifyingKeys
    ];
}

enum InfimumDepositEvent
{
    CoordinatorRegistered = 'CoordinatorRegistered',
    CoordinatorKeysChanged = 'CoordinatorKeysChanged',
    ParticipantRegistered = 'ParticipantRegistered',
    PollCreated = 'PollCreated',
    PollInteraction = 'PollInteraction',
    PollCommitmentUpdated = 'PollCommitmentUpdated',
    PollStateMerged = 'PollStateMerged',
    PollOutcome = 'PollOutcome',
    PollNullified = 'PollNullified',
}

type InfimumExtrinsicEvents = {
    [InfimumExtrinsic.registerAsCoordinator]: [InfimumDepositEvent.CoordinatorRegistered],
    [InfimumExtrinsic.registerAsParticipant]: [InfimumDepositEvent.ParticipantRegistered],
    [InfimumExtrinsic.interactWithPoll]: [InfimumDepositEvent.PollInteraction],
    [InfimumExtrinsic.createPoll]: [InfimumDepositEvent.PollCreated],
    [InfimumExtrinsic.mergePollState]: [InfimumDepositEvent.PollStateMerged],
    [InfimumExtrinsic.commitOutcome]: [InfimumDepositEvent.PollCommitmentUpdated, InfimumDepositEvent.PollOutcome],
    [InfimumExtrinsic.nullifyPoll]: [InfimumDepositEvent.PollNullified],
    [InfimumExtrinsic.rotateKeys]: [InfimumDepositEvent.CoordinatorKeysChanged]
};

interface InfimumDepositEventData
{
    [InfimumDepositEvent.CoordinatorRegistered]: {
        who: string;
        publicKey: any;
        verifyKey: any;
    };
    [InfimumDepositEvent.CoordinatorKeysChanged]: {
        who: string;
        publicKey: any;
        verifyKey: any;
    };
    [InfimumDepositEvent.ParticipantRegistered]: {
        pollId: string;
        count: string;
        block: string;
        publicKey: string;
    };
    [InfimumDepositEvent.PollCreated]: {
        pollId: string;
        coordinator: string;
        startsAt: string;
        endsAt: string;
    };
    [InfimumDepositEvent.PollInteraction]: {
        pollId: string;
        count: string;
        publicKey: any;
        data: any;
    };
    [InfimumDepositEvent.PollCommitmentUpdated]: {
        pollId: string;
        commitment: any;
    };
    [InfimumDepositEvent.PollStateMerged]: {
        pollId: string;
        registrationRoot?: any;
        interactionRoot?: any;
    };
    [InfimumDepositEvent.PollOutcome]: {
        pollId: string;
        outcomeIndex: string;
    };
    [InfimumDepositEvent.PollNullified]: {
        pollId: string;
    };
}

export const extrinsic = (
    api: ApiPromise,
    account: AddressOrPair
) => async <T extends InfimumExtrinsic>(
    type: T,
    args: InfimumExtrinsicArgs[T]
): Promise<{
    error?: string;
    depositEvents: Array<{
        event: InfimumExtrinsicEvents[T][number],
        data: InfimumDepositEventData[InfimumExtrinsicEvents[T][number]]
    }>;
}> =>
{
    return new Promise((resolve, reject) =>
    {
        api.tx.infimum[type](...args)
            .signAndSend(account, ({
                events,
                status
            }) =>
            {
                if (status.isInBlock)
                {
                    const errors = events
                        .filter(({ event }) => api.events.system.ExtrinsicFailed.is(event))
                        .map(({ event: { data: [error] } }) => {
                            const dispatchError = error as DispatchError;
                            if (dispatchError.isModule)
                            {
                                const decoded = api.registry.findMetaError(dispatchError.asModule);
                                const { docs, name, section } = decoded;
                                return `${section}.${name}: ${docs.join(' ')}`;
                            }
                            else return dispatchError.toString();
                        });
                    
                    if (errors.length > 0) resolve({ error: errors.at(0), depositEvents: [] });
                    else
                    {
                        const depositEvents: any = events
                            .filter(({ event }) => event.section === INFIMUM_PALLET_NAME && Object.keys(InfimumDepositEvent).includes(event.method))
                            .map(({ event }) => ({ event: event.method, data: event.data.toHuman() }));

                        resolve({ depositEvents });
                    }
                }
            })
            .catch(error => reject(error.toString()))
    });
};
