#[test]
fn bounded_context_public_surface_is_enforced() {
    let test_cases = trybuild::TestCases::new();

    // ── Should pass: proper facade paths ────────────────────────────
    test_cases.pass("tests/trybuild/explicit_context_facade.rs");
    test_cases.pass("tests/trybuild/explicit_tags_facade.rs");
    test_cases.pass("tests/trybuild/explicit_template_facade.rs");

    // ── Should fail: barrel re-exports / internal access ────────────
    test_cases.compile_fail("tests/trybuild/flat_context_barrel.rs");
    test_cases.compile_fail("tests/trybuild/private_analysis_domain.rs");
    test_cases.compile_fail("tests/trybuild/private_tags_context.rs");
    test_cases.compile_fail("tests/trybuild/private_template_internals.rs");
    test_cases.compile_fail("tests/trybuild/private_checker_custom.rs");
    test_cases.compile_fail("tests/trybuild/private_template_repo_impl.rs");
}

#[test]
fn top_level_extension_module_is_absent() {
    let lib_rs = std::fs::read_to_string("src/lib.rs").expect("failed to read src/lib.rs");

    assert!(
        !std::path::Path::new("src/extension").exists(),
        "top-level src/extension directory should not exist"
    );
    assert!(
        !lib_rs.contains("pub mod extension;"),
        "src/lib.rs should not re-export a top-level extension module"
    );
}
