//! Salvo integration for Injectium dependency injection.
//!
//! This crate provides seamless integration between the [Injectium](https://crates.io/crates/injectium)
//! dependency injection container and the [Salvo](https://salvo.rs) web framework.
//!
//! # Quick Start
//!
//! ## 1. Define your services with `#[derive(Injectable)]`
//!
//! ```ignore
//! use injectium::Injectable;
//!
//! #[derive(Clone)]
//! struct Database {
//!     connection_string: String,
//! }
//!
//! #[derive(Clone, Injectable)]
//! struct UserService {
//!     db: Database,
//! }
//! ```
//!
//! ## 2. Build the container
//!
//! ```ignore
//! use injectium::container;
//! use std::sync::Arc;
//!
//! let container = Arc::new(container! {
//!     singletons: [
//!         Database { connection_string: "postgres://localhost".into() },
//!     ],
//! });
//! ```
//!
//! ## 3. Configure Salvo router
//!
//! ```ignore
//! use salvo::prelude::*;
//! use injectium_salvo::{inject_container, Injected};
//!
//! #[handler]
//! async fn get_users(users: Injected<UserService>) -> String {
//!     format!("Users DB: {}", users.db.connection_string)
//! }
//!
//! let router = Router::new()
//!     .hoop(inject_container(container))
//!     .push(Router::with_path("/users").get(get_users));
//! ```
//!
//! # Key Types
//!
//! - [`Injected<T>`] - Automatically resolves `T` from the container in your
//!   handlers
//! - [`inject_container`] - Registers the container into Salvo's request scope
//!
//! # Features
//!
//! - `oapi` - Enable OpenAPI support for `Injected<T>` types
//!
//! # Error Handling
//!
//! If the container is not registered or a dependency is missing, the handler
//! will panic with a helpful message. Use [`Container::validate`] at startup to
//! catch missing dependencies early.
//!
//! # Thread Safety
//!
//! All injected types must implement `Send + Sync + 'static` to be safely
//! shared across async tasks.
//!
//! # Example
//!
//! See the [injectium examples](https://github.com/so1ve/injectium/tree/main/examples) for a complete working example.

use std::ops::Deref;
use std::sync::Arc;

use cfg_block::cfg_block;
use injectium::{Container, Injectable};
use salvo::extract::{Extractible, Metadata};
use salvo::http::{ParseError, Request};
use salvo::prelude::*;

/// A wrapper type for injecting dependencies from an Injectium container into
/// Salvo handlers.
///
/// `Injected<T>` automatically resolves `T` from the [`Container`] stored in
/// the Salvo [`Depot`]. The type `T` must implement [`injectium::Injectable`]
/// and `Send + Sync + 'static`.
///
/// # Example
///
/// ```ignore
/// use injectium::{Injectable, container};
/// use injectium_salvo::{Injected, inject_container};
/// use salvo::prelude::*;
/// use std::sync::Arc;
///
/// #[derive(Clone, Injectable)]
/// struct DbService {
///     conn: String,
/// }
///
/// #[handler]
/// async fn hello(user: Injected<DbService>) -> String {
///     format!("DB: {}", user.conn)
/// }
///
/// let container = Arc::new(container! {
///     singletons: [DbService { conn: "localhost".into() }],
/// });
///
/// let router = Router::new()
///     .hoop(inject_container(container))
///     .push(Router::with_path("/hello").get(hello));
/// ```
pub struct Injected<T>(pub T);

impl<T> Deref for Injected<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'ex, T> Extractible<'ex> for Injected<T>
where
    T: Injectable + Send + Sync + 'static,
{
    fn metadata() -> &'static Metadata {
        static METADATA: Metadata = Metadata::new("");

        &METADATA
    }

    #[allow(refining_impl_trait)]
    async fn extract(_req: &'ex mut Request, depot: &'ex mut Depot) -> Result<Self, ParseError> {
        let container = depot
            .obtain::<Arc<Container>>()
            .map_err(|_| ParseError::other("container not found in depot"))?;
        let value = T::from_container(container);

        Ok(Self(value))
    }
}

cfg_block! {
    #[cfg(feature = "oapi")] {
        use salvo::oapi::{Components, EndpointArgRegister, Operation};

        impl<T> EndpointArgRegister for Injected<T>
        where
            T: Injectable + Send + Sync + 'static,
        {
            fn register(_components: &mut Components, _operation: &mut Operation, _arg: &str) {}
        }
    }
}

/// Register a [`Container`] into the Salvo [`Depot`] so that [`Injected<T>`]
/// can resolve dependencies from it.
///
/// Call this in your router setup, e.g.:
///
/// ```ignore
/// Router::new()
///     .hoop(inject_container(Arc::new(container)))
///     ...
/// ```
#[must_use]
pub fn inject_container(container: Arc<Container>) -> impl Handler {
    affix_state::inject(container)
}
