use injectium_core::container;
#[rustfmt::skip]
fn main() {
    let _both = {
        ::injectium_core::Container::builder_with_capacity(
                <[()]>::len(&[(), ()]),
                <[()]>::len(&[(), ()]),
            )
            .singleton(1_u32)
            .singleton(2_u64)
            .factory(|_c| 3_u8)
            .factory(|_c| 4_u16)
            .build()
    };
    let _singletons = {
        ::injectium_core::Container::builder_with_capacity(<[()]>::len(&[(), ()]), 0)
            .singleton(1_u32)
            .singleton(2_u64)
            .build()
    };
    let _providers = {
        ::injectium_core::Container::builder_with_capacity(0, <[()]>::len(&[(), ()]))
            .factory(|_c| 3_u8)
            .factory(|_c| 4_u16)
            .build()
    };
}
