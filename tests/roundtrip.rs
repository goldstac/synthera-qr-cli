use qrcode::QrCode;
use syntheraqr::output::{encode, Format};
use syntheraqr::qr::{layout, render, DotStyle, RenderOptions};

fn opts(style: DotStyle) -> RenderOptions {
    RenderOptions {
        fg: [15, 23, 42],
        fg2: None,
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
    let img = render(&qr, &o).unwrap();

    // layout: dotSize = floor((300-8)/21) = 13, offset = floor((300-273)/2) = 13
    let (dot, offset) = layout(o.size, o.margin, qr.width() as u32).unwrap();
    assert_eq!(dot, 13);
    assert_eq!(offset, 13);
    // margin corner must be background
    assert_eq!(img.get_pixel(1, 1).0[..3], [255, 255, 255]);
    assert_eq!(img.get_pixel(1, 1).0[3], 255);
    // finder ring outer corner module (dark)
    let center = |mx: u32, my: u32| -> (u32, u32) {
        (offset + mx * dot + dot / 2, offset + my * dot + dot / 2)
    };
    let (x, y) = center(0, 0);
    assert_eq!(img.get_pixel(x, y).0[..3], [0x25, 0x63, 0xeb]);
    // finder inner dot (dark)
    let (x, y) = center(3, 3);
    assert_eq!(img.get_pixel(x, y).0[..3], [0x25, 0x63, 0xeb]);
    // finder light separator ring
    let (x, y) = center(1, 1);
    assert_eq!(img.get_pixel(x, y).0[..3], [255, 255, 255]);
}

#[test]
fn square_roundtrip() {
    let text = "https://syntheraqr.site";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let img = render(&qr, &opts(DotStyle::Square)).unwrap();
    assert_eq!(decode(&img).unwrap(), text);
}

#[test]
fn rounded_roundtrip() {
    let text = "hello world";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let img = render(&qr, &opts(DotStyle::Rounded)).unwrap();
    assert_eq!(decode(&img).unwrap(), text);
}

#[test]
fn dots_roundtrip() {
    let text = "dots style";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Dots);
    o.size = 800;
    let img = render(&qr, &o).unwrap();
    assert_eq!(decode(&img).unwrap(), text);
}

#[test]
fn transparent_png_has_alpha() {
    let text = "alpha test";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Square);
    o.bg = None;
    let img = render(&qr, &o).unwrap();
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
    assert!(
        svg.contains("<path") || svg.contains("<circle"),
        "rounded SVG should contain web-engine shapes"
    );
    assert!(svg.ends_with("</svg>\n"));

    let o = opts(DotStyle::Square);
    let svg = encode(&img_placeholder(), Format::Svg, &qr, &o).unwrap();
    let svg = String::from_utf8(svg).unwrap();
    assert!(svg.contains("<rect"), "square SVG should contain rects");

    let o = opts(DotStyle::Dots);
    let svg = encode(&img_placeholder(), Format::Svg, &qr, &o).unwrap();
    let svg = String::from_utf8(svg).unwrap();
    assert!(svg.contains("<circle"), "dots SVG should contain circles");
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
    let img = render(&qr, &o).unwrap();
    let bytes = encode(&img, Format::Jpg, &qr, &o).unwrap();
    let decoded = image::load_from_memory(&bytes).unwrap();
    assert_eq!(decoded.color(), image::ColorType::Rgb8);
}

#[test]
fn gradient_renders_and_svg_uses_gradient() {
    let text = "gradient test";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Rounded);
    o.fg = [0xff, 0x00, 0x00];
    o.fg2 = Some([0x00, 0x00, 0xff]);
    let img = render(&qr, &o).unwrap();
    assert_eq!(decode(&img).unwrap(), text);

    let svg = encode(&img_placeholder(), Format::Svg, &qr, &o).unwrap();
    let svg = String::from_utf8(svg).unwrap();
    assert!(svg.contains("<linearGradient"), "SVG should contain a linearGradient");
    assert!(svg.contains("url(#g)"), "SVG shapes should reference the gradient");

    let plain = encode(&img_placeholder(), Format::Svg, &qr, &opts(DotStyle::Rounded)).unwrap();
    let plain = String::from_utf8(plain).unwrap();
    assert!(!plain.contains("linearGradient"), "no gradient without fg2");
}

#[test]
fn canvas_too_small_errors() {
    let text = "tiny";
    let qr = QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::M).unwrap();
    let mut o = opts(DotStyle::Square);
    o.size = 10;
    assert!(render(&qr, &o).is_err(), "size smaller than module count must fail");
}