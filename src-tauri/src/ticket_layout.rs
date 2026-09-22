//! تخطيط تذكرة الطباعة.
//!
//! يُبنى التخطيط مرة واحدة بقياسات مرجعية (تذكرة 80mm على 203 نقطة/بوصة) ثم
//! يرسمه كل نظام بطريقته: GDI على ويندوز، وصورة نقطية تُرسَل إلى CUPS على
//! macOS/Linux. الهدف أن تخرج التذكرة بالمعلومات والترتيب نفسه على كل نظام.

use crate::models::{AppSettings, Ticket};

/// العرض المرجعي بالنقاط الذي عُويرت عليه أحجام الخطوط أدناه.
pub const REFERENCE_WIDTH: i32 = 576;

/// تسمية سطر التاريخ والوقت الذي يظهر في أعلى التذكرة تحت اللوغو.
pub const TIME_LABEL: &str = "التاريخ والوقت";

const HEADER_SIZE: i32 = 48;
const TIME_SIZE: i32 = 26;
const LABEL_SIZE: i32 = 32;
const NUMBER_SIZE: i32 = 108;
const PRIORITY_SIZE: i32 = 30;
const NOTE_SIZE: i32 = 27;
const THANKS_SIZE: i32 = 26;
const APP_FOOTER_SIZE: i32 = 20;
/// اسم التطبيق الذي يُطبع كسطر صغير في أسفل كل تذكرة.
const APP_FOOTER_TEXT: &str = "OS Tickets";

/// سطر واحد في التذكرة.
pub struct Line {
    pub text: String,
    pub size: i32,
    pub bold: bool,
    /// مسافة إضافية قبل السطر بالنقاط المرجعية.
    pub space_before: i32,
}

impl Line {
    fn new(text: impl Into<String>, size: i32, bold: bool) -> Self {
        Self {
            text: text.into(),
            size,
            bold,
            space_before: 0,
        }
    }

    fn spaced(text: impl Into<String>, size: i32, bold: bool, space_before: i32) -> Self {
        Self {
            space_before,
            ..Self::new(text, size, bold)
        }
    }
}

/// محتوى التذكرة جاهزاً للرسم، بدون أي تفاصيل خاصة بنظام تشغيل.
pub struct Layout {
    pub lines: Vec<Line>,
}

pub fn build(ticket: &Ticket, settings: &AppSettings) -> Layout {
    let arabic_digits = settings.number_format == "ar";
    let mut lines = Vec::new();

    let header = settings.print_header.trim();
    if !header.is_empty() {
        lines.push(Line::new(header, HEADER_SIZE, true));
    }

    // التاريخ والوقت في الأعلى مع اللوغو حتى يظهر وقت إنشاء الدور بوضوح.
    lines.push(Line::new(
        time_text(&ticket.created_at, arabic_digits),
        TIME_SIZE,
        false,
    ));

    lines.push(Line::spaced("رقم الدور", LABEL_SIZE, false, 14));
    lines.push(Line::spaced(
        format_digits(&ticket.ticket_number, arabic_digits),
        NUMBER_SIZE,
        true,
        6,
    ));

    if ticket.is_priority {
        lines.push(Line::spaced("⚡ دور ذو أولوية", PRIORITY_SIZE, true, 10));
    }

    lines.push(Line::spaced("الرجاء انتظار دوركم", NOTE_SIZE, false, 12));
    lines.push(Line::spaced("شكراً لزيارتكم", THANKS_SIZE, false, 6));
    lines.push(Line::spaced(APP_FOOTER_TEXT, APP_FOOTER_SIZE, false, 8));

    Layout { lines }
}

/// نص سطر التاريخ والوقت كاملاً مع التسمية، مع تحويل الأرقام إن طُلب.
pub fn time_text(created_at: &str, arabic_digits: bool) -> String {
    format_digits(
        &format!("{}  {}", TIME_LABEL, format_ticket_time(created_at)),
        arabic_digits,
    )
}

pub fn format_digits(text: &str, arabic: bool) -> String {
    if !arabic {
        return text.to_string();
    }
    text.chars()
        .map(|c| match c {
            '0'..='9' => char::from_u32('٠' as u32 + (c as u32 - '0' as u32)).unwrap_or(c),
            other => other,
        })
        .collect()
}

pub fn format_ticket_time(created_at: &str) -> String {
    chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S")
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| created_at.to_string())
}

