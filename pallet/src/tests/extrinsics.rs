use sp_std::vec;
use ark_bn254::{Fr};
use ark_ff::{PrimeField};
use frame_support::{
    assert_ok, 
    assert_err, 
    error
};
use crate::{
    mock::*,
    Error,
    Event
};
use crate::tests::{
    run_to_block,
    get_coordinator_data,
    get_coordinator_data_malformed,
    get_proof,
    get_participant,
    get_participants,
    get_poll_config,
    get_poll_scenario
};
use crate::poll::{
    CommitmentData,
    HashBytes,
    PublicKey,
    ProofData,
    provider::PollProvider
};
use crate::hash::{
    Poseidon,
    PoseidonHasher
};

/// Coordinators should be able to register.
#[test]
fn coordinator_registration_successful()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();

        // Successful registration
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk.clone()));
        assert_eq!(Infimum::coordinators(0).is_some(), true);
        System::assert_has_event(Event::CoordinatorRegistered { who: 0, public_key: pk, verify_key: vk.clone() }.into());
        assert_eq!(System::events().len(), 1);
    })
}

/// Coordinators should only be able to register once.
#[test]
fn coordinator_registration_duplicated()
{
    new_test_ext().execute_with(|| {
        let (pk, vk) = get_coordinator_data();

        // We should only be able to register a single coordinator once
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk.clone()));
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk), Error::<Test>::CoordinatorAlreadyRegistered);
    })
}

/// Coordinators must have a signed origin.
#[test]
fn coordinator_registration_unsigned()
{
    new_test_ext().execute_with(|| {
        let (pk, vk) = get_coordinator_data();
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::none(), pk, vk.clone()), error::BadOrigin);
    })
}

/// Coordinator verification keys must be serializable.
#[test]
fn coordinator_registration_malformed()
{
    new_test_ext().execute_with(|| {
        let (pk, vk) = get_coordinator_data_malformed();
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk), Error::<Test>::MalformedKeys);
    })
}

/// Coordinators should be able to rotate their keys.
#[test]
fn coordinator_key_rotation_successful() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk1, vk1) = get_coordinator_data();
        let (pk2, vk2) = get_coordinator_data();

        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk1, vk1.clone()), Error::<Test>::CoordinatorNotRegistered);
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk1, vk1));
        assert_ok!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk2, vk2.clone()));
        System::assert_has_event(Event::CoordinatorKeysChanged { who: 0, public_key: pk2, verify_key: vk2 }.into());
    })
}

/// Coordinators should not be able to rotate their keys during a poll.
#[test]
fn coordinator_key_rotation_during_poll() 
{
    new_test_ext().execute_with(|| {
        let (pk1, vk1) = get_coordinator_data();
        let (pk2, vk2) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk1, vk1));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));
        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk2, vk2), Error::<Test>::PollCurrentlyActive);
    })
}

/// Coordinators should be able to rotate their keys after a poll.
#[test]
fn coordinator_key_rotation_after_poll() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk1, vk1) = get_coordinator_data();
        let (pk2, vk2) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk1, vk1));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));
        
        run_to_block(signup_period + voting_period + 2);
        assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
        assert_ok!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk2, vk2));
    })
}

/// Coordinator key rotation should maintain integrity of keys.
#[test]
fn coordinator_key_rotation_malformed() 
{
    new_test_ext().execute_with(|| {
        let (pk1, vk1) = get_coordinator_data();
        let (pk2, vk2) = get_coordinator_data_malformed();

        assert_err!(Infimum::rotate_keys(RuntimeOrigin::none(), pk1, vk1.clone()), error::BadOrigin);
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk1, vk1));
        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk2, vk2), Error::<Test>::MalformedKeys);
    })
}

/// Coordinators should be able to create polls.
#[test]
fn poll_creation_successful() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));

        assert_eq!(Infimum::coordinators(0).unwrap().last_poll, Some(0));
        assert_eq!(Infimum::poll_ids(0).len(), 1);        

        System::assert_has_event(Event::PollCreated {
            coordinator: 0,
            poll_id: 0,
            starts_at: 1 + signup_period,
            ends_at: 2 + signup_period + voting_period
        }.into());
    })
}

