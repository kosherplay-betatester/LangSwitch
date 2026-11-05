use std::ptr;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, SelectObject, SetBkMode, SetTextColor,
    BitBlt, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, DT_CENTER,
    DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_RTLREADING, DT_SINGLELINE, DT_VCENTER,
    DT_WORDBREAK, FW_BOLD, FW_MEDIUM, FW_NORMAL, HDC, HFONT, OUT_DEFAULT_PRECIS,
    PAINTSTRUCT, SRCCOPY, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_SELECTED, WC_BUTTONW};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, LoadImageW, PostQuitMessage, RegisterClassExW,
    SendMessageW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    BS_OWNERDRAW, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HMENU, HTCAPTION, ICON_BIG, ICON_SMALL,
    IMAGE_ICON, LR_DEFAULTCOLOR, SM_CXICON, SM_CXSCREEN, SM_CXSMICON, SM_CYICON, SM_CYSCREEN,
    SM_CYSMICON, SW_SHOW, WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_DRAWITEM, WM_ERASEBKGND,
    WM_KEYDOWN, WM_LBUTTONDOWN, WM_NCLBUTTONDOWN, WM_PAINT, WM_SETICON, WNDCLASSEXW, WS_CAPTION,
    WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

// ============================================================================
// Theme Color Constants (Light Blue <-> Light Purple Palette)
// ============================================================================

/// Window background gradient: Ice Light Blue (#E8F4FF) -> Soft Lavender Purple (#F3E8FF)
pub const BG_GRAD_TOP_BLUE: (u8, u8, u8) = (232, 244, 255);
pub const BG_GRAD_BOT_PURPLE: (u8, u8, u8) = (243, 232, 255);

/// Header banner gradient: Sky Blue (#0EA5E9) -> Indigo (#6366F1) -> Light Purple (#A855F7)
pub const HEADER_BLUE: (u8, u8, u8) = (14, 165, 233);
pub const HEADER_PURPLE: (u8, u8, u8) = (147, 51, 234);

/// Card border gradient endpoints: Vibrant Light Blue (#38BDF8) <-> Vibrant Light Purple (#C084FC)
pub const BORDER_LIGHT_BLUE: (u8, u8, u8) = (56, 189, 248);
pub const BORDER_LIGHT_PURPLE: (u8, u8, u8) = (168, 85, 247);

/// Soft outer glow around cards
pub const GLOW_LIGHT_BLUE: (u8, u8, u8) = (186, 230, 253);
pub const GLOW_LIGHT_PURPLE: (u8, u8, u8) = (233, 213, 255);

/// Crisp frosted card surface color (#FCFDFF)
pub const CARD_SURFACE_RGB: u32 = rgb(252, 253, 255);

/// Text colors
pub const TEXT_DARK_SLATE: u32 = rgb(30, 41, 59);
pub const TEXT_MUTED_SLATE: u32 = rgb(71, 85, 105);
pub const TEXT_DEEP_PURPLE: u32 = rgb(109, 40, 217);
pub const TEXT_SKY_BLUE: u32 = rgb(2, 132, 199);
pub const TEXT_INDIGO: u32 = rgb(67, 56, 202);

/// Linearly interpolates between two RGB tuples (`t` in `0.0..=1.0`)
pub fn lerp_rgb(c1: (u8, u8, u8), c2: (u8, u8, u8), t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let r = (c1.0 as f32 + (c2.0 as f32 - c1.0 as f32) * t).round() as u8;
    let g = (c1.1 as f32 + (c2.1 as f32 - c1.1 as f32) * t).round() as u8;
    let b = (c1.2 as f32 + (c2.2 as f32 - c1.2 as f32) * t).round() as u8;
    rgb(r, g, b)
}

/// Computes horizontal X-inset for a rounded rectangle of corner `radius` at vertical distance `dy` from top/bottom edge
fn corner_inset(dy: i32, radius: i32) -> i32 {
    if radius <= 0 || dy >= radius {
        return 0;
    }
    let r = radius as f32;
    let d = (radius - dy) as f32;
    let sq = (r * r - d * d).max(0.0).sqrt();
    (r - sq).round() as i32
}

