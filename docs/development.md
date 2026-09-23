# Development

## Commands

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All four must pass. Warnings get fixed at the cause, never silenced with `allow`. The suite is 222 tests: unit tests inside `src` plus integration suites in `tests/` that build synthetic families and run the real `example.ged` and `demo.ged` files.

## Coverage

No coverage tool is vendored, but `llvm-cov` works with the stock toolchain:

```sh
rm -f cov-*.profraw
RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE="$PWD/cov-%p-%m.profraw" cargo test
llvm-profdata merge -sparse cov-*.profraw -o coverage.profdata
llvm-cov report --instr-profile=coverage.profdata --object <test-binary> --summary-only
```

List one `--object` per binary in `target/debug/deps`. Last measured totals were about 90% regions and functions and 89% lines. `src/main.rs` sits at zero because integration tests drive the library, not the binary. That is normal and not a gap worth closing.

## Release profile

`Cargo.toml` tunes the release build for size: `opt-level = "z"`, full LTO, one codegen unit, `panic = "abort"`, stripped symbols. A release build of the CLI lands around 420 KB with no dependencies to bundle.

```sh
cargo build --release
```

## House rules

- No comments in source. Names carry the meaning.
- No dependencies. The crate is standard library only.
- Keep files near 100 to 200 lines. Split by topic when one grows past that.
- `cargo clean` when you are done. The `target` directory is gitignored and should stay small.
