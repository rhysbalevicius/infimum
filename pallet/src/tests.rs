use super::*;
use crate::{
    mock::*,
    // Error
};
use frame_support::{
    assert_ok, 
    // assert_err, 
    // error
};
use frame_support::pallet_prelude::Hooks;

use ark_bn254::{Fr};
use ark_ff::{PrimeField};
use crate::hash::{Poseidon, PoseidonHasher};
// use num_bigint::BigUint;

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

// #[test]
// fn coordinator_registration()
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         let pk = PublicKey { x:[0;4], y: [0;4] };
//         let vk = vec![0; 4];

//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::none(), pk, vk.clone()), error::BadOrigin);
//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![]), Error::<Test>::MalformedKeys);
//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![0;5]), Error::<Test>::MalformedKeys);

//         // Successful registration
//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk.clone()));
//         assert_eq!(Infimum::coordinators(0).is_some(), true);
//         System::assert_has_event(Event::CoordinatorRegistered { who: 0, public_key: pk, verify_key: VerifyKey::<Test>::truncate_from(vk.clone())}.into());
//         assert_eq!(System::events().len(), 1);

//         // We should only be able to register a single coordinator once
//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk), Error::<Test>::CoordinatorAlreadyRegistered);
//     })
// }

// #[test]
// fn coordinator_registration_duplicated()
// {
//     new_test_ext().execute_with(|| {
//         let pk = PublicKey { x:[0;4], y: [0;4] };
//         let vk = vec![0; 4];
//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk.clone()));
//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk, vk), Error::<Test>::CoordinatorAlreadyRegistered);
//     })
// }

// #[test]
// fn coordinator_registration_malformed()
// {
//     new_test_ext().execute_with(|| {
//         let pk = PublicKey { x:[0;4], y: [0;4] };
//         let vk = vec![0; 4];

//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::none(), pk, vk.clone()), error::BadOrigin);
//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![]), Error::<Test>::MalformedKeys);
//         assert_err!(Infimum::register_as_coordinator(RuntimeOrigin::signed(1), pk, vec![0;5]), Error::<Test>::MalformedKeys);
//     })
// }

// #[test]
// fn coordinator_key_rotation() 
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         let pk_1 = PublicKey { x:[0;4], y: [0;4] };
//         let pk_2 = PublicKey { x:[1;4], y: [1;4] };
//         let vk_1 = vec![0; 4];
//         let vk_2 = vec![1; 4];

//         assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk_1, vk_1.clone()), Error::<Test>::CoordinatorNotRegistered);
//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk_1, vk_1));
//         assert_ok!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk_2, vk_2.clone()));
//         System::assert_has_event(Event::CoordinatorKeysChanged { who: 0, public_key: pk_2, verify_key: VerifyKey::<Test>::truncate_from(vk_2.clone()) }.into());
//     })
// }

// #[test]
// fn coordinator_key_rotation_during_poll() 
// {
//     new_test_ext().execute_with(|| {
//         let pk_1 = PublicKey { x:[0;4], y: [0;4] };
//         let pk_2 = PublicKey { x:[1;4], y: [1;4] };
//         let vk_1 = vec![0; 4];
//         let vk_2 = vec![1; 4];

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), pk_1, vk_1));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

//         assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), pk_2, vk_2), Error::<Test>::PollCurrentlyActive);
//     })
// }

// #[test]
// fn coordinator_key_rotation_malformed() 
// {
//     new_test_ext().execute_with(|| {
//         assert_err!(Infimum::rotate_keys(RuntimeOrigin::none(), PublicKey { x:[0;4], y: [0;4] }, vec![]), error::BadOrigin);
//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![]), Error::<Test>::MalformedKeys);
//         assert_err!(Infimum::rotate_keys(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0;5]), Error::<Test>::MalformedKeys);
//     })
// }

// #[test]
// fn poll_creation() 
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
//         if let Some(c) = Infimum::coordinators(0) { assert_eq!(c.last_poll, Some(0)); }
        
//         let poll_ids = Infimum::poll_ids(0);
//         assert_eq!(poll_ids.len(), 1);

//         System::assert_has_event(Event::PollCreated { coordinator: 0, poll_id: 0, starts_at: 2, ends_at: 3 }.into());
//     })
// }

// #[test]
// fn poll_creation_malformed() 
// {
//     new_test_ext().execute_with(|| {

//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::CoordinatorNotRegistered);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
        
//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 10, vec![0,0]), Error::<Test>::PollConfigInvalid);
//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![]), Error::<Test>::PollConfigInvalid);
//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0;10]), Error::<Test>::PollConfigInvalid);
//     })
// }

// #[test]
// fn poll_creation_by_non_coordinator() 
// {
//     new_test_ext().execute_with(|| {
//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::CoordinatorNotRegistered);
//     })
// }

// #[test]
// fn poll_nullify_during_extant() 
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
//         assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::PollCurrentlyActive);
//     })
// }

// #[test]
// fn poll_nullify_missing_outcome() 
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
       
//         run_to_block(1);
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
//         run_to_block(2);
//         assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
//         run_to_block(10);
//         assert_err!(Infimum::nullify_poll(RuntimeOrigin::signed(0)), Error::<Test>::PollCurrentlyActive);
//     })
// }

// #[test]
// fn poll_creation_beyond_limit() 
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

//         run_to_block(10);
//         assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

//         run_to_block(20);
//         assert_ok!(Infimum::nullify_poll(RuntimeOrigin::signed(0)));
//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::CoordinatorPollLimitReached);
//     })
// }

// #[test]
// fn poll_creation_during_extant() 
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
//         assert_err!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]), Error::<Test>::PollCurrentlyActive);
//     })
// }

// #[test]
// fn register_as_participant()
// {
//     new_test_ext().execute_with(|| { 
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        
//         run_to_block(1);
//         assert_err!(Infimum::register_as_participant(RuntimeOrigin::none(), 0, PublicKey {x:[1;4], y:[1;4]}), error::BadOrigin);
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        
//         let poll = Infimum::polls(0);
//         assert_eq!(poll.is_some(), true);
//         if let Some(p) = poll { assert_eq!(p.state.registrations.count, 1); }

//         System::assert_has_event(Event::ParticipantRegistered { poll_id: 0, count: 1, public_key: PublicKey {x:[1;4], y:[1;4]}, block: 1 }.into());
//     })
// }

