#[allow(unused_attributes)]
#[rustversion::attr(not(nightly), ignore = "requires nightly")]
#[cfg_attr(skip_ui_tests, ignore = "disabled by `--cfg=skip_ui_tests`")]
#[cfg_attr(miri, ignore = "incompatible with miri")]
#[test]
fn ui() {
    let check = trybuild::TestCases::new();
    check.compile_fail("tests/ui/*.rs");
    // Tests that require cargo build instead of cargo check
    let build = trybuild::TestCases::new();
    // Having passing cases forces cargo build instead of cargo check
    build.pass("tests/ui_pass/*.rs");
    build.compile_fail("tests/ui_build/*.rs");
}
