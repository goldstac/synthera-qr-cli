use std::io::{IsTerminal, Write};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

const CHUNK_LIMIT: usize = 4096;

pub fn supports_kitty(force_kitty: bool, force_block: bool) -> bool {
    if force_block {
        return false;
    }
    if force_kitty {
        return true;
    }
    if std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("WEZTERM_PANE").is_ok()
        || std::env::var("GHOSTTY_RESOURCES_DIR").is_ok()
    {
        return true;
    }
    if std::env::var("TERM")
        .map(|t| t.to_ascii_lowercase().contains("kitty"))
        .unwrap_or(false)
    {
        return true;
    }
    probe_kitty_protocol()
}

#[cfg(unix)]
fn probe_kitty_protocol() -> bool {
    use std::time::Instant;

    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return false;
    }
    let fd = libc::STDIN_FILENO;
    unsafe {
        let mut term: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut term) != 0 {
            return false;
        }
        let original = term;
        term.c_lflag &= !(libc::ICANON | libc::ECHO);
        term.c_cc[libc::VMIN] = 0;
        term.c_cc[libc::VTIME] = 0;
        if libc::tcsetattr(fd, libc::TCSANOW, &term) != 0 {
            return false;
        }

        let found = (|| {
            // Capability query: reply is ESC _ G i=<id> ; OK ESC \
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(b"\x1b_Ga=q,i=1\x1b\\");
            let _ = stdout.flush();

            let deadline = Instant::now() + std::time::Duration::from_millis(150);
            let mut buf = [0u8; 512];
            let mut acc: Vec<u8> = Vec::new();
            while Instant::now() < deadline {
                let mut fds = [libc::pollfd {
                    fd,
                    events: libc::POLLIN,
                    revents: 0,
                }];
                let remaining = deadline.saturating_duration_since(Instant::now());
                let timeout_ms = remaining.as_millis().min(u32::MAX as u128) as i32;
                if libc::poll(fds.as_mut_ptr(), 1, timeout_ms) <= 0 {
                    break;
                }
                let n = libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len());
                if n <= 0 {
                    break;
                }
                acc.extend_from_slice(&buf[..n as usize]);
                if String::from_utf8_lossy(&acc).contains("OK") {
                    return true;
                }
            }
            false
        })();

        libc::tcsetattr(fd, libc::TCSANOW, &original);
        found
    }
}

#[cfg(not(unix))]
fn probe_kitty_protocol() -> bool {
    false
}

pub fn terminal_size() -> (u32, u32) {
    #[cfg(unix)]
    {
        unsafe {
            let mut ws: libc::winsize = std::mem::zeroed();
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0
                && ws.ws_col > 0
                && ws.ws_row > 0
            {
                return (ws.ws_col as u32, ws.ws_row as u32);
            }
        }
    }
    let cols = std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok());
    let rows = std::env::var("LINES").ok().and_then(|v| v.parse().ok());
    (cols.unwrap_or(80), rows.unwrap_or(24))
}

/// Transmit a PNG through the kitty graphics protocol and display it inline.
/// Sizes the image to fill ~75% of the terminal while preserving square aspect
/// (terminal cells are ~2:1, so a square image is 2x as wide as it is tall).
pub fn transmit_kitty(png: &[u8], img_size: u32) {
    let (cols, rows) = terminal_size();
    let cols_avail = (cols as f64 * 0.75).max(20.0);
    let rows_avail = (rows as f64 * 0.75).max(10.0);
    let cells = (cols_avail.min(rows_avail * 2.0)).floor() as u32;
    let cells = cells.clamp(20, 160);
    let rows = ((cells as f64 / 2.0).ceil() as u32).max(1);

    let b64 = B64.encode(png);
    let first_control = format!(
        "a=T,f=100,s={img_size},v={img_size},c={cells},r={rows},q=2"
    );

    let mut stdout = std::io::stdout();
    let total_chunks = b64.len().div_ceil(CHUNK_LIMIT);
    for (i, chunk) in b64.as_bytes().chunks(CHUNK_LIMIT).enumerate() {
        let last = i + 1 == total_chunks;
        let control = if i == 0 {
            format!("{first_control},m={}", if last { 0 } else { 1 })
        } else {
            format!("m={}", if last { 0 } else { 1 })
        };
        let _ = write!(stdout, "\x1b_G{control};{}\x1b\\", std::str::from_utf8(chunk).unwrap());
    }
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}
