use std::io::Read;
use std::process::Command;

const RELEASE_API: &str = "https://api.github.com/repos/goldstac/synthera-qr-cli/releases/latest";
const DOWNLOAD_BASE: &str = "https://github.com/goldstac/synthera-qr-cli/releases/latest/download";

fn agent() -> String {
    format!("syntheraqr-update/{}", env!("CARGO_PKG_VERSION"))
}

/// Fetch the latest release tag name from the GitHub API.
fn latest_tag() -> Result<String, String> {
    let body = ureq::get(RELEASE_API)
        .set("User-Agent", &agent())
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(15))
        .call()
        .map_err(|e| format!("failed to reach GitHub: {e}"))?
        .into_string()
        .map_err(|e| format!("failed to read GitHub response: {e}"))?;
    let start = body
        .find("\"tag_name\":\"")
        .ok_or_else(|| "unexpected GitHub API response".to_string())?;
    let rest = &body[start + "\"tag_name\":\"".len()..];
    let end = rest.find('"').ok_or_else(|| "unexpected GitHub API response".to_string())?;
    Ok(rest[..end].to_string())
}

/// Download a file to memory from GitHub's release download redirect.
fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut body = ureq::get(url)
        .set("User-Agent", &agent())
        .timeout(std::time::Duration::from_secs(120))
        .call()
        .map_err(|e| format!("failed to download {url}: {e}"))?
        .into_reader();
    let mut data = Vec::new();
    body.read_to_end(&mut data)
        .map_err(|e| format!("failed to read download: {e}"))?;
    if data.is_empty() {
        return Err(format!("downloaded file from {url} is empty"));
    }
    Ok(data)
}

fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim_start_matches('v');
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// The release asset name for this platform, if a prebuilt binary exists.
fn platform_name() -> Option<&'static str> {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        return None;
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return None;
    };
    Some(match (os, arch) {
        ("linux", "x86_64") => "syntheraqr-linux-x86_64",
        ("linux", "aarch64") => "syntheraqr-linux-aarch64",
        ("macos", "x86_64") => "syntheraqr-macos-x86_64",
        ("macos", "aarch64") => "syntheraqr-macos-aarch64",
        _ => return None,
    })
}

/// Report the current and latest versions without downloading anything.
pub fn check() -> Result<(), String> {
    let latest = latest_tag()?;
    let latest_v = parse_version(&latest).ok_or_else(|| format!("unexpected latest tag {latest}"))?;
    let current = parse_version(env!("CARGO_PKG_VERSION")).unwrap_or((0, 0, 0));
    println!("syntheraqr {} (installed)", env!("CARGO_PKG_VERSION"));
    if latest_v > current {
        println!("syntheraqr {latest} (available)");
    } else if latest_v < current {
        println!("installed version is newer than the latest release ({latest})");
    } else {
        println!("you are up to date");
    }
    Ok(())
}

/// Download and atomically replace the running binary with the latest release.
pub fn update(force: bool) -> Result<(), String> {
    let latest = latest_tag()?;
    let latest_v = parse_version(&latest).ok_or_else(|| format!("unexpected latest tag {latest}"))?;
    let current = parse_version(env!("CARGO_PKG_VERSION")).unwrap_or((0, 0, 0));

    if !force && latest_v <= current {
        println!("syntheraqr is already up to date ({})", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let platform = platform_name().ok_or_else(|| {
        "self-update is not supported on this platform; reinstall with the installer".to_string()
    })?;
    println!("downloading {platform} ({latest})...");
    let data = download(&format!("{DOWNLOAD_BASE}/{platform}"))?;

    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot resolve the current executable: {e}"))?
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_exe().expect("current exe"));
    let dir = exe
        .parent()
        .ok_or_else(|| format!("cannot resolve the install directory of {}", exe.display()))?;
    let tmp = dir.join(format!(".syntheraqr-update-{}", std::process::id()));

    std::fs::write(&tmp, &data).map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
    let fail = |msg: String| -> String {
        let _ = std::fs::remove_file(&tmp);
        msg
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp)
            .map_err(|e| fail(format!("failed to stat {}: {e}", tmp.display())))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tmp, perms)
            .map_err(|e| fail(format!("failed to chmod {}: {e}", tmp.display())))?;
    }

    let out = Command::new(&tmp)
        .arg("--version")
        .output()
        .map_err(|e| fail(format!("downloaded binary failed to run: {e}")))?;
    if !out.status.success() {
        return Err(fail("downloaded binary failed its self-check; update aborted".to_string()));
    }

    std::fs::rename(&tmp, &exe)
        .map_err(|e| fail(format!("failed to replace {}: {e}", exe.display())))?;
    println!("updated syntheraqr to {latest}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_version;

    #[test]
    fn parses_version_tags() {
        assert_eq!(parse_version("v0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_version("v10.20.30"), Some((10, 20, 30)));
        assert_eq!(parse_version("latest"), None);
        assert_eq!(parse_version("v0.1"), None);
    }

    #[test]
    fn version_ordering() {
        assert!(parse_version("v0.1.0").unwrap() < parse_version("v0.2.0").unwrap());
        assert!(parse_version("v0.2.0").unwrap() <= parse_version("v0.2.0").unwrap());
        assert!(parse_version("v0.9.9").unwrap() < parse_version("v1.0.0").unwrap());
    }
}