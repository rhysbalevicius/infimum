import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keypair, PCommand } from 'maci-domainobjs';
import { MaciState, Poll } from 'maci-core';
import {
    serializeMessage,
    provePollResults,
    waitForBlock,
    Coordinator,
    Participant
} from '../src';

describe("End to end tests", function test()
{
    jest.setTimeout(5 * 60 * 1000);

    let api: ApiPromise;
    let coordinator: Coordinator;
    const STATE_TREE_DEPTH = 10;
    const SIGNUP_PERIOD = 4;
    const VOTING_PERIOD = 4;
    const REGISTRATION_DEPTH = 2;
    const INTERACTION_DEPTH = 2;
    const PROCESS_SUBTREE_DEPTH = 1;
    const TALLY_SUBTREE_DEPTH = 1;
    const VOTE_OPTION_DEPTH = 2;
    const VOTE_OPTIONS = Array(25).fill(null).map((_, i) => i);
    const treeDepths = {
        intStateTreeDepth: TALLY_SUBTREE_DEPTH,
        messageTreeDepth: INTERACTION_DEPTH,
        messageTreeSubDepth: TALLY_SUBTREE_DEPTH,
        voteOptionTreeDepth: VOTE_OPTION_DEPTH
    };

    beforeAll(async () =>
    {
        const wsProvider = new WsProvider('ws://127.0.0.1:9944');
        api = await ApiPromise.create({ provider: wsProvider });

        coordinator = new Coordinator(api);
        const isUnregistered = (await api.query.infimum.coordinators(coordinator.address())).toHuman() === null;
        if (isUnregistered) await coordinator.registerAsCoordinator();
    });

    afterAll(async () =>
    {
        await api.disconnect();
    });

    describe("Only user 1 votes", () =>
    {
        it("Should produce the correct outcome index", async () =>
        {
            const OUTCOME_INDEX = '5';

            const createPoll = await coordinator.createPoll(
                SIGNUP_PERIOD,
                VOTING_PERIOD,
                REGISTRATION_DEPTH,
                INTERACTION_DEPTH,
                PROCESS_SUBTREE_DEPTH,
                TALLY_SUBTREE_DEPTH,
                VOTE_OPTION_DEPTH,
                VOTE_OPTIONS
            );

            const votingStartsAt = parseInt(createPoll.startsAt);
            const pollEndsAt = parseInt(createPoll.endsAt);

            const participant = new Participant(api);
            const registerAsParticipant = await participant.registerAsParticipant(parseInt(createPoll.pollId));
    
            await waitForBlock(api, votingStartsAt + 1);

            // Replay current actions internally.
            const maciState = new MaciState(STATE_TREE_DEPTH);
            const stateIndex = maciState.signUp(
                participant.keypair().pubKey,
                BigInt(1),
                BigInt(registerAsParticipant.block)
            );
            const pollId = maciState.deployPoll(
                BigInt(votingStartsAt + VOTING_PERIOD),
                treeDepths,
                5 ** TALLY_SUBTREE_DEPTH,
                coordinator.keypair()
            );
            const poll = maciState.polls.get(pollId) as Poll;
            poll.updatePoll(BigInt(maciState.stateLeaves.length));

            // Participant will vote for fifth option.
            const command = new PCommand(
                BigInt(stateIndex),
                participant.keypair().pubKey,
                BigInt(OUTCOME_INDEX),
                BigInt(1),
                BigInt(1),
                pollId
            );
            const signature = command.sign(participant.keypair().privKey); 
            const ecdhKeypair = new Keypair(); 
            const sharedKey = Keypair.genEcdhSharedKey(ecdhKeypair.privKey, coordinator.keypair().pubKey);
            const message = command.encrypt(signature, sharedKey);
            poll.publishMessage(message, ecdhKeypair.pubKey);

            // Interact with the poll.
            const interactionWithPoll = await participant.interactWithPoll(
                parseInt(createPoll.pollId),
                ecdhKeypair,
                serializeMessage(message)
            );
            console.log(interactionWithPoll);

            // Merge registration state tree.
            const mergeRegistrations = await coordinator.mergePollState();
            console.log(mergeRegistrations)

            await waitForBlock(api, pollEndsAt + 1);

            // Merge interaction state tree.
            const mergeInteractions = await coordinator.mergePollState();
            console.log(mergeInteractions);

            // Compute the proof inputs and outcome. 
            const { batches, outcome } = await provePollResults(poll, '../__tests__/data');

            const result = await coordinator.commitOutcome(batches, outcome);
            console.dir(result, { depth: null });

            const data = result.find(e => e.event === 'PollOutcome')?.data as unknown as { outcomeIndex: string; };
            expect(data?.outcomeIndex).toBe(OUTCOME_INDEX);
        });
    });
});
