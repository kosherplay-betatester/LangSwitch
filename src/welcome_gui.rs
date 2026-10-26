use crate::config::AppConfig;
use crate::updater::CURRENT_VERSION;
use std::ptr;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CreateSolidBrush, DeleteObject, SetBkMode, SetTextColor,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY,
    FW_BOLD, FW_NORMAL, HBRUSH, HDC, HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{WC_BUTTONW, WC_STATICW};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    GetSystemMetrics, PostQuitMessage, RegisterClassExW, SendMessageW, SetForegroundWindow,
    SetWindowLongPtrW, GetWindowLongPtrW, ShowWindow, TranslateMessage, BS_DEFPUSHBUTTON,
    BS_GROUPBOX, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HMENU, SM_CXSCREEN, SM_CYSCREEN,
    SW_SHOW, WM_CLOSE, WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLORSTATIC,
    WM_DESTROY, WM_SETFONT, WNDCLASSEXW, WS_CHILD, WS_OVERLAPPED, WS_TABSTOP,
    WS_VISIBLE, WS_CAPTION, WS_SYSMENU,
};

const ID_BTN_START: usize = 101;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

struct WelcomeState {
    bg_brush: HBRUSH,
    card_brush: HBRUSH,
    hfont_title: HFONT,
    hfont_body: HFONT,
    hfont_bold: HFONT,
    hfont_btn: HFONT,
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

