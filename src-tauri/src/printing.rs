use crate::error::AppResult;
#[cfg(windows)]
use crate::error::AppError;
use crate::models::{AppSettings, PrinterInfo, Ticket};
use crate::ticket_layout::{self, Layout};

pub fn list_printers() -> AppResult<Vec<PrinterInfo>> {
    #[cfg(windows)]
    {
        windows_print::list_printers()
    }

    #[cfg(not(windows))]
    {
        crate::printing_cups::list_printers()
    }
}

pub fn print_ticket(ticket: &Ticket, settings: &AppSettings) -> Result<(), String> {
    let layout = ticket_layout::build(ticket, settings);
    let logo = load_logo(settings);
    submit(&layout, logo.as_ref(), settings)
}

fn submit(
    layout: &Layout,
    logo: Option<&image::RgbImage>,
    settings: &AppSettings,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows_print::print(layout, logo, settings)
    }

    #[cfg(not(windows))]
    {
        crate::printing_cups::print(layout, logo, settings)
    }
}

/// يحمّل اللوغو من الإعدادات. لا نُفشل الطباعة إن تعذّر تحميله،
/// لكن نسجّل السبب بدل تجاهله بصمت.
fn load_logo(settings: &AppSettings) -> Option<image::RgbImage> {
    if settings.logo_path.trim().is_empty() {
        return None;
    }
    match ticket_layout::load_logo(&settings.logo_path) {
        Ok(logo) => Some(logo),
        Err(err) => {
            eprintln!("تحذير: لم تُطبَع صورة اللوغو - {err}");
            None
        }
    }
}

pub fn test_print(settings: &AppSettings) -> Result<(), String> {
    // نتحقق من اللوغو هنا حتى يظهر سببه للمستخدم مباشرة عند اختبار الطباعة.
    if !settings.logo_path.trim().is_empty() {
        ticket_layout::load_logo(&settings.logo_path)?;
    }

    let sample = Ticket {
        id: 0,
        ticket_number: format_ticket_sample(settings),
        sequence: 0,
        status: "WAITING".into(),
        bay_id: None,
        created_at: crate::models::now_iso(),
        started_at: None,
        completed_at: None,
        cancelled_at: None,
        is_priority: false,
    };

    print_ticket(&sample, settings)
}

fn format_ticket_sample(settings: &AppSettings) -> String {
    crate::models::format_ticket_number(&settings.ticket_prefix, 1)
}

#[cfg(windows)]
mod windows_print {
    use super::*;

    use std::mem::size_of;

    use windows::core::{PCWSTR, PWSTR};

    use windows::Win32::Graphics::Gdi::{
        CreateDCW,
        CreateFontW,
        DeleteDC,
        DeleteObject,
        DrawTextW,
        GetDeviceCaps,
        SelectObject,
        SetBkMode,
        SetTextColor,
        StretchDIBits,
        BITMAPINFO,
        BITMAPINFOHEADER,
        CLEARTYPE_QUALITY,
        CLIP_DEFAULT_PRECIS,
        DEFAULT_PITCH,
        DIB_RGB_COLORS,
        DT_CENTER,
        DT_RTLREADING,
        DT_WORDBREAK,
        FW_BOLD,
        FW_NORMAL,
        HDC,
        HGDIOBJ,
        HORZRES,
        OUT_TT_PRECIS,
        SRCCOPY,
        TRANSPARENT,
        FONT_CHARSET,
        FONT_CLIP_PRECISION,
        FONT_OUTPUT_PRECISION,
        FONT_QUALITY,
    };

    use windows::Win32::Graphics::Printing::{
        EnumPrintersW,
        GetDefaultPrinterW,
        PRINTER_ENUM_CONNECTIONS,
        PRINTER_ENUM_LOCAL,
        PRINTER_INFO_4W,
    };

    /*
     * Win32 GDI printing APIs.
     *
     * windows 0.61 no longer exposes these APIs through the same
     * generated module layout used by the old code, so we call the
     * native GDI functions directly.
     */

    #[repr(C)]
    struct DOCINFOW {
        cb_size: i32,
        lpsz_doc_name: PCWSTR,
        lpsz_output: PCWSTR,
        lpsz_datatype: PCWSTR,
        fw_type: u32,
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn StartDocW(hdc: HDC, lpdi: *const DOCINFOW) -> i32;
        fn StartPage(hdc: HDC) -> i32;
        fn EndPage(hdc: HDC) -> i32;
        fn EndDoc(hdc: HDC) -> i32;
    }

