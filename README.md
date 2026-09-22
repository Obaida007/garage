# OS Tickets (OS Tickets)

تطبيق Windows مكتبي (Offline) لإدارة أدوار الزبائن في OS Tickets لخدمات وصيانة السيارات.

التقنيات: React + TypeScript + Vite + Tailwind CSS + shadcn/ui + Tauri + SQLite.

## متطلبات التطوير

- Windows 10/11
- Node.js 20+
- Rust stable (`rustup`)
- Visual Studio Build Tools مع workload **Desktop development with C++**
- WebView2 (مثبت عادةً مع Windows 10)

## تشغيل المشروع للتطوير

```bash
npm install
npm run tauri:dev
```

يفتح التطبيق كنافذة Windows وليس في المتصفح. شاشة الانتظار تُنشأ تلقائياً على الشاشة الثانية إن وُجدت.

اختبارات منطق الأدوار:

```bash
npm test
cd src-tauri
cargo test
```

## قاعدة البيانات

SQLite تُنشأ تلقائياً عند أول تشغيل في مجلد بيانات التطبيق:

`%AppData%\com.ostickets.garage\garage.db`

عند أول تشغيل بعد تحديث الإصدار، يُنشئ التطبيق مجلد البيانات الجديد وينسخ إليه `garage.db` والصورة المخصصة من المجلدات القديمة `%AppData%\com.albaroudi.garage` و `%AppData%\com.osorders.garage` تلقائياً، فلا تفقد الأدوار أو الإعدادات.

لا حاجة لتثبيت SQLite أو تشغيل خادم.

## الطابعة

من الإعدادات اختر الطابعة الحرارية وحجم الورق 58mm أو 80mm ثم **اختبار الطباعة**.

الطباعة تتم مباشرة إلى الطابعة دون نافذة طباعة المتصفح على ويندوز و macOS و Linux:

- **ويندوز**: عبر Windows GDI، والنص العربي بخط Segoe UI من النظام.
- **macOS و Linux**: تُرسم التذكرة كصورة بنقاط الطابعة الحرارية (cosmic-text لتشكيل العربية من اليمين إلى اليسار) ثم تُرسل إلى النظام عبر CUPS (`lp`).

على macOS/Linux يجب أن تكون الطابعة مضافة في إعدادات النظام — يظهر منها كل طابعات النظام في الإعدادات.

إذا تعذر تحميل صورة اللوغو المحفوظة في الإعدادات تستمر الطباعة بدونها، ويظهر سبب المشكلة عند ضغط **اختبار الطباعة**.

إذا فشلت الطباعة يبقى الدور صالحاً ويمكن **إعادة طباعة**.

## الشاشة الثانية

1. اجعل الشاشة الثانية Extended (توسيع) وليس Duplicate (تكرار).
2. شغّل التطبيق.
3. تُفتح شاشة الانتظار تلقائياً Fullscreen على الشاشة غير الرئيسية.
4. من الإعدادات يمكن اختيار الشاشة يدوياً واختبارها.

## التشغيل التلقائي مع Windows

الإعدادات ← تفعيل **تشغيل التطبيق تلقائياً مع Windows**.

## إنشاء Windows Installer

```bash
npm run tauri:build
```

الملف الناتج يكون عادةً داخل:

`src-tauri/target/release/bundle/nsis/`

مثل:

`OS Tickets_1.0.0_x64-setup.exe`

اسم الملف مشتق من `productName` ورقم الإصدار في `src-tauri/tauri.conf.json`.

بعد التثبيت يظهر البرنامج باسم **OS Tickets** مع اختصار سطح المكتب وقائمة ابدأ. المستخدم النهائي لا يحتاج Node.js أو Rust أو npm.

## إصدار Production

1. حدّث رقم الإصدار في `package.json` و `src-tauri/tauri.conf.json` و `src-tauri/Cargo.toml`.
2. شغّل `npm test` و `cargo test --manifest-path src-tauri/Cargo.toml`.
3. شغّل `npm run tauri:build`.
4. وزّع ملف الـ NSIS فقط.
--------------
use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, PrinterInfo, Ticket};

