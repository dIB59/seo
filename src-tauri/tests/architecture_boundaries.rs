#[test]
fn bounded_context_public_surface_is_enforced() {
    let test_cases = trybuild::TestCases::new();

    test_cases.pass("tests/trybuild/explicit_context_facade.rs");
    test_cases.compile_fail("tests/trybuild/flat_context_barrel.rs");
    test_cases.compile_fail("tests/trybuild/private_analysis_domain.rs");
}