    const ARABIC_CHARSET_VALUE: u32 = 178;

    pub fn list_printers() -> AppResult<Vec<PrinterInfo>> {
        unsafe {
            let default_name = default_printer().unwrap_or_default();

            let mut needed = 0u32;
            let mut returned = 0u32;

            let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;

            let _ = EnumPrintersW(
                flags,
                PCWSTR::null(),
                4,
                None,
                &mut needed,
                &mut returned,
            );

            if needed == 0 {
                return Ok(Vec::new());
            }

            let mut buffer = vec![0u8; needed as usize];

            EnumPrintersW(
                flags,
                PCWSTR::null(),
                4,
                Some(buffer.as_mut_slice()),
                &mut needed,
                &mut returned,
            )
            .map_err(|e| AppError::msg(format!("تعذر قراءة الطابعات: {e}")))?;

            let count = returned as usize;
            let infos = buffer.as_ptr() as *const PRINTER_INFO_4W;

            let mut printers = Vec::new();

            for i in 0..count {
                let info = infos.add(i);

                let name = pwstr_to_string((*info).pPrinterName);

                if name.is_empty() {
                    continue;
                }

                let is_default =
                    !default_name.is_empty() && name == default_name;

                printers.push(PrinterInfo {
                    name,
                    is_default,
                });
            }

            Ok(printers)
        }
    }

    pub fn print(
        layout: &Layout,
        logo: Option<&image::RgbImage>,
        settings: &AppSettings,
    ) -> Result<(), String> {
        unsafe {
            let printer = resolve_printer(&settings.printer_name)?;

            let printer_wide: Vec<u16> = printer
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let hdc = CreateDCW(
                windows::core::w!("WINSPOOL"),
                PCWSTR(printer_wide.as_ptr()),
                PCWSTR::null(),
                None,
            );

            if hdc.is_invalid() {
                return Err(
                    "تعذر الاتصال بالطابعة. تحقق أنها متصلة وليست Offline."
                        .into(),
                );
            }

            let doc_name: Vec<u16> = "تذكرة البارودي"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let doc = DOCINFOW {
                cb_size: size_of::<DOCINFOW>() as i32,
                lpsz_doc_name: PCWSTR(doc_name.as_ptr()),
                lpsz_output: PCWSTR::null(),
                lpsz_datatype: PCWSTR::null(),
                fw_type: 0,
            };

            let start_doc_result = StartDocW(hdc, &doc);

            if start_doc_result <= 0 {
                let _ = DeleteDC(hdc);

                return Err(
                    "تعذر بدء مهمة الطباعة. تحقق من حالة الطابعة."
                        .into(),
                );
            }

            let start_page_result = StartPage(hdc);

            if start_page_result <= 0 {
                let _ = EndDoc(hdc);
                let _ = DeleteDC(hdc);

                return Err("تعذر بدء صفحة الطباعة".into());
            }

            let width = GetDeviceCaps(Some(hdc), HORZRES);

            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(
                hdc,
                windows::Win32::Foundation::COLORREF(rgb(0, 0, 0)),
            );

            let mut y = 5;

            if let Some(logo) = logo {
                y += draw_logo(hdc, width, y, logo);
            }

            for line in &layout.lines {
                y += line.space_before;
                y += draw_line(
                    hdc,
                    width,
                    y,
                    line.size,
                    line.bold,
                    &line.text,
                );
            }

            let _ = y;

            let end_page_result = EndPage(hdc);

            if end_page_result <= 0 {
                let _ = EndDoc(hdc);
                let _ = DeleteDC(hdc);

                return Err("فشل إنهاء صفحة الطباعة".into());
            }

            let end_doc_result = EndDoc(hdc);

            if end_doc_result <= 0 {
                let _ = DeleteDC(hdc);

                return Err("فشل إنهاء مستند الطباعة".into());
            }

            let _ = DeleteDC(hdc);

            Ok(())
        }
    }

