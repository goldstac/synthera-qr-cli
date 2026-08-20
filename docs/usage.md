# Usage

## Quick start

```sh
syntheraqr "https://syntheraqr.site"                    # inline terminal preview
syntheraqr "https://syntheraqr.site" -o qr.png          # save as PNG
syntheraqr "hello" --style square -o hello.jpg          # JPG
syntheraqr "hello" -o hello.svg --style dots            # SVG
syntheraqr "hello" -o hello.webp --transparent          # WebP
```

## Input

Text comes from (in order of precedence):

1. The positional `TEXT` argument: `syntheraqr "hello"`
2. Piped stdin: `echo "hello" | syntheraqr`
3. An interactive prompt (`Text to encode:`) when run bare in a terminal

## Options

| Option | Description | Default |
| --- | --- | --- |
| `--fg <HEX>` | Foreground (dot) color | `#0f172a` |
| `--fg2 <HEX>` | Secondary color → diagonal gradient | off |
| `--bg <HEX>` | Background color | `#f8fafc` |
| `--transparent` | Transparent background | off |
| `--style rounded\|square\|dots` | Dot style | `rounded` |
| `-e, --error L\|M\|Q\|H` | Error correction level | `M` |
| `-s, --size <PX>` | Output size in pixels | `260` |
| `-m, --margin <PX>` | Quiet zone in pixels | `4` |
| `-o, --output <FILE>` | Save to file (png/jpg/jpeg/svg/webp) | — |
| `-f, --format <FMT>` | Force output format | inferred |
| `--stdout` | Write image bytes to stdout | — |
| `--no-show` | Skip the terminal preview | off |
| `--open` | Open the saved file in the system viewer | off |
| `-q, --quiet` | Suppress informational messages | off |
| `--completions <SHELL>` | Print shell completion script | — |
| `--force-kitty` | Force kitty graphics preview | — |
| `--force-block` | Force half-block character preview | — |

## Commands

### Decode

Read a QR image back to text:

```sh
syntheraqr decode qr.png
syntheraqr decode qr.jpg | wc -c
```

### Update

Self-update from GitHub Releases — see [Install](install.md#updating).

### Completions

```sh
syntheraqr --completions bash > /etc/bash_completion.d/syntheraqr
syntheraqr --completions zsh > _syntheraqr
syntheraqr --completions fish > syntheraqr.fish
```

## Examples

```sh
# Brand-colored QR
syntheraqr "https://syntheraqr.site" --fg "#0f172a" --bg "#f8fafc"

# Gradient
syntheraqr "grad" --fg "#0f172a" --fg2 "#6366f1" -o grad.svg

# Unix-filter style
echo "https://example.com" | syntheraqr --stdout > qr.png

# Quiet, no preview, saved
syntheraqr "data" -o qr.png --no-show --quiet
```