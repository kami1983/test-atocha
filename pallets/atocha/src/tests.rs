use crate::{Error, mock::*};
use crate::pallet::{*};
use super::Event as AtochaEvent;
use frame_support::{assert_ok, assert_noop};

const CONST_ORIGIN_IS_CREATOR: u64 = 1;

#[test]
fn it_works_for_default_value() {

	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		let origin = Origin::signed(CONST_ORIGIN_IS_CREATOR);
		let puzzle_owner = CONST_ORIGIN_IS_CREATOR ; // The same as origin, so it is a create puzzle operation.
		let puzzle_hash = "PUZZLE_HASH".as_bytes().to_vec();
		let answer_hash = "ANSWER_HASH".as_bytes().to_vec();
		let ticket: PuzzleTicket = 10;
		let relation_type: PuzzleRelationType = 1;
		let duration: u64 = 50;

		// Dispatch a signed extrinsic.
		assert_ok!(AtochaModule::create_puzzle(
			origin,
			puzzle_owner.clone(),
			puzzle_hash.clone(),
			answer_hash.clone(),
			ticket.clone(),
			relation_type.clone(),
			duration.clone()
		));

		// (PuzzleSubjectHash, PuzzleAnswerHash, PuzzleTicket, PuzzleRelationType, PuzzleStatus, u64, u64 )
		let relation_info_vec = AtochaModule::puzzle_relation(CONST_ORIGIN_IS_CREATOR).unwrap();
		println!("==== {:?}", relation_info_vec);
		assert_eq!(relation_info_vec[0], (puzzle_hash.clone(), answer_hash.clone(), ticket, relation_type, PUZZLE_STATUS_IS_SOLVING, 5, 5 + duration));

		System::assert_last_event(AtochaEvent::PuzzleCreated(CONST_ORIGIN_IS_CREATOR, puzzle_hash, 5, 5 + duration).into());
	});
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
