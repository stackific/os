# Shared Rust settings. Import this from any module that builds or lints Rust code (installer/).

# Pinned rustup (static.rust-lang.org/rustup/archive/<version>/<target>/rustup-init.sha256, read 2026-11-06)
rustup_version := "1.29.3"
rustup_target := if os() == "macos" { arch() + "-apple-darwin" } else { arch() + "-unknown-linux-gnu" }
rustup_sha256 := if rustup_target == "aarch64-apple-darwin" { "63752a1919a3a4793173ec70f8f212621a7ce38ba6a246095984482f4de20f9f" } else if rustup_target == "aarch64-unknown-linux-gnu" { "9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282" } else { "b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55" }

# The toolchain: the Rust release Arch and Fedora 45 package, so `just installer test` builds what the packages build
rust_version := "1.99.1"

rust_dir := justfile_directory() / ".tools" / "rust"
rustup_init := rust_dir / "rustup-init"
rustup_home := rust_dir / "rustup"
cargo_home := rust_dir / "cargo"
rustup := cargo_home / "bin" / "rustup"
cargo := cargo_home / "bin" / "cargo"

# Keeps everything rustup and cargo write under .tools/, on the pinned toolchain: `{{ rust_env }} '{{ cargo }}' …`
rust_env := "env RUSTUP_HOME='" + rustup_home + "' CARGO_HOME='" + cargo_home + "' RUSTUP_TOOLCHAIN=" + rust_version