pub fn list_printers() -> AppResult<Vec<PrinterInfo>> {
    #[cfg(windows)]
    {
        windows_print::list_printers()
    }

    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

pub fn print_ticket(ticket: &Ticket, settings: &AppSettings) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows_print::print_ticket(ticket, settings)
    }

    #[cfg(not(windows))]
    {
        let _ = (ticket, settings);
        Err("الطباعة متاحة على Windows فقط".into())
    }
}

pub fn test_print(settings: &AppSettings) -> Result<(), String> {
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
        VERTRES,
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

    pub fn print_ticket(
        ticket: &Ticket,
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
            let height = GetDeviceCaps(Some(hdc), VERTRES);

            // paper_width_mm is currently used by the caller/settings layer.
            // Keep it here so the existing settings contract remains unchanged.
            let _ = (settings.paper_width_mm, height);

            SetBkMode(hdc, TRANSPARENT);
SetTextColor(
    hdc,
    windows::Win32::Foundation::COLORREF(rgb(0, 0, 0)),
);
            let mut y = 40;

            y += draw_logo(hdc, width, y, &settings.logo_path);

            y += draw_line(
                hdc,
                width,
                y,
                48,
                true,
                &settings.print_header,
            );

            y += 16;

            y += draw_line(
                hdc,
                width,
                y,
                32,
                false,
                "رقم الدور",
            );

            y += 8;

            let use_arabic_digits = settings.number_format == "ar";
            let ticket_number_display = format_digits(&ticket.ticket_number, use_arabic_digits);

            y += draw_line(
                hdc,
                width,
                y,
                96,
                true,
                &ticket_number_display,
            );

            y += 24;

            if ticket.is_priority {
                y += draw_line(
                    hdc,
                    width,
                    y,
                    30,
                    true,
                    "⚡ دور ذو أولوية",
                );
                y += 12;
            }

            let time_display = format_digits(&format_ticket_time(&ticket.created_at), use_arabic_digits);

            y += draw_line(
                hdc,
                width,
                y,
                24,
                false,
                &time_display,
            );

            y += 16;

            y += draw_line(
                hdc,
                width,
                y,
                28,
                false,
                "الرجاء انتظار دوركم",
            );

            y += 12;

            y += draw_line(
                hdc,
                width,
                y,
                26,
                false,
                "شكراً لزيارتكم",
            );

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

    fn format_digits(text: &str, arabic: bool) -> String {
        if !arabic {
            return text.to_string();
        }
        text.chars()
            .map(|c| match c {
                '0'..='9' => {
                    char::from_u32('٠' as u32 + (c as u32 - '0' as u32)).unwrap_or(c)
                }
                other => other,
            })
            .collect()
    }

    fn format_ticket_time(created_at: &str) -> String {
        chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%dT%H:%M:%S")
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|_| created_at.to_string())
    }

    /// Decodes the logo image and blits it centered at the top of the ticket.
    /// Returns the vertical space consumed (0 if there is no logo or it failed to load,
    /// so a broken/missing logo never aborts the print job).
    unsafe fn draw_logo(hdc: HDC, page_width: i32, y: i32, path: &str) -> i32 {
        if path.is_empty() {
            return 0;
        }

        let img = match image::open(path) {
            Ok(img) => img.into_rgb8(),
            Err(_) => return 0,
        };

        let (src_w, src_h) = img.dimensions();
        if src_w == 0 || src_h == 0 {
            return 0;
        }

        let max_w = (page_width - 40).max(1) as f64;
        let max_h = 220.0f64;
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

        dest_h + 20
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
            return size + 28;
        }

        let old = SelectObject(hdc, HGDIOBJ(font.0));

        let mut wide: Vec<u16> = text.encode_utf16().collect();

        let mut rect = windows::Win32::Foundation::RECT {
            left: 20,
            top: y,
            right: page_width.saturating_sub(20),
            bottom: y + size + 24,
        };

        let _ = DrawTextW(
            hdc,
            &mut wide,
            &mut rect,
            DT_CENTER | DT_RTLREADING | DT_WORDBREAK,
        );

        let _ = SelectObject(hdc, old);

        let _ = DeleteObject(HGDIOBJ(font.0));

        size + 28
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