/// Polls can only be created by registered coordinators.
#[test]
fn poll_creation_by_non_coordinator() 
{
    new_test_ext().execute_with(|| {
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options), Error::<Test>::CoordinatorNotRegistered);
    })
}

/// Polls should be able to be nullified.
#[test]
fn poll_nullify_error() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::CoordinatorNotRegistered);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::PollDoesNotExist);

        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));

        let (pk, shared_pk, message) = get_participant();
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, pk));
        
        run_to_block(1 + signup_period);
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));
        assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::PollCurrentlyActive);
    })
}

/// Coordinators can only create the allowed maximum number of polls.
#[test]
fn poll_creation_beyond_limit() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();
        let duration = signup_period + voting_period;

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options.clone()));

        run_to_block(2 + duration);
        assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options.clone()));

        run_to_block(2 + 2 * duration);
        assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options), Error::<Test>::CoordinatorPollLimitReached);
    })
}

/// A coordinator can only manage a single poll at a time.
#[test]
fn poll_creation_during_extant() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options.clone()));
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options), Error::<Test>::PollCurrentlyActive);
    })
}

/// Users should be able to register as participants.
#[test]
fn register_as_participant()
{
    new_test_ext().execute_with(|| { 
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options.clone()));
        
        let participant = get_participant();

        assert_err!(Infimum::register_as_participant(RuntimeOrigin::none(), 0, participant.0), error::BadOrigin);
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, participant.0));
        
        assert_eq!(Infimum::polls(0).is_some(), true);
        assert_eq!(Infimum::polls(0).unwrap().state.registrations.count, 1);

        System::assert_has_event(Event::ParticipantRegistered { poll_id: 0, count: 1, public_key: participant.0, block: 1 }.into());
    })
}

/// Users can only register during the registration period.
#[test]
fn register_as_participant_outside_period()
{
    new_test_ext().execute_with(|| { 
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options.clone()));
        
        let participant = get_participant();

        run_to_block(1 + signup_period);
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, participant.0), Error::<Test>::PollRegistrationHasEnded);
    })
}

/// Only the allowable number of participants should be allowed to register.
#[test]
fn participant_limit_reached()
{
    new_test_ext().execute_with(|| { 
        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, _registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, 2, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options.clone()));
        
        let participant = get_participant();
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, participant.0));
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(2), 0, participant.0));
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(3), 0, participant.0));
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(4), 0, participant.0), Error::<Test>::ParticipantRegistrationLimitReached);
    })
}

/// Users can only register in existing polls.
#[test]
fn participant_registration_no_poll()
{
    new_test_ext().execute_with(|| { 
        let participant = get_participant();
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, participant.0), Error::<Test>::PollDoesNotExist);
    })
}

/// Participants should be able to interact with polls they are registered in.
#[test]
fn participant_interaction()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));

        let (pk, shared_pk, message) = get_participant();
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, pk));
        
        run_to_block(1 + signup_period);
        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::none(), 0, shared_pk, message), error::BadOrigin);
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));

        assert_eq!(Infimum::polls(0).is_some(), true);
        assert_eq!(Infimum::polls(0).unwrap().state.interactions.count, 1);

        System::assert_has_event(Event::PollInteraction { poll_id: 0, count: 1, public_key: shared_pk, data: message }.into());
    })
}

/// Participants should only be able to interact during the voting period.
#[test]
fn participant_interaction_outside_period()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));

        let (pk, shared_pk, message) = get_participant();
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, pk));

        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message), Error::<Test>::PollRegistrationInProgress);
        run_to_block(2 + signup_period + voting_period);

        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message), Error::<Test>::PollVotingHasEnded);
    })
}

