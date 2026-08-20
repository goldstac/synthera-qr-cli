use std::io::{IsTerminal, Read, Write};
use std::process::Command;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use qrcode::{EcLevel, QrCode};

use syntheraqr::block::print_block;
use syntheraqr::output::{encode, Format};
use syntheraqr::qr::{render, DotStyle, RenderOptions};
use syntheraqr::terminal;

#[derive(Parser, Debug)]
#[command(
    name = "syntheraqr",
    version,
    about = "Ultra-modern QR code generator with terminal preview",
    long_about = "Generate, customize, and save QR codes — with an inline terminal preview \
                  via the kitty graphics protocol (half-block fallback elsewhere)."
)]
struct Cli {
    /// Text or URL to encode (reads from stdin if omitted and stdin is piped)
    text: Option<String>,

    /// Foreground (dot) color, e.g. #0f172a
    #[arg(long, default_value = "#0f172a")]
    fg: String,

    /// Secondary foreground color: enables a diagonal fg → fg2 gradient
    #[arg(long)]
    fg2: Option<String>,

    /// Background color, e.g. #f8fafc
    #[arg(long, default_value = "#f8fafc")]
    bg: String,

    /// Transparent background
    #[arg(long)]
    transparent: bool,

    /// Dot style
    #[arg(long, value_enum, default_value_t = DotStyleArg::Rounded)]
    style: DotStyleArg,

    /// Error correction level
    #[arg(short = 'e', long, value_enum, default_value_t = EccArg::Medium)]
    error: EccArg,

    /// Output size in pixels
    #[arg(short, long, default_value_t = 260)]
    size: u32,

    /// Quiet zone in modules
    #[arg(short, long, default_value_t = 4)]
    margin: u32,

    /// Save to file (format inferred from extension: png/jpg/jpeg/svg/webp)
    #[arg(short, long)]
    output: Option<String>,

    /// Force output format (ignores/overrides the file extension)
    #[arg(short, long, value_enum)]
    format: Option<FormatArg>,

    /// Write image bytes to stdout instead of previewing
    #[arg(long)]
    stdout: bool,

    /// Skip the terminal preview
    #[arg(long)]
    no_show: bool,

    /// Force kitty graphics protocol preview
    #[arg(long)]
    force_kitty: bool,

    /// Force half-block character preview
    #[arg(long)]
    force_block: bool,

    /// Open the saved file with the system viewer
    #[arg(long)]
    open: bool,

    /// Suppress informational messages
    #[arg(short, long)]
    quiet: bool,

    /// Print shell completion script and exit
    #[arg(long, value_enum)]
    completions: Option<clap_complete::Shell>,

    #[command(subcommand)]
    cmd: Option<SubCmd>,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Decode a QR code image file and print its content
    Decode { image: String },

