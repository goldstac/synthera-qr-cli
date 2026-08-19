use image::imageops::FilterType;
use image::{Rgba, RgbaImage};
use qrcode::{Color, QrCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DotStyle {
    Rounded,
    Square,
    Dots,
}

pub struct RenderOptions {
    pub fg: [u8; 3],
    pub fg2: Option<[u8; 3]>,
    pub bg: Option<[u8; 3]>,
    pub style: DotStyle,
    /// Quiet zone in pixels (matches qr-code-styling; the web app uses 4).
    pub margin: u32,
    pub size: u32,
}

/// Direction bitmask used by the neighbor-aware shapes.
const DIR_L: u8 = 1;
const DIR_R: u8 = 2;
const DIR_U: u8 = 4;
const DIR_D: u8 = 8;

/// Finder pattern masks, copied verbatim from qr-code-styling (QRCanvas.ts).
const SQUARE_MASK: [[u8; 7]; 7] = [
    [1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1],
];

const DOT_MASK: [[u8; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
];

/// The per-module shape drawn by the web engine (`rounded` type of
/// qr-code-styling): isolated modules become circles, 1-neighbor modules
/// become pills, 2-adjacent-neighbor modules become squares with the opposite
/// corner rounded, anything else stays a plain square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Square,
    Circle,
    Pill(u8),
    /// Corner code: 0 = top-right cut, 1 = bottom-right, 2 = bottom-left, 3 = top-left.
    Corner(u8),
}

#[derive(Clone, Copy)]
enum Ctx {
    Data,
    Ring { li: u32, lj: u32 },
    Dot { li: u32, lj: u32 },
}

fn ctx_at(i: u32, j: u32, count: u32) -> Ctx {
    let in_finder = (i < 7 && j < 7) || (i >= count - 7 && j < 7) || (i < 7 && j >= count - 7);
    if !in_finder {
        return Ctx::Data;
    }
    let li = if i >= count - 7 { i - (count - 7) } else { i };
    let lj = if j >= count - 7 { j - (count - 7) } else { j };
    if (2..5).contains(&li) && (2..5).contains(&lj) {
        Ctx::Dot { li, lj }
    } else {
        Ctx::Ring { li, lj }
    }
}

/// Grid layout exactly as qr-code-styling computes it (margin is in pixels):
/// `dotSize = floor((size - margin*2) / count)`, leftover centered.
pub fn layout(size: u32, margin: u32, count: u32) -> Result<(u32, u32), String> {
    if count > size {
        return Err("the canvas is too small".into());
    }
    let min_size = size.saturating_sub(margin * 2);
    let dot_size = min_size / count;
    if dot_size == 0 {
        return Err("the canvas is too small".into());
    }
    let offset = (size - count * dot_size) / 2;
    Ok((dot_size, offset))
}

pub fn module_shape(qr: &QrCode, i: u32, j: u32, style: DotStyle) -> Option<Shape> {
    let count = qr.width() as u32;
    let ctx = ctx_at(i, j, count);
    if !is_dark(ctx, i, j, qr) {
        return None;
    }
    let nb = neighbors(ctx, i, j, count, qr);
    Some(shape_for(style, nb))
}

fn is_dark(ctx: Ctx, _i: u32, _j: u32, qr: &QrCode) -> bool {
    match ctx {
        Ctx::Data => qr[(_i as usize, _j as usize)] == Color::Dark,
        Ctx::Ring { li, lj } => SQUARE_MASK[lj as usize][li as usize] == 1,
        Ctx::Dot { li, lj } => DOT_MASK[lj as usize][li as usize] == 1,
    }
}

fn mask_neighbor(mask: &[[u8; 7]; 7], li: u32, lj: u32, dx: i64, dy: i64) -> bool {
    let x = li as i64 + dx;
    let y = lj as i64 + dy;
    x >= 0 && y >= 0 && x < 7 && y < 7 && mask[y as usize][x as usize] == 1
}

fn neighbors(ctx: Ctx, i: u32, j: u32, count: u32, qr: &QrCode) -> u8 {
    let mut nb = 0;
    match ctx {
        Ctx::Data => {
            let dark = |dx: i64, dy: i64| -> bool {
                let x = i as i64 + dx;
                let y = j as i64 + dy;
                x >= 0 && y >= 0 && x < count as i64 && y < count as i64
                    && qr[(x as usize, y as usize)] == Color::Dark
            };
            if dark(-1, 0) {
                nb |= DIR_L;
            }
            if dark(1, 0) {
                nb |= DIR_R;
            }
            if dark(0, -1) {
                nb |= DIR_U;
            }
            if dark(0, 1) {
                nb |= DIR_D;
            }
        }
        Ctx::Ring { li, lj } => {
            if mask_neighbor(&SQUARE_MASK, li, lj, -1, 0) {
                nb |= DIR_L;
            }
            if mask_neighbor(&SQUARE_MASK, li, lj, 1, 0) {
                nb |= DIR_R;
            }
            if mask_neighbor(&SQUARE_MASK, li, lj, 0, -1) {
                nb |= DIR_U;
            }
            if mask_neighbor(&SQUARE_MASK, li, lj, 0, 1) {
                nb |= DIR_D;
            }
        }
        Ctx::Dot { li, lj } => {
            if mask_neighbor(&DOT_MASK, li, lj, -1, 0) {
                nb |= DIR_L;
            }
            if mask_neighbor(&DOT_MASK, li, lj, 1, 0) {
                nb |= DIR_R;
            }
            if mask_neighbor(&DOT_MASK, li, lj, 0, -1) {
                nb |= DIR_U;
            }
            if mask_neighbor(&DOT_MASK, li, lj, 0, 1) {
                nb |= DIR_D;
            }
        }
    }
    nb
}

fn shape_for(style: DotStyle, nb: u8) -> Shape {
    match style {
        DotStyle::Square => Shape::Square,
        DotStyle::Dots => Shape::Circle,
        DotStyle::Rounded => {
            let n = nb.count_ones();
            if n == 0 {
                Shape::Circle
            } else if n == 1 {
                Shape::Pill(nb)
            } else if n == 2 && !(nb & DIR_L != 0 && nb & DIR_R != 0)
                && !(nb & DIR_U != 0 && nb & DIR_D != 0)
            {
                let cut = if nb & DIR_U != 0 && nb & DIR_L != 0 {
                    1
                } else if nb & DIR_U != 0 && nb & DIR_R != 0 {
                    2
                } else if nb & DIR_D != 0 && nb & DIR_R != 0 {
                    3
                } else {
                    0
                };
                Shape::Corner(cut)
            } else {
                Shape::Square
            }
        }
    }
}

fn covered(shape: Shape, x: f64, y: f64, cx: f64, cy: f64, r: f64) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    match shape {
        Shape::Square => true,
        Shape::Circle => dx * dx + dy * dy <= r * r,
        Shape::Pill(d) => {
            let in_rect = if d & DIR_L != 0 {
                x <= cx
            } else if d & DIR_R != 0 {
                x >= cx
            } else if d & DIR_U != 0 {
                y <= cy
            } else {
                y >= cy
            };
            in_rect || dx * dx + dy * dy <= r * r
        }
        Shape::Corner(cut) => {
            let cut_quadrant = match cut {
                0 => x > cx && y < cy,
                1 => x > cx && y > cy,
                2 => x < cx && y > cy,
                _ => x < cx && y < cy,
            };
            !(cut_quadrant && dx * dx + dy * dy > r * r)
        }
    }
}

