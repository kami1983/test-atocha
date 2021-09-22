#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


#[frame_support::pallet]
pub mod pallet {

	pub type PuzzleRelationType = u8;
	pub type PuzzleStatus = u8;
	pub type PuzzleSubjectHash = Vec<u8>;
	pub type PuzzleAnswerHash = Vec<u8>;
	pub type PuzzleTicket = u64;

	// 1=solving, 2=up to time, 3=solve
	pub const PUZZLE_STATUS_IS_SOLVING: PuzzleStatus = 1;
	pub const PUZZLE_STATUS_IS_UP_TO_TIME: PuzzleStatus = 2;
	pub const PUZZLE_STATUS_IS_SOLVED: PuzzleStatus = 3;


	// 引入需要的包
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
	};
	// 比较粗暴的引用 frame_system 所有宏函数，和系统类型信息
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn my_puzzle)]
	pub type Puzzle<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		( T::AccountId, Vec<u8> )
	>;

	#[pallet::storage]
	#[pallet::getter(fn puzzle_relation)]
	pub type PuzzleRelation<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		// puzzle_hash, answer_hash, ticket (Balance type), relation_type (1=Creater, 2=Answer), status (1=solving, 2=up to time, 3=solve), create_bn, expired_bn
		Vec<(PuzzleSubjectHash, PuzzleAnswerHash, PuzzleTicket, PuzzleRelationType, PuzzleStatus, u64, u64 )>,
	>;

	#[pallet::event]
	// Make a metadata, used by WebUI
	#[pallet::metadata(T::AccountId = "AccountId")]
	// Make a help methods, used by the caller
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		// creator id, puzzle_hash, create block number , duration block number,
		PuzzleCreated(T::AccountId, PuzzleSubjectHash, u64, u64),
		PuzzleRevoked(T::AccountId, Vec<u8>),
	}


	#[pallet::error]
	pub enum Error<T> {
		// 定义一个错误信息，存证已经存在
		ProofAlreadyExist,
		ClaimNotExist,
		NotClaimOwner,
		PuzzleAlreadyExist,
		PuzzleNotExist,
		NotPuzzleOwner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
		where u64: From<<T as frame_system::Config>::BlockNumber>
	{

		// {
		// "puzzle_hash": "QmYiqDpdbkekTsz1dFsgd5jpVcGqEmN6nmGuJ1tdCmJApQ",
		// "answer_hash": "QmZvMUwrciwAgCrEpesLiiWz41TaMTsqzZF1Z74sLj6pFU",
		// "relation_type": 1,
		// "account_id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
		// }

		// (PuzzleSubjectHash,
		// PuzzleAnswerHash,
		// PuzzleTicket,
		// PuzzleRelationType,
		// PuzzleStatus,
		// T::BlockNumber,
		// T::BlockNumber )
		#[pallet::weight(1234)]
		pub fn create_puzzle(
			origin: OriginFor<T>,
			puzzle_owner: T::AccountId,
			puzzle_hash: PuzzleSubjectHash,
			answer_hash: PuzzleAnswerHash,
			ticket: PuzzleTicket,
			// relation type to identify the puzzle relation who create puzzle who answer puzzle.
			relation_type: PuzzleRelationType,
			duration: u64,
		) -> DispatchResultWithPostInfo { // 返回值是一个Result类型的别名它会包含一些weight的信息，这是通过use引入进来的
			// check signer
			let who = ensure_signed(origin)?;

			// check current origin is creator or answer
			let mut relation_type = 2;
			if who == puzzle_owner {
				relation_type = 1;
			}

			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let dration_block_number =  duration.checked_add(current_block_number.into()).unwrap();

			type PuzzleRealtionContent = Vec<(PuzzleSubjectHash, PuzzleAnswerHash, PuzzleTicket, PuzzleRelationType, PuzzleStatus, u64, u64)>;

			// check owner has exists
 			if <PuzzleRelation<T>>::contains_key(&puzzle_owner) {
				// check puzzle_hash has exists
				let mut puzzle_relation_vec: PuzzleRealtionContent = <PuzzleRelation<T>>::get(&puzzle_owner).unwrap();
				let puzzle_exists = puzzle_relation_vec.iter().any(|(old_puzzle_hash, _, _, _, _, _, _,)| { old_puzzle_hash == &puzzle_hash });
				// tip exists and break call.
				ensure!(!puzzle_exists, Error::<T>::PuzzleAlreadyExist);
				// add puzzle Vec<(PuzzleSubjectHash, PuzzleAnswerHash, PuzzleTicket, PuzzleRelationType, PuzzleStatus, T::BlockNumber, T::BlockNumber )>
				puzzle_relation_vec.push((puzzle_hash.clone(), answer_hash, ticket, relation_type, PUZZLE_STATUS_IS_SOLVING, current_block_number.into(), dration_block_number ));
				<PuzzleRelation<T>>::insert(&puzzle_owner, puzzle_relation_vec);
			} else {
				// create puzzle
				let mut puzzle_relation_vec: PuzzleRealtionContent = Vec::new();
				puzzle_relation_vec.push((puzzle_hash.clone(), answer_hash, ticket, relation_type, PUZZLE_STATUS_IS_SOLVING, current_block_number.into(), dration_block_number ));
				<PuzzleRelation<T>>::insert(&puzzle_owner, puzzle_relation_vec);
			}

			// send event
			Self::deposit_event(Event::PuzzleCreated(who, puzzle_hash, current_block_number.into(), dration_block_number));
			//
			Ok(().into())
		}

		// 删除一个谜题 测试
		#[pallet::weight(1235)]
		pub fn revoke_puzzle(
			origin: OriginFor<T>,
			puzzle_hash: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			// 先验证 origin
			let sender = ensure_signed(origin)?;
			let (owner, _) = Puzzle::<T>::get(&puzzle_hash).ok_or(Error::<T>::PuzzleNotExist)?;
			// 判断发送者和存证所有者是否是同一个人
			ensure!(owner == sender, Error::<T>::NotPuzzleOwner);
			// 删除存储的puzzle
			Puzzle::<T>::remove(&puzzle_hash);
			// 发送存证Revoked事件
			Self::deposit_event(Event::PuzzleRevoked(sender, puzzle_hash));
			// 返回函数成功结果
			Ok(().into())
		}

	}
}