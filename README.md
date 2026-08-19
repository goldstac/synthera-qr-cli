# SyntheraQr CLI

Ultra-modern QR code generator for the terminal — the CLI sibling of
[SyntheraQr](https://github.com/goldstac/SyntheraQr). Generate, customize,
preview, and save QR codes entirely from the command line.

- **Same engine as the website**: the saved images are rendered with the exact
  drawing algorithm of [`qr-code-styling`](https://github.com/kozakdenys/qr-code-styling)
  1.5.0 (the library the web app uses) — rounded-corner modules, pills,
  finder patterns, margins, everything
- Inline **terminal preview** via the [kitty graphics
  protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) with a
  half-block ANSI fallback for other terminals
- **Full parity** with the web tool: foreground/background colors,
  transparent background, rounded/square/dots dot styles, error correction
  levels, quiet-zone margin
- Save as **PNG, JPG, SVG, or WebP** (all rendered locally, no network)

## Install

Linux and macOS:

```sh
curl -fsSL https://syntheraqr.netlify.app/install | bash
```

Installs to `~/.local/bin/syntheraqr` (prebuilt binary from GitHub
Releases, falling back to a source build if no binary matches your system).

## Build

```sh
cargo build --release
# binary at target/release/syntheraqr
```

## Usage

```sh
# Save a QR to a file (format from the extension)
syntheraqr "https://syntheraqr.site" -o qr.png

# Customize like the web version
syntheraqr "https://syntheraqr.site" --fg "#0f172a" --bg "#f8fafc" --transparent -o qr.webp

# Rounded dots (default), crisp squares, or clean dots
syntheraqr "hello" --style square --fg "#16a34a" -o hello.png
syntheraqr "hello" --style dots --fg "#e11d48" -o dots.png

# Diagonal gradient (fg → fg2)
syntheraqr "grad" --fg "#0f172a" --fg2 "#6366f1" -o grad.svg

# Decode a QR code back to text
syntheraqr decode qr.png

# Open the saved file in the system viewer
syntheraqr "hello" -o qr.png --open

# Interactive prompt when no text is given (TTY)
syntheraqr

# Shell completions (bash/zsh/fish/powershell/elvish)
syntheraqr --completions zsh > _syntheraqr

# Show an inline preview in the terminal (default when stdout is a TTY)
syntheraqr "hello" --size 200

# Pipe output to files or other tools (unix-filter style)
echo "https://example.com" | syntheraqr --stdout > qr.png
syntheraqr "data" --stdout --format svg > qr.svg
```

### Options

| Option | Description | Default |
| --- | --- | --- |
| `TEXT` | Text or URL to encode (or pipe via stdin) | — |
| `--fg <HEX>` | Foreground (dot) color | `#0f172a` |
| `--fg2 <HEX>` | Secondary color → diagonal gradient | off |
| `--bg <HEX>` | Background color | `#f8fafc` |
| `--transparent` | Transparent background | off |
| `--style rounded\|square\|dots` | Dot style | `rounded` |
| `-e, --error L\|M\|Q\|H` (`--ecc`) | Error correction level | `M` |
| `-s, --size <PX>` | Output size in pixels | `260` |
| `-m, --margin <PX>` | Quiet zone in pixels (matches the web app) | `4` |
| `-o, --output <FILE>` | Save to file (png/jpg/jpeg/svg/webp) | — |
| `-f, --format <FMT>` | Force output format | inferred |
| `--stdout` | Write image bytes to stdout | — |
| `--no-show` | Skip the terminal preview | off |
| `--open` | Open the saved file with the system viewer | off |
| `-q, --quiet` | Suppress informational messages | off |
| `--completions <SHELL>` | Print shell completion script | — |
| `--force-kitty` | Force kitty graphics preview | — |
| `--force-block` | Force half-block character preview | — |

## Terminal preview

When stdout is a TTY, the QR is rendered inline automatically. The preview
**strips the quiet zone** so the code fills the screen; saved files keep your
`--margin`. A status line (size, ECC, modules) is printed to stderr. Both the
preview and the saved files come from the same rendering engine as the web
app — the preview is just rendered at a smaller resolution.

1. **Kitty graphics protocol** — used when supported (kitty, Ghostty, WezTerm,
   iTerm2, Warp, Terminal.app…). Detected via environment hints plus a live
   protocol query with a 150 ms timeout. Sizes itself to ~75% of the terminal
   and adapts when you resize the window.
2. **Half-block fallback** — `▀` glyphs with 24-bit truecolor (fg = upper
   pixel, bg = lower pixel), used everywhere else (Alacritty, xterm, VS Code,
   tmux, SSH).

Use `--force-kitty` / `--force-block` to override detection. When stdout is
not a TTY, image bytes are written to stdout instead (no preview).

## Decoding

```sh
syntheraqr decode qr.png          # prints the encoded text
syntheraqr decode qr.png | wc -c   # pipe it like any text
```

## Tests

```sh
cargo test      # round-trip decode (rqrr), colors, transparency, SVG structure, gradients
cargo clippy    # zero warnings
```

## License

MIT — see [LICENSE](LICENSE). Built by [LI Productions](https://linktr.ee/liproductions).