use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbaImage};
use qrcode::{Color, QrCode};

use crate::qr::{DotStyle, RenderOptions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpg,
    Svg,
    Webp,
}

impl Format {
    pub fn from_extension(ext: &str) -> Option<Format> {
        match ext.to_ascii_lowercase().as_str() {
            "png" => Some(Format::Png),
            "jpg" | "jpeg" => Some(Format::Jpg),
            "svg" => Some(Format::Svg),
            "webp" => Some(Format::Webp),
            _ => None,
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Jpg => "jpg",
            Format::Svg => "svg",
            Format::Webp => "webp",
        }
    }

    fn image_format(self) -> Option<ImageFormat> {
        match self {
            Format::Png => Some(ImageFormat::Png),
            Format::Jpg => Some(ImageFormat::Jpeg),
            Format::Webp => Some(ImageFormat::WebP),
            Format::Svg => None,
        }
    }
}

pub fn encode(img: &RgbaImage, format: Format, qr: &QrCode, opts: &RenderOptions) -> Result<Vec<u8>, String> {
    if format == Format::Svg {
        return Ok(to_svg(qr, opts).into_bytes());
    }
    let image_format = format.image_format().expect("raster format");
    let dynamic = match format {
        Format::Jpg => DynamicImage::ImageRgb8(flatten_alpha(img)),
        _ => DynamicImage::ImageRgba8(img.clone()),
    };
    let mut buf = Cursor::new(Vec::new());
    dynamic
        .write_to(&mut buf, image_format)
        .map_err(|e| format!("failed to encode {}: {e}", format.ext()))?;
    Ok(buf.into_inner())
}

fn flatten_alpha(img: &RgbaImage) -> image::RgbImage {
    let mut out = image::RgbImage::new(img.width(), img.height());
    for (x, y, p) in img.enumerate_pixels() {
        let a = p[3] as u32;
        if a == 255 {
            out.put_pixel(x, y, image::Rgb([p[0], p[1], p[2]]));
        } else if a == 0 {
            out.put_pixel(x, y, image::Rgb([255, 255, 255]));
        } else {
            let blend = |c: u8| -> u8 { ((c as u32 * a + 255 * (255 - a)) / 255) as u8 };
            out.put_pixel(x, y, image::Rgb([blend(p[0]), blend(p[1]), blend(p[2])]));
        }
    }
    out
}

pub fn to_svg(qr: &QrCode, opts: &RenderOptions) -> String {
    let n = qr.width() as u32;
    let total = n + 2 * opts.margin;
    let scale = opts.size as f64 / total as f64;
    let fg = format!("#{:02x}{:02x}{:02x}", opts.fg[0], opts.fg[1], opts.fg[2]);

    let mut s = String::new();
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        opts.size, opts.size, opts.size, opts.size
    ));
    if let Some(c) = opts.bg {
        s.push_str(&format!(
            "<rect width=\"100%\" height=\"100%\" fill=\"#{:02x}{:02x}{:02x}\"/>\n",
            c[0], c[1], c[2]
        ));
    }
    match opts.style {
        DotStyle::Square => {
            for my in 0..n {
                for mx in 0..n {
                    if qr[(mx as usize, my as usize)] == Color::Dark {
                        s.push_str(&format!(
                            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                            (mx as f64 + 0.0) * scale,
                            (my as f64 + 0.0) * scale,
                            scale.ceil(),
                            scale.ceil(),
                            fg
                        ));
                    }
                }
            }
        }
        DotStyle::Rounded => {
            let r = scale / 2.0;
            let t = scale / 2.0;
            for my in 0..n {
                for mx in 0..n {
                    if qr[(mx as usize, my as usize)] == Color::Light {
                        continue;
                    }
                    let cx = (mx as f64 + 0.5) * scale;
                    let cy = (my as f64 + 0.5) * scale;
                    if in_finder(mx, my, n) {
                        s.push_str(&format!(
                            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                            mx as f64 * scale,
                            my as f64 * scale,
                            scale,
                            scale,
                            fg
                        ));
                        continue;
                    }
                    s.push_str(&format!(
                        "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>\n",
                        cx, cy, r, fg
                    ));
                    if mx + 1 < n && qr[(mx as usize + 1, my as usize)] == Color::Dark {
                        s.push_str(&format!(
                            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                            cx,
                            cy - t / 2.0,
                            scale,
                            t,
                            fg
                        ));
                    }
                    if my + 1 < n && qr[(mx as usize, my as usize + 1)] == Color::Dark {
                        s.push_str(&format!(
                            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
                            cx - t / 2.0,
                            cy,
                            t,
                            scale,
                            fg
                        ));
                    }
                }
            }
        }
    }
    s.push_str("</svg>\n");
    s
}

fn in_finder(mx: u32, my: u32, n: u32) -> bool {
    (mx < 7 && my < 7) || (mx >= n - 7 && my < 7) || (mx < 7 && my >= n - 7)
}
