#[test]
fn container_macro_expansion_snapshot() {
    macrotest::expand("tests/expand/*.rs");
}
