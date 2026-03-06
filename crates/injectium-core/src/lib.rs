//! A minimal dependency-injection container for Rust.
//!
//! Injectium provides a runtime DI container built around providers.
//!
//! Providers can be closure-backed providers, shared values registered as
//! `Arc<T>`, or explicit value providers created with [`cloned`] / [`copied`].
//!
//! # Quick Start
//!
//! ```
//! use injectium_core::{Container, container, copied};
//!
//! // Build a container with providers
//! let c = container! {
//!     providers: [
//!         copied(42_u32),
//!         |c: &Container| format!("value is {}", c.get::<u32>()),
//!     ],
//! };
//!
//! assert_eq!(c.get::<u32>(), 42);
//! assert_eq!(c.get::<String>(), "value is 42");
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
//!     providers: [
//!         Arc::new(Connection::new()),
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
mod provider;
mod types;

use cfg_block::cfg_block;
pub use container::{Container, ContainerBuilder};
pub use inject::Injectable;
pub use provider::{CloneProvider, CopyProvider, Provider, cloned, copied};
pub use types::SyncBounds;

/// Declarative macro for building a container from providers.
///
/// # Example
///
/// ```ignore
/// let container = container! {
///     providers: [
///         Arc::new(db),
///         config,
///         jwt_secret,
///         |c: &Container| MyService::new(c.get::<Arc<Db>>()),
///         |c: &Container| AnotherService::new(c.get::<Config>()),
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
    (providers: [$( $provider:expr ),* $(,)?] $(,)?) => {{
        $crate::Container::builder_with_capacity(
            $crate::container!(@count_exprs $($provider),*),
        )
            $(.provider($provider))*
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
    use std::sync::Arc;

    use crate::{Container, cloned, copied};

    #[test]
    fn empty_providers() {
        let c = container! {
            providers: []
        };
        assert_eq!(c.provider_count(), 0);
    }

    #[test]
    fn only_closure_providers() {
        let c = container! {
            providers: [|_c: &Container| 1_u32, |_c: &Container| 2_u64]
        };
        assert_eq!(c.provider_count(), 2);
    }

    #[test]
    fn cloned_and_closure_providers() {
        let c = container! {
            providers: [
                cloned(41_u16),
                |_c: &Container| 42_u32,
                |_c: &Container| "hello",
            ]
        };
        assert_eq!(c.provider_count(), 3);
    }

    #[test]
    fn providers_with_trailing_comma() {
        let c = container! {
            providers: [Arc::new(1_u32), |_c: &Container| 2_u64,]
        };
        assert_eq!(c.provider_count(), 2);
    }

    #[test]
    fn only_providers_empty() {
        let c = container! {
            providers: []
        };
        assert_eq!(c.provider_count(), 0);
    }

    #[test]
    fn get_from_arc_and_closure_providers() {
        let c = container! {
            providers: [Arc::new(1_u32), |_c: &Container| 99_u64]
        };
        assert_eq!(c.provider_count(), 2);
        assert_eq!(*c.get::<Arc<u32>>(), 1);
        assert_eq!(c.get::<u64>(), 99);
    }

    #[test]
    fn with_capacity_builder() {
        let c = Container::builder_with_capacity(3)
            .provider(Arc::new(1_u32))
            .provider(|_c: &Container| 2_u64)
            .provider(|_c: &Container| 3_u8)
            .build();

        assert_eq!(c.provider_count(), 3);
        assert_eq!(c.get::<u8>(), 3);
    }

    #[test]
    fn get_clones_explicit_clone_providers() {
        let c = container! { providers: [cloned(String::from("shared"))] };

        assert_eq!(c.get::<String>(), "shared");
    }

    #[test]
    fn get_copies_explicit_copy_providers() {
        let c = container! { providers: [copied(7_u32)] };

        assert_eq!(c.get::<u32>(), 7);
    }

    #[test]
    fn get_executes_closure_providers() {
        let c = container! {
            providers: [|_c: &Container| String::from("factory")]
        };

        assert_eq!(c.get::<String>(), "factory");
    }

    #[test]
    fn builder_contains_registered_output_type() {
        let builder = Container::builder().provider(Arc::new(String::from("shared")));

        assert!(builder.contains::<Arc<String>>());
        assert!(builder.contains_provider::<Arc<String>>());
        assert!(!builder.contains::<u32>());
    }

    #[test]
    #[should_panic(
        expected = "provider already registered for `alloc::sync::Arc<alloc::string::String>`"
    )]
    fn duplicate_arc_provider_panics() {
        let _ = Container::builder()
            .provider(Arc::new(String::from("first")))
            .provider(Arc::new(String::from("second")));
    }

    #[test]
    #[should_panic(
        expected = "provider already registered for `alloc::sync::Arc<alloc::string::String>`"
    )]
    fn closure_and_arc_provider_conflict_panics() {
        let _ = Container::builder()
            .provider(Arc::new(String::from("shared")))
            .provider(|_c: &Container| Arc::new(String::from("factory")));
    }

    #[test]
    #[should_panic(expected = "provider already registered for `alloc::string::String`")]
    fn duplicate_closure_provider_panics() {
        let _ = Container::builder()
            .provider(|_c: &Container| String::from("first"))
            .provider(|_c: &Container| String::from("second"));
    }
}
