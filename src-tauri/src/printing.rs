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
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        CreateDCW, CreateFontW, DeleteDC, DeleteObject, DrawTextW, EndDoc, EndPage, GetDeviceCaps,
        SelectObject, SetBkMode, SetTextColor, StartDocW, StartPage, CLEARTYPE_QUALITY,
        CLIP_DEFAULT_PRECIS, DEFAULT_PITCH, DOCINFOW, DT_CENTER, DT_RTLREADING, DT_WORDBREAK,
        FW_BOLD, FW_NORMAL, HGDIOBJ, HORZRES, OUT_TT_PRECIS, RGB, TRANSPARENT, VERTRES,
    };
    use windows::Win32::Graphics::Printing::{
        EnumPrintersW, GetDefaultPrinterW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL,
        PRINTER_INFO_4W,
    };

    const ARABIC_CHARSET: i32 = 178;

    pub fn list_printers() -> AppResult<Vec<PrinterInfo>> {
        unsafe {
            let default_name = default_printer().unwrap_or_default();
            let mut needed = 0u32;
            let mut returned = 0u32;
            let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;
            let _ = EnumPrintersW(flags, PCWSTR::null(), 4, None, &mut needed, &mut returned);
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
                let is_default = !default_name.is_empty() && name == default_name;
                printers.push(PrinterInfo { name, is_default });
            }
            Ok(printers)
        }
    }

    pub fn print_ticket(ticket: &Ticket, settings: &AppSettings) -> Result<(), String> {
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
                return Err("تعذر الاتصال بالطابعة. تحقق أنها متصلة وليست Offline.".into());
            }

            let doc_name: Vec<u16> = "Al-Sahil Ticket".encode_utf16().chain(std::iter::once(0)).collect();
            let doc = DOCINFOW {
                cbSize: size_of::<DOCINFOW>() as i32,
                lpszDocName: PCWSTR(doc_name.as_ptr()),
                lpszOutput: PCWSTR::null(),
                lpszDatatype: PCWSTR::null(),
                fwType: 0,
            };
            if StartDocW(hdc, &doc) <= 0 {
                let _ = DeleteDC(hdc);
                return Err("تعذر بدء مهمة الطباعة".into());
            }
            if StartPage(hdc) <= 0 {
                let _ = EndDoc(hdc);
                let _ = DeleteDC(hdc);
                return Err("تعذر بدء صفحة الطباعة".into());
            }

            let width = GetDeviceCaps(hdc, HORZRES);
            let height = GetDeviceCaps(hdc, VERTRES);
            let _ = (settings.paper_width_mm, height);
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, RGB(0, 0, 0));

            let mut y = 40;
            y += draw_line(
                hdc,
                width,
                y,
                48,
                true,
                &settings.print_header,
            );
            y += 16;
            y += draw_line(hdc, width, y, 32, false, "رقم الدور");
            y += 8;
            y += draw_line(hdc, width, y, 96, true, &ticket.ticket_number);
            y += 24;
            y += draw_line(hdc, width, y, 28, false, "الرجاء انتظار دوركم");
            y += 12;
            y += draw_line(hdc, width, y, 26, false, "شكراً لزيارتكم");
            let _ = y;

            if EndPage(hdc) <= 0 {
                let _ = EndDoc(hdc);
                let _ = DeleteDC(hdc);
                return Err("فشل إنهاء صفحة الطباعة".into());
            }
            if EndDoc(hdc) <= 0 {
                let _ = DeleteDC(hdc);
                return Err("فشل إنهاء مستند الطباعة".into());
            }
            let _ = DeleteDC(hdc);
            Ok(())
        }
    }

    unsafe fn draw_line(hdc: windows::Win32::Graphics::Gdi::HDC, page_width: i32, y: i32, size: i32, bold: bool, text: &str) -> i32 {
        let font = CreateFontW(
            size,
            0,
            0,
            0,
            if bold { FW_BOLD.0 as i32 } else { FW_NORMAL.0 as i32 },
            0,
            0,
            0,
            ARABIC_CHARSET as u32,
            OUT_TT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            DEFAULT_PITCH.0 as u32,
            windows::core::w!("Segoe UI"),
        );
        let old = SelectObject(hdc, HGDIOBJ(font.0));
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        let mut rect = windows::Win32::Foundation::RECT {
            left: 20,
            top: y,
            right: page_width.saturating_sub(20),
            bottom: y + size + 24,
        };
        DrawTextW(
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
        default_printer().ok_or_else(|| "لا توجد طابعة افتراضية على النظام".into())
    }

    fn default_printer() -> Option<String> {
        unsafe {
            let mut size = 0u32;
            let _ = GetDefaultPrinterW(PWSTR::null(), &mut size);
            if size == 0 {
                return None;
            }
            let mut buf = vec![0u16; size as usize];
            GetDefaultPrinterW(PWSTR(buf.as_mut_ptr()), &mut size)
                .ok()
                .ok()?;
            Some(String::from_utf16_lossy(&buf).trim_end_matches('\0').to_string())
        }
    }

    unsafe fn pwstr_to_string(value: PWSTR) -> String {
        if value.is_null() {
            return String::new();
        }
        value.to_string().unwrap_or_default()
    }
}
