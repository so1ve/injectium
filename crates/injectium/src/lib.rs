//! Injectium – a minimal dependency-injection implementation for Rust.
//!
//! This is the user-facing crate that re-exports everything from
//! [`injectium_core`](https://docs.rs/injectium_core) and adds the
//! `#[derive(Injectable)]` proc-macro.
//!
//! # Quick Start
//!
//! ```
//! use injectium::{Injectable, container};
//!
//! // Define services that need DI
//! #[derive(Clone, Injectable)]
//! struct Db {
//!     conn: String,
//! }
//!
//! #[derive(Injectable)]
//! struct Service {
//!     db: Db,
//! }
//!
//! // At startup, build the container
//! let c = container! {
//!     singletons: [
//!         Db { conn: "postgres://localhost".into() },
//!     ],
//! };
//!
//! // Validate everything is wired up
//! c.validate();
//!
//! // Later, resolve services
//! let svc = Service::from_container(&c);
//! ```
//!
//! # Key Types
//!
//! - [`Container`] – the runtime container holding singletons and factories.
//! - [`ContainerBuilder`] – fluent builder for constructing a container.
//! - [`Injectable`] – trait for types that can construct themselves from a
//!   container. Implement via `#[derive(Injectable)]`.
//! - [`container`] – macro for building a container with singletons and
//!   factories.
//! - [`declare_dependency!`] – manually declare a type is required (usually
//!   automatic via `#[derive(Injectable)]`).

pub use injectium_core::{Container, ContainerBuilder, DeclaredDependency, Injectable, container};
#[cfg(feature = "derive")]
pub use injectium_macro::Injectable;
// Re-export inventory so users don't need a direct dependency on it.
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
