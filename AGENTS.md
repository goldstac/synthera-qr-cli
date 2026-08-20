# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

**syntheraqr** is a CLI QR code generator written in Rust — the terminal sibling
of the web app [SyntheraQr](https://github.com/goldstac/SyntheraQr) (plain
HTML/Tailwind + `qr-code-styling` JS, hosted on GitHub Pages/Netlify at
https://syntheraqr.netlify.app). Feature parity with the web version is a core
goal — the raster **and** SVG rendering replicate `qr-code-styling@1.5.0`'s
drawing algorithm exactly (see `src/qr.rs`).

What it does:

- Generates a QR code from a text argument, stdin, or an interactive prompt
- Full web parity: `--fg`/`--bg` colors, `--fg2` gradient, `--transparent`,
  `--style rounded|square|dots`, `--error L|M|Q|H`, `--margin`, `--size`
- Saves as PNG / JPG / SVG / WebP (format inferred from `-o` extension,
  overridable with `-f`); `--open` opens the file with the system viewer
- Inline terminal preview via the **kitty graphics protocol** with a
  half-block (`▀`) ANSI truecolor fallback
- `syntheraqr decode <image>` reads a QR back to text (rqrr)
- `syntheraqr update [--check] [--force]` self-updates: fetches the latest
  release tag from the GitHub API, downloads the matching `syntheraqr-<os>-<arch>`
  asset, sanity-checks it with `--version`, and atomically `rename`s it over
  the running binary (`src/update.rs`)
- `--completions bash|zsh|fish|powershell|elvish` prints shell completions
- Unix-filter style: when stdout is not a TTY, image bytes are written to
  stdout instead of previewing
- Installable on Linux/macOS via `curl -fsSL https://syntheraqr.netlify.app/install | bash`
  (script lives in the web repo `../SyntheraQr/install.sh`; prebuilt binaries
  are GitHub Release assets named `syntheraqr-<os>-<arch>`)

## Commands

```sh
cargo build                 # debug
cargo build --release       # stripped binary: target/release/syntheraqr
cargo test                  # integration tests in tests/roundtrip.rs
cargo clippy --all-targets  # must be zero warnings
```

Keep clippy at 0 warnings and all tests green before finishing any change.

## Project layout

- `src/main.rs` — clap CLI definition, orchestration, hex color parsing,
  `decode` subcommand, shell completions, `--open`/`--quiet` handling
- `src/lib.rs` — re-exports the library modules (used by integration tests)
- `src/qr.rs` — QR matrix → styled RGBA bitmap replicating the
  `qr-code-styling@1.5.0` drawing algorithm. `render` supersamples the canvas
  (≤4608 px, 16× at 260 px) and Lanczos-downscales, approximating the
  browser's anti-aliasing. Geometry matches the JS library exactly:
  `dot_size = floor((size - margin*2) / count)` with leftover pixels centered
  (**margin is in pixels**, not modules — the web app uses 4). Styles:
  `Rounded` (neighbor-aware shapes: isolated → circle, 1 neighbor → pill,
  2 adjacent neighbors → corner-cut square, else square), `Dots` (circles
  everywhere), `Square` (rectangles). Finder patterns are drawn module-by-
  module from the `SQUARE_MASK`/`DOT_MASK` tables copied from QRCanvas.ts —
  keep these tables and the `module_shape`/`covered` logic in sync with the
  JS if it ever changes. `--fg2` adds a diagonal fg→fg2 gradient per module
  (`module_color`).
- `src/output.rs` — PNG/JPG/WebP encoding (`image` crate; JPG flattens alpha
  onto white) and hand-rolled SVG generator using the same `module_shape`
  shapes as the raster (paths for pills/corners, circles, rects;
  `userSpaceOnUse` linearGradient for `--fg2` spanning the matrix area)
- `src/terminal.rs` — kitty graphics protocol: support detection (env hints
  like `KITTY_WINDOW_ID`/`WEZTERM_PANE`/`GHOSTTY_RESOURCES_DIR`, then a live
  `ESC_G a=q` probe with 150 ms termios/poll timeout) and chunked transmission
  (`m=1`/`m=0`, ≤4096-byte base64 chunks, `q=2`). Display sizing fills ~75%
  of the terminal assuming 2:1 cells.
- `src/block.rs` — half-block fallback renderer (upper pixel = `38;2` fg,
  lower = `48;2` bg, run-length grouped, `▀` glyphs, Triangle downscale)
- `src/update.rs` — self-update: GitHub API version check, download, sanity
  check (`--version`), atomic self-replacement via `rename` (Unix). Adds
  `ureq` (HTTP) and small manual JSON parse — no serde. Update happens only
  when the installed binary supports it (0.2.0+); older installs must
  reinstall once via the installer.
- `tests/roundtrip.rs` — decode generated images with `rqrr` and assert the
  content round-trips; also color/margin/alpha/SVG/gradient/layout checks

## Key behaviors (don't break these)

- **Preview strips the quiet zone**: the terminal preview re-renders with
  `margin = 0` so the QR fills the display; saved files keep the user's margin
  (default 4 pixels). Kitty preview renders at 512 px; the block fallback
  renders at 2× terminal columns (then Triangle-downscales for cleaner edges).
  Keep these separate from the saved image.
- **Rounded/dots style = neighbor-aware shapes + finder modules** in both
  raster and SVG: the three 7×7 finders and their 3×3 center dots are drawn
  module-by-module from the `SQUARE_MASK`/`DOT_MASK` tables using the same
  dot style, exactly like the web engine (do NOT draw them as plain squares —
  that diverges from the website).
- **Kitty detection order**: `--force-block` → false, `--force-kitty` → true,
  env hints, then the protocol probe. Probe only runs when stdin AND stdout
  are TTYs (uses termios raw mode, restores it after).
- **`--fg2` gradient**: raster colors per module along the diagonal
  `(mx+my)/(2(n-1))`; SVG uses one `userSpaceOnUse` linearGradient spanning the
  matrix bbox (margin excluded) so both stay in sync.
- **Margin is in pixels** (like `qr-code-styling`), not modules. Layout:
  `dot_size = floor((size - 2*margin) / count)`, `offset = floor((size -
  count*dot_size) / 2)`. `render` returns `Result` and errors with "the
  canvas is too small" when `count > size` or `dot_size == 0` (web parity).
- Default colors mirror the web app: fg `#0f172a`, bg `#f8fafc`; default ECC
  Medium, size 260, margin 4.
- `qrcode::QrCode` is indexed with `(usize, usize)` and returns
  `qrcode::Color` — compare with `== Color::Dark`, there is no `is_dark()`
  method on `Color`.
- Interactive prompt: when no TEXT is given and stdin is a TTY, the tool
  prompts "Text to encode:" and reads one line (canonical mode).

## Conventions

- Rust edition 2021. Dependencies: `qrcode`, `image` (png/jpeg/webp
  features), `clap` (derive), `clap_complete`, `base64`, `libc`, `rqrr`
  (decode subcommand and tests), `ureq` (self-update).
- No code comments unless they document non-obvious protocol/shape logic
  (existing comments follow this rule).
- README.md documents the CLI surface and terminal-support matrix — update it
  if flags or behavior change.

## Social / announcements

- Announce significant releases and website updates as tweets (drafted for the
  user to post). Style: punchy opener, what's new in bullet form, a usage/
  install one-liner, hashtags (#OpenSource #Rust #QRCode #DevTools
  #Terminal). Keep the tone energetic but factual.
- **Current announcement draft (v0.2.0 + website update):**

  > syntheraqr v0.2.0 is out! 🎉
  >
  > The terminal QR generator that matches the syntheraqr.netlify.app
  > engine just learned how to update itself:
  >
  > ✨ `syntheraqr update` — self-updates from GitHub Releases
  > ✨ `update --check` — see installed vs latest version
  > ✨ `update --force` — reinstall any time
  >
  > Plus the website got a full Docs page for the CLI (install, usage,
  > options, terminal preview, offline) and an expanded CLI section on the
  > homepage.
  >
  > 📦 `curl -fsSL https://syntheraqr.netlify.app/install | bash`
  >
  > 100% offline for generating/previewing/saving — only update + install
  > need the internet.
  >
  > #OpenSource #Rust #QRCode #DevTools #Terminal

- After posting, replace the draft with a short "posted" note (link if the
  user shares one).

## Related resources

- Web repo: https://github.com/goldstac/SyntheraQr (context source for look &
  feature parity)
- Kitty graphics protocol spec: https://sw.kovidgoyal.net/kitty/graphics-protocol/