    /// Check for and apply updates from GitHub Releases
    Update {
        /// Only report the latest version without downloading
        #[arg(long)]
        check: bool,
        /// Install even if the installed version is current
        #[arg(long)]
        force: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DotStyleArg {
    Rounded,
    Square,
    Dots,
}

impl DotStyleArg {
    fn to_style(self) -> DotStyle {
        match self {
            DotStyleArg::Rounded => DotStyle::Rounded,
            DotStyleArg::Square => DotStyle::Square,
            DotStyleArg::Dots => DotStyle::Dots,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EccArg {
    Low,
    Medium,
    Quartile,
    High,
}

impl EccArg {
    fn label(self) -> &'static str {
        match self {
            EccArg::Low => "L",
            EccArg::Medium => "M",
            EccArg::Quartile => "Q",
            EccArg::High => "H",
        }
    }
}

impl From<EccArg> for EcLevel {
    fn from(e: EccArg) -> Self {
        match e {
            EccArg::Low => EcLevel::L,
            EccArg::Medium => EcLevel::M,
            EccArg::Quartile => EcLevel::Q,
            EccArg::High => EcLevel::H,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FormatArg {
    Png,
    Jpg,
    Svg,
    Webp,
}

impl From<FormatArg> for Format {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Png => Format::Png,
            FormatArg::Jpg => Format::Jpg,
            FormatArg::Svg => Format::Svg,
            FormatArg::Webp => Format::Webp,
        }
    }
}

const LOGO: &str = r#"
   _____             __  __
  / ___/__  ______  / /_/ /_  ___  _________ _____ ______
  \__ \/ / / / __ \/ __/ __ \/ _ \/ ___/ __ `/ __ `/ ___/
 ___/ / /_/ / / / / /_/ / / /  __/ /  / /_/ / /_/ / /
/____/\__, /_/ /_/\__/_/ /_/\___/_/   \__,_/\__, /_/
     /____/                                   /_/
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wants_help = args.iter().any(|a| a == "-h" || a == "--help");
    let wants_version = args.iter().any(|a| a == "-V" || a == "--version");
    let sub = args.get(1).map(String::as_str);
    let sub_help = sub.is_some_and(|s| matches!(s, "decode" | "update")) && wants_help;
    if (wants_help || wants_version) && !sub_help {
        print!("{LOGO}");
        if wants_version {
            println!("syntheraqr {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        let mut cmd = Cli::command();
        let _ = cmd.print_long_help();
        return;
    }
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("syntheraqr: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    if let Some(SubCmd::Decode { image }) = &cli.cmd {
        return run_decode(image);
    }

    if let Some(SubCmd::Update { check, force }) = &cli.cmd {
        return if *check {
            syntheraqr::update::check()
        } else {
            syntheraqr::update::update(*force)
        };
    }

    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "syntheraqr", &mut std::io::stdout());
        return Ok(());
    }

    if cli.output.is_some() && cli.stdout {
        return Err("cannot use --output and --stdout together".into());
    }
    if cli.open && cli.output.is_none() {
        return Err("--open requires --output".into());
    }

    let text = match cli.text.clone() {
        Some(t) if !t.is_empty() => t,
        _ => {
            if std::io::stdin().is_terminal() {
                eprint!("Text to encode: ");
                std::io::stderr()
                    .flush()
                    .map_err(|e| format!("failed to flush stderr: {e}"))?;
                let mut buf = String::new();
                std::io::stdin()
                    .read_line(&mut buf)
                    .map_err(|e| format!("failed to read input: {e}"))?;
                let t = buf.trim().to_string();
                if t.is_empty() {
                    return Err("no input given; pass TEXT as an argument or pipe it via stdin"
                        .into());
                }
                t
            } else {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .map_err(|e| format!("failed to read stdin: {e}"))?;
                if buf.trim().is_empty() {
                    return Err("no input given; pass TEXT as an argument or pipe it via stdin"
                        .into());
                }
                buf.trim().to_string()
            }
        }
    };

    let fg = parse_hex(&cli.fg)?;
    let fg2 = cli.fg2.as_deref().map(parse_hex).transpose()?;
    let bg = if cli.transparent { None } else { Some(parse_hex(&cli.bg)?) };

    let opts = RenderOptions {
        fg,
        fg2,
        bg,
        style: cli.style.to_style(),
        margin: cli.margin,
        size: cli.size,
    };

    let qr = QrCode::with_error_correction_level(text.as_bytes(), cli.error.into())
        .map_err(|_| "input too long to fit in a QR code".to_string())?;
    let img = render(&qr, &opts)?;

    let stdout_tty = std::io::stdout().is_terminal();

    if let Some(path) = &cli.output {
        let format = match cli.format {
            Some(f) => f.into(),
            None => {
                let ext = path.rsplit('.').next().unwrap_or("");
                Format::from_extension(ext).ok_or_else(|| {
                    format!("cannot infer format from \"{path}\"; use --format png|jpg|svg|webp")
                })?
            }
        };
        let bytes = encode(&img, format, &qr, &opts)?;
        std::fs::write(path, &bytes).map_err(|e| format!("failed to write {path}: {e}"))?;
        if !cli.quiet {
            eprintln!("saved {} ({:.1} KiB)", path, bytes.len() as f64 / 1024.0);
        }
        if cli.open {
            open_file(path)?;
        }
    } else if cli.stdout || !stdout_tty {
        let format = cli.format.map(Into::into).unwrap_or(Format::Png);
        let bytes = encode(&img, format, &qr, &opts)?;
        std::io::stdout()
            .write_all(&bytes)
            .map_err(|e| format!("failed to write stdout: {e}"))?;
        std::io::stdout()
            .flush()
            .map_err(|e| format!("failed to flush stdout: {e}"))?;
        return Ok(());
    }

    if !cli.quiet {
        eprintln!(
            "QR {}×{} modules · ECC {} · {} px",
            qr.width(),
            qr.width(),
            cli.error.label(),
            cli.size
        );
    }

    if !cli.no_show && stdout_tty {
        let mut disp = opts;
        disp.margin = 0;
        if terminal::supports_kitty(cli.force_kitty, cli.force_block) {
            disp.size = 512;
            let img_disp = render(&qr, &disp)?;
            let png = encode(&img_disp, Format::Png, &qr, &disp)?;
            terminal::transmit_kitty(&png, disp.size);
        } else {
            let (cols, _rows) = terminal::terminal_size();
            disp.size = (cols.saturating_mul(2)).max(32);
            let img_disp = render(&qr, &disp)?;
            print_block(&img_disp, cols);
        }
    }

    Ok(())
}

fn run_decode(path: &str) -> Result<(), String> {
    let img = image::open(path).map_err(|e| format!("cannot open \"{path}\": {e}"))?;
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    for grid in prepared.detect_grids() {
        if let Ok((_, content)) = grid.decode() {
            println!("{content}");
            return Ok(());
        }
    }
    Err(format!("no QR code found in \"{path}\""))
}

fn open_file(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let cmd = ("open", path);
    #[cfg(target_os = "windows")]
    let cmd = ("cmd", path);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let cmd = ("xdg-open", path);
    Command::new(cmd.0)
        .arg(cmd.1)
        .spawn()
        .map_err(|e| format!("failed to open {path}: {e}"))?;
    Ok(())
}

fn parse_hex(s: &str) -> Result<[u8; 3], String> {
    let s = s.trim_start_matches('#');
    let hex = |i: usize, len: usize| {
        u8::from_str_radix(&s[i..i + len], 16).map_err(|_| format!("invalid color \"{s}\""))
    };
    match s.len() {
        3 => {
            let expand = |v: u8| v * 17;
            Ok([
                expand(hex(0, 1)?),
                expand(hex(1, 1)?),
                expand(hex(2, 1)?),
            ])
        }
        6 => Ok([hex(0, 2)?, hex(2, 2)?, hex(4, 2)?]),
        _ => Err(format!(
            "invalid color \"{s}\"; expected a hex color like #0f172a"
        )),
    }
}
