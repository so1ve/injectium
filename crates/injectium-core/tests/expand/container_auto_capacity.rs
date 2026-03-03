use injectium_core::container;

#[rustfmt::skip]
fn main() {
    let _both = container! {
        singletons: [1_u32, 2_u64],
        providers: [|_c| 3_u8, |_c| 4_u16],
    };

    let _singletons = container! {
        singletons: [1_u32, 2_u64],
    };

    let _providers = container! {
        providers: [|_c| 3_u8, |_c| 4_u16],
    };
}
