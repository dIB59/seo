#[test]
fn bounded_context_public_surface_is_enforced() {
    let test_cases = trybuild::TestCases::new();

    test_cases.pass("tests/trybuild/explicit_context_facade.rs");
    test_cases.compile_fail("tests/trybuild/flat_context_barrel.rs");
    test_cases.compile_fail("tests/trybuild/private_analysis_domain.rs");
    test_cases.compile_fail("tests/trybuild/removed_top_level_extension.rs");
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
