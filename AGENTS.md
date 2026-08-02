# AGENTS.md

## Project

Aidoku source extension: a single Rust crate (`comic-source`, edition 2024) compiled to WebAssembly for two sites:
- https://v3.komikcast.fit/
- https://natsu.one/

Source metadata lives in `res/source.json` (id `id.multisource`); keep id/urls/languages in sync with what the code serves.

## Commands

- Build: `cargo build` or `cargo build --release` — builds WASM directly; `.cargo/config.toml` sets the default target to `wasm32-unknown-unknown`, so **no `--target` flag needed for builds**.
- Test: `cargo test` — the default target is wasm, and `.cargo/config.toml` wires the
  `aidoku-test-runner` runner for `wasm32-unknown-unknown`, so tests are compiled to wasm and
  executed by the runner. Tests must use the `#[aidoku_test::aidoku_test]` attribute (not `#[test]`),
  otherwise the runner will not discover them.
- Note: `cargo test --target x86_64-pc-windows-msvc` will fail to link — the aidoku crate imports
  wasm-only host functions that have no x86_64 symbols. Do not use host-target testing.
- Formatting: `cargo fmt`. All code must be rustfmt-compliant.

## Conventions

- The crate is `#![no_std]` (`crate-type = ["cdylib"]`); `String`, `Vec`, etc. must be imported from `alloc` (e.g. `use alloc::vec::Vec;`). Missing imports are the current failure mode of `src/lib.rs`.
- `panic = "abort"` in both dev and release profiles; release uses `opt-level = "s"`, `strip`, `lto`.
- aidoku/aidoku-test are git dependencies pinned by `Cargo.lock`; building requires network access.
- Non-negotiable:
  - No `unwrap()` in production code; no `unsafe`; every error handled via `Result`/`Option`.
  - Never use regex for HTML parsing — CSS selectors only, grouped per site, never duplicated across sites.
  - Never execute site JavaScript; never disable SSL validation; validate URLs and reject invalid responses.
  - Files: prefer <300 lines, hard max 600. Split modules if needed.

## Layout

Sites are independent — a site must never depend on another. Layout:
- `sites/<site>/` — per site: `search.rs`, `manga.rs`, `chapter.rs`, `pages.rs`, `parser.rs`
- `network/` — shared client (timeout, retry, headers, User-Agent, cookies); every request goes through it
- `parser/` — HTML/JSON parsing, separated from networking
- `models/`, `utils/`, `error/`, `source/`

## Testing

Every parser and JSON parser must have tests; every CSS selector must be validated by a test.

## Commits

Small commits; one feature per commit; no unrelated changes.
