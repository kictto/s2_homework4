use frame_support::{
    pallet_prelude::*,
    traits::GetStorageVersion,
    weights::Weight,
};

use mod_extra::Migrate;
pub use v2 as current_version;

use crate::{Config, Pallet};

mod mod_extra;
mod v0;
mod v1;
pub mod v2;

// type FnMigrate<T: Config> = fn() -> Weight;
//
// const version: [(StorageVersion, FnMigrate<>); 2] = [
//     (v1::STORAGE_VERSION, v1::m_test::<T>),
//     (v2::STORAGE_VERSION, v2::m_test::<T>),
// ];

pub fn migrate<T: Config>() -> Weight {
    let version: [(StorageVersion, fn() -> Weight); 3] = [
        (v0::STORAGE_VERSION, v0::Upgrade::migrate::<T>),
        (v1::STORAGE_VERSION, v1::Upgrade::migrate::<T>),
        (v2::STORAGE_VERSION, v2::Upgrade::migrate::<T>),
    ];
    // 链式升级，直至最终版本
    let on_chain_ver: StorageVersion = Pallet::<T>::on_chain_storage_version();
    // 需要一个版本链
    // let idx = version.index(on_chain_ver);
    for (ver, upgrade) in version.iter() {
        if on_chain_ver.lt(ver) {
            // upgrade::migrate::<T>();
            upgrade();
            ver.put::<Pallet::<T>>();
        }
    }
    // 需要一个map，保存从onChainVer升级至下一个版本的方法
    // v0::Upgrade::migrate::<T>();
    Weight::zero()
}
