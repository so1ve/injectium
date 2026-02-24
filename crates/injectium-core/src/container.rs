use std::any::{Any, TypeId};
use std::collections::HashMap;

use cfg_block::cfg_block;

cfg_block! {
    if #[cfg(feature = "sync")] {
        type AnyDyn = dyn Any + Send + Sync;
        type Factory = dyn Fn(&Container) -> Box<AnyDyn> + Send + Sync;
        pub trait SyncBounds: Send + Sync + 'static {}
        impl<T: Send + Sync + 'static> SyncBounds for T {}
    } else {
        type AnyDyn = dyn Any;
        type Factory = dyn Fn(&Container) -> Box<AnyDyn>;
        pub trait SyncBounds: 'static {}
        impl<T: 'static> SyncBounds for T {}
    }
}

/// A dependency declaration collected at link time via [`inventory`].
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

/// A runtime dependency-injection container.
///
/// `Container` stores two kinds of registrations:
///
/// - **Singletons** – a pre-built value of type `T`, returned by shared
///   reference on every call to [`get`](Container::get).
/// - **Factories** – a closure that constructs a fresh `T` on each call to
///   [`resolve`](Container::resolve), with access to the container so it can
///   pull its own dependencies.
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
///     singletons: [42_u32],
///     providers:  [|_| "hello"],
/// };
///
/// assert_eq!(*c.get::<u32>(), 42);
/// assert_eq!(c.resolve::<&str>(), "hello");
/// ```
pub struct Container {
    singletons: HashMap<TypeId, Box<AnyDyn>>,
    factories: HashMap<TypeId, Box<Factory>>,
}

cfg_block! {
    #[cfg(feature = "debug")] {
        use std::fmt;

        impl fmt::Debug for Container {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("Container")
                    .field("singletons", &self.singletons.len())
                    .field("factories", &self.factories.len())
                    .finish()
            }
        }

        impl fmt::Display for Container {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "Container ({} singletons, {} factories)",
                    self.singletons.len(),
                    self.factories.len()
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
/// use injectium_core::Container;
///
/// let c = Container::builder()
///     .singleton(42_u32)
///     .factory(|_| "hello")
///     .build();
///
/// assert_eq!(*c.get::<u32>(), 42);
/// ```
pub struct ContainerBuilder {
    singletons: HashMap<TypeId, Box<AnyDyn>>,
    factories: HashMap<TypeId, Box<Factory>>,
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
        Self {
            singletons: HashMap::new(),
            factories: HashMap::new(),
        }
    }

    /// Registers a singleton value of type `T`.
    ///
    /// The value is stored once and returned by shared reference on every
    /// [`Container::get`] call. Registering a second value of the same type
    /// silently replaces the first.
    #[must_use]
    pub fn singleton<T: SyncBounds>(mut self, value: T) -> Self {
        self.singletons.insert(TypeId::of::<T>(), Box::new(value));

        self
    }

    /// Registers a factory closure for type `T`.
    ///
    /// The closure receives a reference to the finished [`Container`] so it
    /// can resolve its own dependencies, and is called on every
    /// [`Container::resolve`] invocation to produce a fresh `T`. Registering
    /// a second factory for the same type silently replaces the first.
    #[must_use]
    pub fn factory<T, F>(mut self, f: F) -> Self
    where
        T: SyncBounds,
        F: Fn(&Container) -> T + SyncBounds,
    {
        let f = move |c: &Container| -> Box<AnyDyn> { Box::new(f(c)) };
        self.factories.insert(TypeId::of::<T>(), Box::new(f));

        self
    }

    /// Consumes the builder and returns the finished [`Container`].
    #[must_use]
    pub fn build(self) -> Container {
        Container {
            singletons: self.singletons,
            factories: self.factories,
        }
    }
}

impl Container {
    /// Returns a new [`ContainerBuilder`].
    ///
    /// Equivalent to [`ContainerBuilder::new`].
    #[must_use]
    pub fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    /// Returns a shared reference to the singleton of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if no singleton of type `T` has been registered. Use
    /// [`try_get`](Container::try_get) for a non-panicking alternative.
    #[must_use]
    pub fn get<T: SyncBounds>(&self) -> &T {
        self.try_get().expect("dependency not registered")
    }

    /// Returns a shared reference to the singleton of type `T`, or `None` if
    /// it is not registered.
    #[must_use]
    pub fn try_get<T: SyncBounds>(&self) -> Option<&T> {
        self.singletons
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Invokes the factory for type `T` and returns the produced value.
    ///
    /// The factory closure receives a reference to this container, so it can
    /// pull any additional dependencies it needs.
    ///
    /// # Panics
    ///
    /// Panics if no factory for type `T` has been registered, or if the
    /// factory's output cannot be downcast to `T`. Use
    /// [`try_resolve`](Container::try_resolve) for a non-panicking
    /// alternative.
    #[must_use]
    pub fn resolve<T: SyncBounds>(&self) -> T {
        self.try_resolve()
            .expect("factory not registered or failed")
    }

    /// Invokes the factory for type `T` and returns the produced value, or
    /// `None` if no factory is registered for `T`.
    #[must_use]
    pub fn try_resolve<T: SyncBounds>(&self) -> Option<T> {
        let factory = self.factories.get(&TypeId::of::<T>())?;
        let boxed = factory(self);

        boxed.downcast().ok().map(|b| *b)
    }

    /// Returns `true` if a singleton **or** factory is registered for `T`.
    #[must_use]
    pub fn contains<T: 'static>(&self) -> bool {
        self.singletons.contains_key(&TypeId::of::<T>())
            || self.factories.contains_key(&TypeId::of::<T>())
    }

    /// Validates that every dependency declared via `declare_dependency!` is
    /// registered in this container.
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
    pub fn validate(&self) {
        let mut missing: Vec<&'static str> = Vec::new();

        for dep in inventory::iter::<DeclaredDependency> {
            let type_id = (dep.type_id)();
            let registered =
                self.singletons.contains_key(&type_id) || self.factories.contains_key(&type_id);

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

    /// Returns the number of registered singletons.
    #[must_use]
    pub fn singleton_count(&self) -> usize {
        self.singletons.len()
    }

    /// Returns the number of registered factories.
    #[must_use]
    pub fn factory_count(&self) -> usize {
        self.factories.len()
    }
}
