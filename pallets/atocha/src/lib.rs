#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// 首先通过 frame_support::pallet 宏创建 pallet
#[frame_support::pallet]
pub mod pallet {
	// 引入需要的包
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
	};
	// 比较粗暴的引用 frame_system 所有宏函数，和系统类型信息
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	//创建配置接口，通过 config 宏完成
	//继承自系统模块的 Config 接口，只有一个
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// 只有一个关联类型就是 Event，并且约束
		// 可以从本模块的Event 类型进行转换，并且它的类型是一个系统模块的Event 类型。
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// 定义一个结构体类型，来承载整个功能模块，使用 pallet::pallet 这个宏进行定义
	#[pallet::pallet]
	// 表示这个模块依赖的存储单元，一级存储单元依赖的 trait
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn my_puzzle)]
	pub type Puzzle<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		// Puzzle ID
		Vec<u8>,
		// Creator，RecodeJson （TODO:://未来可能需要线下存储先测试）,
		( T::AccountId, Vec<u8> )
	>;

	// 通过 Event 定义一个时间存储类型，这是一个枚举。
	#[pallet::event]
	// 生成一个 转换后的 metadata 方便前段接收
	#[pallet::metadata(T::AccountId = "AccountId")]
	// 生成一个帮助性的方法，方便这个方法进行触发。
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		PuzzleCreated(T::AccountId, Vec<u8>),
		PuzzleRevoked(T::AccountId, Vec<u8>),
	}

	// 通过 error 宏定义一个错误信息
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

	// 定义一个 hooks ，如果有初始化区块的信息可以放到这里面，如果没有这个也必须要加上否则会报错
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// 构建可调用函数，通过 call 这个宏
	#[pallet::call]
	impl<T: Config> Pallet<T> {


		#[pallet::weight(1234)]
		pub fn create_puzzle(
			// 这个参数表示交易的发送方
			origin: OriginFor<T>,
			// 题目的外部识别hash
			puzzle_hash: Vec<u8>,
			// 存储上链（暂时）
			puzzle_content: Vec<u8>,
		) -> DispatchResultWithPostInfo { // 返回值是一个Result类型的别名它会包含一些weight的信息，这是通过use引入进来的
			// 验证签名信息是否合法
			let sender = ensure_signed(origin)?;
			// 判断存证信息是否存在
			ensure!(!Puzzle::<T>::contains_key(&puzzle_hash), Error::<T>::PuzzleAlreadyExist);
			// 插入存证
			Puzzle::<T>::insert(
				&puzzle_hash,
				(sender.clone(), puzzle_content),
			);

			// 发送事件
			Self::deposit_event(Event::PuzzleCreated(sender, puzzle_hash));
			// 返回结果信息，并进行类型转换。
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