        let win_width = 580;
        let win_height = 590;
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
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            pos_x,
            pos_y,
            win_width,
            win_height,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        );

        // Modern fonts
        let font_name = to_wide("Segoe UI");
        let hfont_title = CreateFontW(
            -22, 0, 0, 0, FW_BOLD as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32, CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_body = CreateFontW(
            -14, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0, DEFAULT_CHARSET as u32,
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

        // Modern light palette (#F8FAFC canvas, #FFFFFF card)
        let bg_brush = CreateSolidBrush(0x00FCFAF8);
        let card_brush = CreateSolidBrush(0x00FFFFFF);

        let state = Box::new(WelcomeState {
            bg_brush,
            card_brush,
            hfont_title,
            hfont_body,
            hfont_bold,
            hfont_btn,
        });

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        init_welcome_controls(hwnd, hinstance, hfont_title, hfont_body, hfont_btn, is_en);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn init_welcome_controls(
    hwnd: HWND,
    hinstance: windows_sys::Win32::Foundation::HMODULE,
    hfont_title: HFONT,
    hfont_body: HFONT,
    hfont_btn: HFONT,
    is_en: bool,
) {
    let create_ctrl = |class: *const u16, text: &str, style: u32, x, y, w, h, id: usize| -> HWND {
        let title = to_wide(text);
        unsafe {
            let ctrl = CreateWindowExW(
                0, class, title.as_ptr(), WS_CHILD | WS_VISIBLE | style,
                x, y, w, h, hwnd, id as HMENU, hinstance, ptr::null(),
            );
            ctrl
        }
    };

    // Header Title
    let header_text = format!("⚡ LangSwitch v{}", CURRENT_VERSION);
    let lbl_title = create_ctrl(WC_STATICW, &header_text, 0, 30, 20, 510, 32, 10);
    unsafe {
        SendMessageW(lbl_title, WM_SETFONT, hfont_title as WPARAM, 1);
    }

    // Subtitle
    let subtitle_text = if is_en {
        "Ultra-fast, native Hebrew <-> English layout text switcher"
    } else {
        "\u{200F}כלי חכם, קל משקל וסופר-מהיר להמרת שפה אוטומטית"
    };
    let lbl_sub = create_ctrl(WC_STATICW, subtitle_text, 0, 32, 56, 510, 22, 11);
    unsafe {
        SendMessageW(lbl_sub, WM_SETFONT, hfont_body as WPARAM, 1);
    }

    // Card Box
    create_ctrl(
        WC_BUTTONW,
        if is_en { " What LangSwitch does: " } else { " מה האפליקציה יודעת לעשות? " },
        BS_GROUPBOX as u32,
        25, 90, 515, 385, 12,
    );

    // Features List
    let items = if is_en {
        vec![
            ("⌨️  Primary Hotkey (F10):", "Instantly converts mistyped text AND switches input language layout automatically!"),
            ("🎯  Auto-Line Selection:", "No need to select text! If unselected, the entire current line is converted instantly."),
            ("⚡  Secondary Hotkey (Ctrl + F10):", "Converts text ONLY (without changing keyboard input layout)."),
            ("🔍  Google AI Search (Shift + Win + G):", "Takes any copied/selected text and opens Google AI Mode directly (&udm=50)!"),
            ("🌐  Quick Translation (Ctrl + Shift + T):", "Translates text between Hebrew and English via Google directly in your browser!"),
            ("🪟  Silent & Lightweight:", "Runs quietly in your System Tray with 0.00% CPU and minimal memory."),
        ]
    } else {
        vec![
            ("\u{200F}⌨️  מקש ראשי (F10):", "\u{200F}המרה מיידית של טקסט שנכתב בשפה הפוכה + החלפת שפת המקלדת אוטומטית!"),
            ("\u{200F}🎯  זיהוי שורה אוטומטי:", "\u{200F}אין צורך לסמן בעכבר! אם לא סומן טקסט, כל השורה שהקלדת תומר מיד."),
            ("\u{200F}⚡  מקש משני (Ctrl + F10):", "\u{200F}המרת טקסט בלבד (ללא שינוי שפת קלט המקלדת)."),
            ("\u{200F}🔍  חיפוש ב-Google AI (Shift + Win + G):", "\u{200F}מסמנים כל טקסט ופותחים אותו מיד במצב ה-AI החדש של גוגל (&udm=50)!"),
            ("\u{200F}🌐  תרגום מהיר (Ctrl + Shift + T):", "\u{200F}תרגום מיידי מעברית לאנגלית ולהיפך בגוגל (בית -> באנגלית, house -> בעברית)!"),
            ("\u{200F}🪟  שקט וחסכוני במשאבים:", "\u{200F}פועל ברקע באזור מגש המערכת (ליד השעון) עם 0% ניצול מעבד."),
        ]
    };

    let mut start_y = 115;
    for (idx, (lead, desc)) in items.iter().enumerate() {
        let lbl_lead = create_ctrl(WC_STATICW, lead, 0, 45, start_y, 475, 20, 20 + idx * 2);
        unsafe {
            SendMessageW(lbl_lead, WM_SETFONT, hfont_body as WPARAM, 1);
        }

        let lbl_desc = create_ctrl(WC_STATICW, desc, 0, 50, start_y + 19, 465, 20, 21 + idx * 2);
        unsafe {
            SendMessageW(lbl_desc, WM_SETFONT, hfont_body as WPARAM, 1);
        }

        start_y += 50;
    }

    // Action button: Let's start!
    let btn_text = if is_en { "Got it, let's start! 🚀" } else { "הבנתי, בוא נתחיל! 🚀" };
    let btn = create_ctrl(
        WC_BUTTONW,
        btn_text,
        BS_DEFPUSHBUTTON as u32 | WS_TABSTOP,
        170, 495, 230, 40,
        ID_BTN_START,
    );
    unsafe {
        SendMessageW(btn, WM_SETFONT, hfont_btn as WPARAM, 1);
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
        WM_CTLCOLORDLG => {
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                return state.bg_brush as LRESULT;
            }
        }
        WM_CTLCOLORSTATIC => {
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let hdc = wparam as HDC;
                unsafe {
                    SetBkMode(hdc, TRANSPARENT as i32);
                    SetTextColor(hdc, 0x00332211); // Deep slate text
                }
                return state.bg_brush as LRESULT;
            }
        }
        WM_CTLCOLORBTN => {
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let hdc = wparam as HDC;
                unsafe {
                    SetBkMode(hdc, TRANSPARENT as i32);
                }
                return state.bg_brush as LRESULT;
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
        WM_CLOSE => {
            unsafe {
                DestroyWindow(hwnd);
            }
            return 0;
        }
        WM_DESTROY => {
            if !state_ptr.is_null() {
                let state = unsafe { Box::from_raw(state_ptr) };
                unsafe {
                    DeleteObject(state.bg_brush as *mut _);
                    DeleteObject(state.card_brush as *mut _);
                    DeleteObject(state.hfont_title as *mut _);
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
