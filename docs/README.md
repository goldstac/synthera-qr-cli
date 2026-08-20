# SyntheraQr CLI — Docs

Ultra-modern QR code generator for the terminal. The CLI sibling of
[SyntheraQr](https://github.com/goldstac/SyntheraQr) — the same rendering
engine, right in your shell.

## Getting started

- [Install](install.md) — one-line install, source builds, updating
- [Usage](usage.md) — all commands, flags, and examples
- [Terminal preview](preview.md) — kitty graphics protocol & half-block fallback
- [Offline & network](offline.md) — what works without the internet

## Development

- [Build & test](development.md) — cargo commands, architecture, tests

## Quick reference

```sh
syntheraqr "text"                 # inline preview
syntheraqr "text" -o qr.png       # save as PNG (also JPG/SVG/WebP)
syntheraqr decode qr.png          # read a QR back to text
syntheraqr update                 # self-update from GitHub Releases
curl -fsSL https://syntheraqr.netlify.app/install | bash   # install
```