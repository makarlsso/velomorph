#[test]
fn malformed_morph_attributes_fail_with_clear_errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/malformed_*.rs");
}
