use image::{imageops::FilterType, Rgba, RgbaImage};

const BLOCK: &str = "\u{2580}";

/// Render the image using half-block glyphs (upper pixel = fg, lower = bg) with 24-bit color.
pub fn print_block(img: &RgbaImage, max_cols: u32) {
    let w = img.width();
    let h = img.height();
    let target_w = w.min(max_cols.max(1));
    let target_h = (((h as f64) * (target_w as f64) / (w as f64)).round() as u32).max(2);
    let scaled = if target_w == w && target_h == h {
        img.clone()
    } else {
        image::imageops::resize(img, target_w, target_h, FilterType::Lanczos3)
    };

    let mut out = String::with_capacity(target_w as usize * target_h as usize * 8);
    let rows = (target_h as usize).div_ceil(2);
    for pair in 0..rows {
        let y_upper = pair * 2;
        let y_lower = y_upper + 1;
        let mut line = String::new();
        let mut run_start = 0usize;
        let mut cur: Option<(Rgba<u8>, Rgba<u8>)> = None;
        for x in 0..target_w {
            let upper = *scaled.get_pixel(x, y_upper as u32);
            let lower = if (y_lower as u32) < target_h {
                *scaled.get_pixel(x, y_lower as u32)
            } else {
                Rgba([0, 0, 0, 0])
            };
            let pair = (upper, lower);
            if cur != Some(pair) {
                if cur.is_some() {
                    line.push_str(&BLOCK.repeat(x as usize - run_start));
                }
                line.push_str(&codes(pair));
                run_start = x as usize;
                cur = Some(pair);
            }
        }
        if cur.is_some() {
            line.push_str(&BLOCK.repeat(target_w as usize - run_start));
        }
        line.push_str("\x1b[0m\n");
        out.push_str(&line);
    }
    print!("{out}");
}

fn codes(pair: (Rgba<u8>, Rgba<u8>)) -> String {
    let mut s = String::from("\x1b[0m");
    let (upper, lower) = pair;
    if upper[3] >= 128 {
        s.push_str(&format!("\x1b[38;2;{};{};{}m", upper[0], upper[1], upper[2]));
    }
    if lower[3] >= 128 {
        s.push_str(&format!("\x1b[48;2;{};{};{}m", lower[0], lower[1], lower[2]));
    }
    s
}
