# Contributing

Please open an issue before proposing substantial emulator behavior changes. Keep pull requests focused and include tests for emulation behavior or ROM-loading changes.

Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Do not commit commercial ROMs, extracted ROM archives, generated binaries, or IDE metadata.