// #[test]
// fn register_as_participant_outside_period()
// {
//     new_test_ext().execute_with(|| { 
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));

//         run_to_block(2);
//         assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}), Error::<Test>::PollRegistrationHasEnded);
//     })
// }

// #[test]
// fn participant_limit_reached()
// {
//     new_test_ext().execute_with(|| { 
//         let max_registrations = 3;
//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, max_registrations, vec![0,0]));
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(2), 0, PublicKey {x:[1;4], y:[1;4]}));
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(3), 0, PublicKey {x:[1;4], y:[1;4]}));
//         assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(4), 0, PublicKey {x:[1;4], y:[1;4]}), Error::<Test>::ParticipantRegistrationLimitReached);
//     })
// }

// #[test]
// fn participant_registration_no_poll()
// {
//     new_test_ext().execute_with(|| { 
//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
//         assert_err!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 1, PublicKey {x:[1;4], y:[1;4]}), Error::<Test>::PollDoesNotExist);
//     })
// }

// #[test]
// fn participant_interaction()
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 1, vec![0,0]));
        
//         run_to_block(1);
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
//         run_to_block(2);
//         assert_err!(Infimum::interact_with_poll(RuntimeOrigin::none(), 0, PublicKey { x:[0;4], y: [0;4] }, [[0; 4]; 16]), error::BadOrigin);
//         assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
        
//         let poll = Infimum::polls(0);
//         assert_eq!(poll.is_some(), true);
//         if let Some(p) = poll { assert_eq!(p.state.interactions.count, 1); }

//         System::assert_has_event(Event::PollInteraction { poll_id: 0, count: 1, public_key: PublicKey {x:[1;4], y:[1;4]}, data: [[0; 4]; 16] }.into());
//     })
// }

// #[test]
// fn participant_interaction_outside_period()
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 4, vec![0,0]));
        
//         run_to_block(1);
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
//         assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]), Error::<Test>::PollRegistrationInProgress);

//         run_to_block(4);
//         assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]), Error::<Test>::PollVotingHasEnded);
//     })
// }

// #[test]
// fn participant_interaction_limit()
// {
//     new_test_ext().execute_with(|| {
//         System::set_block_number(1);

//         assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), PublicKey { x:[0;4], y: [0;4] }, vec![0; 4]));
//         assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), 1, 1, 4, vec![0,0]));
        
//         run_to_block(1);
//         assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}));
        
//         run_to_block(2);
//         assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
//         assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(2), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
//         assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(3), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
//         assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(4), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]));
//         assert_err!(Infimum::interact_with_poll(RuntimeOrigin::signed(5), 0, PublicKey {x:[1;4], y:[1;4]}, [[0; 4]; 16]), Error::<Test>::ParticipantInteractionLimitReached);
//     })
// }

