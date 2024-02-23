use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_ok, assert_err, error};
use frame_support::pallet_prelude::Hooks;

pub fn run_to_block(n: u64)
{
    while System::block_number() < n
    {
        if System::block_number() > 1 
        {
            Infimum::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Infimum::on_initialize(System::block_number());
    }
}

#[test]
fn coordinator_registration()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let pk = PublicKey { x:[0;4], y: [0;4] };
        let vk = vec![0; 4];

        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::none(), pk, vk.clone()), error::BadOrigin);
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![]), Error::<Test>::MalformedKeys);
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![0;5]), Error::<Test>::MalformedKeys);

        // Successful registration
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk.clone()));
        assert_eq!(Infimum::coordinators(0).is_some(), true);
        System::assert_has_event(Event::CoordinatorRegistered { who: 0, public_key: pk, verify_key: VerifyKey::<Test>::truncate_from(vk.clone())}.into());
        assert_eq!(System::events().len(), 1);

        // We should only be able to register a single coordinator once
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk), Error::<Test>::CoordinatorAlreadyRegistered);
    })
}

#[test]
fn coordinator_registration_duplicated()
{
    new_test_ext().execute_with(|| {
        let pk = PublicKey { x:[0;4], y: [0;4] };
        let vk = vec![0; 4];
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk.clone()));
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk), Error::<Test>::CoordinatorAlreadyRegistered);
    })
}

#[test]
fn coordinator_registration_malformed()
{
    new_test_ext().execute_with(|| {
        let pk = PublicKey { x:[0;4], y: [0;4] };
        let vk = vec![0; 4];

        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::none(), pk, vk.clone()), error::BadOrigin);
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![]), Error::<Test>::MalformedKeys);
        assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![0;5]), Error::<Test>::MalformedKeys);
    })
}

#[test]
fn coordinator_key_rotation() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let pk_1 = PublicKey { x:[0;4], y: [0;4] };
        let pk_2 = PublicKey { x:[1;4], y: [1;4] };
        let vk_1 = vec![0; 4];
        let vk_2 = vec![1; 4];

        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk_1, vk_1.clone()), Error::<Test>::CoordinatorNotRegistered);
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk_1, vk_1));
        assert_ok!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk_2, vk_2.clone()));
        System::assert_has_event(Event::CoordinatorKeysChanged { who: 0, public_key: pk_2, verify_key: VerifyKey::<Test>::truncate_from(vk_2.clone()) }.into());
    })
}

#[test]
fn coordinator_key_rotation_during_poll() 
{
    new_test_ext().execute_with(|| {
        let pk_1 = PublicKey { x:[0;4], y: [0;4] };
        let pk_2 = PublicKey { x:[1;4], y: [1;4] };
        let vk_1 = vec![0; 4];
        let vk_2 = vec![1; 4];

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk_1, vk_1));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk_2, vk_2), Error::<Test>::PollCurrentlyActive);
    })
}

#[test]
fn coordinator_key_rotation_malformed() 
{
    new_test_ext().execute_with(|| {
        assert_err!(Infimum::rotate_keys(RuntimeOrigin::none(), PublicKey { x:[0;4], y: [0;4] }, vec![]), error::BadOrigin);
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![]), Error::<Test>::MalformedKeys);
        assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0;5]), Error::<Test>::MalformedKeys);
    })
}

#[test]
fn poll_creation() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        if let Some(c) = Infimum::coordinators(0) { assert_eq!(c.last_poll, Some(0)); }
        
        let poll_ids = Infimum::poll_ids(0);
        assert_eq!(poll_ids.len(), 1);

        System::assert_has_event(Event::PollCreated { coordinator: 0, poll_id: 0, starts_at: 2, ends_at: 3 }.into());
    })
}

#[test]
fn poll_creation_malformed() 
{
    new_test_ext().execute_with(|| {

        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::CoordinatorNotRegistered);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 10, vec![0,0]), Error::<Test>::PollConfigInvalid);
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![]), Error::<Test>::PollConfigInvalid);
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0;10]), Error::<Test>::PollConfigInvalid);
    })
}

#[test]
fn poll_creation_by_non_coordinator() 
{
    new_test_ext().execute_with(|| {
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::CoordinatorNotRegistered);
    })
}

#[test]
fn poll_nullify_during_extant() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::PollCurrentlyActive);
    })
}

#[test]
fn poll_nullify_missing_outcome() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
       
        run_to_block(1);
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        run_to_block(2);
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        run_to_block(10);
        assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::PollCurrentlyActive);
    })
}

#[test]
fn poll_creation_beyond_limit() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

        run_to_block(10);
        assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

        run_to_block(20);
        assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::CoordinatorPollLimitReached);
    })
}

#[test]
fn poll_creation_during_extant() 
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::PollCurrentlyActive);
    })
}

#[test]
fn register_as_participant()
{
    new_test_ext().execute_with(|| { 
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        
        run_to_block(1);
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::none(), 0, PublicKey {x:[1;4], y:[1;4]}), error::BadOrigin);
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        
        let poll = Infimum::polls(0);
        assert_eq!(poll.is_some(), true);
        if let Some(p) = poll { assert_eq!(p.state.registrations.count, 1); }

        System::assert_has_event(Event::ParticipantRegistered { poll_id: 0, count: 1, public_key: PublicKey {x:[1;4], y:[1;4]}, block: 1 }.into());
    })
}

#[test]
fn register_as_participant_outside_period()
{
    new_test_ext().execute_with(|| { 
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

        run_to_block(2);
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}), Error::<Test>::PollRegistrationHasEnded);
    })
}

#[test]
fn participant_limit_reached()
{
    new_test_ext().execute_with(|| { 
        let max_registrations = 3;
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, max_registrations, vec![0,0]));
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(2), 0, PublicKey {x:[1;4], y:[1;4]}));
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(3), 0, PublicKey {x:[1;4], y:[1;4]}));
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(4), 0, PublicKey {x:[1;4], y:[1;4]}), Error::<Test>::ParticipantRegistrationLimitReached);
    })
}

#[test]
fn participant_registration_no_poll()
{
    new_test_ext().execute_with(|| { 
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 1, PublicKey {x:[1;4], y:[1;4]}), Error::<Test>::PollDoesNotExist);
    })
}

#[test]
fn participant_interaction()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        
        run_to_block(1);
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        run_to_block(2);
        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::none(), 0, PublicKey { x:[0;4], y: [0;4] }, [[0; 4]; 16]), error::BadOrigin);
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        
        let poll = Infimum::polls(0);
        assert_eq!(poll.is_some(), true);
        if let Some(p) = poll { assert_eq!(p.state.interactions.count, 1); }

        System::assert_has_event(Event::PollInteraction { poll_id: 0, count: 1, public_key: PublicKey {x:[1;4], y:[1;4]}, data: [[0; 4]; 16] }.into());
    })
}

#[test]
fn participant_interaction_outside_period()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 4, vec![0,0]));
        
        run_to_block(1);
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]), Error::<Test>::PollRegistrationInProgress);

        run_to_block(4);
        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]), Error::<Test>::PollVotingHasEnded);
    })
}

#[test]
fn participant_interaction_limit()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 4, vec![0,0]));
        
        run_to_block(1);
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        
        run_to_block(2);
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(2), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(3), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(4), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(5), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]), Error::<Test>::ParticipantInteractionLimitReached);
    })
}

#[test]
fn merge_poll_state()
{
    new_test_ext().execute_with(|| { /* TODO (M2) */ })
}

#[test]
fn commit_outcome()
{
    new_test_ext().execute_with(|| { /* TODO (M2) */ })
}
