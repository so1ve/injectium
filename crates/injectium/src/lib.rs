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
//! #[derive(Clone)]
//! struct Db {
//!     conn: String,
//! }
//!
//! #[derive(Injectable)]
//! struct Service {
//!     db: Db,
//!     random_string: String,
//! }
//!
//! // At startup, build the container
//! let c = container! {
//!     singletons: [
//!         Db { conn: "postgres://localhost".into() },
//!         String::from("connection string"),
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
//! assert_eq!(svc.random_string, "connection string");
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

pub use injectium_core::{
    Container, ContainerBuilder, DeclaredDependency, Injectable, container, declare_dependency,
    inventory,
};
#[cfg(feature = "derive")]
pub use injectium_macro::Injectable;
