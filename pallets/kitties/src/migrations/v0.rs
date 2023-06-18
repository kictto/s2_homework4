use frame_support::{
    pallet_prelude::*,
    weights::Weight,
    storage_alias
};

use crate::{Config, Pallet};

use super::mod_extra::Migrate;

/// 当前版本的定义
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
/// ID
pub type KittyId = u32;

/// 数据存储的类型和长度
#[derive(Encode, Decode, Clone, Copy, RuntimeDebug, PartialEq, Eq, Default, TypeInfo, MaxEncodedLen)]
pub struct Kitty (pub [u8; 16]);

#[storage_alias]
pub(super) type Kitties<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, KittyId, Kitty>;


pub(crate) struct Upgrade;

impl Migrate for Upgrade {
    fn migrate<T: Config>() -> Weight {
        Weight::zero()
    }
}