/// The maximal number of allowable interactions should be enforced.
#[test]
fn participant_interaction_limit()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (pk, vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, _interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, 1, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options));

        let (pk, shared_pk, message) = get_participant();
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, pk));

        run_to_block(1 + signup_period);
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message));
        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, shared_pk, message), Error::<Test>::ParticipantInteractionLimitReached);
    })
}

/// The registration tree should only be mergable after the signup period.
#[test]
fn merge_registration_signup_period()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        assert_err!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)), Error::<Test>::PollRegistrationInProgress);
    })
}

/// The interaction tree should only be mergable after the voting period.
#[test]
fn merge_interaction_voting_period()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(1 + signup_period);

        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));
        assert_err!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)), Error::<Test>::PollVotingInProgress);
    })
}

/// The registration tree should be able to be merged and produce the correct root and commitment value.
#[test]
fn merge_registration_state_success()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }
        
        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        assert_eq!(
            Infimum::polls(0).unwrap().state.registrations.root, 
            Some([16, 44, 202, 10, 154, 154, 255, 162, 164, 69, 231, 62, 33, 104, 15, 112, 88, 216, 113, 111, 70, 122, 146, 189, 80, 94, 79, 213, 137, 73, 176, 205])
        );
        assert_eq!(
            Infimum::polls(0).unwrap().state.commitment.process,
            (0, [42, 172, 65, 18, 133, 85, 171, 69, 236, 46, 172, 46, 31, 229, 218, 229, 163, 201, 108, 165, 174, 141, 40, 17, 128, 246, 71, 216, 46, 235, 135, 32])
        );
    })
}

/// The registration tree should be able to be merged and produce the correct root and expected number of proofs.
#[test]
fn merge_interaction_state_success()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        assert_eq!(
            Infimum::polls(0).unwrap().state.interactions.root, 
            Some([31, 254, 7, 234, 211, 75, 174, 138, 104, 42, 237, 212, 221, 158, 115, 172, 29, 63, 109, 91, 47, 88, 77, 75, 76, 5, 201, 65, 69, 119, 219, 182])
        );

        assert_eq!(Infimum::polls(0).unwrap().state.commitment.expected_process, 1);
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.expected_tally, 2);
    })
}

/// The correct public signals should be produced prior to proving.
#[test]
fn process_messages_public_signals()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        // Assert validity of public proof inputs. Expected:
        // [
        //     "4",
        //     "25",
        //     "14470532103638942535012694444804587998397771818472334093141391431267230669750",
        //     "2",
        //     "1",
        //     "0",
        //     "19920653097131876015283340295735326298336825292385683485447270132525802217807",
        //     "19301486448472428800803584456730803281486402183229406170295981014011957970720",
        //     "15716693934388801634961548270365703551068883178466706335144577413121368892481"
        // ]
        assert_eq!(Infimum::polls(0).unwrap().state.registrations.count + 1, 4);
        assert_eq!(Infimum::polls(0).unwrap().get_voting_period_end(), 25);
        assert_eq!(Infimum::polls(0).unwrap().state.interactions.root, Some([31, 254, 7, 234, 211, 75, 174, 138, 104, 42, 237, 212, 221, 158, 115, 172, 29, 63, 109, 91, 47, 88, 77, 75, 76, 5, 201, 65, 69, 119, 219, 182]));
        assert_eq!(Infimum::polls(0).unwrap().state.registrations.depth, 2);
        let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();
        let coord_pub_key_fr: vec::Vec<Fr> = vec::Vec::from([ alice_pk.x, alice_pk.y ])
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();
        let coord_pub_key_hash = hasher.hash(&coord_pub_key_fr).unwrap().into_bigint().to_string();
        assert_eq!(coord_pub_key_hash, "19920653097131876015283340295735326298336825292385683485447270132525802217807");
        assert_eq!(
            Infimum::polls(0).unwrap().state.commitment.process,
            (0, [42, 172, 65, 18, 133, 85, 171, 69, 236, 46, 172, 46, 31, 229, 218, 229, 163, 201, 108, 165, 174, 141, 40, 17, 128, 246, 71, 216, 46, 235, 135, 32])
        );
    })
}

