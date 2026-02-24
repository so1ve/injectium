//! Procedural macros for Injectium.
//!
//! This crate provides:
//!
//! - `#[derive(Injectable)]` – implements the [`Injectable`] trait for your
//!   structs, automatically pulling each field from the container.

mod inject;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives an implementation of the `injectium_core::Injectable` trait.
///
/// When applied to a named struct, this macro generates:
///
/// 1. An implementation of `from_container(&Container) -> Self` that clones
///    each field from the container using `container.get::<T>()`.
/// 2. An implementation of `try_from_container(&Container) -> Option<Self>`
///    that uses `container.try_get::<T>()` for graceful missing-dependency
///    handling.
/// 3. A `injectium::declare_dependency!` call for each field type, enabling
///    `Container::validate` to check all dependencies at startup.
///
/// # Requirements
///
/// - The struct must be a named struct (not a tuple struct or enum).
/// - All field types must implement `Clone`.
/// - All field types must be `'static`.
///
/// # Example
///
/// ```ignore
/// use injectium::Injectable;
/// use std::sync::Arc;
///
/// #[derive(Clone, Injectable)]
/// struct Database {
///     conn: Arc<Connection>,
/// }
///
/// #[derive(Injectable)]
/// struct Service {
///     db: Database,
/// }
/// ```
#[proc_macro_derive(Injectable)]
pub fn derive_injectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match inject::expand_inject(input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
