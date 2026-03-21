fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let go_version_path = std::path::Path::new(&manifest_dir).join("go-version");

    let go_version = std::fs::read_to_string(&go_version_path).expect("failed to read go-version");
    let go_version = go_version.trim();
    assert!(
        go_version.chars().all(|c| c.is_ascii_digit() || c == '.'),
        "go-version contains invalid characters: {go_version}"
    );

    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        format!("{out_dir}/go_version.rs"),
        format!("pub const GO_VERSION: &str = \"{go_version}\";"),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=go-version");
}
