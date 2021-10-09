use super::Event as AtochaEvent;
use crate::pallet::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::AccountId32;

const CONST_ORIGIN_IS_CREATOR: u64 = 1;
const CONST_ORIGIN_IS_ANSWER_1: u64 = 2;
const CONST_ORIGIN_IS_ANSWER_2: u64 = 3;
const CONST_ORIGIN_IS_ANSWER_3: u64 = 4;

#[test]
fn test_create_puzzle() {
    new_test_ext().execute_with(|| {
        System::set_block_number(5);

        handle_create_puzzle(
            CONST_ORIGIN_IS_CREATOR,
            "PUZZLE_HASH",
            "ANSWER_SIGNED",
            "NONCE",
            10,
            50,
        );

        // (PuzzleSubjectHash, PuzzleAnswerHash, PuzzleTicket, PuzzleRelationType, PuzzleStatus, u64, u64 )
        let relation_info = AtochaModule::puzzle_info(toVec("PUZZLE_HASH")).unwrap();
        // println!("==== {:?}", relation_info);
        // if let (_, _, puzzle_hash, _, _, _, _, _, _, _) = relation_info {
        //     println!("puzzle_hash = {:?}", sp_std::str::from_utf8(&puzzle_hash));
        // };

        assert_eq!(
            relation_info,
            (
                CONST_ORIGIN_IS_CREATOR,
                None,
                toVec("ANSWER_SIGNED"), //.as_bytes().to_vec(),
                toVec("NONCE"),         //.as_bytes().to_vec(),
                10,
                PUZZLE_STATUS_IS_SOLVING,
                5,
                5 + 50,
                0,
                1,
            )
        );
        //
        System::assert_last_event(
            AtochaEvent::PuzzleCreated(
                CONST_ORIGIN_IS_CREATOR,
                toVec("PUZZLE_HASH"), //.as_bytes().to_vec(),
                5,
                5 + 50,
            )
            .into(),
        );
    });
}

#[test]
fn test_answer_puzzle() {
    new_test_ext().execute_with(|| {
        System::set_block_number(5);

        // check initial status.
        let answer_list = AtochaModule::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
        assert_eq!(None, answer_list);

        // if puzzle not exists.
        assert_noop!(
            // Try to call create answer, but the puzzle not exists.
            AtochaModule::answer_puzzle(
                Origin::signed(CONST_ORIGIN_IS_ANSWER_1),
                toVec("PUZZLE_HASH"),
                toVec("ANSWER_HASH"),
                500,
            ),
            Error::<Test>::PuzzleNotExist
        );

        // Create puzzle hash on the chain.
        handle_create_puzzle(
            CONST_ORIGIN_IS_CREATOR,
            "PUZZLE_HASH",
            "ANSWER_SIGNED",
            "NONCE",
            10,
            50,
        );

        // Set the end of the answer period.
        System::set_block_number(5 + 50 + 1);

        assert_noop!(
            // Try to call create answer, but answer period has expired.
            AtochaModule::answer_puzzle(
                Origin::signed(CONST_ORIGIN_IS_ANSWER_1),
                toVec("PUZZLE_HASH"),
                toVec("ANSWER_HASH"),
                500,
            ),
            Error::<Test>::AnswerPeriodHasExpired
        );

        System::set_block_number(5);
        // add answer
        // origin: OriginFor<T>,
        // puzzle_hash: PuzzleSubjectHash,
        // answer_hash: PuzzleAnswerHash,
        // ticket: PuzzleTicket,
        assert_ok!(AtochaModule::answer_puzzle(
            Origin::signed(CONST_ORIGIN_IS_ANSWER_1),
            toVec("PUZZLE_HASH"),
            toVec("ANSWER_HASH"),
            500,
        ));

        // check answer list count.
        let answer_list = AtochaModule::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
        assert!(answer_list.is_some());
        if let Some(answer_list) = answer_list {
            // println!("answer_list = {:?}", answer_list);
            assert_eq!(1, answer_list.len());
            // check list item
            // Vec<(
            //     T::AccountId,
            //     PuzzleAnswerHash,
            //     PuzzleTicket,
            //     PuzzleAnswerStatus,
            //     CreateBn,
            // )>,
            assert_eq!(
                (CONST_ORIGIN_IS_ANSWER_1, toVec("ANSWER_HASH"), 500, 0, 5),
                answer_list[0]
            );
        }
    });
}

#[test]
fn test_handler_reveal_signed_valid() {
    new_test_ext().execute_with(|| {
        use frame_support::sp_runtime::app_crypto::{Pair, Ss58Codec, TryFrom};
        use sp_application_crypto::sr25519::Public;

        let test_signature = &hex::decode("2aeaa98e26062cf65161c68c5cb7aa31ca050cb5bdd07abc80a475d2a2eebc7b7a9c9546fbdff971b29419ddd9982bf4148c81a49df550154e1674a6b58bac84").expect("Hex invalid")[..];
        let public_id =  Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
        assert!(AtochaModule::check_signed_valid(public_id, test_signature, "This is a text message".as_bytes()));
    });
}

