//! طباعة التذاكر على macOS و Linux عبر نظام الطباعة CUPS.
//!
//! لا توجد هنا واجهة GDI كما في ويندوز، لذلك نرسم التذكرة كصورة نقطية بنفس
//! تخطيط ويندوز (و cosmic-text يتولى تشكيل العربية من اليمين إلى اليسار)، ثم
//! نرسل الصورة إلى أمر النظام `lp` الذي يحوّلها ويرسلها للطابعة مباشرة.

use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, PrinterInfo};
use crate::ticket_layout::{self, Layout};

use cosmic_text::{
    Align, Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, Weight,
};
use image::{Rgb, RgbImage};

use std::path::{Path, PathBuf};
use std::process::Command;

/// الطابعات الحرارية تطبع 203 نقطة/بوصة أي 8 نقاط لكل مليمتر تقريباً.
const DOTS_PER_MM: f32 = 8.0;
/// الهامش الكلي (يمين + يسار) بالمليمتر.
const SIDE_MARGIN_MM: f32 = 6.0;
const TOP_MARGIN: i32 = 20;
const BOTTOM_MARGIN: i32 = 18;
/// مسافة إضافية بالمليمتر قبل القطع حتى لا تُقطع التذكرة عند آخر سطر.
const CUT_MARGIN_MM: f32 = 6.0;
/// مسافة إضافية بعد كل سطر، نفس ما يستخدمه رسم ويندوز.
const LINE_ADVANCE_EXTRA: f32 = 16.0;
/// أقصى ارتفاع للوغو بالنقاط المرجعية.
const LOGO_MAX_HEIGHT: f32 = 150.0;
/// المسافة بعد اللوغو وقبل أول سطر نص.
const LOGO_GAP: f32 = 12.0;

/// خطوط عربية مفضلة بالترتيب إن وُجدت على النظام.
const PREFERRED_FAMILIES: [&str; 8] = [
    "Cairo",
    "Noto Naskh Arabic",
    "Noto Sans Arabic",
    "Geeza Pro",
    "Al Nile",
    "DejaVu Sans",
    "Tahoma",
    "Arial",
];

pub fn list_printers() -> AppResult<Vec<PrinterInfo>> {
    let output = run("lpstat", &["-e"]).map_err(|err| {
        AppError::msg(format!("تعذر قراءة طابعات النظام (CUPS): {err}"))
    })?;

    let default = default_printer();

    Ok(parse_printer_list(&output)
        .into_iter()
        .map(|name| {
            let is_default = default.as_deref() == Some(name.as_str());
            PrinterInfo { name, is_default }
        })
        .collect())
}

pub fn print(
    layout: &Layout,
    logo: Option<&RgbImage>,
    settings: &AppSettings,
) -> Result<(), String> {
    let printer = resolve_printer(&settings.printer_name)?;
    let image = render(layout, logo, settings.paper_width_mm);

    // الغالبية العظمى من طابعات الإيصالات الحرارية على macOS/Linux تُضاف كطابعة
    // "خام" بلا أي تعريف CUPS حقيقي يحوّل الصور، فترسل بايتات أي ملف كما هي
    // للطابعة. لذلك نولّد أوامر ESC/POS بأنفسنا (بعد تحويل الصورة كاملة لأبيض
    // وأسود صرف) ونرسلها كمهمة raw، بدل الاعتماد على تحويل CUPS للصورة.
    let bw_image = ticket_layout::dither_to_black_and_white(&image);
    let escpos = encode_escpos(&bw_image);
    let escpos_path = temp_file_path("bin");
    let escpos_result = std::fs::write(&escpos_path, &escpos)
        .map_err(|err| format!("تعذر تجهيز بيانات الطباعة: {err}"))
        .and_then(|()| run_lp(&printer, &escpos_path, &["raw".to_string()]));
    let _ = std::fs::remove_file(&escpos_path);

    if escpos_result.is_ok() {
        return escpos_result;
    }

    // احتياطي: إن فشل استدعاء lp نفسه (لا لسوء جودة الطباعة، التي لا يمكن
    // اكتشافها برمجياً) نجرّب المسار القديم عبر صورة PNG وفلتر CUPS العادي،
    // مفيد لمن لديه تعريف CUPS حقيقي لا يقبل مهام raw.
    let png_path = temp_file_path("png");
    image
        .save_with_format(&png_path, image::ImageFormat::Png)
        .map_err(|err| format!("تعذر تجهيز صورة التذكرة للطباعة: {err}"))?;
    let height_mm = image.height() as f32 / DOTS_PER_MM + CUT_MARGIN_MM;
    let result = submit_png(&printer, &png_path, settings.paper_width_mm, height_mm);
    let _ = std::fs::remove_file(&png_path);
    result
}