#[test]
fn merge_registration_state()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let alice_pk = PublicKey {
            x: [ 47, 251, 84, 72, 124, 5, 250, 184, 108, 105, 233, 65, 173, 6, 148, 178, 97, 59, 169, 24, 37, 253, 56, 60, 116, 29, 135, 209, 153,  55, 138, 1 ],
            y: [ 0, 208, 187, 24, 225, 152, 32, 253, 150, 2, 56, 22, 114, 192, 50, 57, 6, 172, 129, 198, 198, 135, 128, 22, 90, 189, 179, 218, 81, 142, 132, 50 ]
        };
        let alpha_g1: vec::Vec<u8> = vec::Vec::from([ 114, 39, 107, 77, 186, 125, 136, 83, 48, 152, 17, 220, 209, 40, 186, 22, 62, 0, 158, 8, 112, 174, 25, 122, 233, 23, 160, 9, 132, 82, 162, 1, 90, 39, 209, 145, 115, 230, 245, 222, 166, 255, 202, 84, 255, 178, 127, 42, 230, 161, 74, 124, 198, 158, 120, 105, 233, 164, 49, 211, 8, 236, 203, 0 ]);
        let beta_g2: vec::Vec<u8> = vec::Vec::from([ 133, 168, 175, 111, 192, 218, 204, 16, 176, 78, 132, 171, 112, 39, 62, 225, 21, 71, 215, 229, 132, 122, 194, 220, 28, 140, 233, 102, 26, 6, 106, 20, 120, 115, 133, 57, 112, 131, 24, 22, 61, 130, 57, 24, 226, 148, 129, 130, 225, 183, 188, 32, 115, 106, 181, 181, 10, 107, 75, 183, 54, 184, 141, 19, 72, 7, 225, 149, 37, 95, 62, 208, 23, 213, 149, 201, 151, 11, 238, 203, 70, 188, 148, 119, 138, 107, 152, 251, 59, 117, 65, 216, 219, 160, 136, 19, 190, 126, 42, 13, 74, 238, 63, 88, 101, 5, 89, 214, 143, 23, 226, 34, 72, 136, 43, 120, 95, 198, 196, 97, 165, 40, 164, 216, 149, 249, 251, 16 ]);
        let gamma_g2: vec::Vec<u8> = vec::Vec::from([ 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9 ]);
        let delta_g2: vec::Vec<u8> = vec::Vec::from([ 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9 ]);
        let gamma_abc_g1: vec::Vec<vec::Vec<u8>> = vec::Vec::from([
            vec::Vec::from([ 231, 47, 28, 36, 226, 5, 251, 2, 39, 130, 87, 199, 63, 122, 238, 75, 151, 132, 50, 112, 155, 152, 42, 214, 88, 86, 76, 109, 0, 113, 96, 35, 189, 3, 117, 229, 249, 159, 130, 223, 182, 250, 103, 205, 169, 102, 192, 34, 162, 245, 1, 24, 230, 92, 41, 165, 7, 124, 43, 33, 20, 206, 51, 164 ]),
            vec::Vec::from([ 33, 137, 206, 76, 58, 248, 78, 136, 204, 105, 180, 211, 224, 52, 126, 166, 116, 234, 32, 129, 185, 145, 212, 215, 144, 149, 159, 104, 16, 62, 54, 46, 38, 196, 122, 41, 170, 91, 4, 223, 200, 53, 212, 183, 193, 80, 5, 251, 36, 114, 209, 129, 238, 6, 67, 78, 208, 163, 201, 145, 4, 85, 114, 169 ]),
            vec::Vec::from([ 202, 11, 83, 80, 108, 139, 116, 53, 121, 25, 123, 41, 138, 158, 41, 10, 232, 178, 30, 28, 133, 50, 255, 125, 75, 81, 75, 225, 158, 236, 34, 12, 220, 168, 44, 53, 128, 49, 35, 245, 63, 17, 125, 154, 211, 229, 55, 133, 234, 214, 114, 55, 160, 68, 45, 88, 34, 222, 201, 78, 130, 95, 110, 19 ]),
            vec::Vec::from([ 87, 220, 62, 228, 145, 117, 67, 194, 172, 16, 180, 36, 49, 148, 102, 1, 202, 73, 51, 58, 247, 235, 39, 53, 176, 57, 205, 158, 249, 92, 76, 29, 56, 36, 65, 108, 197, 192, 24, 50, 225, 205, 148, 211, 164, 46, 233, 33, 113, 152, 18, 166, 66, 64, 129, 21, 52, 152, 224, 163, 27, 135, 32, 18 ]),
            vec::Vec::from([ 118, 196, 164, 19, 242, 252, 230, 251, 240, 122, 210, 49, 43, 122, 254, 226, 121, 250, 237, 122, 43, 113, 106, 88, 117, 105, 91, 53, 252, 61, 6, 25, 113, 220, 221, 165, 203, 48, 231, 111, 87, 213, 246, 175, 32, 82, 15, 34, 153, 89, 219, 250, 45, 103, 31, 39, 39, 180, 182, 29, 113, 93, 130, 22 ]),
            vec::Vec::from([ 73, 138, 39, 224, 66, 133, 29, 204, 148, 207, 18, 184, 229, 102, 231, 30, 237, 87, 157, 178, 42, 84, 73, 141, 2, 215, 187, 37, 244, 89, 25, 6, 101, 189, 8, 115, 12, 85, 46, 213, 33, 48, 60, 20, 68, 39, 38, 83, 95, 218, 193, 164, 68, 1, 68, 67, 87, 225, 60, 127, 116, 29, 25, 10 ]),
            vec::Vec::from([ 228, 92, 194, 251, 14, 18, 65, 240, 151, 102, 158, 13, 238, 255, 222, 208, 76, 107, 32, 182, 202, 177, 168, 82, 14, 184, 150, 91, 88, 240, 141, 7, 132, 156, 225, 107, 164, 145, 96, 211, 222, 158, 148, 105, 236, 156, 8, 71, 102, 125, 12, 40, 40, 61, 16, 143, 44, 22, 65, 159, 182, 66, 48, 167 ]),
            vec::Vec::from([ 47, 201, 159, 91, 106, 242, 240, 86, 103, 210, 120, 16, 197, 155, 35, 209, 73, 74, 93, 31, 6, 157, 47, 173, 24, 17, 192, 23, 241, 188, 22, 7, 43, 94, 16, 21, 42, 38, 123, 173, 40, 76, 237, 228, 154, 85, 209, 245, 38, 124, 124, 52, 72, 52, 28, 149, 61, 18, 104, 167, 162, 67, 16, 160 ]),
            vec::Vec::from([ 103, 203, 92, 31, 14, 86, 151, 42, 234, 246, 74, 42, 162, 238, 68, 115, 190, 69, 152, 160, 29, 184, 59, 38, 4, 22, 193, 80, 214, 132, 3, 2, 12, 81, 38, 28, 142, 93, 189, 255, 195, 134, 100, 108, 232, 193, 180, 53, 160, 58, 61, 39, 255, 172, 82, 224, 189, 155, 233, 164, 219, 5, 145, 1 ]),
            vec::Vec::from([ 249, 131, 0, 181, 4, 54, 223, 149, 85, 169, 158, 156, 194, 194, 17, 20, 119, 129, 241, 157, 86, 130, 226, 55, 196, 255, 148, 83, 184, 115, 182, 36, 147, 180, 245, 95, 3, 235, 83, 19, 197, 59, 39, 92, 61, 110, 140, 11, 94, 132, 85, 110, 253, 217, 166, 65, 204, 65, 56, 121, 106, 208, 168, 3 ]),
        ]);
        let vk_process = VerifyKey { alpha_g1, beta_g2, gamma_g2, delta_g2, gamma_abc_g1 };
        let vk_tally = vk_process.clone(); // Unused in test
        let vk = VerifyingKeys {
            process: vk_process.clone(),
            tally: vk_tally
        };
        let signup_period = 12;
        let voting_period = 12;
        let registration_depth = 10;
        let interaction_depth = 2;
        let process_subtree_depth = 1;
        let vote_options = vec![ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24 ];
        
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, vote_options));

        run_to_block(2);

        let bob_pk = PublicKey {
            x: [ 37, 65, 89, 247, 81, 66, 57, 66, 160, 59, 9, 185, 3, 52, 188, 122, 132, 221, 26, 200, 129, 243, 234, 120, 128, 23, 19, 96, 94, 154, 207, 196 ],
            y: [ 38, 38, 57, 70, 162, 8, 198, 245, 211, 231, 101, 158, 63, 226, 172, 117, 156, 26, 3, 50, 0, 241, 20, 66, 227, 150, 160, 78, 249, 106, 140, 69 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, bob_pk));

        let charlie_pk = PublicKey {
            x: [ 18, 82, 169, 2, 59, 214, 181, 32, 190, 138, 154, 7, 110, 231, 188, 138, 50, 73, 161, 191, 159, 106, 91, 81, 190, 236, 94, 235, 5, 160, 175, 87 ],
            y: [ 19, 91, 46, 26, 178, 84, 211, 165, 56, 51, 221, 105, 57, 100, 104, 56, 6, 117, 127, 57, 120, 153, 167, 98, 208, 213, 142, 165, 133, 89, 50, 155 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(2), 0, charlie_pk));
        
        let dave_pk = PublicKey {
            x: [ 45, 176, 160, 155, 236, 20, 65, 226, 217, 228, 254, 184, 183, 52, 211, 133, 29, 211, 57, 56, 180, 30, 172, 98, 44, 39, 76, 106, 250, 58, 196, 23 ],
            y: [ 0, 104, 141, 184, 6, 19, 30, 79, 30, 248, 201, 77, 242, 71, 85, 191, 43, 194, 205, 31, 94, 14, 128, 203, 5, 205, 148, 238, 8, 169, 155, 243 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(3), 0, dave_pk));

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        assert_eq!(
            Infimum::polls(0).unwrap().state.registrations.root, 
            Some([16, 44, 202, 10, 154, 154, 255, 162, 164, 69, 231, 62, 33, 104, 15, 112, 88, 216, 113, 111, 70, 122, 146, 189, 80, 94, 79, 213, 137, 73, 176, 205])
        );
        assert_eq!(
            Infimum::polls(0).unwrap().state.commitment,
            (0, [42, 172, 65, 18, 133, 85, 171, 69, 236, 46, 172, 46, 31, 229, 218, 229, 163, 201, 108, 165, 174, 141, 40, 17, 128, 246, 71, 216, 46, 235, 135, 32])
        );
    })
}