    /// Decodes the logo image and blits it centered at the top of the ticket.
    /// Returns the vertical space consumed (0 if there is no logo, so a missing
    /// logo never aborts the print job).
    unsafe fn draw_logo(hdc: HDC, page_width: i32, y: i32, img: &image::RgbImage) -> i32 {
        let (src_w, src_h) = img.dimensions();
        if src_w == 0 || src_h == 0 {
            return 0;
        }

        let max_w = (page_width - 10).max(1) as f64;
        let max_h = 150.0f64;
        let scale = (max_w / src_w as f64).min(max_h / src_h as f64).min(1.0);
        let dest_w = ((src_w as f64) * scale).round().max(1.0) as i32;
        let dest_h = ((src_h as f64) * scale).round().max(1.0) as i32;

        // Bottom-up 24bpp BGR DIB, rows padded to a 4-byte boundary.
        let row_stride = (((src_w * 3) + 3) / 4) * 4;
        let mut bits = vec![0u8; (row_stride * src_h) as usize];
        for (px, py, pixel) in img.enumerate_pixels() {
            let dest_row = src_h - 1 - py;
            let offset = (dest_row * row_stride + px * 3) as usize;
            bits[offset] = pixel[2];
            bits[offset + 1] = pixel[1];
            bits[offset + 2] = pixel[0];
        }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = src_w as i32;
        bmi.bmiHeader.biHeight = src_h as i32;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 24;
        bmi.bmiHeader.biCompression = 0;

        let x = ((page_width - dest_w) / 2).max(0);

        let _ = StretchDIBits(
            hdc,
            x,
            y,
            dest_w,
            dest_h,
            0,
            0,
            src_w as i32,
            src_h as i32,
            Some(bits.as_ptr() as *const core::ffi::c_void),
            &bmi,
            DIB_RGB_COLORS,
            SRCCOPY,
        );

        dest_h + 5
    }

    unsafe fn draw_line(
        hdc: HDC,
        page_width: i32,
        y: i32,
        size: i32,
        bold: bool,
        text: &str,
    ) -> i32 {
        /*
         * windows 0.61 uses strongly typed values for the CreateFontW
         * charset/precision/quality parameters.
         */
        let font = CreateFontW(
            size,
            0,
            0,
            0,
            if bold {
                FW_BOLD.0 as i32
            } else {
                FW_NORMAL.0 as i32
            },
            0,
            0,
            0,
            FONT_CHARSET(ARABIC_CHARSET_VALUE as u8),
            FONT_OUTPUT_PRECISION(OUT_TT_PRECIS.0),
            FONT_CLIP_PRECISION(CLIP_DEFAULT_PRECIS.0),
            FONT_QUALITY(CLEARTYPE_QUALITY.0),
            DEFAULT_PITCH.0 as u32,
            windows::core::w!("Segoe UI"),
        );

        if font.is_invalid() {
            return size + 6;
        }

        let old = SelectObject(hdc, HGDIOBJ(font.0));

        let mut wide: Vec<u16> = text.encode_utf16().collect();

        let mut rect = windows::Win32::Foundation::RECT {
            left: 5,
            top: y,
            right: page_width.saturating_sub(5),
            bottom: y + size + 12,
        };

        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_CENTER | DT_RTLREADING | DT_WORDBREAK,
        );

        let _ = SelectObject(hdc, old);

        let _ = DeleteObject(HGDIOBJ(font.0));

        size + 6
    }

    fn resolve_printer(configured: &str) -> Result<String, String> {
        if !configured.trim().is_empty() {
            return Ok(configured.trim().to_string());
        }

        default_printer()
            .ok_or_else(|| "لا توجد طابعة افتراضية على النظام".into())
    }

    fn default_printer() -> Option<String> {
        unsafe {
            let mut size = 0u32;

            let _ = GetDefaultPrinterW(
                Some(PWSTR::null()),
                &mut size,
            );

            if size == 0 {
                return None;
            }

            let mut buf = vec![0u16; size as usize];

            GetDefaultPrinterW(
                Some(PWSTR(buf.as_mut_ptr())),
                &mut size,
            )
            .ok()
            .ok()?;

            Some(
                String::from_utf16_lossy(&buf)
                    .trim_end_matches('\0')
                    .to_string(),
            )
        }
    }

    unsafe fn pwstr_to_string(value: PWSTR) -> String {
        if value.is_null() {
            return String::new();
        }

        value.to_string().unwrap_or_default()
    }

    #[inline]
    fn rgb(red: u8, green: u8, blue: u8) -> u32 {
        (red as u32)
            | ((green as u32) << 8)
            | ((blue as u32) << 16)
    }
}