#[test]
fn test_reveal_puzzle() {
    new_test_ext().execute_with(|| {
        use frame_support::sp_runtime::app_crypto::{ed25519, sr25519, Pair, Ss58Codec};
        use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
        use sp_application_crypto::sr25519::Public;
        use sp_runtime::MultiSignature;
        use sp_runtime::MultiSigner;

        let account_pair = sr25519::Pair::from_string("//Alice", None).unwrap();
        let make_public = account_pair.public();
        let make_signature = account_pair.sign("This is a text message".as_bytes());
        let multi_sig = MultiSignature::from(make_signature); // OK
        let multi_signer = MultiSigner::from(make_public);
        assert!(multi_sig.verify(
            "This is a text message".as_bytes(),
            &multi_signer.into_account()
        ));

        let puzzle_hash = "PUZZLE_HASH";
        let answer_hash = "ANSWER_HASH";
        let answer_nonce = "NONCE";

        let mut puzzle_hash_vec = toVec(puzzle_hash);
        puzzle_hash_vec.append(&mut toVec(answer_nonce));
        // let linked_str = sp_std::str::from_utf8(&puzzle_hash_vec);
        // println!("linked_str = {:?}", linked_str);
        assert_eq!(
            Ok("PUZZLE_HASHNONCE"),
            sp_std::str::from_utf8(&puzzle_hash_vec)
        );

        let make_signer = account_pair.sign(&puzzle_hash_vec);
        println!("make_signer = {:?}", make_signer);

        // System::set_block_number(5);
        //
        // // Create puzzle hash on the chain.
        // handle_create_puzzle(
        //     CONST_ORIGIN_IS_CREATOR,
        //     "PUZZLE_HASH",
        //     "ANSWER_SIGNED",
        //     "NONCE",
        //     10,
        //     50,
        // );
        //
        // assert_noop!(
        //     // Try to call create answer, but answer period has expired.
        //     AtochaModule::answer_puzzle(
        //         Origin::signed(CONST_ORIGIN_IS_ANSWER_1),
        //         toVec("PUZZLE_HASH"),
        //         toVec("ANSWER_HASH"),
        //         500,
        //     ),
        //     Error::<Test>::AnswerPeriodHasExpired
        // );
    });
}

#[test]
fn test_signed_method() {
    new_test_ext().execute_with(|| {
        System::set_block_number(5);
        //
        use sp_application_crypto::sr25519;
        use sp_application_crypto::sr25519::Signature;
        use sp_runtime::MultiSignature;
        use sp_runtime::MultiSigner;
        use frame_support::sp_runtime::app_crypto::{Pair, Ss58Codec, TryFrom};
        use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
        use sp_application_crypto::sr25519::Public;

        // sp_core::sr25519::Pair(schnorrkel::Keypair).;

        // let result = AuthorityPair::verify(signature.into(), signature.into(), test_address.into());
        // assert!(result, "Result is true.")

        let msg = &b"test-message"[..];
        let (pair, _) = sr25519::Pair::generate();

        let signature = pair.sign(&msg);
        assert!(sr25519::Pair::verify(&signature, msg, &pair.public()));

        println!("msg = {:?}", &msg);
        println!("signature = {:?}", &signature);
        println!("pair.public() = {:?}", &pair.public());
        // println!("multi_signer.into_account() = {:?}", &multi_signer.into_account());


        let multi_sig = MultiSignature::from(signature); // OK
        let multi_signer = MultiSigner::from(pair.public());
        assert!(multi_sig.verify(msg, &multi_signer.into_account()));

        let multi_signer = MultiSigner::from(pair.public());
        assert!(multi_sig.verify(msg, &multi_signer.into_account()));

        //---------


        let test_signature = &hex::decode("2aeaa98e26062cf65161c68c5cb7aa31ca050cb5bdd07abc80a475d2a2eebc7b7a9c9546fbdff971b29419ddd9982bf4148c81a49df550154e1674a6b58bac84").expect("Hex invalid")[..];
        let signature = Signature::try_from(test_signature);
        let signature = signature.unwrap();
        println!(" signature = {:?}", signature);

        // let account_result =  AccountId::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty");
        // let account_id = account_result.unwrap();
        // println!(" account_id = {:?} ", account_id);

        let public_id = Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty");
        let public_id = public_id.unwrap();
        println!(" public_id = {:?} ", public_id);

        let multi_sig = MultiSignature::from(signature); // OK
        let multi_signer = MultiSigner::from(public_id);
        assert!(multi_sig.verify("This is a text message".as_bytes(), &multi_signer.into_account()));

        //
        let account_pair = sr25519::Pair::from_string("blur pioneer frown science banana impose avoid law act strategy have bronze//2//stash", None).unwrap();
        let make_public = account_pair.public();
        let make_signature = account_pair.sign("This is a text message".as_bytes());
        let multi_sig = MultiSignature::from(make_signature); // OK
        let multi_signer = MultiSigner::from(make_public);
        assert!(multi_sig.verify("This is a text message".as_bytes(), &multi_signer.into_account()));

        // println!("make_signature = {:?}", make_signature);
        // verify

    });
}

fn handle_create_puzzle(
    account_id: u64,
    puzzle_hash: &str,
    answer_signed: &str,
    answer_nonce: &str,
    ticket: PuzzleTicket,
    duration: DurationBn,
) {
    let origin = Origin::signed(account_id);
    let puzzle_hash = puzzle_hash.as_bytes().to_vec();
    let answer_signed = answer_signed.as_bytes().to_vec();
    let answer_nonce = answer_nonce.as_bytes().to_vec();
    let puzzle_version: PuzzleVersion = 1;

    // Dispatch a signed extrinsic.
    assert_ok!(AtochaModule::create_puzzle(
        origin,
        puzzle_hash.clone(),
        answer_signed.clone(),
        answer_nonce.clone(),
        ticket.clone(),
        duration.clone(),
        puzzle_version.clone()
    ));
}

fn toVec(to_str: &str) -> Vec<u8> {
    to_str.as_bytes().to_vec()
}

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			TemplateModule::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }
