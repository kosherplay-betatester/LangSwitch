use crate::config::AppConfig;
use crate::ui_theme::{
    self, to_wide, ButtonStyle, BG_GRAD_BOT_PURPLE, BG_GRAD_TOP_BLUE, TEXT_DARK_SLATE,
    TEXT_DEEP_PURPLE, TEXT_INDIGO, TEXT_SKY_BLUE,
};
use crate::updater::CURRENT_VERSION;
use std::ptr;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, DeleteDC,
    DeleteObject, DrawTextW, EndPaint, SelectObject, SetBkMode, SetTextColor, CLIP_DEFAULT_PRECIS,
    DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, DT_LEFT, DT_NOPREFIX, DT_RIGHT,
    DT_RTLREADING, DT_SINGLELINE, DT_VCENTER, FW_BOLD, FW_MEDIUM, FW_NORMAL, HFONT,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, SRCCOPY, TRANSPARENT,
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

const ID_BTN_START: usize = 101;

struct WelcomeState {
    hfont_title: HFONT,
    hfont_sub: HFONT,
    hfont_body: HFONT,
    hfont_bold: HFONT,
    hfont_btn: HFONT,
    is_english: bool,
}

pub fn show_welcome_window(config: &AppConfig) {
    let is_en = config.ui_language == "en";
    unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = to_wide("LangSwitchWelcomeWindow");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(welcome_wnd_proc),
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

        let win_width = 610;
        let win_height = 610;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_width) / 2;
        let pos_y = (screen_h - win_height) / 2;

        let title = if is_en {
            to_wide("Welcome to LangSwitch! ⚡")
        } else {
            to_wide("ברוכים הבאים ל-LangSwitch! ⚡")
        };

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
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

        let icon_sm = LoadImageW(
            hinstance,
            1 as *const u16,
            IMAGE_ICON,
            sm_cx,
            sm_cy,
            LR_DEFAULTCOLOR,
        );
        let icon_big = LoadImageW(
            hinstance,
            1 as *const u16,
            IMAGE_ICON,
            big_cx,
            big_cy,
            LR_DEFAULTCOLOR,
        );

        if !icon_sm.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, icon_sm as LPARAM);
        }
        if !icon_big.is_null() {
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, icon_big as LPARAM);
        }

        let font_name = to_wide("Segoe UI");
        let hfont_title = CreateFontW(
            -21, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_sub = CreateFontW(
            -13, 0, 0, 0, FW_MEDIUM as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_body = CreateFontW(
            -13, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_bold = CreateFontW(
            -14, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_btn = CreateFontW(
            -15, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );

        let state = Box::new(WelcomeState {
            hfont_title,
            hfont_sub,
            hfont_body,
            hfont_bold,
            hfont_btn,
            is_english: is_en,
        });

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        let mut client_rc = RECT {
            left: 0,
            top: 0,
            right: win_width,
            bottom: win_height,
        };
        GetClientRect(hwnd, &mut client_rc);
        let cw = client_rc.right - client_rc.left;
        let ch = client_rc.bottom - client_rc.top;

        let btn_w = 250;
        let btn_h = 44;
        let btn_x = (cw - btn_w) / 2;
        let btn_y = ch - 58;

        let btn_text = if is_en {
            "Got it, let's start! 🚀"
        } else {
            "הבנתי, בוא נתחיל! 🚀"
        };
        let wbtn = to_wide(btn_text);
        CreateWindowExW(
            0,
            WC_BUTTONW,
            wbtn.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_OWNERDRAW as u32 | WS_TABSTOP,
            btn_x,
            btn_y,
            btn_w,
            btn_h,
            hwnd,
            ID_BTN_START as HMENU,
            hinstance,
            ptr::null(),
        );

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn welcome_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WelcomeState };

    match msg {
        WM_ERASEBKGND => return 1,
        WM_LBUTTONDOWN => unsafe {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            return 0;
        },
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let is_en = state.is_english;
                let is_rtl = !is_en;

                let mut rc = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
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
                    ui_theme::fill_vertical_gradient(
                        mem_dc,
                        &rc,
                        BG_GRAD_TOP_BLUE,
                        BG_GRAD_BOT_PURPLE,
                    );

                    // 2. Top Gradient Banner
                    let header_title = format!("⚡ LangSwitch v{}", CURRENT_VERSION);
                    let header_sub = if is_en {
                        "Ultra-fast, native Hebrew <-> English layout switcher for Windows"
                    } else {
                        "כלי חכם, קל משקל ומהיר כברק להמרת טקסט והחלפת שפה אוטומטית"
                    };
                    ui_theme::draw_header_banner(
                        mem_dc,
                        18,
                        12,
                        w - 18,
                        76,
                        &header_title,
                        header_sub,
                        state.hfont_title,
                        state.hfont_sub,
                        is_rtl,
                    );

                    // 3. Feature Cards with alternating Light Blue <-> Light Purple gradient borders
                    let items: [(&str, &str, u32, bool); 6] = if is_en {
                        [
                            (
                                "⌨️  Primary Hotkey (F10) — Convert + Switch Layout",
                                "Instantly converts mistyped text AND switches keyboard layout automatically!",
                                TEXT_SKY_BLUE,
                                true,
                            ),
                            (
                                "🎯  Smart Auto-Line Selection",
                                "No need to highlight text first — converts the current line automatically.",
                                TEXT_DEEP_PURPLE,
                                false,
                            ),
                            (
                                "⚡  Secondary Hotkey (Ctrl + F10) — Convert Only",
                                "Converts mistyped text without switching your active keyboard layout.",
                                TEXT_INDIGO,
                                true,
                            ),
                            (
                                "🔍  Google AI Mode Search (Shift + Win + G)",
                                "Opens your selected or current line text directly in Google AI Mode.",
                                TEXT_SKY_BLUE,
                                false,
                            ),
                            (
                                "🌐  Quick Google Translation (Ctrl + Shift + T)",
                                "Translates Hebrew <-> English instantly in your browser with dictionary cards.",
                                TEXT_DEEP_PURPLE,
                                true,
                            ),
                            (
                                "🪟  0.00% Idle CPU in Windows System Tray",
                                "Runs silently near the clock — right-click the tray icon anytime for Settings.",
                                TEXT_INDIGO,
                                false,
                            ),
                        ]
                    } else {
                        [
                            (
                                "⌨️  מקש ראשי (F10) — המרה + החלפת שפת מקלדת",
                                "ממיר מיידית טקסט שהוקלד בשפה הפוכה ומחליף את שפת המקלדת אוטומטית!",
                                TEXT_SKY_BLUE,
                                true,
                            ),
                            (
                                "🎯  זיהוי וסימון שורה אוטומטי חכם",
                                "אין צורך לסמן טקסט עם העכבר — השורה הנוכחית מומרת בלחיצה אחת.",
                                TEXT_DEEP_PURPLE,
                                false,
                            ),
                            (
                                "⚡  מקש משני (Ctrl + F10) — המרת טקסט בלבד",
                                "ממיר את הטקסט שנכתב בטעות ללא שינוי שפת המקלדת הפעילה.",
                                TEXT_INDIGO,
                                true,
                            ),
                            (
                                "🔍  חיפוש מהיר ב-Google AI (Shift + Win + G)",
                                "פותח את הטקסט המסומן או השורה הנוכחית ישירות במצב ה-AI של גוגל!",
                                TEXT_SKY_BLUE,
                                false,
                            ),
                            (
                                "🌐  תרגום מהיר בגוגל (Ctrl + Shift + T)",
                                "תרגום מיידי מעברית לאנגלית ולהיפך עם כרטיסיית מילון והגייה קולית.",
                                TEXT_DEEP_PURPLE,
                                true,
                            ),
                            (
                                "🪟  שקט וחסכוני במגש המערכת (System Tray)",
                                "פועל ברקע ליד השעון עם 0% מעבד — לחץ על האייקון לפתיחת ההגדרות.",
                                TEXT_INDIGO,
                                false,
                            ),
                        ]
                    };

                    SetBkMode(mem_dc, TRANSPARENT as i32);
                    let flags = if is_rtl {
                        DT_RIGHT | DT_RTLREADING | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX
                    } else {
                        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX
                    };

                    let mut card_y = 88;
                    let card_h = 62;
                    for (lead, desc, title_color, blue_to_purple) in items {
                        ui_theme::draw_gradient_card(
                            mem_dc,
                            18,
                            card_y,
                            w - 18,
                            card_y + card_h,
                            12,
                            blue_to_purple,
                        );

                        SelectObject(mem_dc, state.hfont_bold as *mut _);
                        SetTextColor(mem_dc, title_color);
                        let mut rc_lead = RECT {
                            left: 34,
                            top: card_y + 9,
                            right: w - 34,
                            bottom: card_y + 31,
                        };
                        let wlead = to_wide(lead);
                        DrawTextW(mem_dc, wlead.as_ptr(), -1, &mut rc_lead, flags);

                        SelectObject(mem_dc, state.hfont_body as *mut _);
                        SetTextColor(mem_dc, TEXT_DARK_SLATE);
                        let mut rc_desc = RECT {
                            left: 34,
                            top: card_y + 32,
                            right: w - 34,
                            bottom: card_y + card_h - 6,
                        };
                        let wdesc = to_wide(desc);
                        DrawTextW(mem_dc, wdesc.as_ptr(), -1, &mut rc_desc, flags);

                        card_y += card_h + 7;
                    }

                    BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, SRCCOPY);
                    SelectObject(mem_dc, old_bmp);
                    DeleteObject(mem_bmp as *mut _);
                    DeleteDC(mem_dc);
                }
            }

            unsafe {
                EndPaint(hwnd, &ps);
            }
            return 0;
        }
        WM_DRAWITEM => {
            let dis = unsafe { &*(lparam as *const DRAWITEMSTRUCT) };
            if dis.CtlID as usize == ID_BTN_START && !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let is_en = state.is_english;
                let is_pressed = (dis.itemState & ODS_SELECTED) != 0;
                let text = if is_en {
                    "Got it, let's start! 🚀"
                } else {
                    "הבנתי, בוא נתחיל! 🚀"
                };
                unsafe {
                    ui_theme::draw_gradient_button(
                        dis.hDC,
                        &dis.rcItem,
                        text,
                        state.hfont_btn,
                        ButtonStyle::PrimaryGradient,
                        is_pressed,
                    );
                }
                return 1;
            }
        }
        WM_COMMAND => {
            let ctrl_id = (wparam & 0xFFFF) as usize;
            if ctrl_id == ID_BTN_START {
                unsafe {
                    DestroyWindow(hwnd);
                }
                return 0;
            }
        }
        WM_KEYDOWN => {
            let vk = wparam as u32;
            if vk == 0x0D || vk == 0x1B {
                unsafe {
                    DestroyWindow(hwnd);
                }
                return 0;
            }
        }
        WM_CLOSE => unsafe {
            DestroyWindow(hwnd);
            return 0;
        },
        WM_DESTROY => {
            if !state_ptr.is_null() {
                let state = unsafe { Box::from_raw(state_ptr) };
                unsafe {
                    DeleteObject(state.hfont_title as *mut _);
                    DeleteObject(state.hfont_sub as *mut _);
                    DeleteObject(state.hfont_body as *mut _);
                    DeleteObject(state.hfont_bold as *mut _);
                    DeleteObject(state.hfont_btn as *mut _);
                }
            }
            unsafe {
                PostQuitMessage(0);
            }
            return 0;
        }
        _ => {}
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
