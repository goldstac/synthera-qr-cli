use image::imageops::FilterType;
use image::{Rgba, RgbaImage};
use qrcode::{Color, QrCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DotStyle {
    Rounded,
    Square,
}

pub struct RenderOptions {
    pub fg: [u8; 3],
    pub bg: Option<[u8; 3]>,
    pub style: DotStyle,
    pub margin: u32,
    pub size: u32,
}

const SUPERSAMPLE: u32 = 8;

pub fn render(qr: &QrCode, opts: &RenderOptions) -> RgbaImage {
    match opts.style {
        DotStyle::Square => render_square(qr, opts),
        DotStyle::Rounded => render_rounded(qr, opts),
    }
}

fn render_square(qr: &QrCode, opts: &RenderOptions) -> RgbaImage {
    let n = qr.width() as u32;
    let total = n + 2 * opts.margin;
    let px = opts.size as f64 / total as f64;
    let dark = Rgba([opts.fg[0], opts.fg[1], opts.fg[2], 255]);
    let light = match opts.bg {
        Some(c) => Rgba([c[0], c[1], c[2], 255]),
        None => Rgba([0, 0, 0, 0]),
    };
    let mut img = RgbaImage::new(opts.size, opts.size);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let mx = ((x as f64 + 0.5) / px).floor() as i64 - opts.margin as i64;
        let my = ((y as f64 + 0.5) / px).floor() as i64 - opts.margin as i64;
        *p = if mx >= 0 && my >= 0 && (mx as u32) < n && (my as u32) < n && qr[(mx as usize, my as usize)] == Color::Dark
        {
            dark
        } else {
            light
        };
    }
    img
}

fn render_rounded(qr: &QrCode, opts: &RenderOptions) -> RgbaImage {
    let n = qr.width() as u32;
    let total = n + 2 * opts.margin;
    let canvas = total * SUPERSAMPLE;
    let dark = Rgba([opts.fg[0], opts.fg[1], opts.fg[2], 255]);
    let light = match opts.bg {
        Some(c) => Rgba([c[0], c[1], c[2], 255]),
        None => Rgba([0, 0, 0, 0]),
    };

    let mut buf = vec![0u8; (canvas * canvas * 4) as usize];
    for y in 0..canvas {
        for x in 0..canvas {
            let i = (y * canvas + x) as usize * 4;
            buf[i..i + 4].copy_from_slice(&light.0);
        }
    }
    let half = SUPERSAMPLE as f64 / 2.0;
    let thickness = SUPERSAMPLE as f64 / 2.0;

for my in 0..n {
        for mx in 0..n {
            if qr[(mx as usize, my as usize)] == Color::Light {
                continue;
            }
            let cx = (mx as f64 + 0.5) * SUPERSAMPLE as f64;
            let cy = (my as f64 + 0.5) * SUPERSAMPLE as f64;
            if in_finder(mx, my, n) {
                fill_rect(
                    &mut buf,
                    canvas,
                    cx - half,
                    cy - half,
                    cx + half,
                    cy + half,
                    dark,
                );
                continue;
            }
            fill_circle(&mut buf, canvas, cx, cy, half, dark);
            if mx + 1 < n && qr[(mx as usize + 1, my as usize)] == Color::Dark {
                fill_rect(
                    &mut buf,
                    canvas,
                    cx,
                    cy - thickness / 2.0,
                    cx + SUPERSAMPLE as f64,
                    cy + thickness / 2.0,
                    dark,
                );
            }
            if my + 1 < n && qr[(mx as usize, my as usize + 1)] == Color::Dark {
                fill_rect(
                    &mut buf,
                    canvas,
                    cx - thickness / 2.0,
                    cy,
                    cx + thickness / 2.0,
                    cy + SUPERSAMPLE as f64,
                    dark,
                );
            }
        }
    }

    let img = RgbaImage::from_raw(canvas, canvas, buf).expect("canvas size mismatch");
    scale_premultiplied(&img, opts.size)
}

/// Premultiplied-resize helper: scales `img` to `opts.size` with correct alpha handling.
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

fn in_finder(mx: u32, my: u32, n: u32) -> bool {
    (mx < 7 && my < 7) || (mx >= n - 7 && my < 7) || (mx < 7 && my >= n - 7)
}

fn put_px(buf: &mut [u8], canvas: u32, x: u32, y: u32, color: Rgba<u8>) {
    let i = (y * canvas + x) as usize * 4;
    buf[i..i + 4].copy_from_slice(&color.0);
}

fn fill_circle(buf: &mut [u8], canvas: u32, cx: f64, cy: f64, r: f64, color: Rgba<u8>) {
    let x0 = ((cx - r).floor().max(0.0)) as u32;
    let x1 = ((cx + r).ceil().min(canvas as f64 - 1.0)) as u32;
    let y0 = ((cy - r).floor().max(0.0)) as u32;
    let y1 = ((cy + r).ceil().min(canvas as f64 - 1.0)) as u32;
    let r2 = r * r;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            if dx * dx + dy * dy <= r2 {
                put_px(buf, canvas, x, y, color);
            }
        }
    }
}

fn fill_rect(
    buf: &mut [u8],
    canvas: u32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color: Rgba<u8>,
) {
    let x0 = x0.floor().max(0.0) as u32;
    let x1 = (x1.ceil() - 1.0).min(canvas as f64 - 1.0) as u32;
    let y0 = y0.floor().max(0.0) as u32;
    let y1 = (y1.ceil() - 1.0).min(canvas as f64 - 1.0) as u32;
    for y in y0..=y1 {
        for x in x0..=x1 {
            put_px(buf, canvas, x, y, color);
        }
    }
}
