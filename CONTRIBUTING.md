# Contributing

Please open an issue before proposing substantial emulator behavior changes. Keep pull requests focused and include tests for emulation behavior or ROM-loading changes.

## Workspace package metadata

The shared version, edition, minimum Rust version, and license are defined in
`[workspace.package]` in the root `Cargo.toml`. Member crates inherit these values
with `version.workspace = true`, `edition.workspace = true`,
`rust-version.workspace = true`, and `license.workspace = true`. Use the same
convention when adding a crate. Crate-specific descriptions and other metadata
belong in the member manifest.

## Checks

Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not commit commercial ROMs, extracted ROM archives, generated binaries, or IDE metadata.
