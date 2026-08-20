# Offline & network

## Works fully offline

Everything you'd use day-to-day is computed locally by the binary — no
internet, no external services, no CDN assets, no runtime fonts:

- QR generation (encoding)
- Rendering — all styles, colors, gradients, finder patterns
- Anti-aliasing (supersampling + Lanczos downscale)
- PNG / JPG / SVG / WebP encoding
- Terminal preview (kitty protocol + half-block fallback)
- `decode` — reading QR images back to text
- Shell completions, `--help`, `--version`

## Needs the internet

| Command | Why |
| --- | --- |
| `syntheraqr update` | Fetches the latest version from the GitHub API and downloads the new binary |
| Installing (`curl …/install \| bash`) | Downloads the installer and the binary |

That's it. Generate, style, save, preview, and decode all work on a plane,
in a bunker, or behind a firewall.