/// حجم شريحة الصورة القصوى بالأسطر لكل أمر `GS v 0` — توافقية واسعة مع
/// المخازن المؤقتة الصغيرة في الطابعات الحرارية الرخيصة المستنسخة.
const ESCPOS_BAND_HEIGHT: u32 = 255;

/// يحوّل صورة أبيض/أسود صرف إلى أوامر ESC/POS جاهزة للإرسال كمهمة raw:
/// تهيئة، ثم صورة نقطية (`GS v 0`) مقسّمة لشرائح، ثم تلقيم وقطع الورق.
fn encode_escpos(image: &RgbImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[0x1B, 0x40]); // ESC @ : تهيئة الطابعة

    let width = image.width();
    let height = image.height();
    let bytes_per_row = ((width + 7) / 8) as usize;

    let mut y = 0u32;
    while y < height {
        let band_height = ESCPOS_BAND_HEIGHT.min(height - y);

        bytes.extend_from_slice(&[0x1D, 0x76, 0x30, 0x00]); // GS v 0 m=0
        bytes.push((bytes_per_row & 0xFF) as u8);
        bytes.push(((bytes_per_row >> 8) & 0xFF) as u8);
        bytes.push((band_height & 0xFF) as u8);
        bytes.push(((band_height >> 8) & 0xFF) as u8);

        for row in 0..band_height {
            let mut row_bytes = vec![0u8; bytes_per_row];
            for x in 0..width {
                let is_black = image.get_pixel(x, y + row)[0] < 128;
                if is_black {
                    row_bytes[(x / 8) as usize] |= 0x80 >> (x % 8);
                }
            }
            bytes.extend_from_slice(&row_bytes);
        }

        y += band_height;
    }

    bytes.extend_from_slice(b"\n\n\n\n");
    bytes.extend_from_slice(&[0x1D, 0x56, 0x42, 0x00]); // GS V B 0 : قطع كامل

    bytes
}

/// يرسم التذكرة كاملة على خلفية بيضاء بنقاط الطابعة الحرارية.
fn render(layout: &Layout, logo: Option<&RgbImage>, paper_width_mm: i64) -> RgbImage {
    let width = printable_width(paper_width_mm);
    let scale = width as f32 / ticket_layout::REFERENCE_WIDTH as f32;

    let mut font_system = FontSystem::new();
    let family = pick_family(&font_system);
    let mut cache = SwashCache::new();

    let mut height = TOP_MARGIN;
    let logo_size = logo.map(|logo| fit_logo(logo, width, scale));
    if let Some((_, logo_height)) = logo_size {
        height += logo_height + logo_gap(scale);
    }
    for line in &layout.lines {
        height += scaled(line.space_before as f32, scale)
            + scaled(line.size as f32, scale)
            + scaled(LINE_ADVANCE_EXTRA, scale);
    }
    height += BOTTOM_MARGIN;

    let mut canvas = RgbImage::from_pixel(width.max(1) as u32, height.max(1) as u32, Rgb([255; 3]));

    let mut y = TOP_MARGIN;
    if let (Some(logo), Some(size)) = (logo, logo_size) {
        draw_logo(&mut canvas, logo, size, y);
        y += size.1 + logo_gap(scale);
    }

    for line in &layout.lines {
        y += scaled(line.space_before as f32, scale);
        draw_line(
            &mut canvas,
            &mut font_system,
            &mut cache,
            family.as_deref(),
            line.size as f32 * scale,
            line.bold,
            &line.text,
            y,
        );
        y += scaled(line.size as f32, scale) + scaled(LINE_ADVANCE_EXTRA, scale);
    }

    canvas
}

fn printable_width(paper_width_mm: i64) -> i32 {
    let printable_mm = (paper_width_mm as f32 - SIDE_MARGIN_MM).max(32.0);
    (printable_mm * DOTS_PER_MM).round() as i32
}

fn logo_gap(scale: f32) -> i32 {
    scaled(LOGO_GAP, scale)
}

fn scaled(value: f32, scale: f32) -> i32 {
    (value * scale).round() as i32
}

fn side_margin(scale: f32) -> f32 {
    (SIDE_MARGIN_MM / 2.0) * DOTS_PER_MM * scale
}

