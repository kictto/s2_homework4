use frame_support::weights::Weight;
use crate::Config;

pub trait Migrate {
    fn migrate<T: Config>() -> Weight;
}