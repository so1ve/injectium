use injectium_core::{Container, cloned, container};

#[rustfmt::skip]
fn main() {
    let _providers = container! {
        providers: [
            cloned(1_u32),
            |_c: &Container| 2_u64,
            |_c: &Container| 3_u8,
            |_c: &Container| 4_u16,
        ],
    };
}
