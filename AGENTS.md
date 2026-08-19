# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

**syntheraqr** is a CLI QR code generator written in Rust — the terminal sibling
of the web app [SyntheraQr](https://github.com/goldstac/SyntheraQr) (plain
HTML/Tailwind + `qr-code-styling` JS, hosted on GitHub Pages). Feature parity
with the web version is a core goal.

What it does:

- Generates a QR code from a text argument or stdin
- Full web parity: `--fg`/`--bg` colors, `--transparent`, `--style
  rounded|square`, `--error L|M|Q|H`, `--margin`, `--size`
- Saves as PNG / JPG / SVG / WebP (format inferred from `-o` extension,
  overridable with `-f`)
- Inline terminal preview via the **kitty graphics protocol** with a
  half-block (`▀`) ANSI truecolor fallback
- Unix-filter style: when stdout is not a TTY, image bytes are written to
  stdout instead of previewing

## Commands

```sh
cargo build                 # debug
cargo build --release       # stripped binary: target/release/syntheraqr
cargo test                  # integration tests in tests/roundtrip.rs
cargo clippy --all-targets  # must be zero warnings
```

Keep clippy at 0 warnings and all tests green before finishing any change.

## Project layout

- `src/main.rs` — clap CLI definition, orchestration, hex color parsing
- `src/lib.rs` — re-exports the library modules (used by integration tests)
- `src/qr.rs` — QR matrix → styled RGBA bitmap. `render_square` is direct
  per-pixel; `render_rounded` supersamples 8× (circles + orthogonal connector
  bars), premultiplied-alpha Lanczos downscale to target size. The three 7×7
  **finder patterns are kept square** (matching the web version's default
  cornersSquare style) — do not round them or decoding breaks.
- `src/output.rs` — PNG/JPG/WebP encoding (`image` crate; JPG flattens alpha
  onto white) and hand-rolled SVG generator (mirrors the raster shapes exactly)
- `src/terminal.rs` — kitty graphics protocol: support detection (env hints
  like `KITTY_WINDOW_ID`/`WEZTERM_PANE`/`GHOSTTY_RESOURCES_DIR`, then a live
  `ESC_G a=q` probe with 150 ms termios/poll timeout) and chunked transmission
  (`m=1`/`m=0`, ≤4096-byte base64 chunks, `q=2`). Display sizing fills ~75%
  of the terminal assuming 2:1 cells.
- `src/block.rs` — half-block fallback renderer (upper pixel = `38;2` fg,
  lower = `48;2` bg, run-length grouped, `▀` glyphs)
- `tests/roundtrip.rs` — decode generated images with `rqrr` and assert the
  content round-trips; also color/margin/alpha/SVG-structure checks

## Key behaviors (don't break these)

- **Preview strips the quiet zone**: the terminal preview re-renders with
  `margin = 0` and `size = 512` so the QR fills the display; saved files keep
  the user's margin (default 4 modules). Keep these separate.
- **Rounded style = round dots + square finders** in both raster and SVG.
- **Kitty detection order**: `--force-block` → false, `--force-kitty` → true,
  env hints, then the protocol probe. Probe only runs when stdin AND stdout
  are TTYs (uses termios raw mode, restores it after).
- Default colors mirror the web app: fg `#0f172a`, bg `#f8fafc`; default ECC
  Medium, size 260, margin 4.
- `qrcode::QrCode` is indexed with `(usize, usize)` and returns
  `qrcode::Color` — compare with `== Color::Dark`, there is no `is_dark()`
  method on `Color`.

## Conventions

- Rust edition 2021. Dependencies: `qrcode`, `image` (png/jpeg/webp
  features), `clap` (derive), `base64`, `libc`; dev-dep `rqrr`.
- No code comments unless they document non-obvious protocol/shape logic
  (existing comments follow this rule).
- README.md documents the CLI surface and terminal-support matrix — update it
  if flags or behavior change.

## Related resources

- Web repo: https://github.com/goldstac/SyntheraQr (context source for look &
  feature parity)
- Kitty graphics protocol spec: https://sw.kovidgoyal.net/kitty/graphics-protocol/
