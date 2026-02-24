use crate::Container;

/// Types that can construct themselves from a [`Container`].
///
/// Implement this trait (or derive it with `#[derive(Injectable)]`) to allow
/// a type to be assembled by pulling its dependencies out of a [`Container`].
///
/// # Deriving
///
/// The `#[derive(Injectable)]` macro implements both methods for any named
/// struct whose fields all implement `Clone` and are registered in the
/// container. It also emits a [`declare_dependency!`] call for every field
/// type so that [`Container::validate`] can catch missing registrations at
/// startup.
pub trait Injectable: Sized {
    /// Constructs `Self` by cloning each field's value out of the container.
    ///
    /// # Panics
    ///
    /// Panics if any required dependency is not registered. Prefer
    /// [`try_from_container`](Injectable::try_from_container) if you need
    /// graceful handling of missing dependencies.
    fn from_container(container: &Container) -> Self;

    /// Attempts to construct `Self` from the container, returning `None` if
    /// any required dependency is absent.
    fn try_from_container(container: &Container) -> Option<Self>;
}