/// Fills a rectangular area with a smooth vertical gradient
pub unsafe fn fill_vertical_gradient(
    hdc: HDC,
    rc: &RECT,
    top_color: (u8, u8, u8),
    bottom_color: (u8, u8, u8),
) {
    let height = (rc.bottom - rc.top).max(1);
    let steps = height.min(120).max(1);
    for i in 0..steps {
        let y0 = rc.top + (i * height) / steps;
        let y1 = if i == steps - 1 {
            rc.bottom
        } else {
            rc.top + ((i + 1) * height) / steps
        };
        let t = i as f32 / (steps - 1).max(1) as f32;
        let color = lerp_rgb(top_color, bottom_color, t);
        let band = RECT {
            left: rc.left,
            top: y0,
            right: rc.right,
            bottom: y1,
        };
        unsafe {
            let brush = CreateSolidBrush(color);
            FillRect(hdc, &band, brush);
            DeleteObject(brush as *mut _);
        }
    }
}

/// Fills a rounded rectangle with a vertical gradient from `top_color` to `bottom_color`
pub unsafe fn fill_rounded_vertical_gradient(
    hdc: HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
    top_color: (u8, u8, u8),
    bottom_color: (u8, u8, u8),
) {
    let height = (bottom - top).max(1);
    for y in top..bottom {
        let dy_top = y - top;
        let dy_bot = (bottom - 1) - y;
        let dy = dy_top.min(dy_bot);
        let inset = corner_inset(dy, radius);
        let x0 = left + inset;
        let x1 = right - inset;
        if x1 > x0 {
            let t = (y - top) as f32 / (height - 1).max(1) as f32;
            let color = lerp_rgb(top_color, bottom_color, t);
            let line_rc = RECT {
                left: x0,
                top: y,
                right: x1,
                bottom: y + 1,
            };
            unsafe {
                let brush = CreateSolidBrush(color);
                FillRect(hdc, &line_rc, brush);
                DeleteObject(brush as *mut _);
            }
        }
    }
}

/// Fills a rounded rectangle with a horizontal gradient from `left_color` to `right_color`
pub unsafe fn fill_rounded_horizontal_gradient(
    hdc: HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
    left_color: (u8, u8, u8),
    right_color: (u8, u8, u8),
) {
    let width = (right - left).max(1);
    for x in left..right {
        let dx_left = x - left;
        let dx_right = (right - 1) - x;
        let dx = dx_left.min(dx_right);
        let inset = corner_inset(dx, radius);
        let y0 = top + inset;
        let y1 = bottom - inset;
        if y1 > y0 {
            let t = (x - left) as f32 / (width - 1).max(1) as f32;
            let color = lerp_rgb(left_color, right_color, t);
            let col_rc = RECT {
                left: x,
                top: y0,
                right: x + 1,
                bottom: y1,
            };
            unsafe {
                let brush = CreateSolidBrush(color);
                FillRect(hdc, &col_rc, brush);
                DeleteObject(brush as *mut _);
            }
        }
    }
}

/// Fills a rounded rectangle with a solid color cleanly
pub unsafe fn fill_rounded_solid(
    hdc: HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
    color: u32,
) {
    unsafe {
        let brush = CreateSolidBrush(color);
        for y in top..bottom {
            let dy_top = y - top;
            let dy_bot = (bottom - 1) - y;
            let dy = dy_top.min(dy_bot);
            let inset = corner_inset(dy, radius);
            let x0 = left + inset;
            let x1 = right - inset;
            if x1 > x0 {
                let line_rc = RECT {
                    left: x0,
                    top: y,
                    right: x1,
                    bottom: y + 1,
                };
                FillRect(hdc, &line_rc, brush);
            }
        }
        DeleteObject(brush as *mut _);
    }
}

