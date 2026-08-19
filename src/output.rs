use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbaImage};

use crate::qr::{module_shape, RenderOptions, Shape};

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

pub fn encode(img: &RgbaImage, format: Format, qr: &qrcode::QrCode, opts: &RenderOptions) -> Result<Vec<u8>, String> {
    if format == Format::Svg {
        return Ok(to_svg(qr, opts)?.into_bytes());
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

pub fn to_svg(qr: &qrcode::QrCode, opts: &RenderOptions) -> Result<String, String> {
    let count = qr.width() as u32;
    let size = opts.size;
    let (dot_size, offset) = crate::qr::layout(size, opts.margin, count)?;
    let fg = format!("#{:02x}{:02x}{:02x}", opts.fg[0], opts.fg[1], opts.fg[2]);

    let mut s = String::new();
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        size, size, size, size
    ));
    if let Some(c) = opts.bg {
        s.push_str(&format!(
            "<rect width=\"100%\" height=\"100%\" fill=\"#{:02x}{:02x}{:02x}\"/>\n",
            c[0], c[1], c[2]
        ));
    }
    // Gradient spans the matrix area (excluding the margin), matching the raster render.
    let fill = if let Some(f2) = opts.fg2 {
        let fg2 = format!("#{:02x}{:02x}{:02x}", f2[0], f2[1], f2[2]);
        s.push_str(&format!(
            "<defs><linearGradient id=\"g\" gradientUnits=\"userSpaceOnUse\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\">\
             <stop offset=\"0%\" stop-color=\"{}\"/><stop offset=\"100%\" stop-color=\"{}\"/>\
             </linearGradient></defs>\n",
            offset, offset, size as f64 - offset as f64, size as f64 - offset as f64, fg, fg2
        ));
        "url(#g)".to_string()
    } else {
        fg.clone()
    };

    for j in 0..count {
        for i in 0..count {
            let Some(shape) = module_shape(qr, i, j, opts.style) else {
                continue;
            };
            let x = offset as f64 + i as f64 * dot_size as f64;
            let y = offset as f64 + j as f64 * dot_size as f64;
            s.push_str(&shape_svg(shape, x, y, dot_size as f64, &fill));
        }
    }
    s.push_str("</svg>\n");
    Ok(s)
}

fn shape_svg(shape: Shape, x: f64, y: f64, size: f64, fill: &str) -> String {
    let cx = x + size / 2.0;
    let cy = y + size / 2.0;
    let r = size / 2.0;
    match shape {
        Shape::Square => format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
            x, y, size, size, fill
        ),
        Shape::Circle => format!(
            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{}\"/>\n",
            cx, cy, r, fill
        ),
        Shape::Pill(d) => {
            let path = match d {
                1 => format!("M{cx:.2} {cy:.2} A{r:.2} {r:.2} 0 0 1 {cx:.2} {cy:.2} L{x:.2} {} L{x:.2} {y:.2} Z", cy + size),
                2 => format!("M{cx:.2} {cy:.2} A{r:.2} {r:.2} 0 0 0 {cx:.2} {cy:.2} L{} {} L{} {y:.2} Z", x + size, cy + size, x + size),
                4 => format!("M{} {cy:.2} A{r:.2} {r:.2} 0 0 0 {x:.2} {cy:.2} L{x:.2} {} L{} {} Z", x + size, cy + size, x + size, y + size),
                _ => format!("M{x:.2} {cy:.2} A{r:.2} {r:.2} 0 0 1 {} {cy:.2} L{} {y:.2} L{x:.2} {y:.2} Z", x + size, x + size),
            };
            format!("<path d=\"{}\" fill=\"{}\"/>\n", path, fill)
        }
        Shape::Corner(cut) => {
            let path = match cut {
                0 => format!(
                    "M{cx:.2} {y:.2} A{r:.2} {r:.2} 0 0 1 {} {cy:.2} L{} {} L{x:.2} {} L{x:.2} {y:.2} Z",
                    x + size, x + size, y + size, y + size
                ),
                1 => format!(
                    "M{} {cy:.2} A{r:.2} {r:.2} 0 0 1 {cx:.2} {} L{x:.2} {} L{x:.2} {y:.2} L{} {y:.2} Z",
                    x + size, y + size, y + size, x + size
                ),
                2 => format!(
                    "M{cx:.2} {} A{r:.2} {r:.2} 0 0 1 {x:.2} {cy:.2} L{x:.2} {y:.2} L{} {y:.2} L{} {} Z",
                    y + size, x + size, x + size, y + size
                ),
                _ => format!(
                    "M{x:.2} {cy:.2} A{r:.2} {r:.2} 0 0 1 {cx:.2} {y:.2} L{} {y:.2} L{} {} L{x:.2} {} Z",
                    x + size, x + size, y + size, y + size
                ),
            };
            format!("<path d=\"{}\" fill=\"{}\"/>\n", path, fill)
        }
    }
}