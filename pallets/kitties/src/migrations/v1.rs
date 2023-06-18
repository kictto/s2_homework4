use frame_support::{
    pallet_prelude::*,
    storage::StoragePrefixedMap,
    traits::GetStorageVersion,
    weights::Weight,
    migration::storage_key_iter,
    Blake2_128Concat,
    storage_alias,
};
use crate::{Config, Pallet};
use super::{v0, mod_extra::Migrate};

/// 当前版本的定义
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// ID
pub type KittyId = v0::KittyId;

/// 数据存储的类型和长度
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct Kitty {
    pub dna: [u8; 16],
    pub name: [u8; 4],
}

#[storage_alias]
pub(super) type Kitties<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, KittyId, Kitty>;

/// 上个版本的定义
type OldKitty = v0::Kitty;

pub(crate) struct Upgrade;

/// 从v0~v1
impl Migrate for Upgrade {
    fn migrate<T: Config>() -> Weight {
        let on_chain_version = Pallet::<T>::on_chain_storage_version();
        let current_version = Pallet::<T>::current_storage_version();

        if on_chain_version != 0 {
            return Weight::zero();
        }

        if current_version < 1 {
            return Weight::zero();
        }

        let module = Kitties::<T>::module_prefix();
        let item = Kitties::<T>::storage_prefix();

        for (index, kitty) in storage_key_iter::<KittyId, OldKitty, Blake2_128Concat>(module, item).drain() {
            let new_kitty = Kitty {
                dna: kitty.0,
                name: *b"NULL",
            };
            Kitties::<T>::insert(index, new_kitty);
        }

        Weight::zero()
    }
}