/// يحمّل صورة اللوغو المحددة في الإعدادات على خلفية بيضاء.
///
/// الشفافية تُدمج مع الأبيض لأن الطابعة الحرارية تطبع لوناً واحداً، ولو
/// تركت الشفافية شفافة لطبعت مساحة سوداء حول اللوغو.
pub fn load_logo(path: &str) -> Result<image::RgbImage, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("لم تُحدَّد صورة اللوغو في الإعدادات".into());
    }
    if !std::path::Path::new(trimmed).is_file() {
        return Err(format!("ملف اللوغو غير موجود: {trimmed}"));
    }
    let rgba = image::open(trimmed)
        .map_err(|e| format!("تعذر قراءة صورة اللوغو: {e}"))?
        .into_rgba8();

    let (width, height) = rgba.dimensions();
    let mut rgb = image::RgbImage::new(width, height);
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as u32;
        let blend = |channel: u8| -> u8 {
            ((channel as u32 * alpha + 255 * (255 - alpha)) / 255) as u8
        };
        rgb.put_pixel(
            x,
            y,
            image::Rgb([blend(pixel[0]), blend(pixel[1]), blend(pixel[2])]),
        );
    }
    Ok(dither_to_black_and_white(&rgb))
}

/// يحوّل الصورة إلى أبيض وأسود صرف (بكسل أسود أو أبيض فقط، بلا رمادي) عبر
/// خوارزمية Floyd–Steinberg، بدل ترك التحويل لتعريف الطابعة الذي غالباً ما
/// يُخرج نتيجة غامقة أو مبقّعة على الطابعات الحرارية أحادية اللون. تُستخدم
/// على اللوغو وحده وعلى صورة التذكرة كاملة (لتحويل تنعيم حواف النص أيضاً)
/// قبل تحويلها لأوامر ESC/POS في `printing_cups`.
pub fn dither_to_black_and_white(img: &image::RgbImage) -> image::RgbImage {
    let (width, height) = img.dimensions();
    let w = width as i32;
    let h = height as i32;

    let mut gray: Vec<f32> = img
        .pixels()
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();

    let mut out = image::RgbImage::new(width, height);
    let add_error = |gray: &mut Vec<f32>, x: i32, y: i32, amount: f32| {
        if x >= 0 && x < w && y >= 0 && y < h {
            let idx = (y * w + x) as usize;
            gray[idx] = (gray[idx] + amount).clamp(0.0, 255.0);
        }
    };

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let old = gray[idx];
            let is_black = old < 128.0;
            out.put_pixel(
                x as u32,
                y as u32,
                if is_black { image::Rgb([0, 0, 0]) } else { image::Rgb([255, 255, 255]) },
            );

            let error = old - if is_black { 0.0 } else { 255.0 };
            add_error(&mut gray, x + 1, y, error * 7.0 / 16.0);
            add_error(&mut gray, x - 1, y + 1, error * 3.0 / 16.0);
            add_error(&mut gray, x, y + 1, error * 5.0 / 16.0);
            add_error(&mut gray, x + 1, y + 1, error * 1.0 / 16.0);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_logo_areas_become_white() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("logo.png");
        let mut rgba = image::RgbaImage::new(2, 2);
        rgba.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
        rgba.put_pixel(1, 0, image::Rgba([0, 0, 0, 255]));
        rgba.put_pixel(0, 1, image::Rgba([255, 255, 255, 255]));
        rgba.put_pixel(1, 1, image::Rgba([255, 0, 0, 128]));
        rgba.save(&path).unwrap();

        let rgb = load_logo(path.to_str().unwrap()).unwrap();

        assert_eq!(rgb.get_pixel(0, 0)[0], 255, "البكسل الشفاف يجب أن يصير أبيض");
        assert_eq!(rgb.get_pixel(1, 0)[0], 0, "البكسل الأسود يجب أن يبقى أسود");
        assert!(
            rgb.get_pixel(1, 1)[0] > 100,
            "اللون نصف الشفاف يجب أن يفتح لا أن يبقى غامقاً"
        );
    }

    #[test]
    fn missing_logo_reports_the_reason() {
        assert!(load_logo(" ").unwrap_err().contains("الإعدادات"));
        assert!(load_logo("/no/such/logo.png")
            .unwrap_err()
            .contains("غير موجود"));
    }
}