/// Draws a modern card with a smooth Light Blue <-> Light Purple gradient border,
/// soft outer glow, clean frosted white-lavender interior, and top horizontal gradient accent bar.
/// If `blue_to_purple` is true, border transitions from Light Blue -> Light Purple;
/// if false, border transitions from Light Purple -> Light Blue!
pub unsafe fn draw_gradient_card(
    hdc: HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
    blue_to_purple: bool,
) {
    let (glow_start, glow_end, border_start, border_end) = if blue_to_purple {
        (
            GLOW_LIGHT_BLUE,
            GLOW_LIGHT_PURPLE,
            BORDER_LIGHT_BLUE,
            BORDER_LIGHT_PURPLE,
        )
    } else {
        (
            GLOW_LIGHT_PURPLE,
            GLOW_LIGHT_BLUE,
            BORDER_LIGHT_PURPLE,
            BORDER_LIGHT_BLUE,
        )
    };

    unsafe {
        // 1. Soft 1px outer glow halo
        fill_rounded_vertical_gradient(
            hdc,
            left - 1,
            top - 1,
            right + 1,
            bottom + 1,
            radius + 1,
            glow_start,
            glow_end,
        );

        // 2. 2px Gradient Border (Light Blue <-> Light Purple)
        fill_rounded_vertical_gradient(
            hdc,
            left,
            top,
            right,
            bottom,
            radius,
            border_start,
            border_end,
        );

        // 3. Inner frosted white-lavender card surface
        fill_rounded_solid(
            hdc,
            left + 2,
            top + 2,
            right - 2,
            bottom - 2,
            (radius - 2).max(2),
            CARD_SURFACE_RGB,
        );

        // 4. Subtle top horizontal accent line inside the card
        fill_rounded_horizontal_gradient(
            hdc,
            left + 12,
            top + 2,
            right - 12,
            top + 5,
            1,
            border_start,
            border_end,
        );
    }
}

