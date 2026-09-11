fn main() {
    tauri_build::build();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Tauri's app manifest does not cover Cargo integration-test executables.
        // Their desktop dependencies also need Common Controls v6 to load.
        let manifest = std::path::PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"),
        )
        .join("windows-tests.manifest");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}
