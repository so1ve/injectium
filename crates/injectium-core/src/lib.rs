//! A minimal dependency-injection container for Rust.
//!
//! Injectium provides a runtime DI container with two registration strategies:
//!
//! - **Singletons** – a single instance shared across all consumers.
//! - **Factories** – a closure that produces a fresh instance on each request.
//!
//! # Quick Start
//!
//! ```
//! use injectium_core::{Container, container};
//!
//! // Build a container with singletons and/or factories
//! let c = container! {
//!     singletons: [
//!         42_u32,
//!         String::from("hello"),
//!     ],
//!     providers: [
//!         |c| format!("value is {}", c.get::<u32>()),
//!     ],
//! };
//!
//! // Retrieve a singleton
//! assert_eq!(*c.get::<u32>(), 42);
//!
//! // Resolve a factory (produces a new value each time)
//! assert_eq!(c.resolve::<String>(), "value is 42");
//! ```
//!
//! # Using `#[derive(Injectable)]`
//!
//! The [`injectium`](https://docs.rs/injectium) crate (not this one) re-exports
//! the `#[derive(Injectable)]` macro which automatically implements the
//! [`Injectable`] trait for your structs:
//!
//! ```ignore
//! use injectium::{Injectable, container};
//! use std::sync::Arc;
//!
//! #[derive(Clone, Injectable)]
//! struct Db {
//!     conn: Arc<Connection>,
//! }
//!
//! #[derive(Clone, Injectable)]
//! struct Config {
//!     url: String,
//! }
//!
//! #[derive(Injectable)]
//! struct Service {
//!     db: Db,
//!     config: Config,
//! }
//!
//! // At application startup:
//! let c = container! {
//!     singletons: [
//!         Connection::new(),
//!         Config { url: "localhost".into() },
//!     ],
//! };
//!
//! // Anywhere in your code, resolve a fully-constructed Service:
//! let svc = Service::from_container(&c);
//! ```
//!
//! # Validation
//!
//! Call [`Container::validate`] at startup to ensure every
//! `#[derive(Injectable)]` struct's dependencies are actually registered:
//!
//! ```ignore
//! let c = container! { /* ... */ };
//! c.validate(); // panics with a helpful message if something is missing
//! ```
//!
//! This catches misconfiguration immediately rather than failing on first use.

mod container;
mod inject;

use cfg_block::cfg_block;
pub use container::{Container, ContainerBuilder};
pub use inject::Injectable;

