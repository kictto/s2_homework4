#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod migrations;

//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    use sp_io::hashing::blake2_128;
    use frame_support::traits::{Randomness, Currency, ExistenceRequirement};
    use frame_support::PalletId;
    use sp_runtime::traits::AccountIdConversion;
    use crate::migrations;
    pub use crate::migrations::current_version::*;

    pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        type Currency: Currency<Self::AccountId>;
        #[pallet::constant]
        type KittyPrice: Get<BalanceOf<Self>>;
        type PalletId: Get<PalletId>;
    }

    /// 存储KittyId
    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T> = StorageValue<_, KittyId, ValueQuery>;     // 此处给定了第三个参数，该参数用于给定默认值，对于u32类型的KittyId来说，它就是0

    /// 存储Kitty的数据内容
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;
    /// 存储Kitty的Owner
    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;
    /// 存储Kitty的继承关系
    #[pallet::storage]
    #[pallet::getter(fn kitty_parents)]
    pub type KittyParents<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (KittyId, KittyId), OptionQuery>;
    /// 存储Kitty的Sale状态
    #[pallet::storage]
    #[pallet::getter(fn kitty_on_sale)]
    pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, (), OptionQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/main-docs/build/events-errors/
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Kitty创建成功
        KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
        /// Kitty breed成功
        KittyBred { who: T::AccountId, kitty_id: KittyId, kitty: Kitty },
        /// Kitty 转移成功
        KittyTransferred { who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId },
        /// Kitty 销售上架
        KittyOnSale { who: T::AccountId, kitty_id: KittyId },
        /// Kitty被购买
        KittyBought { who: T::AccountId, kitty_id: KittyId },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// KittyId创建失败
        InvalidKittyId,
        /// KittyId相同
        SameKittyId,
        /// 非Owner
        NotOwner,
        /// 转移给自己
        CanNotTransferToSelf,
        /// 已经处于在售状态
        AlreadyOnSale,
        /// 没有Owner
        NoOwner,
        /// 已经持有了
        AlreadyOwned,
        /// 未上架销售
        NotOnSale,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            // migrations::v1::migrate::<T>()
            // migrations::v2::migrate::<T>()
            migrations::migrate::<T>()
        }
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建Kitty
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn create_kitty(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let kitty_id = Self::get_next_id()?;
            let dna = Self::random_value(&who);
            let kitty = Kitty { dna, name };

            let price = T::KittyPrice::get();
            // T::Currency::reserve(&who, price)?;
            T::Currency::transfer(&who, &Self::get_account_id(),
                                  price, ExistenceRequirement::KeepAlive)?;

            Kitties::<T>::insert(kitty_id, &kitty);
            KittyOwner::<T>::insert(kitty_id, &who);


            // 发布创建成功事件
            Self::deposit_event(Event::KittyCreated { who, kitty_id, kitty });
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        /// 两个kitty，生成一个子kitty
        #[pallet::call_index(1)]
        #[pallet::weight(10_001 + T::DbWeight::get().writes(1).ref_time())]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyId, kitty_id_2: KittyId, name: [u8; 8]) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(kitty_id_1 != kitty_id_2,Error::<T>::SameKittyId);
            ensure!(Kitties::<T>::contains_key(kitty_id_1),Error::<T>::InvalidKittyId);
            ensure!(Kitties::<T>::contains_key(kitty_id_2),Error::<T>::InvalidKittyId);

            let kitty_id = Self::get_next_id()?;

            let kitty_1 = Kitties::<T>::get(kitty_id_1).expect("Checked it Exists");
            let kitty_2 = Kitties::<T>::get(kitty_id_2).expect("Checked it Exists");

            let selector = Self::random_value(&who);
            let mut data = [0u8; 16];
            for i in 0..kitty_1.dna.len() {
                data[i] = (kitty_1.dna[i] & selector[i]) | (kitty_2.dna[i] & !selector[i])
            }
            let kitty = Kitty { dna: data, name };

            let price = T::KittyPrice::get();
            // T::Currency::reserve(&who, price)?;
            T::Currency::transfer(&who, &Self::get_account_id(),
                                  price, ExistenceRequirement::KeepAlive)?;

            Kitties::<T>::insert(kitty_id, &kitty);
            KittyOwner::<T>::insert(kitty_id, &who);
            KittyParents::<T>::insert(kitty_id, (kitty_id_1, kitty_id_2));

            // 发布创建成功事件
            Self::deposit_event(Event::KittyBred { who, kitty_id, kitty });

            Ok(())
        }

        /// 转移kitty
        #[pallet::call_index(2)]
        #[pallet::weight(10_002 + T::DbWeight::get().writes(1).ref_time())]
        pub fn transfer(origin: OriginFor<T>, recipient: T::AccountId, kitty_id: KittyId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Kitties::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);

            ensure!( Self::kitty_owner(kitty_id) == Some(who.clone()),Error::<T>::NotOwner);

            ensure!(recipient != who,Error::<T>::CanNotTransferToSelf);

            KittyOwner::<T>::insert(kitty_id, &recipient);

            Self::deposit_event(Event::KittyTransferred { who, recipient, kitty_id });

            Ok(())
        }

        /// 标记可售
        #[pallet::call_index(3)]
        #[pallet::weight(10_003 + T::DbWeight::get().writes(1).ref_time())]
        pub fn sale(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // kitty存在
            ensure!(Kitties::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);
            // 所有权正确
            ensure!( Self::kitty_owner(kitty_id) == Some(who.clone()),Error::<T>::NotOwner);
            // 已经在售状态
            ensure!(!KittyOnSale::<T>::contains_key(kitty_id), Error::<T>::AlreadyOnSale);
            // 标记在售
            KittyOnSale::<T>::insert(kitty_id, ());

            Self::deposit_event(Event::KittyOnSale { who, kitty_id });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(10_004 + T::DbWeight::get().writes(1).ref_time())]
        pub fn buy(origin: OriginFor<T>, kitty_id: KittyId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(Kitties::<T>::contains_key(kitty_id), Error::<T>::InvalidKittyId);
            let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::NoOwner)?;

            ensure!(owner != who, Error::<T>::AlreadyOwned);
            ensure!(KittyOnSale::<T>::contains_key(kitty_id), Error::<T>::NotOnSale);

            let price = T::KittyPrice::get();
            // 质押
            // T::Currency::reserve(&who, price)?;
            // 解除质押
            // T::Currency::unreserve(&owner, price);
            // 转移
            T::Currency::transfer(&who, &owner,
                                  price, ExistenceRequirement::KeepAlive)?;

            KittyOwner::<T>::insert(kitty_id, &who);
            KittyOnSale::<T>::remove(kitty_id);

            Self::deposit_event(Event::KittyBought { who, kitty_id });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 返回一个kittyId，并+1后保存为下一个kittyId
        fn get_next_id() -> Result<KittyId, DispatchError> {
            NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId, DispatchError> {
                // 读取当前的 此时完成了copy
                let current_id = *next_id;
                // 更新下一个id，可能超出u32的范围，溢出则抛出Error
                *next_id = next_id.checked_add(1).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
                Ok(current_id)
            })
        }
        /// 生成一个随机数
        fn random_value(sender: &T::AccountId) -> [u8; 16] {
            // 多个参数，确保payload唯一
            let payload = (
                T::Randomness::random_seed(),
                &sender,
                <frame_system::Pallet<T>>::extrinsic_index(),
            );
            // 用blake2_128确保长度match
            payload.using_encoded(blake2_128)
        }
        ///
        fn get_account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
