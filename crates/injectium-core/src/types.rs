use std::any::Any;

use cfg_block::cfg_block;

use crate::container::Container;

cfg_block! {
    if #[cfg(feature = "sync")] {
        pub type AnyDyn = dyn Any + Send + Sync;
        pub type ErasedProvider = dyn Fn(&Container) -> Box<AnyDyn> + Send + Sync;

        pub trait SyncBounds: Send + Sync + 'static {}
        impl<T: Send + Sync + 'static> SyncBounds for T {}
    } else {
        pub type AnyDyn = dyn Any;
        pub type ErasedProvider = dyn Fn(&Container) -> Box<AnyDyn>;

        pub trait SyncBounds: 'static {}
        impl<T: 'static> SyncBounds for T {}
    }
}
