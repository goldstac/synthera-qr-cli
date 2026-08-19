# SyntheraQr CLI

Ultra-modern QR code generator for the terminal — the CLI sibling of
[SyntheraQr](https://github.com/goldstac/SyntheraQr). Generate, customize,
preview, and save QR codes entirely from the command line.

- Inline **terminal preview** via the [kitty graphics
  protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) with a
  half-block ANSI fallback for other terminals
- **Full parity** with the web tool: foreground/background colors,
  transparent background, rounded or square dot style, error correction
  levels, quiet-zone margin
- Save as **PNG, JPG, SVG, or WebP** (all rendered locally, no network)

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

# Rounded dots (default) or crisp squares
syntheraqr "hello" --style square --fg "#16a34a" -o hello.png

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
| `--bg <HEX>` | Background color | `#f8fafc` |
| `--transparent` | Transparent background | off |
| `--style rounded\|square` | Dot style | `rounded` |
| `--error L\|M\|Q\|H` (`--ecc`) | Error correction level | `M` |
| `-s, --size <PX>` | Output size in pixels | `260` |
| `-m, --margin <MOD>` | Quiet zone in modules | `4` |
| `-o, --output <FILE>` | Save to file (png/jpg/jpeg/svg/webp) | — |
| `-f, --format <FMT>` | Force output format | inferred |
| `--stdout` | Write image bytes to stdout | — |
| `--no-show` | Skip the terminal preview | off |
| `--force-kitty` | Force kitty graphics preview | — |
| `--force-block` | Force half-block character preview | — |

## Terminal preview

When stdout is a TTY, the QR is rendered inline automatically:

1. **Kitty graphics protocol** — used when supported (kitty, Ghostty, WezTerm,
   iTerm2, Warp, Terminal.app…). Detected via environment hints plus a live
   protocol query with a 150 ms timeout.
2. **Half-block fallback** — `▀` glyphs with 24-bit truecolor (fg = upper
   pixel, bg = lower pixel), used everywhere else (Alacritty, xterm, VS Code,
   tmux, SSH).

Use `--force-kitty` / `--force-block` to override detection. When stdout is
not a TTY, image bytes are written to stdout instead (no preview).

## Tests

```sh
cargo test      # round-trip decode (rqrr), colors, transparency, SVG structure
cargo clippy    # zero warnings
```

## License

MIT — see [LICENSE](LICENSE). Built by [LI Productions](https://linktr.ee/liproductions).