/// Draws the top header banner with a Light Blue <-> Light Purple gradient and inner border glow
pub unsafe fn draw_header_banner(
    hdc: HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    title: &str,
    subtitle: &str,
    hfont_title: HFONT,
    hfont_sub: HFONT,
    is_rtl: bool,
) {
    unsafe {
        // Outer gradient border (Light Purple -> Light Blue)
        fill_rounded_horizontal_gradient(
            hdc,
            left,
            top,
            right,
            bottom,
            14,
            BORDER_LIGHT_PURPLE,
            BORDER_LIGHT_BLUE,
        );

        // Inner vibrant Sky Blue -> Violet-Purple gradient fill
        fill_rounded_horizontal_gradient(
            hdc,
            left + 2,
            top + 2,
            right - 2,
            bottom - 2,
            12,
            HEADER_BLUE,
            HEADER_PURPLE,
        );

        SetBkMode(hdc, TRANSPARENT as i32);

        let align_flags = if is_rtl {
            DT_RIGHT | DT_RTLREADING | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX
        } else {
            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX
        };

        // Title
        SetTextColor(hdc, rgb(255, 255, 255));
        SelectObject(hdc, hfont_title as *mut _);
        let mut rc_title = RECT {
            left: left + 18,
            top: top + 8,
            right: right - 18,
            bottom: top + 36,
        };
        let wtitle = to_wide(title);
        DrawTextW(hdc, wtitle.as_ptr(), -1, &mut rc_title, align_flags);

        // Subtitle
        SetTextColor(hdc, rgb(224, 242, 254));
        SelectObject(hdc, hfont_sub as *mut _);
        let mut rc_sub = RECT {
            left: left + 18,
            top: top + 36,
            right: right - 18,
            bottom: bottom - 8,
        };
        let wsub = to_wide(subtitle);
        DrawTextW(hdc, wsub.as_ptr(), -1, &mut rc_sub, align_flags);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Vibrant Sky Blue -> Light Purple gradient with white text
    PrimaryGradient,
    /// Soft Light Blue gradient with Light Blue -> Light Purple border
    LightBlue,
    /// Soft Light Purple gradient with Light Purple -> Light Blue border
    LightPurple,
    /// Clean neutral slate-lavender with gradient border
    Subtle,
}

/// Draws a modern rounded button with a Light Blue <-> Light Purple gradient border and gradient fill
pub unsafe fn draw_gradient_button(
    hdc: HDC,
    rc: &RECT,
    text: &str,
    hfont: HFONT,
    style: ButtonStyle,
    is_pressed: bool,
) {
    let radius = 10;
    let (border_left, border_right, fill_left, fill_right, text_color) = match style {
        ButtonStyle::PrimaryGradient => {
            if is_pressed {
                (
                    BORDER_LIGHT_PURPLE,
                    BORDER_LIGHT_BLUE,
                    (126, 34, 206),
                    (2, 132, 199),
                    rgb(255, 255, 255),
                )
            } else {
                (
                    BORDER_LIGHT_BLUE,
                    BORDER_LIGHT_PURPLE,
                    (14, 165, 233),
                    (147, 51, 234),
                    rgb(255, 255, 255),
                )
            }
        }
        ButtonStyle::LightBlue => {
            if is_pressed {
                (
                    BORDER_LIGHT_PURPLE,
                    BORDER_LIGHT_BLUE,
                    (186, 230, 253),
                    (221, 214, 254),
                    rgb(3, 105, 161),
                )
            } else {
                (
                    BORDER_LIGHT_BLUE,
                    BORDER_LIGHT_PURPLE,
                    (224, 242, 254),
                    (243, 232, 255),
                    rgb(2, 132, 199),
                )
            }
        }
        ButtonStyle::LightPurple => {
            if is_pressed {
                (
                    BORDER_LIGHT_BLUE,
                    BORDER_LIGHT_PURPLE,
                    (221, 214, 254),
                    (186, 230, 253),
                    rgb(91, 33, 182),
                )
            } else {
                (
                    BORDER_LIGHT_PURPLE,
                    BORDER_LIGHT_BLUE,
                    (243, 232, 255),
                    (224, 242, 254),
                    rgb(109, 40, 217),
                )
            }
        }
        ButtonStyle::Subtle => {
            if is_pressed {
                (
                    BORDER_LIGHT_PURPLE,
                    BORDER_LIGHT_BLUE,
                    (226, 232, 240),
                    (233, 213, 255),
                    TEXT_DARK_SLATE,
                )
            } else {
                (
                    (148, 163, 184),
                    (192, 132, 252),
                    (248, 250, 252),
                    (243, 238, 255),
                    TEXT_MUTED_SLATE,
                )
            }
        }
    };

    unsafe {
        // Clear corner background with card/window surface
        let bg_brush = CreateSolidBrush(CARD_SURFACE_RGB);
        FillRect(hdc, rc, bg_brush);
        DeleteObject(bg_brush as *mut _);

        // 1. Gradient border (2px)
        fill_rounded_horizontal_gradient(
            hdc,
            rc.left,
            rc.top,
            rc.right,
            rc.bottom,
            radius,
            border_left,
            border_right,
        );

        // 2. Gradient interior
        fill_rounded_horizontal_gradient(
            hdc,
            rc.left + 2,
            rc.top + 2,
            rc.right - 2,
            rc.bottom - 2,
            radius - 2,
            fill_left,
            fill_right,
        );

        // 3. Button label
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, text_color);
        if !hfont.is_null() {
            SelectObject(hdc, hfont as *mut _);
        }
        let mut text_rc = *rc;
        if is_pressed {
            text_rc.top += 1;
            text_rc.left += 1;
        }
        let wtext = to_wide(text);
        DrawTextW(
            hdc,
            wtext.as_ptr(),
            -1,
            &mut text_rc,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
}

/// Draws a modern pill/chip modifier toggle button (`Ctrl`, `Alt`, `Shift`, `Win`)
pub unsafe fn draw_modifier_pill(
    hdc: HDC,
    rc: &RECT,
    label: &str,
    hfont: HFONT,
    is_checked: bool,
    is_pressed: bool,
) {
    let radius = 8;
    unsafe {
        let bg_brush = CreateSolidBrush(CARD_SURFACE_RGB);
        FillRect(hdc, rc, bg_brush);
        DeleteObject(bg_brush as *mut _);

        if is_checked {
            // Checked: Light Blue <-> Light Purple gradient border & soft gradient fill
            let (b_start, b_end) = if is_pressed {
                (BORDER_LIGHT_PURPLE, BORDER_LIGHT_BLUE)
            } else {
                (BORDER_LIGHT_BLUE, BORDER_LIGHT_PURPLE)
            };
            fill_rounded_horizontal_gradient(
                hdc,
                rc.left,
                rc.top,
                rc.right,
                rc.bottom,
                radius,
                b_start,
                b_end,
            );
            fill_rounded_horizontal_gradient(
                hdc,
                rc.left + 2,
                rc.top + 2,
                rc.right - 2,
                rc.bottom - 2,
                radius - 2,
                (224, 242, 254),
                (243, 232, 255),
            );
            SetTextColor(hdc, rgb(109, 40, 217));
        } else {
            // Unchecked: subtle border & crisp white-slate interior
            fill_rounded_horizontal_gradient(
                hdc,
                rc.left,
                rc.top,
                rc.right,
                rc.bottom,
                radius,
                (186, 230, 253),
                (221, 214, 254),
            );
            fill_rounded_solid(
                hdc,
                rc.left + 1,
                rc.top + 1,
                rc.right - 1,
                rc.bottom - 1,
                radius - 1,
                rgb(248, 250, 252),
            );
            SetTextColor(hdc, TEXT_MUTED_SLATE);
        }

        SetBkMode(hdc, TRANSPARENT as i32);
        if !hfont.is_null() {
            SelectObject(hdc, hfont as *mut _);
        }
        let display = if is_checked {
            format!("✓ {}", label)
        } else {
            label.to_string()
        };
        let wtext = to_wide(&display);
        let mut text_rc = *rc;
        DrawTextW(
            hdc,
            wtext.as_ptr(),
            -1,
            &mut text_rc,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
}

/// Draws a modern custom checkbox row with a Light Blue <-> Light Purple gradient checkbox indicator
/// and proper RTL/LTR text alignment with zero background box artifacts.
pub unsafe fn draw_custom_checkbox_row(
    hdc: HDC,
    rc: &RECT,
    text: &str,
    hfont: HFONT,
    is_checked: bool,
    is_rtl: bool,
) {
    unsafe {
        // Clear with card surface color
        let bg_brush = CreateSolidBrush(CARD_SURFACE_RGB);
        FillRect(hdc, rc, bg_brush);
        DeleteObject(bg_brush as *mut _);

        let box_size = 20;
        let cy = (rc.top + rc.bottom - box_size) / 2;

        let (box_left, text_left, text_right) = if is_rtl {
            (rc.right - box_size - 4, rc.left + 4, rc.right - box_size - 14)
        } else {
            (rc.left + 4, rc.left + box_size + 14, rc.right - 4)
        };
        let box_right = box_left + box_size;
        let box_bottom = cy + box_size;

        if is_checked {
            // Checked: Light Blue -> Light Purple gradient box with white checkmark
            fill_rounded_horizontal_gradient(
                hdc,
                box_left,
                cy,
                box_right,
                box_bottom,
                6,
                HEADER_BLUE,
                HEADER_PURPLE,
            );
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, rgb(255, 255, 255));
            if !hfont.is_null() {
                SelectObject(hdc, hfont as *mut _);
            }
            let mut chk_rc = RECT {
                left: box_left,
                top: cy - 1,
                right: box_right,
                bottom: box_bottom,
            };
            let wchk = to_wide("✓");
            DrawTextW(
                hdc,
                wchk.as_ptr(),
                -1,
                &mut chk_rc,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
            );
        } else {
            // Unchecked: Light Blue <-> Light Purple gradient border + white interior
            fill_rounded_horizontal_gradient(
                hdc,
                box_left,
                cy,
                box_right,
                box_bottom,
                6,
                BORDER_LIGHT_BLUE,
                BORDER_LIGHT_PURPLE,
            );
            fill_rounded_solid(
                hdc,
                box_left + 2,
                cy + 2,
                box_right - 2,
                box_bottom - 2,
                4,
                rgb(255, 255, 255),
            );
        }

        // Draw label text
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, TEXT_DARK_SLATE);
        if !hfont.is_null() {
            SelectObject(hdc, hfont as *mut _);
        }
        let mut text_rc = RECT {
            left: text_left,
            top: rc.top,
            right: text_right,
            bottom: rc.bottom,
        };
        let flags = if is_rtl {
            DT_RIGHT | DT_RTLREADING | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS
        } else {
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS
        };
        let wtext = to_wide(text);
        DrawTextW(hdc, wtext.as_ptr(), -1, &mut text_rc, flags);
    }
}

// ============================================================================
// Modern Custom Dialog Box (Replaces raw Windows MessageBoxW everywhere!)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    Primary,   // OK / Yes / Install
    Secondary, // No / Cancel / Portable
}

const ID_DLG_PRIMARY: usize = 501;
const ID_DLG_SECONDARY: usize = 502;

struct ModernDialogState {
    title: String,
    subtitle: String,
    body: String,
    primary_text: String,
    secondary_text: Option<String>,
    is_rtl: bool,
    result: DialogResult,
    hfont_title: HFONT,
    hfont_sub: HFONT,
    hfont_body: HFONT,
    hfont_btn: HFONT,
}

/// Displays a sleek, custom-styled Light Blue <-> Light Purple gradient modal dialog box.
pub fn show_modern_dialog(
    title: &str,
    subtitle: &str,
    body: &str,
    primary_btn_text: &str,
    secondary_btn_text: Option<&str>,
    is_rtl: bool,
) -> DialogResult {
    unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = to_wide("LangSwitchModernDialogClass");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(modern_dialog_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: ptr::null_mut(),
        };
        RegisterClassExW(&wnd_class);

        let line_count = body.lines().count().max(2) as i32;
        let win_width = 500;
        let win_height = (240 + line_count * 24).clamp(275, 440);
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_width) / 2;
        let pos_y = (screen_h - win_height) / 2;

        let font_name = to_wide("Segoe UI");
        let hfont_title = CreateFontW(
            -19, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_sub = CreateFontW(
            -13, 0, 0, 0, FW_MEDIUM as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_body = CreateFontW(
            -15, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_btn = CreateFontW(
            -14, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );

        let wtitle = to_wide(title);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            wtitle.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            pos_x,
            pos_y,
            win_width,
            win_height,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        );

        // Set application icons
        let sm_cx = GetSystemMetrics(SM_CXSMICON);
        let sm_cy = GetSystemMetrics(SM_CYSMICON);
        let big_cx = GetSystemMetrics(SM_CXICON);
        let big_cy = GetSystemMetrics(SM_CYICON);
        let icon_sm = LoadImageW(hinstance, 1 as *const u16, IMAGE_ICON, sm_cx, sm_cy, LR_DEFAULTCOLOR);
        let icon_big = LoadImageW(hinstance, 1 as *const u16, IMAGE_ICON, big_cx, big_cy, LR_DEFAULTCOLOR);
        if !icon_sm.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, icon_sm as LPARAM);
        }
        if !icon_big.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, icon_big as LPARAM);
        }

        let mut client_rc = RECT { left: 0, top: 0, right: win_width, bottom: win_height };
        GetClientRect(hwnd, &mut client_rc);
        let cw = client_rc.right - client_rc.left;
        let ch = client_rc.bottom - client_rc.top;

        let btn_y = ch - 54;
        let btn_h = 38;

        if let Some(sec_txt) = secondary_btn_text {
            let btn_w = 185;
            let gap = 16;
            let total_w = btn_w * 2 + gap;
            let start_x = (cw - total_w) / 2;

            let (pri_x, sec_x) = if is_rtl {
                (start_x + btn_w + gap, start_x)
            } else {
                (start_x, start_x + btn_w + gap)
            };

            let wpri = to_wide(primary_btn_text);
            CreateWindowExW(
                0, WC_BUTTONW, wpri.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
                pri_x, btn_y, btn_w, btn_h,
                hwnd, ID_DLG_PRIMARY as HMENU, hinstance, ptr::null(),
            );

            let wsec = to_wide(sec_txt);
            CreateWindowExW(
                0, WC_BUTTONW, wsec.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
                sec_x, btn_y, btn_w, btn_h,
                hwnd, ID_DLG_SECONDARY as HMENU, hinstance, ptr::null(),
            );
        } else {
            let btn_w = 180;
            let pri_x = (cw - btn_w) / 2;
            let wpri = to_wide(primary_btn_text);
            CreateWindowExW(
                0, WC_BUTTONW, wpri.as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
                pri_x, btn_y, btn_w, btn_h,
                hwnd, ID_DLG_PRIMARY as HMENU, hinstance, ptr::null(),
            );
        }

        let state = Box::new(ModernDialogState {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            body: body.to_string(),
            primary_text: primary_btn_text.to_string(),
            secondary_text: secondary_btn_text.map(|s| s.to_string()),
            is_rtl,
            result: DialogResult::Secondary,
            hfont_title,
            hfont_sub,
            hfont_body,
            hfont_btn,
        });
        let state_raw = Box::into_raw(state);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_raw as isize);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let final_state = Box::from_raw(state_raw);
        let res = final_state.result;
        DeleteObject(final_state.hfont_title as *mut _);
        DeleteObject(final_state.hfont_sub as *mut _);
        DeleteObject(final_state.hfont_body as *mut _);
        DeleteObject(final_state.hfont_btn as *mut _);
        res
    }
}

