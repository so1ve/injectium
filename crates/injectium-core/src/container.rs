use std::any::{TypeId, type_name};

use cfg_block::cfg_block;

use crate::provider::Provider;
use crate::types::{AnyDyn, ErasedProvider, SyncBounds};

cfg_block! {
    #[cfg(feature = "validation")] {
        /// A dependency declaration collected at link time via [`inventory`].
        ///
        /// Available when the `validation` feature is enabled.
        ///
        /// Each `#[derive(Injectable)]` struct registers one `DeclaredDependency`
        /// entry per field type. [`Container::validate`] iterates all collected
        /// entries at startup to confirm every required type is present.
        ///
        /// You rarely construct this manually; use `declare_dependency!` instead.
        pub struct DeclaredDependency {
            /// A function pointer returning the [`TypeId`] of the required type.
            /// Stored as a fn pointer rather than a value because [`TypeId`] is not
            /// usable in `const` contexts on stable/nightly without a feature flag.
            pub type_id: fn() -> TypeId,
            /// Human-readable name of the type, produced by `stringify!`.
            pub type_name: &'static str,
        }

        inventory::collect!(DeclaredDependency);
    }
}

struct RegistrationEntry {
    type_id: TypeId,
    provider: Box<ErasedProvider>,
}

#[inline]
fn registrations_contain_type_id<'a>(
    registrations: impl IntoIterator<Item = &'a RegistrationEntry>,
    type_id: TypeId,
) -> bool {
    registrations
        .into_iter()
        .any(|entry| entry.type_id == type_id)
}

/// A runtime dependency-injection container.
///
/// `Container` stores one provider per type.
///
/// A provider can be a closure that constructs a fresh value, or an [`Arc`]
/// that clones and returns shared state. Consumers use [`get`](Container::get)
/// to retrieve an owned value regardless of how that provider is implemented.
///
/// Containers are built through [`ContainerBuilder`] (or the
/// [`container!`](crate::container!) macro) and are typically wrapped in an
/// [`Arc`](std::sync::Arc) for sharing across threads.
///
/// # Example
///
/// ```
/// use injectium_core::{Container, container};
///
/// let c = container! {
///     providers: [|_: &Container| 42_u32, |_: &Container| "hello"],
/// };
///
/// assert_eq!(c.get::<u32>(), 42);
/// assert_eq!(c.get::<&str>(), "hello");
/// ```
pub struct Container {
    registrations: Box<[RegistrationEntry]>,
}

cfg_block! {
    #[cfg(feature = "debug")] {
        use std::fmt;

        impl fmt::Debug for Container {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("Container")
                    .field("providers", &self.provider_count())
                    .finish()
            }
        }

        impl fmt::Display for Container {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "Container ({} providers)",
                    self.provider_count()
                )
            }
        }
    }
}

/// A builder for [`Container`].
///
/// Obtain one via [`Container::builder`] or the
/// [`container!`](crate::container!) macro. All methods take `self` by value
/// and return `Self`, enabling a fluent builder chain. Call
/// [`build`](ContainerBuilder::build) to finalise.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
///
/// use injectium_core::Container;
///
/// let c = Container::builder()
///     .provider(Arc::new(42_u32))
///     .provider(|_: &Container| "hello")
///     .build();
///
/// assert_eq!(c.get::<Arc<u32>>().as_ref(), &42);
/// ```
pub struct ContainerBuilder {
    registrations: Vec<RegistrationEntry>,
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerBuilder {
    /// Creates an empty builder with no registrations.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates an empty builder with preallocated storage capacity for the
    /// expected number of providers.
    ///
    /// These values are only allocation hints. Duplicate registrations are not
    /// allowed, so the final number of stored registrations may still be
    /// smaller than the requested capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            registrations: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    fn contains_type_id(&self, type_id: TypeId) -> bool {
        registrations_contain_type_id(self.registrations.iter(), type_id)
    }

