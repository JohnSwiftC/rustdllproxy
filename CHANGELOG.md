# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.0] — 2026-05-05

### Added

- `build` subcommand. Run `rustdllproxy build` from the proxy crate directory
  to parse `src/lib.rs`, regenerate the `.def` file so hooked functions stop
  forwarding and unhooked functions still do, and then invoke `cargo build`.
  Replaces the previous manual `.def` edit + force-rebuild dance.
- Original DLL name is recovered, in priority order, from the hook macro's
  first string argument, the `//<dllname>.dll` trailing comment on the
  attribute line, or the existing `.def` forwarding entry.
- Build flags: `--profile <name>` (defaults to `release`), `--no-build` to
  regenerate the `.def` only, and trailing `-- <cargo args>` forwarded to
  `cargo build` verbatim.
- Forced relink: every `build` invocation bumps `src/lib.rs`'s mtime so cargo
  always re-runs the linker against the freshly written `.def`. (Cargo does
  not fingerprint `.def` files on its own.)

### Changed

- **Breaking:** the CLI now uses subcommands. The previous flat invocation
  `rustdllproxy -p <dll> -n <name>` is now `rustdllproxy new -p <dll> -n <name>`.
  The `new` subcommand's flags (`-p`, `-o`, `-n`, `-a`) are otherwise unchanged.

### Migration from 2.x

- Replace `rustdllproxy <args>` with `rustdllproxy new <args>`.
- After implementing hooks, run `rustdllproxy build` from the proxy crate root
  instead of hand-editing the `.def` file and running `cargo build --release`.
- Keep the `//<dllname>.dll` trailing comment that `new` emits next to each
  `#[no_mangle]` — `build` reads it to recover the original DLL name for
  unhooked functions.
