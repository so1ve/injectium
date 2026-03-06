//! Injectium – a minimal dependency-injection implementation for Rust.
//!
//! This is the user-facing crate that re-exports everything from
//! [`injectium_core`](https://docs.rs/injectium_core) and adds the
//! `#[derive(Injectable)]` proc-macro.
//!
//! # Quick Start
//!
//! ```
//! use injectium::{Container, Injectable, cloned, container};
//!
//! #[derive(Clone)]
//! struct DbConnection {
//!     conn: String,
//! }
//!
//! #[derive(Injectable)]
//! struct Service {
//!     db: DbConnection,
//!     greeting: String,
//! }
//!
//! // At startup, build the container
//! let c = container! {
//!     providers: [
//!         cloned(DbConnection { conn: "postgres://localhost".into() }),
//!         |_: &Container| String::from("hello from provider"),
//!     ],
//! };
//!
//! // Validate everything is wired up
//! c.validate();
//!
//! // Later, resolve services
//! let svc = Service::from_container(&c);
//!
//! assert_eq!(svc.db.conn, "postgres://localhost");
//! assert_eq!(svc.greeting, "hello from provider");
//! ```
//!
//! # Key Types
//!
//! - [`Container`] – the runtime container holding typed providers.
//! - [`ContainerBuilder`] – fluent builder for constructing a container.
//! - [`Injectable`] – trait for types that can construct themselves from a
//!   container. Implement via `#[derive(Injectable)]`.
//! - [`Provider`] – trait implemented by closure providers and `Arc<T>` values.
//! - [`cloned`] – helper for explicitly registering clone-on-read providers.
//! - [`copied`] – helper for explicitly registering copy-on-read providers.
//! - [`container`] – macro for building a container from providers.
//! - [`declare_dependency!`] – manually declare a type is required (usually
//!   automatic via `#[derive(Injectable)]`).

pub use injectium_core::{
    CloneProvider, Container, ContainerBuilder, CopyProvider, DeclaredDependency, Injectable,
    Provider, cloned, container, copied, declare_dependency, inventory,
};
#[cfg(feature = "derive")]
pub use injectium_macro::Injectable;
