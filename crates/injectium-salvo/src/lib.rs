use std::ops::Deref;
use std::sync::Arc;

use cfg_block::cfg_block;
use injectium::Container;
use salvo::extract::{Extractible, Metadata};
use salvo::http::{ParseError, Request};
use salvo::prelude::*;

pub struct Injected<T>(pub T);

impl<T> Deref for Injected<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'ex, T> Extractible<'ex> for Injected<T>
where
    T: injectium::Injectable + Send + Sync + 'static,
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
            T: injectium::Injectable + Send + Sync + 'static,
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
/// ```rust,ignore
/// Router::new()
///     .hoop(inject_container(Arc::new(container)))
///     ...
/// ```
#[must_use]
pub fn inject_container(container: Arc<Container>) -> impl Handler {
    affix_state::inject(container)
}