/// A single valid message processing proof should be successfully verifiable.
#[test]
fn commit_outcome_single_batch()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (proof_data, new_proof_commitment, _tpf, _tc) = get_proof();
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(proof_data, new_proof_commitment)]);
    
        assert_ok!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None));
    
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.process, (1, new_proof_commitment));
    })
}

/// An invalid message processing proof should be rejected.
#[test]
fn commit_outcome_invalid_proof()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pf, new_proof_commitment, _tpf, _tc) = get_proof();
        let proof_data = ProofData {
            pi_a: vec::Vec::from([ 1, 90, 132, 178, 53, 72, 162, 190, 174, 234, 202, 225, 124, 15, 203, 241, 24, 166, 28, 140, 33, 166, 32, 142, 98, 204, 176, 252, 230, 140, 192, 20, 139, 39, 230, 152, 184, 129, 60, 181, 238, 20, 200, 162, 172, 120, 43, 154, 8, 140, 169, 102, 4, 146, 94, 64, 88, 220, 77, 63, 11, 46, 20, 23 ]),
            pi_b: vec::Vec::from([ 84, 30, 183, 52, 30, 16, 193, 22, 207, 118, 249, 89, 64, 160, 107, 10, 205, 244, 52, 202, 249, 228, 234, 172, 175, 156, 23, 220, 186, 234, 66, 12, 83, 150, 12, 48, 176, 8, 107, 225, 135, 4, 133, 97, 30, 180, 200, 113, 196, 162, 63, 247, 68, 183, 181, 125, 165, 1, 27, 178, 151, 4, 100, 27, 235, 67, 144, 49, 36, 228, 17, 171, 138, 32, 78, 235, 17, 96, 110, 90, 181, 238, 134, 153, 143, 241, 126, 140, 110, 231, 89, 76, 11, 204, 229, 24, 29, 255, 158, 244, 198, 108, 64, 92, 228, 96, 63, 226, 6, 159, 93, 250, 157, 181, 97, 183, 8, 78, 34, 241, 253, 29, 119, 62, 9, 19, 207, 164 ]),
            pi_c: vec::Vec::from([ 182, 96, 48, 82, 178, 199, 89, 110, 195, 62, 134, 21, 179, 247, 238, 14, 188, 181, 110, 68, 123, 104, 180, 13, 224, 126, 126, 197, 175, 15, 10, 21, 13, 52, 132, 172, 241, 121, 20, 152, 135, 139, 30, 106, 85, 16, 123, 212, 179, 189, 37, 237, 139, 45, 248, 83, 70, 14, 234, 82, 234, 229, 157, 8 ])
        };
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(proof_data, new_proof_commitment)]);
    
        assert_err!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None), Error::<Test>::MalformedProof);
    })
}

/// An valid message processing proof with an invalid commitment should be rejected.
#[test]
fn commit_outcome_invalid_commitment()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (proof_data, _c, _tpf, _tc) = get_proof();
        let new_proof_commitment: HashBytes = [1, 191, 85, 98, 25, 92, 104, 227, 66, 252, 50, 63, 42, 27, 108, 81, 67, 38, 115, 38, 128, 126, 14, 99, 203, 194, 61, 124, 1, 119, 164, 65];
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(proof_data, new_proof_commitment)]);
    
        assert_err!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None), Error::<Test>::MalformedProof);
    })
}

/// A valid message processing proof with mismatched data should be rejected.
#[test]
fn commit_outcome_mismatched_state()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        let bob_pk = PublicKey {
            x: [ 1, 65, 89, 247, 81, 66, 57, 66, 160, 59, 9, 185, 3, 52, 188, 122, 132, 221, 26, 200, 129, 243, 234, 120, 128, 23, 19, 96, 94, 154, 207, 196 ],
            y: [ 38, 38, 57, 70, 162, 8, 198, 245, 211, 231, 101, 158, 63, 226, 172, 117, 156, 26, 3, 50, 0, 241, 20, 66, 227, 150, 160, 78, 249, 106, 140, 69 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, bob_pk));

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (proof_data, new_proof_commitment, _tpf, _tc) = get_proof();
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(proof_data, new_proof_commitment)]);
    
        assert_err!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None), Error::<Test>::MalformedProof);
    })
}

