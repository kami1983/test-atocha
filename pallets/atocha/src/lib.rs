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
    pub type PuzzleAnswerOption = Option<PuzzleAnswerHash>;
    pub type PuzzleTicket = u64;

    pub type PuzzleAnswerSigned = Vec<u8>;
    pub type PuzzleAnswerNonce = Vec<u8>;
    pub type CreateBn = u64;
    pub type DurationBn = u64;
    pub type RevealBn = u64;
    pub type PuzzleVersion = u64;

    pub type PuzzleAnswerStatus = u8;

    // 1=solving, 2=up to time, 3=solve
    pub const PUZZLE_STATUS_IS_SOLVING: PuzzleStatus = 1;
    pub const PUZZLE_STATUS_IS_UP_TO_TIME: PuzzleStatus = 2;
    pub const PUZZLE_STATUS_IS_SOLVED: PuzzleStatus = 3;

    // 引入需要的包
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
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

    // #[pallet::storage]
    // #[pallet::getter(fn my_puzzle)]
    // pub type Puzzle<T: Config> = StorageMap<
    // 	_,
    // 	Blake2_128Concat,
    // 	Vec<u8>,
    // 	( T::AccountId, Vec<u8> )
    // >;

    #[pallet::storage]
    #[pallet::getter(fn puzzle_info)]
    pub type PuzzleInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PuzzleSubjectHash,
        (
            T::AccountId,
            PuzzleAnswerOption,
            PuzzleAnswerSigned,
            PuzzleAnswerNonce,
            PuzzleTicket,
            PuzzleStatus,
            CreateBn,
            DurationBn,
            RevealBn,
            PuzzleVersion,
        ),
    >;

    #[pallet::storage]
    #[pallet::getter(fn puzzle_direct_answer)]
    pub type PuzzleDirectAnswer<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PuzzleSubjectHash,
        // puzzle_hash, answer_hash, ticket (Balance type), relation_type (1=Creater, 2=Answer), status (1=solving, 2=up to time, 3=solve), create_bn, expired_bn
        Vec<(
            T::AccountId,
            PuzzleAnswerHash,
            PuzzleTicket,
            PuzzleAnswerStatus,
            CreateBn,
        )>,
    >;

    #[pallet::event]
    // Make a metadata, used by WebUI
    #[pallet::metadata(T::AccountId = "AccountId")]
    // Make a help methods, used by the caller
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        // creator id, puzzle_hash, create block number , duration block number,
        PuzzleCreated(T::AccountId, PuzzleSubjectHash, CreateBn, DurationBn),
        AnswerCreated(T::AccountId, PuzzleAnswerHash, PuzzleSubjectHash, CreateBn),
        PuzzleRevoked(T::AccountId, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        // 定义一个错误信息，存证已经存在
        ProofAlreadyExist,
        ClaimNotExist,
        NotClaimOwner,
        PuzzleAlreadyExist,
        AnswerAlreadyExist,
        PuzzleNotExist,
        NotPuzzleOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T>
    where
        u64: From<<T as frame_system::Config>::BlockNumber>,
    {
        #[pallet::weight(1234)]
        pub fn create_puzzle(
            origin: OriginFor<T>,
            puzzle_hash: PuzzleSubjectHash,
            answer_signed: PuzzleAnswerSigned,
            answer_nonce: PuzzleAnswerNonce,
            ticket: PuzzleTicket,
            duration: DurationBn,
            puzzle_version: PuzzleVersion,
        ) -> DispatchResultWithPostInfo {
            // check signer
            let who = ensure_signed(origin)?;
            //
            let current_block_number = <frame_system::Pallet<T>>::block_number();
            let dration_block_number = duration.checked_add(current_block_number.into()).unwrap();

            ensure!(
                !<PuzzleInfo<T>>::contains_key(&puzzle_hash),
                Error::<T>::PuzzleAlreadyExist
            );

            type PuzzleContent<T> = (
                <T as frame_system::Config>::AccountId,
                PuzzleAnswerOption,
                PuzzleAnswerSigned,
                PuzzleAnswerNonce,
                PuzzleTicket,
                PuzzleStatus,
                CreateBn,
                DurationBn,
                RevealBn,
                PuzzleVersion,
            );

            let puzzle_content: PuzzleContent<T> = (
                who.clone(),
                None,
                answer_signed.clone(),
                answer_nonce.clone(),
                ticket,
                PUZZLE_STATUS_IS_SOLVING,
                current_block_number.clone().into(),
                dration_block_number.clone(),
                0,
                1,
            );
            <PuzzleInfo<T>>::insert(puzzle_hash.clone(), puzzle_content);

            // send event
            Self::deposit_event(Event::PuzzleCreated(
                who,
                puzzle_hash,
                current_block_number.into(),
                dration_block_number,
            ));
            //
            Ok(().into())
        }

        #[pallet::weight(1234)]
        pub fn answer_puzzle(
            origin: OriginFor<T>,
            puzzle_hash: PuzzleSubjectHash,
            answer_hash: PuzzleAnswerHash,
            ticket: PuzzleTicket,
        ) -> DispatchResultWithPostInfo {
            // check signer
            let who = ensure_signed(origin)?;

            //
            let current_block_number = <frame_system::Pallet<T>>::block_number();

            //(T::AccountId, PuzzleAnswerHash, PuzzleTicket, PuzzleAnswerStatus, CreateBn)
            type AnswerContent<T> = (
                <T as frame_system::Config>::AccountId,
                PuzzleAnswerHash,
                PuzzleTicket,
                PuzzleAnswerStatus,
                CreateBn,
            );

            // Puzzle need exists.
            ensure!(
                <PuzzleInfo<T>>::contains_key(&puzzle_hash),
                Error::<T>::PuzzleNotExist
            );

            let mut answer_store_list: Vec<AnswerContent<T>> = Vec::new();
            let answer_list_opt = <PuzzleDirectAnswer<T>>::get(&puzzle_hash);
            if let Some(answer_list) = answer_list_opt {
                // Determine whether the answer already exists.
                let answer_exists = answer_list
                    .iter()
                    .any(|(_, old_answer_hash, _, _, _)| &answer_hash == old_answer_hash);
                ensure!(!answer_exists, Error::<T>::AnswerAlreadyExist);
                answer_store_list = answer_list;
            }

            // create new answer tuple.
            let answer_content: AnswerContent<T> = (
                who.clone(),
                answer_hash.clone(),
                ticket,
                0, // 1=solving ???, 2=NONE, 3=solved ????4=closed ???
                current_block_number.clone().into(),
            );
            answer_store_list.push(answer_content);

            <PuzzleDirectAnswer<T>>::insert(puzzle_hash.clone(), answer_store_list);

            // send event
            Self::deposit_event(Event::AnswerCreated(
                who,
                answer_hash,
                puzzle_hash,
                current_block_number.into(),
            ));
            //
            Ok(().into())
        }
    }
}