/// يقيس اللوغو داخل عرض التذكرة دون تكبيره فوق حجمه الأصلي.
fn fit_logo(logo: &RgbImage, page_width: i32, scale: f32) -> (i32, i32) {
    let (src_w, src_h) = logo.dimensions();
    if src_w == 0 || src_h == 0 {
        return (0, 0);
    }
    let max_w = (page_width - scaled(24.0, scale)).max(1) as f64;
    let max_h = scaled(LOGO_MAX_HEIGHT, scale).max(1) as f64;
    let factor = (max_w / src_w as f64)
        .min(max_h / src_h as f64)
        .min(1.0);
    (
        ((src_w as f64 * factor).round().max(1.0)) as i32,
        ((src_h as f64 * factor).round().max(1.0)) as i32,
    )
}

fn draw_logo(canvas: &mut RgbImage, logo: &RgbImage, size: (i32, i32), top: i32) {
    let resized = image::imageops::resize(
        logo,
        size.0.max(1) as u32,
        size.1.max(1) as u32,
        image::imageops::FilterType::Triangle,
    );
    let left = (canvas.width() as i32 - size.0) / 2;
        image::imageops::overlay(canvas, &resized, left.max(0) as i64, top.max(0) as i64);
}

fn draw_line(
    canvas: &mut RgbImage,
    font_system: &mut FontSystem,
    cache: &mut SwashCache,
    family: Option<&str>,
    font_size: f32,
    bold: bool,
    text: &str,
    top: i32,
) {
    if text.trim().is_empty() || font_size < 1.0 {
        return;
    }

    let scale = canvas.width() as f32 / ticket_layout::REFERENCE_WIDTH as f32;
    let margin = side_margin(scale);
    let width = (canvas.width() as f32 - 2.0 * margin).max(1.0);
    let line_height = (font_size * 1.45).max(font_size + 4.0);

    let mut buffer = Buffer::new(font_system, Metrics::new(font_size, line_height));
    buffer.set_size(Some(width), Some(line_height));

    let mut attrs = Attrs::new();
    if let Some(family) = family {
        attrs = attrs.family(Family::Name(family));
    }
    if bold {
        attrs = attrs.weight(Weight::BOLD);
    }

    buffer.set_text(text, &attrs, Shaping::Advanced, Some(Align::Center));
    buffer.shape_until_scroll(font_system, false);

    let offset_x = margin.round() as i32;
    buffer.draw(
        font_system,
        cache,
        Color::rgb(0, 0, 0),
        |x, y, width, height, color| {
            blend(canvas, x + offset_x, y + top, width, height, color);
        },
    );
}

/// يدمج بكسلات الحرف (بالشفافية) فوق الخلفية البيضاء.
fn blend(canvas: &mut RgbImage, x: i32, y: i32, width: u32, height: u32, color: Color) {
    let alpha = color.a() as u32;
    if alpha == 0 {
        return;
    }
    let inverse = 255 - alpha;

    for row in 0..height as i32 {
        for column in 0..width as i32 {
            let px = x + column;
            let py = y + row;
            if px < 0 || py < 0 || px >= canvas.width() as i32 || py >= canvas.height() as i32 {
                continue;
            }
            let pixel = canvas.get_pixel_mut(px as u32, py as u32);
            let mix = |foreground: u8, background: u8| -> u8 {
                ((foreground as u32 * alpha + background as u32 * inverse) / 255) as u8
            };
            *pixel = Rgb([
                mix(color.r(), pixel[0]),
                mix(color.g(), pixel[1]),
                mix(color.b(), pixel[2]),
            ]);
        }
    }
}

/// يختار أول خط عربي متاح على النظام ليكون خط التذكرة.
fn pick_family(font_system: &FontSystem) -> Option<String> {
    let available: Vec<String> = font_system
        .db()
        .faces()
        .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
        .collect();

    PREFERRED_FAMILIES
        .iter()
        .find(|preferred| {
            available
                .iter()
                .any(|name| name.eq_ignore_ascii_case(preferred))
        })
        .map(|preferred| preferred.to_string())
}

fn submit_png(
    printer: &str,
    path: &Path,
    paper_width_mm: i64,
    height_mm: f32,
) -> Result<(), String> {
    let media = media_option(paper_width_mm, height_mm);

    match run_lp(printer, path, &[media]) {
        Ok(()) => Ok(()),
        Err(first_error) => {
            // بعض التعريفات لا تقبل مقاساً مخصصاً، فنجرب مقاس الطابعة الافتراضي.
            match run_lp(printer, path, &["fit-to-page".to_string()]) {
                Ok(()) => Ok(()),
                Err(_) => Err(format!(
                    "تعذر إرسال التذكرة إلى الطابعة {printer}: {first_error}"
                )),
            }
        }
    }
}