#[test]
fn merge_interaction_state()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let alice_pk = PublicKey {
            x: [ 47, 251, 84, 72, 124, 5, 250, 184, 108, 105, 233, 65, 173, 6, 148, 178, 97, 59, 169, 24, 37, 253, 56, 60, 116, 29, 135, 209, 153,  55, 138, 1 ],
            y: [ 0, 208, 187, 24, 225, 152, 32, 253, 150, 2, 56, 22, 114, 192, 50, 57, 6, 172, 129, 198, 198, 135, 128, 22, 90, 189, 179, 218, 81, 142, 132, 50 ]
        };
        let alpha_g1: vec::Vec<u8> = vec::Vec::from([ 114, 39, 107, 77, 186, 125, 136, 83, 48, 152, 17, 220, 209, 40, 186, 22, 62, 0, 158, 8, 112, 174, 25, 122, 233, 23, 160, 9, 132, 82, 162, 1, 90, 39, 209, 145, 115, 230, 245, 222, 166, 255, 202, 84, 255, 178, 127, 42, 230, 161, 74, 124, 198, 158, 120, 105, 233, 164, 49, 211, 8, 236, 203, 0 ]);
        let beta_g2: vec::Vec<u8> = vec::Vec::from([ 133, 168, 175, 111, 192, 218, 204, 16, 176, 78, 132, 171, 112, 39, 62, 225, 21, 71, 215, 229, 132, 122, 194, 220, 28, 140, 233, 102, 26, 6, 106, 20, 120, 115, 133, 57, 112, 131, 24, 22, 61, 130, 57, 24, 226, 148, 129, 130, 225, 183, 188, 32, 115, 106, 181, 181, 10, 107, 75, 183, 54, 184, 141, 19, 72, 7, 225, 149, 37, 95, 62, 208, 23, 213, 149, 201, 151, 11, 238, 203, 70, 188, 148, 119, 138, 107, 152, 251, 59, 117, 65, 216, 219, 160, 136, 19, 190, 126, 42, 13, 74, 238, 63, 88, 101, 5, 89, 214, 143, 23, 226, 34, 72, 136, 43, 120, 95, 198, 196, 97, 165, 40, 164, 216, 149, 249, 251, 16 ]);
        let gamma_g2: vec::Vec<u8> = vec::Vec::from([ 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9 ]);
        let delta_g2: vec::Vec<u8> = vec::Vec::from([ 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9 ]);
        let gamma_abc_g1: vec::Vec<vec::Vec<u8>> = vec::Vec::from([
            vec::Vec::from([ 231, 47, 28, 36, 226, 5, 251, 2, 39, 130, 87, 199, 63, 122, 238, 75, 151, 132, 50, 112, 155, 152, 42, 214, 88, 86, 76, 109, 0, 113, 96, 35, 189, 3, 117, 229, 249, 159, 130, 223, 182, 250, 103, 205, 169, 102, 192, 34, 162, 245, 1, 24, 230, 92, 41, 165, 7, 124, 43, 33, 20, 206, 51, 164 ]),
            vec::Vec::from([ 33, 137, 206, 76, 58, 248, 78, 136, 204, 105, 180, 211, 224, 52, 126, 166, 116, 234, 32, 129, 185, 145, 212, 215, 144, 149, 159, 104, 16, 62, 54, 46, 38, 196, 122, 41, 170, 91, 4, 223, 200, 53, 212, 183, 193, 80, 5, 251, 36, 114, 209, 129, 238, 6, 67, 78, 208, 163, 201, 145, 4, 85, 114, 169 ]),
            vec::Vec::from([ 202, 11, 83, 80, 108, 139, 116, 53, 121, 25, 123, 41, 138, 158, 41, 10, 232, 178, 30, 28, 133, 50, 255, 125, 75, 81, 75, 225, 158, 236, 34, 12, 220, 168, 44, 53, 128, 49, 35, 245, 63, 17, 125, 154, 211, 229, 55, 133, 234, 214, 114, 55, 160, 68, 45, 88, 34, 222, 201, 78, 130, 95, 110, 19 ]),
            vec::Vec::from([ 87, 220, 62, 228, 145, 117, 67, 194, 172, 16, 180, 36, 49, 148, 102, 1, 202, 73, 51, 58, 247, 235, 39, 53, 176, 57, 205, 158, 249, 92, 76, 29, 56, 36, 65, 108, 197, 192, 24, 50, 225, 205, 148, 211, 164, 46, 233, 33, 113, 152, 18, 166, 66, 64, 129, 21, 52, 152, 224, 163, 27, 135, 32, 18 ]),
            vec::Vec::from([ 118, 196, 164, 19, 242, 252, 230, 251, 240, 122, 210, 49, 43, 122, 254, 226, 121, 250, 237, 122, 43, 113, 106, 88, 117, 105, 91, 53, 252, 61, 6, 25, 113, 220, 221, 165, 203, 48, 231, 111, 87, 213, 246, 175, 32, 82, 15, 34, 153, 89, 219, 250, 45, 103, 31, 39, 39, 180, 182, 29, 113, 93, 130, 22 ]),
            vec::Vec::from([ 73, 138, 39, 224, 66, 133, 29, 204, 148, 207, 18, 184, 229, 102, 231, 30, 237, 87, 157, 178, 42, 84, 73, 141, 2, 215, 187, 37, 244, 89, 25, 6, 101, 189, 8, 115, 12, 85, 46, 213, 33, 48, 60, 20, 68, 39, 38, 83, 95, 218, 193, 164, 68, 1, 68, 67, 87, 225, 60, 127, 116, 29, 25, 10 ]),
            vec::Vec::from([ 228, 92, 194, 251, 14, 18, 65, 240, 151, 102, 158, 13, 238, 255, 222, 208, 76, 107, 32, 182, 202, 177, 168, 82, 14, 184, 150, 91, 88, 240, 141, 7, 132, 156, 225, 107, 164, 145, 96, 211, 222, 158, 148, 105, 236, 156, 8, 71, 102, 125, 12, 40, 40, 61, 16, 143, 44, 22, 65, 159, 182, 66, 48, 167 ]),
            vec::Vec::from([ 47, 201, 159, 91, 106, 242, 240, 86, 103, 210, 120, 16, 197, 155, 35, 209, 73, 74, 93, 31, 6, 157, 47, 173, 24, 17, 192, 23, 241, 188, 22, 7, 43, 94, 16, 21, 42, 38, 123, 173, 40, 76, 237, 228, 154, 85, 209, 245, 38, 124, 124, 52, 72, 52, 28, 149, 61, 18, 104, 167, 162, 67, 16, 160 ]),
            vec::Vec::from([ 103, 203, 92, 31, 14, 86, 151, 42, 234, 246, 74, 42, 162, 238, 68, 115, 190, 69, 152, 160, 29, 184, 59, 38, 4, 22, 193, 80, 214, 132, 3, 2, 12, 81, 38, 28, 142, 93, 189, 255, 195, 134, 100, 108, 232, 193, 180, 53, 160, 58, 61, 39, 255, 172, 82, 224, 189, 155, 233, 164, 219, 5, 145, 1 ]),
            vec::Vec::from([ 249, 131, 0, 181, 4, 54, 223, 149, 85, 169, 158, 156, 194, 194, 17, 20, 119, 129, 241, 157, 86, 130, 226, 55, 196, 255, 148, 83, 184, 115, 182, 36, 147, 180, 245, 95, 3, 235, 83, 19, 197, 59, 39, 92, 61, 110, 140, 11, 94, 132, 85, 110, 253, 217, 166, 65, 204, 65, 56, 121, 106, 208, 168, 3 ]),
        ]);
        let vk_process = VerifyKey { alpha_g1, beta_g2, gamma_g2, delta_g2, gamma_abc_g1 };
        let vk_tally = vk_process.clone(); // Unused in test
        let vk = VerifyingKeys {
            process: vk_process.clone(),
            tally: vk_tally
        };
        let signup_period = 12;
        let voting_period = 12;
        let registration_depth = 31;
        let interaction_depth = 2;
        let process_subtree_depth = 1;
        let vote_options = vec![ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24 ];
        
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, vote_options));

        run_to_block(2);

        let bob_pk = PublicKey {
            x: [ 37, 65, 89, 247, 81, 66, 57, 66, 160, 59, 9, 185, 3, 52, 188, 122, 132, 221, 26, 200, 129, 243, 234, 120, 128, 23, 19, 96, 94, 154, 207, 196 ],
            y: [ 38, 38, 57, 70, 162, 8, 198, 245, 211, 231, 101, 158, 63, 226, 172, 117, 156, 26, 3, 50, 0, 241, 20, 66, 227, 150, 160, 78, 249, 106, 140, 69 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, bob_pk));

        let charlie_pk = PublicKey {
            x: [ 18, 82, 169, 2, 59, 214, 181, 32, 190, 138, 154, 7, 110, 231, 188, 138, 50, 73, 161, 191, 159, 106, 91, 81, 190, 236, 94, 235, 5, 160, 175, 87 ],
            y: [ 19, 91, 46, 26, 178, 84, 211, 165, 56, 51, 221, 105, 57, 100, 104, 56, 6, 117, 127, 57, 120, 153, 167, 98, 208, 213, 142, 165, 133, 89, 50, 155 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(2), 0, charlie_pk));
        
        let dave_pk = PublicKey {
            x: [ 45, 176, 160, 155, 236, 20, 65, 226, 217, 228, 254, 184, 183, 52, 211, 133, 29, 211, 57, 56, 180, 30, 172, 98, 44, 39, 76, 106, 250, 58, 196, 23 ],
            y: [ 0, 104, 141, 184, 6, 19, 30, 79, 30, 248, 201, 77, 242, 71, 85, 191, 43, 194, 205, 31, 94, 14, 128, 203, 5, 205, 148, 238, 8, 169, 155, 243 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(3), 0, dave_pk));

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let bob_shared_pk = PublicKey {
            x: [ 40, 162, 73, 223, 129, 218, 20, 106, 227, 221, 21, 198, 229, 247, 95, 63, 67, 107, 48, 80, 66, 13, 114, 203, 227, 83, 110, 211, 1, 230, 208, 15 ],
            y: [ 16, 186, 146, 190, 25, 247, 51, 27, 61, 209, 71, 23, 169, 166, 156, 229, 156, 148, 80, 67, 232, 167, 99, 179, 33, 97, 164, 231, 182, 54, 24, 193 ]
        };
        let message_data: [[u8; 32]; 10] = [
            [ 7, 67, 213, 234, 220, 97, 174, 242, 201, 152, 25, 95, 27, 13, 252, 170, 94, 174, 253, 35, 57, 94, 19, 196, 112, 180, 128, 126, 94, 23, 170, 243 ],
            [ 16, 6, 13, 207, 130, 125, 169, 104, 61, 143, 251, 235, 246, 140, 40, 104, 64, 244, 251, 219, 221, 75, 102, 219, 224, 12, 45, 222, 165, 143, 198, 218 ],
            [ 48, 14, 148, 209, 150, 143, 205, 99, 181, 243, 72, 165, 163, 218, 126, 162, 6, 70, 136, 74, 194, 113, 139, 169, 239, 129, 146, 8, 1, 233, 54, 20 ],
            [ 32, 143, 168, 111, 55, 55, 61, 175, 174, 81, 178, 220, 43, 32, 73, 181, 249, 133, 200, 38, 182, 149, 31, 180, 39, 163, 73, 7, 100, 115, 193, 114 ],
            [ 23, 138, 11, 56, 255, 95, 192, 15, 9, 86, 246, 255, 37, 44, 75, 92, 26, 160, 102, 136, 7, 110, 102, 60, 163, 6, 85, 19, 141, 192, 41, 35 ],
            [ 8, 209, 138, 22, 230, 23, 29, 238, 151, 14, 38, 138, 187, 103, 37, 161, 132, 153, 152, 0, 209, 179, 198, 172, 66, 3, 134, 30, 173, 149, 199, 121 ],
            [ 23, 84, 9, 67, 16, 37, 196, 141, 251, 221, 247, 106, 49, 213, 158, 127, 111, 191, 75, 45, 55, 163, 28, 214, 149, 84, 146, 69, 201, 106, 153, 227 ],
            [ 18, 200, 65, 136, 248, 83, 148, 255, 255, 171, 174, 130, 144, 91, 252, 229, 28, 32, 207, 195, 168, 175, 242, 97, 144, 6, 159, 92, 140, 155, 45, 98 ],
            [ 36, 7, 169, 100, 46, 245, 143, 92, 177, 43, 180, 138, 2, 181, 106, 63, 90, 190, 254, 24, 162, 226, 99, 96, 221, 92, 120, 113, 255, 247, 232, 253 ],
            [ 3, 128, 185, 64, 119, 206, 73, 138, 23, 207, 169, 168, 119, 210, 224, 86, 77, 102, 207, 34, 172, 53, 38, 23, 74, 130, 238, 215, 111, 175, 86, 3 ]
        ];

        assert_ok!(Infimum::interact_with_poll(RuntimeOrigin::signed(1), 0, bob_shared_pk, message_data));

        run_to_block(26);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        assert_eq!(
            Infimum::polls(0).unwrap().state.interactions.root, 
            Some([31, 254, 7, 234, 211, 75, 174, 138, 104, 42, 237, 212, 221, 158, 115, 172, 29, 63, 109, 91, 47, 88, 77, 75, 76, 5, 201, 65, 69, 119, 219, 182])
        );
    })
}

