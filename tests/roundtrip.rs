use qrcode::QrCode;
use syntheraqr::output::{encode, Format};
use syntheraqr::qr::{render, DotStyle, RenderOptions};

fn opts(style: DotStyle) -> RenderOptions {
    RenderOptions {
        fg: [15, 23, 42],
        bg: Some([248, 250, 252]),
        style,
        margin: 4,
        size: 400,
    }
}

fn decode(img: &image::RgbaImage) -> Result<String, String> {
    let mut prepared = rqrr::PreparedImage::prepare(image::DynamicImage::ImageRgba8(img.clone()).to_luma8());
    for grid in prepared.detect_grids() {
        if let Ok((_, content)) = grid.decode() {
            return Ok(content);
        }
    }
    Err("no QR grid decoded".into())
}

#[test]
fn colors_and_margin_are_respected() {
    let text = "color test";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Square);
    o.fg = [0x25, 0x63, 0xeb];
    o.bg = Some([0xff, 0xff, 0xff]);
    o.size = 300;
    let img = render(&qr, &o);

    let w = img.width();
    // margin corner must be background
    assert_eq!(img.get_pixel(1, 1).0[..3], [255, 255, 255]);
    assert_eq!(img.get_pixel(1, 1).0[3], 255);
    // finder pattern corner module is inside the margin (margin=4, module = 300/29 ≈ 10.3px)
    let m = 300.0 / (21 + 8) as f64;
    let px = |mx: u32, my: u32| {
        let x = ((mx as f64 + 4.0 + 0.5) * m).round() as u32;
        let y = ((my as f64 + 4.0 + 0.5) * m).round() as u32;
        img.get_pixel(x, y)
    };
    // finder outer corner (dark)
    assert_eq!(px(0, 0).0[..3], [0x25, 0x63, 0xeb]);
    // finder inner dot (dark)
    assert_eq!(px(3, 3).0[..3], [0x25, 0x63, 0xeb]);
    // finder light separator ring
    assert_eq!(px(1, 1).0[..3], [255, 255, 255]);
    let _ = w;
}

#[test]
fn square_roundtrip() {
    let text = "https://syntheraqr.site";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let img = render(&qr, &opts(DotStyle::Square));
    assert_eq!(decode(&img).unwrap(), text);
}

#[test]
fn rounded_roundtrip() {
    let text = "hello world";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let img = render(&qr, &opts(DotStyle::Rounded));
    assert_eq!(decode(&img).unwrap(), text);
}

#[test]
fn transparent_png_has_alpha() {
    let text = "alpha test";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Square);
    o.bg = None;
    let img = render(&qr, &o);
    let bytes = encode(&img, Format::Png, &qr, &o).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap().to_rgba8();
    let corner = decoded.get_pixel(0, 0);
    assert_eq!(corner[3], 0, "corner must be fully transparent");
}

#[test]
fn svg_has_structure() {
    let text = "svg test";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let o = opts(DotStyle::Rounded);
    let svg = encode(&img_placeholder(), Format::Svg, &qr, &o).unwrap();
    let svg = String::from_utf8(svg).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<circle"), "rounded SVG should contain circles");
    assert!(svg.ends_with("</svg>\n"));

    let o = opts(DotStyle::Square);
    let svg = encode(&img_placeholder(), Format::Svg, &qr, &o).unwrap();
    let svg = String::from_utf8(svg).unwrap();
    assert!(svg.contains("<rect"), "square SVG should contain rects");
}

fn img_placeholder() -> image::RgbaImage {
    image::RgbaImage::new(1, 1)
}

#[test]
fn jpg_flattens_to_rgb() {
    let text = "jpg test";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Square);
    o.bg = None;
    let img = render(&qr, &o);
    let bytes = encode(&img, Format::Jpg, &qr, &o).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    assert_eq!(decoded.color(), image::ColorType::Rgb8);
}