fn run_lp(printer: &str, path: &Path, options: &[String]) -> Result<(), String> {    let mut command = Command::new("lp");
    command.arg("-d").arg(printer).arg("-n").arg("1");
    for option in options {
        command.arg("-o").arg(option);
    }
    command.arg(path);

    let output = command
        .output()
        .map_err(|err| format!("تعذر تشغيل أمر الطباعة lp: {err}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err("فشل أمر الطباعة lp بدون رسالة".into())
    } else {
        Err(stderr)
    }
}

fn resolve_printer(configured: &str) -> Result<String, String> {
    let configured = configured.trim();
    if !configured.is_empty() {
        return Ok(configured.to_string());
    }
    default_printer().ok_or_else(|| "لا توجد طابعة افتراضية على النظام".into())
}

/// مقاس ورق مخصص بعرض التذكرة وارتفاع محتواها الفعلي.
fn media_option(paper_width_mm: i64, height_mm: f32) -> String {
    let height = height_mm.ceil().max(50.0) as i64;
    format!("media=Custom.{paper_width_mm}x{height}mm")
}

fn default_printer() -> Option<String> {
    let output = run("lpstat", &["-d"]).ok()?;
    parse_default_printer(&output)
}

/// يستخرج أسماء الطابعات من مخرجات `lpstat`.
fn parse_printer_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// يستخرج الطابعة الافتراضية من مخرجات `lpstat -d`.
/// النص يختلف باختلاف لغة النظام، لذلك نأخذ ما بعد النقطتين.
fn parse_default_printer(output: &str) -> Option<String> {
    let line = output.lines().last()?.trim();
    let (_, name) = line.rsplit_once(':')?;
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|err| format!("{program}: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("{program} فشل بدون رسالة")
        } else {
            stderr
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn temp_file_path(extension: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "os-orders-ticket-{}-{nanos}.{extension}",
        std::process::id()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppSettings, Ticket};

    #[test]
    fn media_size_follows_the_ticket_height() {
        assert_eq!(media_option(80, 96.2), "media=Custom.80x97mm");
        assert_eq!(media_option(58, 10.0), "media=Custom.58x50mm");
    }

    #[test]
    fn parses_printer_list_from_lpstat() {
        let output = "HP_Thermal\nKitchen_Printer\n\n";
        assert_eq!(
            parse_printer_list(output),
            vec!["HP_Thermal".to_string(), "Kitchen_Printer".to_string()]
        );
    }

    #[test]
    fn parses_default_printer_from_lpstat() {
        assert_eq!(
            parse_default_printer("system default destination: HP_Thermal\n"),
            Some("HP_Thermal".to_string())
        );
        assert_eq!(
            parse_default_printer("no system default destination\n"),
            None
        );
    }

    #[test]
    fn renders_ticket_that_is_not_blank() {
        let ticket = Ticket {
            id: 1,
            ticket_number: "007".into(),
            sequence: 7,
            status: "WAITING".into(),
            bay_id: None,
            created_at: "2026-09-18T09:30:00".into(),
            started_at: None,
            completed_at: None,
            cancelled_at: None,
            is_priority: false,
        };
        let settings = AppSettings::default();

        let image = render(&ticket_layout::build(&ticket, &settings), None, 80);

        assert_eq!(image.width(), printable_width(80) as u32);
        assert!(image.height() > 300);
        assert!(
            image.pixels().any(|pixel| pixel[0] < 128),
            "التذكرة خرجت فارغة بلا أي نص"
        );
    }

    /// يرسم تذكرة كاملة مع اللوغو ويحفظها في `target/ticket-preview.png`
    /// ليمكن مراجعة الشكل النهائي والخط العربي يدوياً.
    #[test]
    fn renders_preview_with_logo_and_header() {
        let ticket = Ticket {
            id: 1,
            ticket_number: "007".into(),
            sequence: 7,
            status: "WAITING".into(),
            bay_id: None,
            created_at: "2026-09-18T09:30:00".into(),
            started_at: None,
            completed_at: None,
            cancelled_at: None,
            is_priority: true,
        };
        let settings = AppSettings {
            print_header: "OS Tickets".into(),
            ..AppSettings::default()
        };

        let logo = ticket_layout::load_logo("icons/128x128.png").ok();
        let image = render(&ticket_layout::build(&ticket, &settings), logo.as_ref(), 80);

        let preview = Path::new("target/ticket-preview.png");
        std::fs::create_dir_all("target").unwrap();
        image.save_with_format(preview, image::ImageFormat::Png).unwrap();
        eprintln!("معاينة التذكرة: {}", preview.display());

        assert!(image.pixels().any(|pixel| pixel[0] < 128));

        // اللوغو المحدد في الإعدادات يجب أن يظهر في أعلى التذكرة فعلاً.
        if let Some(logo) = logo.as_ref() {
            let scale = image.width() as f32 / ticket_layout::REFERENCE_WIDTH as f32;
            let (_, logo_height) = fit_logo(logo, image.width() as i32, scale);
            let dark_pixels: usize = (TOP_MARGIN as u32..TOP_MARGIN as u32 + logo_height as u32)
                .map(|y| {
                    (0..image.width())
                        .filter(|x| image.get_pixel(*x, y)[0] < 128)
                        .count()
                })
                .sum();
            assert!(dark_pixels > 500, "اللوغو لم يُرسم في أعلى التذكرة");
        }
    }

    #[test]
    fn arabic_text_is_shaped_and_ordered_right_to_left() {
        let text = "رقم الدور";
        let mut font_system = FontSystem::new();
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(32.0, 46.0));
        buffer.set_size(Some(500.0), Some(46.0));
        buffer.set_text(text, &Attrs::new(), Shaping::Advanced, Some(Align::Center));
        buffer.shape_until_scroll(&mut font_system, false);

        assert_eq!(
            buffer.is_rtl(0),
            Some(true),
            "النص العربي لم يُشكَّل باتجاه اليمين إلى اليسار"
        );

        let run = buffer.layout_runs().next().expect("لا يوجد سطر مرسوم");
        assert!(!run.glyphs.is_empty(), "لم يُرسم أي حرف عربي");
        assert!(
            run.glyphs.iter().all(|glyph| glyph.glyph_id != 0),
            "خط النظام لا يحتوي الحروف العربية المطلوبة"
        );

        let first_character = run.glyphs.iter().find(|glyph| glyph.start == 0).unwrap();
        let last_character = run.glyphs.iter().max_by_key(|glyph| glyph.end).unwrap();
        assert!(
            first_character.x > last_character.x,
            "ترتيب الحروف معكوس: أول حرف يجب أن يظهر يمين آخر حرف"
        );
    }

    #[test]
    fn escpos_starts_with_init_and_ends_with_a_cut() {
        let image = RgbImage::from_pixel(16, 3, Rgb([255, 255, 255]));
        let bytes = encode_escpos(&image);

        assert_eq!(&bytes[0..2], &[0x1B, 0x40], "يجب أن تبدأ المهمة بأمر تهيئة ESC @");

        let raster_header = &bytes[2..6];
        assert_eq!(
            raster_header,
            &[0x1D, 0x76, 0x30, 0x00],
            "يجب أن تلي التهيئة ترويسة صورة نقطية GS v 0"
        );

        // عرض 16 بكسل = بايتان لكل سطر بالضبط.
        assert_eq!(bytes[6], 2, "bytes-per-row يجب أن يكون ⌈16/8⌉ = 2");
        assert_eq!(bytes[7], 0);
        // ارتفاع الشريحة 3 أسطر كما في الصورة التجريبية.
        assert_eq!(bytes[8], 3);
        assert_eq!(bytes[9], 0);

        assert_eq!(
            &bytes[bytes.len() - 4..],
            &[0x1D, 0x56, 0x42, 0x00],
            "يجب أن تنتهي المهمة بأمر قطع الورق"
        );
    }

    #[test]
    fn escpos_marks_dark_pixels_as_set_bits() {
        // بكسل أسود واحد في أقصى يسار أول سطر يجب أن يُشعل البت الأعلى فقط.
        let mut image = RgbImage::from_pixel(8, 1, Rgb([255, 255, 255]));
        image.put_pixel(0, 0, Rgb([0, 0, 0]));

        let bytes = encode_escpos(&image);
        let row_byte = bytes[10]; // أول بايت بيانات مباشرة بعد ترويسة 10 بايتات (2 تهيئة + 8 GS v 0)

        assert_eq!(row_byte, 0b1000_0000);
    }

    #[test]
    fn escpos_bands_tall_images_into_multiple_raster_commands() {
        let image = RgbImage::from_pixel(8, (ESCPOS_BAND_HEIGHT * 2 + 1) as u32, Rgb([255; 3]));
        let bytes = encode_escpos(&image);

        let raster_command_count = bytes
            .windows(3)
            .filter(|window| *window == [0x1D, 0x76, 0x30])
            .count();
        assert_eq!(
            raster_command_count, 3,
            "صورة أطول من حد الشريحة يجب أن تُقسَّم لثلاثة أوامر GS v 0"
        );
    }
}
