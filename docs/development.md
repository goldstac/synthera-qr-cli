# Development

## Commands

```sh
cargo build                 # debug build
cargo build --release       # stripped, LTO binary at target/release/syntheraqr
cargo test                  # 11 tests: round-trip decode, colors, SVG, gradients, versions
cargo clippy --all-targets  # must stay at zero warnings
```

## Architecture

| Module | Purpose |
| --- | --- |
| `src/main.rs` | clap CLI definition, orchestration, hex colors, `decode`, `update`, completions, ASCII logo on `--help`/`--version` |
| `src/qr.rs` | QR matrix → RGBA bitmap replicating `qr-code-styling@1.5.0`: neighbor-aware shapes, `SQUARE_MASK`/`DOT_MASK` finders, pixel margins, supersampled Lanczos rendering |
| `src/output.rs` | PNG/JPG/WebP encoding + hand-rolled SVG generator mirroring the raster shapes |
| `src/terminal.rs` | Kitty graphics protocol: detection (env hints + probe), chunked transmission, adaptive sizing |
| `src/block.rs` | Half-block `▀` fallback renderer with 24-bit color |
| `src/update.rs` | Self-update: GitHub API version check, download, sanity check, atomic replace (`ureq`, manual JSON parse) |
| `tests/roundtrip.rs` | Integration tests using `rqrr` to decode generated images |

## Conventions

- Rust edition 2021
- Dependencies: `qrcode`, `image`, `clap`, `clap_complete`, `base64`, `libc`,
  `rqrr`, `ureq`
- No code comments unless they document non-obvious protocol/shape logic
- README + docs stay in sync when flags or behavior change

## Releasing

1. Bump the version in `Cargo.toml`
2. `cargo build --release`
3. `gh release create vX.Y.Z target/release/syntheraqr --title "syntheraqr vX.Y.Z"`
4. The installer and `syntheraqr update` pick it up automatically (`latest`)