#[test]
fn commit_outcome_process_proof_batch()
{
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let alice_pk = PublicKey {
            x: [ 47, 251, 84, 72, 124, 5, 250, 184, 108, 105, 233, 65, 173, 6, 148, 178, 97, 59, 169, 24, 37, 253, 56, 60, 116, 29, 135, 209, 153,  55, 138, 1 ],
            y: [ 0, 208, 187, 24, 225, 152, 32, 253, 150, 2, 56, 22, 114, 192, 50, 57, 6, 172, 129, 198, 198, 135, 128, 22, 90, 189, 179, 218, 81, 142, 132, 50 ]
        };
        let alpha_g1: vec::Vec<u8> = vec::Vec::from([ 114, 39, 107, 77, 186, 125, 136, 83, 48, 152, 17, 220, 209, 40, 186, 22, 62, 0, 158, 8, 112, 174, 25, 122, 233, 23, 160, 9, 132, 82, 162, 1, 90, 39, 209, 145, 115, 230, 245, 222, 166, 255, 202, 84, 255, 178, 127, 42, 230, 161, 74, 124, 198, 158, 120, 105, 233, 164, 49, 211, 8, 236, 203, 0 ]);
        let beta_g2: vec::Vec<u8> = vec::Vec::from([ 133, 168, 175, 111, 192, 218, 204, 16, 176, 78, 132, 171, 112, 39, 62, 225, 21, 71, 215, 229, 132, 122, 194, 220, 28, 140, 233, 102, 26, 6, 106, 20, 120, 115, 133, 57, 112, 131, 24, 22, 61, 130, 57, 24, 226, 148, 129, 130, 225, 183, 188, 32, 115, 106, 181, 181, 10, 107, 75, 183, 54, 184, 141, 19, 72, 7, 225, 149, 37, 95, 62, 208, 23, 213, 149, 201, 151, 11, 238, 203, 70, 188, 148, 119, 138, 107, 152, 251, 59, 117, 65, 216, 219, 160, 136, 19, 190, 126, 42, 13, 74, 238, 63, 88, 101, 5, 89, 214, 143, 23, 226, 34, 72, 136, 43, 120, 95, 198, 196, 97, 165, 40, 164, 216, 149, 249, 251, 16 ]);
        let gamma_g2: vec::Vec<u8> = vec::Vec::from([ 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9 ]);
        let delta_g2: vec::Vec<u8> = vec::Vec::from([ 237, 246, 146, 217, 92, 189, 222, 70, 221, 218, 94, 247, 212, 34, 67, 103, 121, 68, 92, 94, 102, 0, 106, 66, 118, 30, 31, 18, 239, 222, 0, 24, 194, 18, 243, 174, 183, 133, 228, 151, 18, 231, 169, 53, 51, 73, 170, 241, 37, 93, 251, 49, 183, 191, 96, 114, 58, 72, 13, 146, 147, 147, 142, 25, 170, 125, 250, 102, 1, 204, 230, 76, 123, 211, 67, 12, 105, 231, 209, 227, 143, 64, 203, 141, 128, 113, 171, 74, 235, 109, 140, 219, 165, 94, 200, 18, 91, 151, 34, 209, 220, 218, 172, 85, 243, 142, 179, 112, 51, 49, 75, 188, 149, 51, 12, 105, 173, 153, 158, 236, 117, 240, 95, 88, 208, 137, 6, 9 ]);
        let gamma_abc_g1: vec::Vec<vec::Vec<u8>> = vec::Vec::from([
            vec::Vec::from([ 231, 47, 28, 36, 226, 5, 251, 2, 39, 130, 87, 199, 63, 122, 238, 75, 151, 132, 50, 112, 155, 152, 42, 214, 88, 86, 76, 109, 0, 113, 96, 35, 189, 3, 117, 229, 249, 159, 130, 223, 182, 250, 103, 205, 169, 102, 192, 34, 162, 245, 1, 24, 230, 92, 41, 165, 7, 124, 43, 33, 20, 206, 51, 164 ]),
            vec::Vec::from([ 33, 137, 206, 76, 58, 248, 78, 136, 204, 105, 180, 211, 224, 52, 126, 166, 116, 234, 32, 129, 185, 145, 212, 215, 144, 149, 159, 104, 16, 62, 54, 46, 38, 196, 122, 41, 170, 91, 4, 223, 200, 53, 212, 183, 193, 80, 5, 251, 36, 114, 209, 129, 238, 6, 67, 78, 208, 163, 201, 145, 4, 85, 114, 169 ]),
            vec::Vec::from([ 202, 11, 83, 80, 108, 139, 116, 53, 121, 25, 123, 41, 138, 158, 41, 10, 232, 178, 30, 28, 133, 50, 255, 125, 75, 81, 75, 225, 158, 236, 34, 12, 220, 168, 44, 53, 128, 49, 35, 245, 63, 17, 125, 154, 211, 229, 55, 133, 234, 214, 114, 55, 160, 68, 45, 88, 34, 222, 201, 78, 130, 95, 110, 19 ]),
            vec::Vec::from([ 87, 220, 62, 228, 145, 117, 67, 194, 172, 16, 180, 36, 49, 148, 102, 1, 202, 73, 51, 58, 247, 235, 39, 53, 176, 57, 205, 158, 249, 92, 76, 29, 56, 36, 65, 108, 197, 192, 24, 50, 225, 205, 148, 211, 164, 46, 233, 33, 113, 152, 18, 166, 66, 64, 129, 21, 52, 152, 224, 163, 27, 135, 32, 18 ]),
            vec::Vec::from([ 118, 196, 164, 19, 242, 252, 230, 251, 240, 122, 210, 49, 43, 122, 254, 226, 121, 250, 237, 122, 43, 113, 106, 88, 117, 105, 91, 53, 252, 61, 6, 25, 113, 220, 221, 165, 203, 48, 231, 111, 87, 213, 246, 175, 32, 82, 15, 34, 153, 89, 219, 250, 45, 103, 31, 39, 39, 180, 182, 29, 113, 93, 130, 22 ]),
            vec::Vec::from([ 73, 138, 39, 224, 66, 133, 29, 204, 148, 207, 18, 184, 229, 102, 231, 30, 237, 87, 157, 178, 42, 84, 73, 141, 2, 215, 187, 37, 244, 89, 25, 6, 101, 189, 8, 115, 12, 85, 46, 213, 33, 48, 60, 20, 68, 39, 38, 83, 95, 218, 193, 164, 68, 1, 68, 67, 87, 225, 60, 127, 116, 29, 25, 10 ]),
            vec::Vec::from([ 228, 92, 194, 251, 14, 18, 65, 240, 151, 102, 158, 13, 238, 255, 222, 208, 76, 107, 32, 182, 202, 177, 168, 82, 14, 184, 150, 91, 88, 240, 141, 7, 132, 156, 225, 107, 164, 145, 96, 211, 222, 158, 148, 105, 236, 156, 8, 71, 102, 125, 12, 40, 40, 61, 16, 143, 44, 22, 65, 159, 182, 66, 48, 167 ]),
            vec::Vec::from([ 47, 201, 159, 91, 106, 242, 240, 86, 103, 210, 120, 16, 197, 155, 35, 209, 73, 74, 93, 31, 6, 157, 47, 173, 24, 17, 192, 23, 241, 188, 22, 7, 43, 94, 16, 21, 42, 38, 123, 173, 40, 76, 237, 228, 154, 85, 209, 245, 38, 124, 124, 52, 72, 52, 28, 149, 61, 18, 104, 167, 162, 67, 16, 160 ]),
            vec::Vec::from([ 103, 203, 92, 31, 14, 86, 151, 42, 234, 246, 74, 42, 162, 238, 68, 115, 190, 69, 152, 160, 29, 184, 59, 38, 4, 22, 193, 80, 214, 132, 3, 2, 12, 81, 38, 28, 142, 93, 189, 255, 195, 134, 100, 108, 232, 193, 180, 53, 160, 58, 61, 39, 255, 172, 82, 224, 189, 155, 233, 164, 219, 5, 145, 1 ]),
            vec::Vec::from([ 249, 131, 0, 181, 4, 54, 223, 149, 85, 169, 158, 156, 194, 194, 17, 20, 119, 129, 241, 157, 86, 130, 226, 55, 196, 255, 148, 83, 184, 115, 182, 36, 147, 180, 245, 95, 3, 235, 83, 19, 197, 59, 39, 92, 61, 110, 140, 11, 94, 132, 85, 110, 253, 217, 166, 65, 204, 65, 56, 121, 106, 208, 168, 3 ]),
        ]);
        let vk_process = VerifyKey { alpha_g1, beta_g2, gamma_g2, delta_g2, gamma_abc_g1 };
        let vk_tally = vk_process.clone(); // Unused in test
        let vk = VerifyingKeys {
            process: vk_process.clone(),
            tally: vk_tally
        };

        let signup_period = 12;
        let voting_period = 12;
        let registration_depth = 31;
        let interaction_depth = 2;
        let process_subtree_depth = 1;
        let vote_options = vec![ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24 ];
        
        assert_ok!(Infimum::register_as_coordinator(RuntimeOrigin::signed(0), alice_pk, vk));
        assert_ok!(Infimum::create_poll(RuntimeOrigin::signed(0), signup_period, voting_period, registration_depth, interaction_depth, process_subtree_depth, vote_options));

        run_to_block(2);

        let bob_pk = PublicKey {
            x: [ 37, 65, 89, 247, 81, 66, 57, 66, 160, 59, 9, 185, 3, 52, 188, 122, 132, 221, 26, 200, 129, 243, 234, 120, 128, 23, 19, 96, 94, 154, 207, 196 ],
            y: [ 38, 38, 57, 70, 162, 8, 198, 245, 211, 231, 101, 158, 63, 226, 172, 117, 156, 26, 3, 50, 0, 241, 20, 66, 227, 150, 160, 78, 249, 106, 140, 69 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(1), 0, bob_pk));

        let charlie_pk = PublicKey {
            x: [ 18, 82, 169, 2, 59, 214, 181, 32, 190, 138, 154, 7, 110, 231, 188, 138, 50, 73, 161, 191, 159, 106, 91, 81, 190, 236, 94, 235, 5, 160, 175, 87 ],
            y: [ 19, 91, 46, 26, 178, 84, 211, 165, 56, 51, 221, 105, 57, 100, 104, 56, 6, 117, 127, 57, 120, 153, 167, 98, 208, 213, 142, 165, 133, 89, 50, 155 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(2), 0, charlie_pk));
        
        let dave_pk = PublicKey {
            x: [ 45, 176, 160, 155, 236, 20, 65, 226, 217, 228, 254, 184, 183, 52, 211, 133, 29, 211, 57, 56, 180, 30, 172, 98, 44, 39, 76, 106, 250, 58, 196, 23 ],
            y: [ 0, 104, 141, 184, 6, 19, 30, 79, 30, 248, 201, 77, 242, 71, 85, 191, 43, 194, 205, 31, 94, 14, 128, 203, 5, 205, 148, 238, 8, 169, 155, 243 ]
        };
        assert_ok!(Infimum::register_as_participant(RuntimeOrigin::signed(3), 0, dave_pk));

        run_to_block(14);
        assert_ok!(Infimum::merge_poll_state(RuntimeOrigin::signed(0)));

        let bob_shared_pk = PublicKey {
            x: [ 40, 162, 73, 223, 129, 218, 20, 106, 227, 221, 21, 198, 229, 247, 95, 63, 67, 107, 48, 80, 66, 13, 114, 203, 227, 83, 110, 211, 1, 230, 208, 15 ],
            y: [ 16, 186, 146, 190, 25, 247, 51, 27, 61, 209, 71, 23, 169, 166, 156, 229, 156, 148, 80, 67, 232, 167, 99, 179, 33, 97, 164, 231, 182, 54, 24, 193 ]
        };
        let message_data: [[u8; 32]; 10] = [
            [ 7, 67, 213, 234, 220, 97, 174, 242, 201, 152, 25, 95, 27, 13, 252, 170, 94, 174, 253, 35, 57, 94, 19, 196, 112, 180, 128, 126, 94, 23, 170, 243 ],
            [ 16, 6, 13, 207, 130, 125, 169, 104, 61, 143, 251, 235, 246, 140, 40, 104, 64, 244, 251, 219, 221, 75, 102, 219, 224, 12, 45, 222, 165, 143, 198, 218 ],
            [ 48, 14, 148, 209, 150, 143, 205, 99, 181, 243, 72, 165, 163, 218, 126, 162, 6, 70, 136, 74, 194, 113, 139, 169, 239, 129, 146, 8, 1, 233, 54, 20 ],
            [ 32, 143, 168, 111, 55, 55, 61, 175, 174, 81, 178, 220, 43, 32, 73, 181, 249, 133, 200, 38, 182, 149, 31, 180, 39, 163, 73, 7, 100, 115, 193, 114 ],
            [ 23, 138, 11, 56, 255, 95, 192, 15, 9, 86, 246, 255, 37, 44, 75, 92, 26, 160, 102, 136, 7, 110, 102, 60, 163, 6, 85, 19, 141, 192, 41, 35 ],
            [ 8, 209, 138, 22, 230, 23, 29, 238, 151, 14, 38, 138, 187, 103, 37, 161, 132, 153, 152, 0, 209, 179, 198, 172, 66, 3, 134, 30, 173, 149, 199, 121 ],
            [ 23, 84, 9, 67, 16, 37, 196, 141, 251, 221, 247, 106, 49, 213, 158, 127, 111, 191, 75, 45, 55, 163, 28, 214, 149, 84, 146, 69, 201, 106, 153, 227 ],
            [ 18, 200, 65, 136, 248, 83, 148, 255, 255, 171, 174, 130, 144, 91, 252, 229, 28, 32, 207, 195, 168, 175, 242, 97, 144, 6, 159, 92, 140, 155, 45, 98 ],
            [ 36, 7, 169, 100, 46, 245, 143, 92, 177, 43, 180, 138, 2, 181, 106, 63, 90, 190, 254, 24, 162, 226, 99, 96, 221, 92, 120, 113, 255, 247, 232, 253 ],
            [ 3, 128, 185, 64, 119, 206, 73, 138, 23, 207, 169, 168, 119, 210, 224, 86, 77, 102, 207, 34, 172, 53, 38, 23, 74, 130, 238, 215, 111, 175, 86, 3 ]
        ];

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
            Infimum::polls(0).unwrap().state.commitment,
            (0, [42, 172, 65, 18, 133, 85, 171, 69, 236, 46, 172, 46, 31, 229, 218, 229, 163, 201, 108, 165, 174, 141, 40, 17, 128, 246, 71, 216, 46, 235, 135, 32])
        );

        let proof_data = ProofData {
            pi_a: vec::Vec::from([ 105, 90, 132, 178, 53, 72, 162, 190, 174, 234, 202, 225, 124, 15, 203, 241, 24, 166, 28, 140, 33, 166, 32, 142, 98, 204, 176, 252, 230, 140, 192, 20, 139, 39, 230, 152, 184, 129, 60, 181, 238, 20, 200, 162, 172, 120, 43, 154, 8, 140, 169, 102, 4, 146, 94, 64, 88, 220, 77, 63, 11, 46, 20, 23 ]),
            pi_b: vec::Vec::from([ 84, 30, 183, 52, 30, 16, 193, 22, 207, 118, 249, 89, 64, 160, 107, 10, 205, 244, 52, 202, 249, 228, 234, 172, 175, 156, 23, 220, 186, 234, 66, 12, 83, 150, 12, 48, 176, 8, 107, 225, 135, 4, 133, 97, 30, 180, 200, 113, 196, 162, 63, 247, 68, 183, 181, 125, 165, 1, 27, 178, 151, 4, 100, 27, 235, 67, 144, 49, 36, 228, 17, 171, 138, 32, 78, 235, 17, 96, 110, 90, 181, 238, 134, 153, 143, 241, 126, 140, 110, 231, 89, 76, 11, 204, 229, 24, 29, 255, 158, 244, 198, 108, 64, 92, 228, 96, 63, 226, 6, 159, 93, 250, 157, 181, 97, 183, 8, 78, 34, 241, 253, 29, 119, 62, 9, 19, 207, 164 ]),
            pi_c: vec::Vec::from([ 182, 96, 48, 82, 178, 199, 89, 110, 195, 62, 134, 21, 179, 247, 238, 14, 188, 181, 110, 68, 123, 104, 180, 13, 224, 126, 126, 197, 175, 15, 10, 21, 13, 52, 132, 172, 241, 121, 20, 152, 135, 139, 30, 106, 85, 16, 123, 212, 179, 189, 37, 237, 139, 45, 248, 83, 70, 14, 234, 82, 234, 229, 157, 8 ])
        };
        let new_proof_commitment: HashBytes = [34, 191, 85, 98, 25, 92, 104, 227, 66, 252, 50, 63, 42, 27, 108, 81, 67, 38, 115, 38, 128, 126, 14, 99, 203, 194, 61, 124, 1, 119, 164, 65];
        let proof_batches: vec::Vec<(ProofData, CommitmentData)> = vec::Vec::from([(proof_data, new_proof_commitment)]);
        assert_ok!(Infimum::commit_outcome(RuntimeOrigin::signed(0), proof_batches, None));
    
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.0, 1);
        assert_eq!(Infimum::polls(0).unwrap().state.commitment.1, new_proof_commitment);
    })
}