/// A valid tally proof should successfully chain to a valid message processing proof.
#[test]
fn commit_outcome_full_batch()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (process_proof_data, process_commitment, tally_proof_data, tally_commitment) = get_proof();
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(process_proof_data, process_commitment), (tally_proof_data, tally_commitment)]);

        assert_ok!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None));
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.process, (1, process_commitment));
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.tally, (1, tally_commitment));
    })
}

/// A partial chain of valid proofs should be successfully verified, but not produce an outcome.
#[test]
fn commit_outcome_partial_success()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (process_proof_data, process_commitment, tally_proof_data, tally_commitment) = get_proof();
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(process_proof_data, process_commitment), (tally_proof_data, tally_commitment)]);
        let scenario = get_poll_scenario(0);

        assert_ok!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, Some(scenario.outcome)));
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.process, (1, process_commitment));
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.tally, (1, tally_commitment));
        assert_eq!(Infimum::polls(0).unwrap().state.outcome, scenario.expected);
    })
}

/// An out of order chain of proofs should be rejected.
#[test]
fn commit_outcome_permuted()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let (alice_pk, alice_vk) = get_coordinator_data();
        let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
        assert_ok!(
            Infimum::create_poll(
                RuntimeOrigin::signed(0),
                signup_period,
                voting_period,
                registration_depth,
                interaction_depth,
                process_subtree_depth,
                tally_subtree_depth,
                vote_option_tree_depth,
                vote_options
            )
        );

        run_to_block(2);

        for (origin, pk) in &get_participants()
        {
            assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
        }

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (_pk, bob_shared_pk, message_data) = get_participant();

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let (process_proof_data, process_commitment, tally_proof_data, tally_commitment) = get_proof();
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(tally_proof_data, tally_commitment), (process_proof_data, process_commitment)]);

        assert_err!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None), Error::<Test>::MalformedProof);
    })
}

macro_rules! invoke_test_poll_scenario {
    ($test_name:ident, $scenario_index:expr) =>
    {
        #[test]
        fn $test_name() {
            new_test_ext().execute_with(|| {
                System::set_block_number(1);

                let (alice_pk, alice_vk) = get_coordinator_data();
                let (signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, tally_subtree_depth, vote_option_tree_depth, vote_options) = get_poll_config();

                assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, alice_vk));
                assert_ok!(
                    Infimum::create_poll(
                        RuntimeOrigin::signed(0),
                        signup_period,
                        voting_period,
                        registration_depth,
                        interaction_depth,
                        process_subtree_depth,
                        tally_subtree_depth,
                        vote_option_tree_depth,
                        vote_options
                    )
                );

                for (origin, pk) in &get_participants()
                {
                    assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(*origin), 0, *pk));
                }

                run_to_block(1 + signup_period);
                assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

                let scenario = get_poll_scenario($scenario_index);
                for (pk, data) in &scenario.interactions
                {
                    assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, *pk, *data));
                }

                if scenario.interactions.len() > 0
                {
                    run_to_block(1 + signup_period + voting_period);
                    assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));
                }

                if scenario.proof_batches.len() > 0
                {
                    assert_ok!(Infimum::commit_outcome(RuntimeOrigin::signed(0), scenario.proof_batches, Some(scenario.outcome)));
                }

                assert_eq!(Infimum::polls(0).unwrap().state.outcome, scenario.expected);
            })
        }
    };
}

// A full chain of valid proofs should produce the correct outcome.
invoke_test_poll_scenario!(commit_outcome_success, 1);