/// Declarative macro for building a container with singletons and/or factory
/// providers.
///
/// # Example
///
/// ```ignore
/// let container = container! {
///     singletons: [
///         db,
///         config,
///         jwt_secret,
///     ],
///     providers: [
///         |c| MyService::new(c.get::<Db>().clone()),
///         |c| AnotherService::new(c.get::<Config>().clone()),
///     ]
/// };
/// ```
#[macro_export]
macro_rules! container {
    (@replace_expr $_expr:expr, $sub:expr) => {
        $sub
    };
    (@count_exprs $( $expr:expr ),* $(,)?) => {
        <[()]>::len(&[$($crate::container!(@replace_expr $expr, ())),*])
    };
    (capacity: { singletons: $singleton_capacity:expr, factories: $factory_capacity:expr } $(,)? singletons: [$( $singleton:expr ),* $(,)?] $(,)? providers: [$( $factory:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity($singleton_capacity, $factory_capacity)
            $(.singleton($singleton))*
            $(.factory($factory))*
            .build()
    }};
    (capacity: { singletons: $singleton_capacity:expr, factories: $factory_capacity:expr } $(,)? singletons: [$( $singleton:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity($singleton_capacity, $factory_capacity)
            $(.singleton($singleton))*
            .build()
    }};
    (capacity: { singletons: $singleton_capacity:expr, factories: $factory_capacity:expr } $(,)? providers: [$( $factory:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity($singleton_capacity, $factory_capacity)
            $(.factory($factory))*
            .build()
    }};
    // Both singletons and providers
    (singletons: [$( $singleton:expr ),* $(,)?] $(,)? providers: [$( $factory:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity(
            $crate::container!(@count_exprs $($singleton),*),
            $crate::container!(@count_exprs $($factory),*),
        )
            $(.singleton($singleton))*
            $(.factory($factory))*
            .build()
    }};
    // Only singletons
    (singletons: [$( $singleton:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity(
            $crate::container!(@count_exprs $($singleton),*),
            0,
        )
            $(.singleton($singleton))*
            .build()
    }};
    // Only providers
    (providers: [$( $factory:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity(
            0,
            $crate::container!(@count_exprs $($factory),*),
        )
            $(.factory($factory))*
            .build()
    }};
}

cfg_block! {
    if #[cfg(feature = "validation")] {
        pub use container::DeclaredDependency;
        pub use inventory;

        /// Declare that a type `$ty` must be present in the [`Container`].
        ///
        /// Registers a [`DeclaredDependency`] entry collected at link time by
        /// [`inventory`]. Call [`Container::validate`] at startup to assert all
        /// declared types are registered.
        ///
        /// `#[derive(Injectable)]` calls this automatically for every field type, so
        /// manual use is only needed when calling `container.get::<T>()` directly
        /// without going through [`Injectable`].
        ///
        /// # Example
        ///
        /// ```ignore
        /// injectium::declare_dependency!(MyService);
        /// ```
        #[macro_export]
        macro_rules! declare_dependency {
            ($ty:ty) => {
                $crate::inventory::submit! {
                    $crate::DeclaredDependency {
                        type_id: ::std::any::TypeId::of::<$ty>,
                        type_name: ::std::stringify!($ty),
                    }
                }
            };
        }
    } else {
        /// No-op when validation is disabled.
        #[macro_export]
        macro_rules! declare_dependency {
            ($ty:ty) => {};
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Container;

    #[test]
    fn empty_both() {
        let c = container! {
            singletons: [],
            providers: []
        };
        assert_eq!(c.singleton_count(), 0);
        assert_eq!(c.factory_count(), 0);
    }

    #[test]
    fn only_singletons() {
        let c = container! {
            singletons: [1_u32, 2_u64],
            providers: []
        };
        assert_eq!(c.singleton_count(), 2);
        assert_eq!(c.factory_count(), 0);
    }

    #[test]
    fn only_providers() {
        let c = container! {
            singletons: [],
            providers: [
                |_c| 42_u32,
                |_c| "hello",
            ]
        };
        assert_eq!(c.singleton_count(), 0);
        assert_eq!(c.factory_count(), 2);
    }

    #[test]
    fn only_singletons_no_providers_key() {
        let c = container! {
            singletons: [1_u32, 2_u64]
        };
        assert_eq!(c.singleton_count(), 2);
        assert_eq!(c.factory_count(), 0);
    }

    #[test]
    fn only_singletons_no_providers_key_empty() {
        let c = container! {
            singletons: []
        };
        assert_eq!(c.singleton_count(), 0);
        assert_eq!(c.factory_count(), 0);
    }

    #[test]
    fn only_singletons_no_providers_key_trailing_comma() {
        let c = container! {
            singletons: [1_u32,],
        };
        assert_eq!(c.singleton_count(), 1);
        assert_eq!(c.factory_count(), 0);
    }

    #[test]
    fn only_providers_no_singletons_key() {
        let c = container! {
            providers: [|_c| 42_u32, |_c| "hello"]
        };
        assert_eq!(c.singleton_count(), 0);
        assert_eq!(c.factory_count(), 2);
    }

    #[test]
    fn only_providers_no_singletons_key_empty() {
        let c = container! {
            providers: []
        };
        assert_eq!(c.singleton_count(), 0);
        assert_eq!(c.factory_count(), 0);
    }

    #[test]
    fn only_providers_no_singletons_key_trailing_comma() {
        let c = container! {
            providers: [|_c| 42_u32,],
        };
        assert_eq!(c.singleton_count(), 0);
        assert_eq!(c.factory_count(), 1);
    }

    #[test]
    fn singletons_and_providers() {
        let c = container! {
            singletons: [1_u32],
            providers: [|_c| 99_u64]
        };
        assert_eq!(c.singleton_count(), 1);
        assert_eq!(c.factory_count(), 1);
        assert_eq!(*c.get::<u32>(), 1);
        assert_eq!(c.resolve::<u64>(), 99);
    }

    #[test]
    fn trailing_commas() {
        let c = container! {
            singletons: [1_u32,],
            providers: [|_c| 2_u64,],
        };
        assert_eq!(c.singleton_count(), 1);
        assert_eq!(c.factory_count(), 1);
    }

    #[test]
    fn with_capacity_builder() {
        let c = Container::builder_with_capacity(2, 1)
            .singleton(1_u32)
            .singleton(2_u64)
            .factory(|_c| 3_u8)
            .build();

        assert_eq!(c.singleton_count(), 2);
        assert_eq!(c.factory_count(), 1);
        assert_eq!(c.resolve::<u8>(), 3);
    }

    #[test]
    fn with_capacity_macro() {
        let c = container! {
            capacity: { singletons: 2, factories: 1 },
            singletons: [1_u32, 2_u64],
            providers: [|_c| 3_u8],
        };

        assert_eq!(c.singleton_count(), 2);
        assert_eq!(c.factory_count(), 1);
        assert_eq!(c.resolve::<u8>(), 3);
    }
}