unsafe extern "system" fn modern_dialog_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ModernDialogState };

    match msg {
        WM_ERASEBKGND => return 1,
        WM_LBUTTONDOWN => unsafe {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            return 0;
        },
        WM_PAINT => {
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
                let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

                let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                unsafe {
                    GetClientRect(hwnd, &mut rc);
                }
                let w = rc.right - rc.left;
                let h = rc.bottom - rc.top;

                unsafe {
                    let mem_dc = CreateCompatibleDC(hdc);
                    let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
                    let old_bmp = SelectObject(mem_dc, mem_bmp as *mut _);

                    // 1. Light Blue -> Light Purple background gradient
                    fill_vertical_gradient(mem_dc, &rc, BG_GRAD_TOP_BLUE, BG_GRAD_BOT_PURPLE);

                    // 2. Top gradient header banner
                    draw_header_banner(
                        mem_dc,
                        16,
                        12,
                        w - 16,
                        72,
                        &state.title,
                        &state.subtitle,
                        state.hfont_title,
                        state.hfont_sub,
                        state.is_rtl,
                    );

                    // 3. Message card with Light Blue <-> Light Purple gradient border
                    let card_top = 82;
                    let card_bottom = h - 66;
                    draw_gradient_card(mem_dc, 16, card_top, w - 16, card_bottom, 12, true);

                    // 4. Body text inside card
                    SetBkMode(mem_dc, TRANSPARENT as i32);
                    SetTextColor(mem_dc, TEXT_DARK_SLATE);
                    SelectObject(mem_dc, state.hfont_body as *mut _);

                    let mut body_rc = RECT {
                        left: 34,
                        top: card_top + 16,
                        right: w - 34,
                        bottom: card_bottom - 14,
                    };
                    let flags = if state.is_rtl {
                        DT_RIGHT | DT_RTLREADING | DT_WORDBREAK | DT_NOPREFIX
                    } else {
                        DT_LEFT | DT_WORDBREAK | DT_NOPREFIX
                    };
                    let wbody = to_wide(&state.body);
                    DrawTextW(mem_dc, wbody.as_ptr(), -1, &mut body_rc, flags);

                    BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);
                    SelectObject(mem_dc, old_bmp);
                    DeleteObject(mem_bmp as *mut _);
                    DeleteDC(mem_dc);
                    EndPaint(hwnd, &ps);
                }
                return 0;
            }
        }
        WM_DRAWITEM => {
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
                let ctrl_id = dis.CtlID as usize;
                let is_pressed = (dis.itemState & ODS_SELECTED) != 0;

                unsafe {
                    if ctrl_id == ID_DLG_PRIMARY {
                        draw_gradient_button(
                            dis.hDC,
                            &dis.rcItem,
                            &state.primary_text,
                            state.hfont_btn,
                            ButtonStyle::PrimaryGradient,
                            is_pressed,
                        );
                        return 1;
                    } else if ctrl_id == ID_DLG_SECONDARY {
                        let txt = state.secondary_text.as_deref().unwrap_or("Cancel");
                        draw_gradient_button(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_btn,
                            ButtonStyle::LightPurple,
                            is_pressed,
                        );
                        return 1;
                    }
                }
            }
        }
        WM_COMMAND => {
            let ctrl_id = (wparam & 0xFFFF) as usize;
            if !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                if ctrl_id == ID_DLG_PRIMARY {
                    state.result = DialogResult::Primary;
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return 0;
                } else if ctrl_id == ID_DLG_SECONDARY {
                    state.result = DialogResult::Secondary;
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return 0;
                }
            }
        }
        WM_KEYDOWN => {
            let vk = wparam as u32;
            if !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                if vk == 0x0D {
                    // Enter -> Primary
                    state.result = DialogResult::Primary;
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return 0;
                } else if vk == 0x1B {
                    // Esc -> Secondary
                    state.result = DialogResult::Secondary;
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return 0;
                }
            }
        }
        WM_CLOSE => unsafe {
            DestroyWindow(hwnd);
            return 0;
        },
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
            return 0;
        },
        _ => {}
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
