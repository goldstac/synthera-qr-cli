# Terminal preview

When stdout is a TTY, syntheraqr renders the QR inline automatically. The
preview strips the quiet zone so the code fills the screen; saved files keep
your `--margin`. A status line (`QR 21×21 modules · ECC M · 260 px`) is
printed to stderr.

## Kitty graphics protocol

Used when the terminal supports it (kitty, Ghostty, WezTerm, iTerm2, Warp,
Terminal.app…). Detection order:

1. `--force-block` → disabled
2. `--force-kitty` → enabled
3. Environment hints (`KITTY_WINDOW_ID`, `WEZTERM_PANE`, `GHOSTTY_RESOURCES_DIR`, …)
4. A live protocol probe with a 150 ms timeout (only when stdin and stdout are TTYs)

The image is transmitted as chunked base64 `ESC_G` escape sequences and sized
to ~75% of the terminal, adapting when you resize the window.

## Half-block fallback

`▀` glyphs with 24-bit truecolor (fg = upper pixel, bg = lower pixel),
run-length optimized. Used everywhere else: Alacritty, xterm, VS Code, tmux,
SSH.

## Overrides

```sh
syntheraqr "hello" --force-kitty   # always use the kitty protocol
syntheraqr "hello" --force-block   # always use half-blocks
syntheraqr "hello" --no-show       # no preview at all
```

## Piping

When stdout is not a TTY, image bytes are written to stdout instead of
previewing:

```sh
syntheraqr "hello" > qr.png
syntheraqr "hello" --stdout --format svg > qr.svg
```