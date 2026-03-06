use injectium_core::{Container, cloned, container};
#[rustfmt::skip]
fn main() {
    let _providers = {
        ::injectium_core::Container::builder_with_capacity(
                <[()]>::len(&[(), (), (), ()]),
            )
            .provider(cloned(1_u32))
            .provider(|_c: &Container| 2_u64)
            .provider(|_c: &Container| 3_u8)
            .provider(|_c: &Container| 4_u16)
            .build()
    };
}