    /// Returns `true` if a provider producing `T` is already registered in this
    /// builder.
    #[must_use]
    pub fn contains<T: 'static>(&self) -> bool {
        self.contains_type_id(TypeId::of::<T>())
    }

    /// Returns `true` if a provider of type `P` would conflict with an existing
    /// registration in this builder.
    #[must_use]
    pub fn contains_provider<P>(&self) -> bool
    where
        P: Provider + SyncBounds,
    {
        self.contains::<P::Output>()
    }

    /// Registers a provider for values of type `T`.
    ///
    /// Providers are automatically supported for closures and [`Arc`] values.
    ///
    /// # Panics
    ///
    /// Panics if a provider for the same output type has already been
    /// registered in this builder.
    #[must_use]
    pub fn provider<P>(self, provider: P) -> Self
    where
        P: Provider + SyncBounds,
    {
        let type_id = TypeId::of::<P::Output>();
        assert!(
            !self.contains_type_id(type_id),
            "provider already registered for `{}`",
            type_name::<P::Output>()
        );

        let provider =
            move |container: &Container| -> Box<AnyDyn> { Box::new(provider.provide(container)) };

        let mut registrations = self.registrations;
        registrations.push(RegistrationEntry {
            type_id,
            provider: Box::new(provider),
        });

        Self { registrations }
    }

    /// Consumes the builder and returns the finished [`Container`].
    #[must_use]
    pub fn build(self) -> Container {
        Container {
            registrations: self.registrations.into_boxed_slice(),
        }
    }
}

impl Container {
    #[inline]
    fn provider_for(&self, type_id: TypeId) -> Option<&ErasedProvider> {
        self.registrations
            .iter()
            .rev()
            .find(|entry| entry.type_id == type_id)
            .map(|entry| entry.provider.as_ref())
    }

    #[inline]
    fn contains_type_id(&self, type_id: TypeId) -> bool {
        registrations_contain_type_id(self.registrations.iter(), type_id)
    }

    #[inline]
    fn cast_owned_unchecked<T: SyncBounds>(owned_erased: Box<AnyDyn>) -> T {
        debug_assert!((*owned_erased).is::<T>());
        let ptr = Box::into_raw(owned_erased).cast::<T>();

        unsafe {
            // SAFETY: `provider::<T>` stores providers under `TypeId::of::<T>()` and
            // each stored provider returns `Box::new(provider.provide(c))` where the
            // concrete value is exactly `T`. `try_get::<T>` uses the same key, so
            // this cast is valid and ownership is preserved when reconstructing the box.
            *Box::from_raw(ptr)
        }
    }

    /// Returns a new [`ContainerBuilder`].
    ///
    /// Equivalent to [`ContainerBuilder::new`].
    #[must_use]
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    /// Returns a new [`ContainerBuilder`] with preallocated storage capacity.
    #[must_use]
    pub fn builder_with_capacity(capacity: usize) -> ContainerBuilder {
        ContainerBuilder::with_capacity(capacity)
    }

    /// Returns the current value for type `T`.
    ///
    /// # Panics
    ///
    /// Panics if no provider of type `T` has been registered. Use
    /// [`try_get`](Container::try_get) for a non-panicking alternative.
    #[must_use]
    pub fn get<T: SyncBounds>(&self) -> T {
        self.try_get().expect("dependency not registered")
    }

    /// Returns the current value for type `T`, or `None` if no provider is
    /// registered.
    #[must_use]
    pub fn try_get<T: SyncBounds>(&self) -> Option<T> {
        let boxed = (self.provider_for(TypeId::of::<T>())?)(self);

        Some(Self::cast_owned_unchecked::<T>(boxed))
    }

    /// Returns `true` if a provider is registered for `T`.
    #[must_use]
    pub fn contains<T: 'static>(&self) -> bool {
        self.contains_type_id(TypeId::of::<T>())
    }

    /// Validates that every dependency declared via `declare_dependency!` is
    /// registered in this container.
    ///
    /// Available when the `validation` feature is enabled.
    ///
    /// Intended to be called once at application startup, immediately after
    /// the container is built. If any declared dependency is absent, the
    /// method panics with a message that lists every missing type by name,
    /// making misconfiguration easy to diagnose.
    ///
    /// `#[derive(Injectable)]` automatically calls `declare_dependency!` for
    /// each field type, so this check covers all structs that use the derive
    /// macro without any manual bookkeeping.
    ///
    /// # Panics
    ///
    /// Panics if one or more declared dependencies are not registered,
    /// printing the names of all missing types.
    #[cfg(feature = "validation")]
    pub fn validate(&self) {
        let mut missing: Vec<&'static str> = Vec::new();

        for dep in inventory::iter::<DeclaredDependency> {
            let type_id = (dep.type_id)();
            let registered = self.contains_type_id(type_id);

            if !registered {
                missing.push(dep.type_name);
            }
        }

        if !missing.is_empty() {
            panic!(
                "Container is missing {} declared dependenc{}: [{}]",
                missing.len(),
                if missing.len() == 1 { "y" } else { "ies" },
                missing.join(", ")
            );
        }
    }

    /// Returns the number of registered providers.
    #[must_use]
    pub fn provider_count(&self) -> usize {
        self.registrations.len()
    }
}