/// Diagonal (top-left → bottom-right) gradient color for a module.
fn module_color(opts: &RenderOptions, mx: u32, my: u32, n: u32) -> Rgba<u8> {
    let c = match opts.fg2 {
        None => opts.fg,
        Some(f2) => {
            let t = if n <= 1 {
                0.0
            } else {
                ((mx + my) as f64 / (2 * (n - 1)) as f64).clamp(0.0, 1.0)
            };
            [
                (opts.fg[0] as f64 + (f2[0] as f64 - opts.fg[0] as f64) * t) as u8,
                (opts.fg[1] as f64 + (f2[1] as f64 - opts.fg[1] as f64) * t) as u8,
                (opts.fg[2] as f64 + (f2[2] as f64 - opts.fg[2] as f64) * t) as u8,
            ]
        }
    };
    Rgba([c[0], c[1], c[2], 255])
}

pub fn render(qr: &QrCode, opts: &RenderOptions) -> Result<RgbaImage, String> {
    let count = qr.width() as u32;
    let size = opts.size;
    let (dot_size, offset) = layout(size, opts.margin, count)?;

    // Supersample factor: the canvas is rendered at up to ~4600 px then
    // Lanczos-downscaled, which approximates the browser's anti-aliasing.
    let ss = (4608 / size.max(1)).clamp(2, 16);
    let c = size * ss;
    let d = dot_size * ss;
    let o = offset * ss;
    let r = d as f64 / 2.0;

    let light = match opts.bg {
        Some(c) => Rgba([c[0], c[1], c[2], 255]),
        None => Rgba([0, 0, 0, 0]),
    };
    let mut buf = vec![0u8; (c * c * 4) as usize];
    for chunk in buf.chunks_exact_mut(4) {
        chunk.copy_from_slice(&light.0);
    }

    for py in 0..c {
        if py < o {
            continue;
        }
        let j = (py - o) / d;
        if j >= count {
            continue;
        }
        for px in 0..c {
            if px < o {
                continue;
            }
            let i = (px - o) / d;
            if i >= count {
                continue;
            }
            let Some(shape) = module_shape(qr, i, j, opts.style) else {
                continue;
            };
            let x = px as f64 + 0.5;
            let y = py as f64 + 0.5;
            let cx = (o + i * d) as f64 + r;
            let cy = (o + j * d) as f64 + r;
            if covered(shape, x, y, cx, cy, r) {
                let color = module_color(opts, i, j, count);
                let idx = (py * c + px) as usize * 4;
                buf[idx..idx + 4].copy_from_slice(&color.0);
            }
        }
    }

    let img = RgbaImage::from_raw(c, c, buf).expect("canvas size mismatch");
    Ok(scale_premultiplied(&img, size))
}

/// Premultiplied-resize helper: scales `img` to `size` with correct alpha handling.
pub fn scale_premultiplied(img: &RgbaImage, size: u32) -> RgbaImage {
    let mut premul = img.clone();
    for p in premul.pixels_mut() {
        let a = p[3] as u32;
        if a == 0 {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        } else if a != 255 {
            p[0] = (p[0] as u32 * a / 255) as u8;
            p[1] = (p[1] as u32 * a / 255) as u8;
            p[2] = (p[2] as u32 * a / 255) as u8;
        }
    }
    let resized = image::imageops::resize(&premul, size, size, FilterType::Lanczos3);
    let mut out = resized.clone();
    for p in out.pixels_mut() {
        let a = p[3] as u32;
        if a != 0 && a != 255 {
            p[0] = ((p[0] as u32 * 255) / a).min(255) as u8;
            p[1] = ((p[1] as u32 * 255) / a).min(255) as u8;
            p[2] = ((p[2] as u32 * 255) / a).min(255) as u8;
        }
    }
    out
}