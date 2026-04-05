fn main() {
    tauri_build::build();

    // Embed the build date so the app can enforce soft expiry:
    // if build_date > license.expires_at the app still works but shows a renewal banner.
    let build_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    println!("cargo:rustc-env=BUILD_DATE={build_date}");
    println!("cargo:rerun-if-changed=build.rs");
}
