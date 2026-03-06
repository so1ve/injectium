use std::sync::Arc;

use crate::container::Container;
use crate::types::SyncBounds;

/// A source that can provide values from a [`Container`].
pub trait Provider: 'static {
    /// The value produced by this provider.
    type Output: SyncBounds;

    /// Produces a value using the current container.
    fn provide(&self, container: &Container) -> Self::Output;
}

impl<T: SyncBounds> Provider for Arc<T> {
    type Output = Arc<T>;

    fn provide(&self, _container: &Container) -> Self::Output {
        Arc::clone(self)
    }
}

/// An explicit provider that clones a stored value on each request.
pub struct CloneProvider<T>(T);

impl<T> CloneProvider<T> {
    /// Wraps a cloneable value as an explicit clone-on-read provider.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Provider for CloneProvider<T>
where
    T: SyncBounds + Clone,
{
    type Output = T;

    fn provide(&self, _container: &Container) -> Self::Output {
        self.0.clone()
    }
}

/// Wraps a cloneable value as an explicit clone-on-read provider.
#[must_use]
pub fn cloned<T>(value: T) -> CloneProvider<T>
where
    T: SyncBounds + Clone,
{
    CloneProvider::new(value)
}

/// An explicit provider that copies a stored value on each request.
pub struct CopyProvider<T>(T);

impl<T> CopyProvider<T> {
    /// Wraps a copyable value as an explicit copy-on-read provider.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Provider for CopyProvider<T>
where
    T: SyncBounds + Copy,
{
    type Output = T;

    fn provide(&self, _container: &Container) -> Self::Output {
        self.0
    }
}

/// Wraps a copyable value as an explicit copy-on-read provider.
#[must_use]
pub fn copied<T>(value: T) -> CopyProvider<T>
where
    T: SyncBounds + Copy,
{
    CopyProvider::new(value)
}

impl<T, F> Provider for F
where
    T: SyncBounds,
    F: Fn(&Container) -> T + SyncBounds,
{
    type Output = T;

    fn provide(&self, container: &Container) -> Self::Output {
        self(container)
    }
}
