# Install

Linux and macOS:

```sh
curl -fsSL https://syntheraqr.netlify.app/install | bash
```

Installs to `~/.local/bin/syntheraqr`. The installer:

1. Detects your OS (Linux/macOS) and architecture (x86_64/aarch64)
2. Downloads the matching prebuilt binary from GitHub Releases
3. Falls back to a source build (`cargo build --release`) if no prebuilt
   binary exists for your platform
4. Prints a PATH hint if `~/.local/bin` isn't already on your `PATH`

## Environment

| Variable | Effect |
| --- | --- |
| `SYNTHERAQR_DEST` | Install destination instead of `~/.local/bin` |

## Build from source

```sh
cargo build --release
# binary at target/release/syntheraqr
```

Requires a Rust toolchain: <https://rustup.rs>

## Updating

```sh
syntheraqr update --check     # installed vs latest version
syntheraqr update             # download and install the latest version
syntheraqr update --force     # reinstall even if already up to date
```

`update` requires an internet connection. It downloads the new binary,
verifies it runs (`--version`), and atomically replaces the running
executable — safe to run while the binary is in use.

> **Note:** self-update exists in 0.2.0+. If you installed an older version,
> reinstall once with the curl command above, then `syntheraqr update` works
> forever after.