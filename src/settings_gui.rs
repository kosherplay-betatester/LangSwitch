use crate::config::{
    is_windows_conflict, ALL_KEYBOARD_KEYS, AppConfig, HotkeyConfig, VK_F10,
};
use crate::ui_theme::{
    self, to_wide, ButtonStyle, DialogResult, BG_GRAD_BOT_PURPLE, BG_GRAD_TOP_BLUE,
    CARD_SURFACE_RGB, TEXT_DARK_SLATE, TEXT_DEEP_PURPLE, TEXT_INDIGO, TEXT_SKY_BLUE,
};
use crate::updater::{self, UpdateCheckResult, CURRENT_VERSION};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
    CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, InvalidateRect, SelectObject,
    SetBkMode, SetTextColor, UpdateWindow, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH,
    DEFAULT_QUALITY, DT_LEFT, DT_NOPREFIX, DT_RIGHT, DT_RTLREADING, DT_SINGLELINE, DT_VCENTER,
    FW_BOLD, FW_MEDIUM, FW_NORMAL, HBRUSH, HDC, HFONT, OUT_DEFAULT_PRECIS, PAINTSTRUCT, SRCCOPY,
    TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{DRAWITEMSTRUCT, ODS_SELECTED, WC_BUTTONW, WC_COMBOBOXW};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClientRect, GetDlgItem,
    GetMessageW, GetSystemMetrics, GetWindowLongPtrW, IsWindow, LoadImageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, SetForegroundWindow, SetWindowLongPtrW, SetWindowTextW,
    ShowWindow, TranslateMessage, BS_OWNERDRAW, CBS_DROPDOWNLIST, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, HMENU, HTCAPTION, ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_DEFAULTCOLOR, SM_CXICON,
    SM_CXSCREEN, SM_CXSMICON, SM_CYICON, SM_CYSCREEN, SM_CYSMICON, SW_RESTORE, SW_SHOW, WM_CLOSE,
    WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLORDLG, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC, WM_DESTROY,
    WM_DRAWITEM, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_NCLBUTTONDOWN, WM_PAINT, WM_SETFONT,
    WM_SETICON, WNDCLASSEXW, WS_CAPTION, WS_CHILD, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP,
    WS_VISIBLE, WS_VSCROLL, SetWindowPos, SWP_NOSIZE, SWP_NOZORDER,
};

static SETTINGS_HWND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(ptr::null_mut());

// Control IDs
const ID_CHK_PRI_CTRL: usize = 101;
const ID_CHK_PRI_ALT: usize = 102;
const ID_CHK_PRI_SHIFT: usize = 103;
const ID_CHK_PRI_WIN: usize = 104;
const ID_CMB_PRI_KEY: usize = 105;

const ID_CHK_SEC_CTRL: usize = 106;
const ID_CHK_SEC_ALT: usize = 107;
const ID_CHK_SEC_SHIFT: usize = 108;
const ID_CHK_SEC_WIN: usize = 109;
const ID_CMB_SEC_KEY: usize = 110;

const ID_CHK_AUTO_SWITCH: usize = 111;
const ID_CHK_AUTO_SELECT_LINE: usize = 112;
const ID_CHK_START_WINDOWS: usize = 113;
const ID_CHK_RESTORE_CLIPBOARD: usize = 114;

const ID_BTN_CHECK_UPDATE: usize = 115;
const ID_BTN_SAVE: usize = 116;
const ID_BTN_CANCEL: usize = 117;
const ID_BTN_RESTORE_DEFAULTS: usize = 118;
const ID_CMB_LANG: usize = 119;

const ID_CHK_AI_CTRL: usize = 120;
const ID_CHK_AI_ALT: usize = 121;
const ID_CHK_AI_SHIFT: usize = 122;
const ID_CHK_AI_WIN: usize = 123;
const ID_CMB_AI_KEY: usize = 124;

const ID_CHK_TRANS_CTRL: usize = 125;
const ID_CHK_TRANS_ALT: usize = 126;
const ID_CHK_TRANS_SHIFT: usize = 127;
const ID_CHK_TRANS_WIN: usize = 128;
const ID_CMB_TRANS_KEY: usize = 129;

const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CB_RESETCONTENT: u32 = 0x014B;

struct SettingsState {
    on_save: Box<dyn Fn(AppConfig) + Send + 'static>,
    hfont_body: HFONT,
    hfont_bold: HFONT,
    hfont_section: HFONT,
    hfont_title: HFONT,
    hfont_sub: HFONT,
    card_brush: HBRUSH,
    pri_mods: u32,
    pri_vk: u32,
    sec_mods: u32,
    sec_vk: u32,
    ai_mods: u32,
    ai_vk: u32,
    trans_mods: u32,
    trans_vk: u32,
    auto_switch_layout: bool,
    auto_select_line: bool,
    start_with_windows: bool,
    restore_clipboard: bool,
    is_english: bool,
    last_welcome_version: Option<String>,
}

impl SettingsState {
    fn is_ctrl_checked(&self, id: usize) -> bool {
        match id {
            ID_CHK_PRI_CTRL => self.pri_mods & MOD_CONTROL != 0,
            ID_CHK_PRI_ALT => self.pri_mods & MOD_ALT != 0,
            ID_CHK_PRI_SHIFT => self.pri_mods & MOD_SHIFT != 0,
            ID_CHK_PRI_WIN => self.pri_mods & MOD_WIN != 0,

            ID_CHK_SEC_CTRL => self.sec_mods & MOD_CONTROL != 0,
            ID_CHK_SEC_ALT => self.sec_mods & MOD_ALT != 0,
            ID_CHK_SEC_SHIFT => self.sec_mods & MOD_SHIFT != 0,
            ID_CHK_SEC_WIN => self.sec_mods & MOD_WIN != 0,

            ID_CHK_AI_CTRL => self.ai_mods & MOD_CONTROL != 0,
            ID_CHK_AI_ALT => self.ai_mods & MOD_ALT != 0,
            ID_CHK_AI_SHIFT => self.ai_mods & MOD_SHIFT != 0,
            ID_CHK_AI_WIN => self.ai_mods & MOD_WIN != 0,

            ID_CHK_TRANS_CTRL => self.trans_mods & MOD_CONTROL != 0,
            ID_CHK_TRANS_ALT => self.trans_mods & MOD_ALT != 0,
            ID_CHK_TRANS_SHIFT => self.trans_mods & MOD_SHIFT != 0,
            ID_CHK_TRANS_WIN => self.trans_mods & MOD_WIN != 0,

            ID_CHK_AUTO_SWITCH => self.auto_switch_layout,
            ID_CHK_AUTO_SELECT_LINE => self.auto_select_line,
            ID_CHK_START_WINDOWS => self.start_with_windows,
            ID_CHK_RESTORE_CLIPBOARD => self.restore_clipboard,
            _ => false,
        }
    }

    fn toggle_ctrl(&mut self, id: usize) {
        match id {
            ID_CHK_PRI_CTRL => self.pri_mods ^= MOD_CONTROL,
            ID_CHK_PRI_ALT => self.pri_mods ^= MOD_ALT,
            ID_CHK_PRI_SHIFT => self.pri_mods ^= MOD_SHIFT,
            ID_CHK_PRI_WIN => self.pri_mods ^= MOD_WIN,

            ID_CHK_SEC_CTRL => self.sec_mods ^= MOD_CONTROL,
            ID_CHK_SEC_ALT => self.sec_mods ^= MOD_ALT,
            ID_CHK_SEC_SHIFT => self.sec_mods ^= MOD_SHIFT,
            ID_CHK_SEC_WIN => self.sec_mods ^= MOD_WIN,

            ID_CHK_AI_CTRL => self.ai_mods ^= MOD_CONTROL,
            ID_CHK_AI_ALT => self.ai_mods ^= MOD_ALT,
            ID_CHK_AI_SHIFT => self.ai_mods ^= MOD_SHIFT,
            ID_CHK_AI_WIN => self.ai_mods ^= MOD_WIN,

            ID_CHK_TRANS_CTRL => self.trans_mods ^= MOD_CONTROL,
            ID_CHK_TRANS_ALT => self.trans_mods ^= MOD_ALT,
            ID_CHK_TRANS_SHIFT => self.trans_mods ^= MOD_SHIFT,
            ID_CHK_TRANS_WIN => self.trans_mods ^= MOD_WIN,

            ID_CHK_AUTO_SWITCH => self.auto_switch_layout = !self.auto_switch_layout,
            ID_CHK_AUTO_SELECT_LINE => self.auto_select_line = !self.auto_select_line,
            ID_CHK_START_WINDOWS => self.start_with_windows = !self.start_with_windows,
            ID_CHK_RESTORE_CLIPBOARD => self.restore_clipboard = !self.restore_clipboard,
            _ => {}
        }
    }
}

pub fn show_settings_window(
    current_config: AppConfig,
    on_save: Box<dyn Fn(AppConfig) + Send + 'static>,
) -> Option<std::thread::JoinHandle<()>> {
    let existing = SETTINGS_HWND.load(Ordering::SeqCst) as HWND;
    if !existing.is_null() && unsafe { IsWindow(existing) } != 0 {
        unsafe {
            ShowWindow(existing, SW_RESTORE);
            SetForegroundWindow(existing);
        }
        return None;
    }

    Some(std::thread::spawn(move || unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = to_wide("LangSwitchSettingsWindow");

        let card_brush = CreateSolidBrush(CARD_SURFACE_RGB);

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_wnd_proc),
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

        let win_width = 650;
        let win_height = 740;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_width) / 2;
        let pos_y = (screen_h - win_height) / 2;

        let is_en = current_config.ui_language == "en";
        let title = if is_en {
            to_wide("LangSwitch - Settings")
        } else {
            to_wide("LangSwitch - הגדרות")
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

        SETTINGS_HWND.store(hwnd as *mut _, Ordering::SeqCst);

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

        // Fonts
        let font_name = to_wide("Segoe UI");
        let hfont_title = CreateFontW(
            -20, 0, 0, 0, FW_BOLD as i32, 0, 0, 0,
            DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_sub = CreateFontW(
            -13, 0, 0, 0, FW_MEDIUM as i32, 0, 0, 0,
            DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_section = CreateFontW(
            -15, 0, 0, 0, FW_BOLD as i32, 0, 0, 0,
            DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_bold = CreateFontW(
            -13, 0, 0, 0, FW_BOLD as i32, 0, 0, 0,
            DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );
        let hfont_body = CreateFontW(
            -14, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
            DEFAULT_CHARSET as u32, OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32, DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32, font_name.as_ptr(),
        );

        // Initialize state BEFORE creating controls so WM_DRAWITEM / WM_CTLCOLOR always have valid state!
        let state = Box::new(SettingsState {
            on_save,
            hfont_body,
            hfont_bold,
            hfont_section,
            hfont_title,
            hfont_sub,
            card_brush,
            pri_mods: current_config.hotkey_primary.modifiers,
            pri_vk: current_config.hotkey_primary.vk,
            sec_mods: current_config.hotkey_secondary.modifiers,
            sec_vk: current_config.hotkey_secondary.vk,
            ai_mods: current_config.hotkey_ai_search.modifiers,
            ai_vk: current_config.hotkey_ai_search.vk,
            trans_mods: current_config.hotkey_translate.modifiers,
            trans_vk: current_config.hotkey_translate.vk,
            auto_switch_layout: current_config.auto_switch_layout,
            auto_select_line: current_config.auto_select_line,
            start_with_windows: current_config.start_with_windows,
            restore_clipboard: current_config.restore_clipboard,
            is_english: is_en,
            last_welcome_version: current_config.last_welcome_version.clone(),
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        init_controls(hwnd, hinstance, hfont_body, &current_config, is_en);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        SETTINGS_HWND.store(ptr::null_mut(), Ordering::SeqCst);
    }))
}

unsafe fn init_controls(
    hwnd: HWND,
    hinstance: windows_sys::Win32::Foundation::HMODULE,
    hfont_body: HFONT,
    config: &AppConfig,
    is_en: bool,
) {
    let create_ctrl = |class: *const u16, text: &str, style: u32, x, y, w, h, id: usize| -> HWND {
        let title = to_wide(text);
        unsafe {
            let ctrl = CreateWindowExW(
                0,
                class,
                title.as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                x,
                y,
                w,
                h,
                hwnd,
                id as HMENU,
                hinstance,
                ptr::null(),
            );
            if !hfont_body.is_null() {
                SendMessageW(ctrl, WM_SETFONT, hfont_body as WPARAM, 1);
            }
            ctrl
        }
    };

    // Language ComboBox inside Header Banner (Left side in RTL Hebrew, Right side in LTR English)
    let lang_x = if is_en { 442 } else { 32 };
    let cmb_lang = create_ctrl(
        WC_COMBOBOXW,
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP | WS_VSCROLL,
        lang_x,
        30,
        155,
        120,
        ID_CMB_LANG,
    );
    let lang_he = to_wide("🇮🇱 עברית (Hebrew)");
    let lang_en = to_wide("🇺🇸 English");
    unsafe {
        SendMessageW(cmb_lang, CB_ADDSTRING, 0, lang_he.as_ptr() as LPARAM);
        SendMessageW(cmb_lang, CB_ADDSTRING, 0, lang_en.as_ptr() as LPARAM);
        SendMessageW(cmb_lang, CB_SETCURSEL, if is_en { 1 } else { 0 }, 0);
    }

    // Helper for creating a 4-modifier pill row + key dropdown
    let create_hotkey_row = |y_controls: i32,
                                 id_ctrl: usize,
                                 id_alt: usize,
                                 id_shift: usize,
                                 id_win: usize,
                                 id_cmb: usize| {
        let pill_w = 58;
        let pill_h = 26;
        let gap = 6;
        let start_x = 34;

        create_ctrl(
            WC_BUTTONW,
            "Ctrl",
            BS_OWNERDRAW as u32 | WS_TABSTOP,
            start_x,
            y_controls,
            pill_w,
            pill_h,
            id_ctrl,
        );
        create_ctrl(
            WC_BUTTONW,
            "Alt",
            BS_OWNERDRAW as u32 | WS_TABSTOP,
            start_x + (pill_w + gap),
            y_controls,
            pill_w,
            pill_h,
            id_alt,
        );
        create_ctrl(
            WC_BUTTONW,
            "Shift",
            BS_OWNERDRAW as u32 | WS_TABSTOP,
            start_x + (pill_w + gap) * 2,
            y_controls,
            pill_w,
            pill_h,
            id_shift,
        );
        create_ctrl(
            WC_BUTTONW,
            "Win",
            BS_OWNERDRAW as u32 | WS_TABSTOP,
            start_x + (pill_w + gap) * 3,
            y_controls,
            pill_w,
            pill_h,
            id_win,
        );

        create_ctrl(
            WC_COMBOBOXW,
            "",
            CBS_DROPDOWNLIST as u32 | WS_TABSTOP | WS_VSCROLL,
            300,
            y_controls + 1,
            298,
            380,
            id_cmb,
        );
    };

    // Card 1 Hotkey Rows:
    // Row 1: Primary (y_label = 114, y_controls = 136)
    create_hotkey_row(
        136,
        ID_CHK_PRI_CTRL,
        ID_CHK_PRI_ALT,
        ID_CHK_PRI_SHIFT,
        ID_CHK_PRI_WIN,
        ID_CMB_PRI_KEY,
    );
    // Row 2: Secondary (y_label = 172, y_controls = 194)
    create_hotkey_row(
        194,
        ID_CHK_SEC_CTRL,
        ID_CHK_SEC_ALT,
        ID_CHK_SEC_SHIFT,
        ID_CHK_SEC_WIN,
        ID_CMB_SEC_KEY,
    );
    // Row 3: Google AI Search (y_label = 230, y_controls = 252)
    create_hotkey_row(
        252,
        ID_CHK_AI_CTRL,
        ID_CHK_AI_ALT,
        ID_CHK_AI_SHIFT,
        ID_CHK_AI_WIN,
        ID_CMB_AI_KEY,
    );
    // Row 4: Quick Translation (y_label = 288, y_controls = 310)
    create_hotkey_row(
        310,
        ID_CHK_TRANS_CTRL,
        ID_CHK_TRANS_ALT,
        ID_CHK_TRANS_SHIFT,
        ID_CHK_TRANS_WIN,
        ID_CMB_TRANS_KEY,
    );

    // Populate comboboxes
    unsafe {
        populate_combobox(
            hwnd,
            ID_CMB_PRI_KEY,
            config.hotkey_primary.modifiers,
            config.hotkey_primary.vk,
        );
        populate_combobox(
            hwnd,
            ID_CMB_SEC_KEY,
            config.hotkey_secondary.modifiers,
            config.hotkey_secondary.vk,
        );
        populate_combobox(
            hwnd,
            ID_CMB_AI_KEY,
            config.hotkey_ai_search.modifiers,
            config.hotkey_ai_search.vk,
        );
        populate_combobox(
            hwnd,
            ID_CMB_TRANS_KEY,
            config.hotkey_translate.modifiers,
            config.hotkey_translate.vk,
        );
    }

    // Card 2: 4 Owner-Drawn Custom Checkbox Rows (inside Card 2: y = 362..534)
    let opt_x = 34;
    let opt_w = 564;
    let opt_h = 28;
    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        opt_x,
        402,
        opt_w,
        opt_h,
        ID_CHK_AUTO_SWITCH,
    );
    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        opt_x,
        433,
        opt_w,
        opt_h,
        ID_CHK_AUTO_SELECT_LINE,
    );
    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        opt_x,
        464,
        opt_w,
        opt_h,
        ID_CHK_START_WINDOWS,
    );
    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        opt_x,
        495,
        opt_w,
        opt_h,
        ID_CHK_RESTORE_CLIPBOARD,
    );

    // Card 3: Updates & Version (y = 546..618)
    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        380,
        566,
        218,
        36,
        ID_BTN_CHECK_UPDATE,
    );

    // Bottom Action Buttons (y = 634)
    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        18,
        634,
        175,
        40,
        ID_BTN_RESTORE_DEFAULTS,
    );

    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        318,
        634,
        165,
        40,
        ID_BTN_SAVE,
    );

    create_ctrl(
        WC_BUTTONW,
        "",
        BS_OWNERDRAW as u32 | WS_TABSTOP,
        495,
        634,
        120,
        40,
        ID_BTN_CANCEL,
    );
}

unsafe fn populate_combobox(hwnd: HWND, cmb_id: usize, modifiers: u32, desired_vk: u32) {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
    let cmb = unsafe { GetDlgItem(hwnd, cmb_id as i32) };
    unsafe {
        SendMessageW(cmb, CB_RESETCONTENT, 0, 0);
    }

    let valid_keys: Vec<(&str, u32)> = ALL_KEYBOARD_KEYS
        .iter()
        .filter(|(_, vk)| !is_windows_conflict(modifiers, *vk))
        .copied()
        .collect();

    let mut selected_idx = 0;
    let mut found = false;
    for (i, (name, vk)) in valid_keys.iter().enumerate() {
        let wname = to_wide(name);
        unsafe {
            SendMessageW(cmb, CB_ADDSTRING, 0, wname.as_ptr() as LPARAM);
        }
        if *vk == desired_vk {
            selected_idx = i;
            found = true;
        }
    }

    if !found && !valid_keys.is_empty() {
        if let Some(pos) = valid_keys.iter().position(|(_, vk)| *vk == VK_F10) {
            selected_idx = pos;
        } else {
            selected_idx = 0;
        }
    }

    unsafe {
        SendMessageW(cmb, CB_SETCURSEL, selected_idx, 0);
        InvalidateRect(cmb, ptr::null(), 1);
        UpdateWindow(cmb);
    }

    if !state_ptr.is_null() {
        let state = unsafe { &mut *state_ptr };
        if let Some((_, vk)) = valid_keys.get(selected_idx) {
            if cmb_id == ID_CMB_PRI_KEY {
                state.pri_vk = *vk;
            } else if cmb_id == ID_CMB_SEC_KEY {
                state.sec_vk = *vk;
            } else if cmb_id == ID_CMB_AI_KEY {
                state.ai_vk = *vk;
            } else if cmb_id == ID_CMB_TRANS_KEY {
                state.trans_vk = *vk;
            }
        }
    }
}

unsafe fn refresh_hotkey_combobox(hwnd: HWND, target: u8) {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
    if state_ptr.is_null() {
        return;
    }
    let state = unsafe { &*state_ptr };

    unsafe {
        match target {
            0 => populate_combobox(hwnd, ID_CMB_PRI_KEY, state.pri_mods, state.pri_vk),
            1 => populate_combobox(hwnd, ID_CMB_SEC_KEY, state.sec_mods, state.sec_vk),
            2 => populate_combobox(hwnd, ID_CMB_AI_KEY, state.ai_mods, state.ai_vk),
            _ => populate_combobox(hwnd, ID_CMB_TRANS_KEY, state.trans_mods, state.trans_vk),
        }
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => return 1,
        WM_LBUTTONDOWN => unsafe {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0);
            return 0;
        },
        WM_PAINT => {
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
            let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                let is_en = state.is_english;
                let is_rtl = !is_en;

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

                    // 1. Smooth Light Blue -> Light Purple window background gradient
                    ui_theme::fill_vertical_gradient(
                        mem_dc,
                        &rc,
                        BG_GRAD_TOP_BLUE,
                        BG_GRAD_BOT_PURPLE,
                    );

                    // 2. Top Header Banner (Sky Blue <-> Light Purple Gradient)
                    let header_title = if is_en {
                        "⚡ LangSwitch Settings"
                    } else {
                        "⚡ הגדרות LangSwitch"
                    };
                    let header_sub = if is_en {
                        "Instant Hebrew <-> English Layout Text Switcher for Windows"
                    } else {
                        "המרת טקסט והחלפת שפה מהירה בין עברית לאנגלית עבור Windows"
                    };
                    ui_theme::draw_header_banner(
                        mem_dc,
                        18,
                        12,
                        w - 18,
                        74,
                        header_title,
                        header_sub,
                        state.hfont_title,
                        state.hfont_sub,
                        is_rtl,
                    );

                    // 3. Card 1: Hotkeys (Light Blue -> Light Purple gradient border)
                    ui_theme::draw_gradient_card(mem_dc, 18, 84, w - 18, 350, 14, true);

                    // 4. Card 2: Options & Behavior (Light Purple -> Light Blue gradient border!)
                    ui_theme::draw_gradient_card(mem_dc, 18, 362, w - 18, 534, 14, false);

                    // 5. Card 3: Updates & Version (Light Blue -> Light Purple gradient border)
                    ui_theme::draw_gradient_card(mem_dc, 18, 546, w - 18, 620, 14, true);

                    SetBkMode(mem_dc, TRANSPARENT as i32);

                    let draw_label = |text: &str,
                                      x_left: i32,
                                      y_top: i32,
                                      x_right: i32,
                                      y_bot: i32,
                                      font: HFONT,
                                      color: u32,
                                      rtl: bool| {
                        SelectObject(mem_dc, font as *mut _);
                        SetTextColor(mem_dc, color);
                        let mut lrc = RECT {
                            left: x_left,
                            top: y_top,
                            right: x_right,
                            bottom: y_bot,
                        };
                        let flags = if rtl {
                            DT_RIGHT | DT_RTLREADING | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX
                        } else {
                            DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX
                        };
                        let wtxt = to_wide(text);
                        DrawTextW(mem_dc, wtxt.as_ptr(), -1, &mut lrc, flags);
                    };

                    // Section 1 Header
                    draw_label(
                        if is_en {
                            "⌨️  Keyboard Shortcuts (Hotkeys)"
                        } else {
                            "⌨️  מקשי קיצור (Hotkeys)"
                        },
                        34,
                        91,
                        w - 34,
                        113,
                        state.hfont_section,
                        TEXT_DEEP_PURPLE,
                        is_rtl,
                    );

                    // Hotkey Row Labels
                    draw_label(
                        if is_en {
                            "Primary Hotkey — Convert text + Switch keyboard layout:"
                        } else {
                            "מקש ראשי — המרת טקסט + החלפת שפת מקלדת:"
                        },
                        34,
                        114,
                        w - 34,
                        134,
                        state.hfont_bold,
                        TEXT_SKY_BLUE,
                        is_rtl,
                    );

                    draw_label(
                        if is_en {
                            "Secondary Hotkey — Convert text only (keep current layout):"
                        } else {
                            "מקש משני — המרת טקסט בלבד (ללא החלפת שפת מקלדת):"
                        },
                        34,
                        172,
                        w - 34,
                        192,
                        state.hfont_bold,
                        TEXT_INDIGO,
                        is_rtl,
                    );

                    draw_label(
                        if is_en {
                            "Google AI Search Hotkey — Open selected text in Google AI Mode:"
                        } else {
                            "מקש חיפוש ב-Google AI — פתיחת הטקסט במצב ה-AI של גוגל:"
                        },
                        34,
                        230,
                        w - 34,
                        250,
                        state.hfont_bold,
                        TEXT_SKY_BLUE,
                        is_rtl,
                    );

                    draw_label(
                        if is_en {
                            "Quick Translation Hotkey — Instant Hebrew <-> English via Google:"
                        } else {
                            "מקש תרגום מהיר בגוגל — תרגום מיידי בין עברית לאנגלית:"
                        },
                        34,
                        288,
                        w - 34,
                        308,
                        state.hfont_bold,
                        TEXT_DEEP_PURPLE,
                        is_rtl,
                    );

                    // Section 2 Header
                    draw_label(
                        if is_en {
                            "⚙️  Options & Behavior"
                        } else {
                            "⚙️  אפשרויות והתנהגות"
                        },
                        34,
                        372,
                        w - 34,
                        396,
                        state.hfont_section,
                        TEXT_DEEP_PURPLE,
                        is_rtl,
                    );

                    // Section 3 Header & Version Info
                    draw_label(
                        if is_en {
                            "🚀  Updates & Version Info"
                        } else {
                            "🚀  עדכונים ומידע על הגרסה"
                        },
                        34,
                        554,
                        365,
                        578,
                        state.hfont_section,
                        TEXT_INDIGO,
                        is_rtl,
                    );

                    let ver_str = if is_en {
                        format!("Current Version: v{} (Windows Edition)", CURRENT_VERSION)
                    } else {
                        format!("גרסה נוכחית: v{} (מהדורת Windows)", CURRENT_VERSION)
                    };
                    draw_label(
                        &ver_str,
                        34,
                        582,
                        365,
                        606,
                        state.hfont_bold,
                        TEXT_DARK_SLATE,
                        is_rtl,
                    );

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
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
            if state_ptr.is_null() {
                return 0;
            }
            let state = unsafe { &*state_ptr };
            let is_en = state.is_english;
            let is_rtl = !is_en;
            let ctrl_id = dis.CtlID as usize;
            let is_pressed = (dis.itemState & ODS_SELECTED) != 0;

            unsafe {
                match ctrl_id {
                    // Modifier Pills (Ctrl, Alt, Shift, Win)
                    ID_CHK_PRI_CTRL | ID_CHK_SEC_CTRL | ID_CHK_AI_CTRL | ID_CHK_TRANS_CTRL => {
                        ui_theme::draw_modifier_pill(
                            dis.hDC,
                            &dis.rcItem,
                            "Ctrl",
                            state.hfont_bold,
                            state.is_ctrl_checked(ctrl_id),
                            is_pressed,
                        );
                        return 1;
                    }
                    ID_CHK_PRI_ALT | ID_CHK_SEC_ALT | ID_CHK_AI_ALT | ID_CHK_TRANS_ALT => {
                        ui_theme::draw_modifier_pill(
                            dis.hDC,
                            &dis.rcItem,
                            "Alt",
                            state.hfont_bold,
                            state.is_ctrl_checked(ctrl_id),
                            is_pressed,
                        );
                        return 1;
                    }
                    ID_CHK_PRI_SHIFT | ID_CHK_SEC_SHIFT | ID_CHK_AI_SHIFT | ID_CHK_TRANS_SHIFT => {
                        ui_theme::draw_modifier_pill(
                            dis.hDC,
                            &dis.rcItem,
                            "Shift",
                            state.hfont_bold,
                            state.is_ctrl_checked(ctrl_id),
                            is_pressed,
                        );
                        return 1;
                    }
                    ID_CHK_PRI_WIN | ID_CHK_SEC_WIN | ID_CHK_AI_WIN | ID_CHK_TRANS_WIN => {
                        ui_theme::draw_modifier_pill(
                            dis.hDC,
                            &dis.rcItem,
                            "Win",
                            state.hfont_bold,
                            state.is_ctrl_checked(ctrl_id),
                            is_pressed,
                        );
                        return 1;
                    }

                    // 4 Custom Gradient Behavior Checkbox Rows
                    ID_CHK_AUTO_SWITCH => {
                        let txt = if is_en {
                            "Switch keyboard layout automatically on conversion (Primary hotkey)"
                        } else {
                            "החלף שפת מקלדת אוטומטית בעת המרת טקסט (עבור המקש הראשי)"
                        };
                        ui_theme::draw_custom_checkbox_row(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_body,
                            state.auto_switch_layout,
                            is_rtl,
                        );
                        return 1;
                    }
                    ID_CHK_AUTO_SELECT_LINE => {
                        let txt = if is_en {
                            "Auto-select current line if no text is highlighted"
                        } else {
                            "סמן אוטומטית את השורה הנוכחית אם לא נבחר טקסט"
                        };
                        ui_theme::draw_custom_checkbox_row(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_body,
                            state.auto_select_line,
                            is_rtl,
                        );
                        return 1;
                    }
                    ID_CHK_START_WINDOWS => {
                        let txt = if is_en {
                            "Start LangSwitch automatically when Windows starts"
                        } else {
                            "הפעל את LangSwitch אוטומטית עם הפעלת Windows"
                        };
                        ui_theme::draw_custom_checkbox_row(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_body,
                            state.start_with_windows,
                            is_rtl,
                        );
                        return 1;
                    }
                    ID_CHK_RESTORE_CLIPBOARD => {
                        let txt = if is_en {
                            "Restore previous clipboard contents after conversion"
                        } else {
                            "שחזר את תוכן לוח הגזירים המקורי לאחר ההמרה"
                        };
                        ui_theme::draw_custom_checkbox_row(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_body,
                            state.restore_clipboard,
                            is_rtl,
                        );
                        return 1;
                    }

                    // Action Buttons
                    ID_BTN_SAVE => {
                        let txt = if is_en {
                            "💾  Save & Apply"
                        } else {
                            "💾  שמור והחל"
                        };
                        ui_theme::draw_gradient_button(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_bold,
                            ButtonStyle::PrimaryGradient,
                            is_pressed,
                        );
                        return 1;
                    }
                    ID_BTN_RESTORE_DEFAULTS => {
                        let txt = if is_en {
                            "↺  Restore Defaults"
                        } else {
                            "↺  איפוס לברירת מחדל"
                        };
                        ui_theme::draw_gradient_button(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_bold,
                            ButtonStyle::LightPurple,
                            is_pressed,
                        );
                        return 1;
                    }
                    ID_BTN_CHECK_UPDATE => {
                        let txt = if is_en {
                            "🔍  Check for Updates"
                        } else {
                            "🔍  בדיקת עדכונים ב-GitHub"
                        };
                        ui_theme::draw_gradient_button(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_bold,
                            ButtonStyle::LightBlue,
                            is_pressed,
                        );
                        return 1;
                    }
                    ID_BTN_CANCEL => {
                        let txt = if is_en { "Cancel" } else { "ביטול" };
                        ui_theme::draw_gradient_button(
                            dis.hDC,
                            &dis.rcItem,
                            txt,
                            state.hfont_bold,
                            ButtonStyle::Subtle,
                            is_pressed,
                        );
                        return 1;
                    }
                    _ => return 0,
                }
            }
        }
        WM_COMMAND => {
            let ctrl_id = (wparam & 0xFFFF) as usize;
            let notif = ((wparam >> 16) & 0xFFFF) as u32;
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };

            // Handle combobox selection changes (CBN_SELCHANGE == 1)
            if notif == 1 && !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                if ctrl_id == ID_CMB_PRI_KEY {
                    let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_PRI_KEY as i32) };
                    let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as isize };
                    if idx >= 0 {
                        let valid: Vec<_> = ALL_KEYBOARD_KEYS
                            .iter()
                            .filter(|(_, vk)| !is_windows_conflict(state.pri_mods, *vk))
                            .collect();
                        if let Some((_, vk)) = valid.get(idx as usize) {
                            state.pri_vk = *vk;
                        }
                    }
                } else if ctrl_id == ID_CMB_SEC_KEY {
                    let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_SEC_KEY as i32) };
                    let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as isize };
                    if idx >= 0 {
                        let valid: Vec<_> = ALL_KEYBOARD_KEYS
                            .iter()
                            .filter(|(_, vk)| !is_windows_conflict(state.sec_mods, *vk))
                            .collect();
                        if let Some((_, vk)) = valid.get(idx as usize) {
                            state.sec_vk = *vk;
                        }
                    }
                } else if ctrl_id == ID_CMB_AI_KEY {
                    let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_AI_KEY as i32) };
                    let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as isize };
                    if idx >= 0 {
                        let valid: Vec<_> = ALL_KEYBOARD_KEYS
                            .iter()
                            .filter(|(_, vk)| !is_windows_conflict(state.ai_mods, *vk))
                            .collect();
                        if let Some((_, vk)) = valid.get(idx as usize) {
                            state.ai_vk = *vk;
                        }
                    }
                } else if ctrl_id == ID_CMB_TRANS_KEY {
                    let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_TRANS_KEY as i32) };
                    let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as isize };
                    if idx >= 0 {
                        let valid: Vec<_> = ALL_KEYBOARD_KEYS
                            .iter()
                            .filter(|(_, vk)| !is_windows_conflict(state.trans_mods, *vk))
                            .collect();
                        if let Some((_, vk)) = valid.get(idx as usize) {
                            state.trans_vk = *vk;
                        }
                    }
                } else if ctrl_id == ID_CMB_LANG {
                    let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_LANG as i32) };
                    let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as usize };
                    state.is_english = idx == 1;
                    unsafe {
                        update_ui_language(hwnd, state.is_english);
                    }
                }
            }

            // Handle button / pill / checkbox clicks (BN_CLICKED == 0)
            if notif == 0 && !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                match ctrl_id {
                    ID_CHK_PRI_CTRL | ID_CHK_PRI_ALT | ID_CHK_PRI_SHIFT | ID_CHK_PRI_WIN => unsafe {
                        state.toggle_ctrl(ctrl_id);
                        let ctrl = GetDlgItem(hwnd, ctrl_id as i32);
                        InvalidateRect(ctrl, ptr::null(), 0);
                        refresh_hotkey_combobox(hwnd, 0);
                        return 0;
                    },
                    ID_CHK_SEC_CTRL | ID_CHK_SEC_ALT | ID_CHK_SEC_SHIFT | ID_CHK_SEC_WIN => unsafe {
                        state.toggle_ctrl(ctrl_id);
                        let ctrl = GetDlgItem(hwnd, ctrl_id as i32);
                        InvalidateRect(ctrl, ptr::null(), 0);
                        refresh_hotkey_combobox(hwnd, 1);
                        return 0;
                    },
                    ID_CHK_AI_CTRL | ID_CHK_AI_ALT | ID_CHK_AI_SHIFT | ID_CHK_AI_WIN => unsafe {
                        state.toggle_ctrl(ctrl_id);
                        let ctrl = GetDlgItem(hwnd, ctrl_id as i32);
                        InvalidateRect(ctrl, ptr::null(), 0);
                        refresh_hotkey_combobox(hwnd, 2);
                        return 0;
                    },
                    ID_CHK_TRANS_CTRL | ID_CHK_TRANS_ALT | ID_CHK_TRANS_SHIFT
                    | ID_CHK_TRANS_WIN => unsafe {
                        state.toggle_ctrl(ctrl_id);
                        let ctrl = GetDlgItem(hwnd, ctrl_id as i32);
                        InvalidateRect(ctrl, ptr::null(), 0);
                        refresh_hotkey_combobox(hwnd, 3);
                        return 0;
                    },
                    ID_CHK_AUTO_SWITCH
                    | ID_CHK_AUTO_SELECT_LINE
                    | ID_CHK_START_WINDOWS
                    | ID_CHK_RESTORE_CLIPBOARD => unsafe {
                        state.toggle_ctrl(ctrl_id);
                        let ctrl = GetDlgItem(hwnd, ctrl_id as i32);
                        InvalidateRect(ctrl, ptr::null(), 0);
                        return 0;
                    },
                    ID_BTN_RESTORE_DEFAULTS => unsafe {
                        handle_restore_defaults(hwnd);
                        return 0;
                    },
                    ID_BTN_CANCEL => unsafe {
                        DestroyWindow(hwnd);
                        return 0;
                    },
                    ID_BTN_SAVE => unsafe {
                        save_and_close(hwnd);
                        return 0;
                    },
                    ID_BTN_CHECK_UPDATE => unsafe {
                        trigger_check_update(hwnd);
                        return 0;
                    },
                    _ => {}
                }
            }
            return 0;
        }
        WM_CTLCOLORDLG | WM_CTLCOLORSTATIC | WM_CTLCOLORBTN | WM_CTLCOLORLISTBOX => {
            let hdc = wparam as HDC;
            unsafe {
                SetBkMode(hdc, TRANSPARENT as i32);
                SetTextColor(hdc, TEXT_DARK_SLATE);
            }
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
            if !state_ptr.is_null() {
                let state = unsafe { &*state_ptr };
                return state.card_brush as LRESULT;
            }
        }
        WM_CLOSE => unsafe {
            DestroyWindow(hwnd);
            return 0;
        },
        WM_DESTROY => unsafe {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState;
            if !state_ptr.is_null() {
                let state = Box::from_raw(state_ptr);
                DeleteObject(state.hfont_body as *mut _);
                DeleteObject(state.hfont_bold as *mut _);
                DeleteObject(state.hfont_section as *mut _);
                DeleteObject(state.hfont_title as *mut _);
                DeleteObject(state.hfont_sub as *mut _);
                DeleteObject(state.card_brush as *mut _);
            }
            PostQuitMessage(0);
            return 0;
        },
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn update_ui_language(hwnd: HWND, is_en: bool) {
    let title = if is_en {
        "LangSwitch - Settings"
    } else {
        "LangSwitch - הגדרות"
    };
    let w_title = to_wide(title);
    unsafe {
        SetWindowTextW(hwnd, w_title.as_ptr());
        let cmb_lang = GetDlgItem(hwnd, ID_CMB_LANG as i32);
        let lang_x = if is_en { 442 } else { 32 };
        SetWindowPos(cmb_lang, ptr::null_mut(), lang_x, 30, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
        InvalidateRect(hwnd, ptr::null(), 1);

        let ids = [
            ID_CHK_AUTO_SWITCH,
            ID_CHK_AUTO_SELECT_LINE,
            ID_CHK_START_WINDOWS,
            ID_CHK_RESTORE_CLIPBOARD,
            ID_BTN_CHECK_UPDATE,
            ID_BTN_RESTORE_DEFAULTS,
            ID_BTN_SAVE,
            ID_BTN_CANCEL,
        ];
        for id in ids {
            let ctrl = GetDlgItem(hwnd, id as i32);
            InvalidateRect(ctrl, ptr::null(), 1);
        }
    }
}

unsafe fn handle_restore_defaults(hwnd: HWND) {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
    if state_ptr.is_null() {
        return;
    }
    let is_en = unsafe { (*state_ptr).is_english };

    let res = if is_en {
        ui_theme::show_modern_dialog(
            "Restore Default Settings",
            "Reset all hotkeys and preferences",
            "Are you sure you want to reset all LangSwitch hotkeys and behavior options back to their recommended default values?",
            "Yes, Restore Defaults",
            Some("Cancel"),
            false,
        )
    } else {
        ui_theme::show_modern_dialog(
            "איפוס לברירת מחדל",
            "שחזור הגדרות ומקשי קיצור מומלצים",
            "האם אתה בטוח שברצונך לאפס את כל מקשי הקיצור והאפשרויות של LangSwitch לערכי ברירת המחדל המומלצים?",
            "כן, אפס הגדרות",
            Some("ביטול"),
            true,
        )
    };

    if res != DialogResult::Primary {
        return;
    }

    let state = unsafe { &mut *state_ptr };
    state.pri_mods = 0;
    state.pri_vk = VK_F10;
    state.sec_mods = MOD_CONTROL;
    state.sec_vk = VK_F10;
    state.ai_mods = MOD_SHIFT | MOD_WIN;
    state.ai_vk = 0x47; // 'G'
    state.trans_mods = MOD_CONTROL | MOD_SHIFT;
    state.trans_vk = 0x54; // 'T'

    state.auto_switch_layout = true;
    state.auto_select_line = true;
    state.start_with_windows = false;
    state.restore_clipboard = false;

    unsafe {
        populate_combobox(hwnd, ID_CMB_PRI_KEY, state.pri_mods, state.pri_vk);
        populate_combobox(hwnd, ID_CMB_SEC_KEY, state.sec_mods, state.sec_vk);
        populate_combobox(hwnd, ID_CMB_AI_KEY, state.ai_mods, state.ai_vk);
        populate_combobox(hwnd, ID_CMB_TRANS_KEY, state.trans_mods, state.trans_vk);
        InvalidateRect(hwnd, ptr::null(), 1);
    }
}

unsafe fn trigger_check_update(hwnd: HWND) {
    let hwnd_val = hwnd as usize;
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
    let is_en = if !state_ptr.is_null() {
        unsafe { (*state_ptr).is_english }
    } else {
        false
    };

    std::thread::spawn(move || {
        let res = updater::check_for_updates();
        let target_hwnd = hwnd_val as HWND;
        unsafe {
            match res {
                UpdateCheckResult::UpdateAvailable {
                    version,
                    url,
                    notes,
                } => {
                    let answer = if is_en {
                        ui_theme::show_modern_dialog(
                            "New Update Available! 🎉",
                            &format!("LangSwitch {} is ready to download", version),
                            &format!(
                                "A new version ({}) is available on GitHub!\n\nWhat's new:\n{}\n\nWould you like to open the download page now?",
                                version, notes
                            ),
                            "Download Update 🚀",
                            Some("Later"),
                            false,
                        )
                    } else {
                        ui_theme::show_modern_dialog(
                            "עדכון חדש זמין! 🎉",
                            &format!("גרסה {} זמינה כעת להורדה", version),
                            &format!(
                                "גרסה חדשה ({}) זמינה ב-GitHub!\n\nמה חדש:\n{}\n\nהאם ברצונך לפתוח את עמוד ההורדה כעת?",
                                version, notes
                            ),
                            "הורד עדכון 🚀",
                            Some("מאוחר יותר"),
                            true,
                        )
                    };
                    if answer == DialogResult::Primary {
                        let wurl = to_wide(&url);
                        let wopen = to_wide("open");
                        ShellExecuteW(
                            target_hwnd,
                            wopen.as_ptr(),
                            wurl.as_ptr(),
                            ptr::null(),
                            ptr::null(),
                            SW_SHOW,
                        );
                    }
                }
                UpdateCheckResult::UpToDate => {
                    if is_en {
                        ui_theme::show_modern_dialog(
                            "LangSwitch is Up to Date ⚡",
                            &format!("Running latest version v{}", CURRENT_VERSION),
                            &format!(
                                "You are running the latest version of LangSwitch (v{}).\nNo updates are needed right now.",
                                CURRENT_VERSION
                            ),
                            "Awesome!",
                            None,
                            false,
                        );
                    } else {
                        ui_theme::show_modern_dialog(
                            "התוכנה מעודכנת! ⚡",
                            &format!("אתה משתמש בגרסה העדכנית ביותר v{}", CURRENT_VERSION),
                            &format!(
                                "התוכנה מעודכנת לגרסה האחרונה (v{}).\nאין צורך בעדכונים נוספים כרגע.",
                                CURRENT_VERSION
                            ),
                            "מעולה!",
                            None,
                            true,
                        );
                    }
                }
                UpdateCheckResult::NoReleasesFound => {
                    if is_en {
                        ui_theme::show_modern_dialog(
                            "Update Check",
                            "GitHub Releases",
                            "No releases were found on GitHub yet.",
                            "OK",
                            None,
                            false,
                        );
                    } else {
                        ui_theme::show_modern_dialog(
                            "בדיקת עדכונים",
                            "שחרורים ב-GitHub",
                            "לא נמצאו גרסאות שחרור (Releases) ב-GitHub עדיין.",
                            "אישור",
                            None,
                            true,
                        );
                    }
                }
                UpdateCheckResult::Error(e) => {
                    if is_en {
                        ui_theme::show_modern_dialog(
                            "Update Check Error",
                            "Could not connect to GitHub",
                            &format!("An error occurred while checking for updates:\n{}", e),
                            "Close",
                            None,
                            false,
                        );
                    } else {
                        ui_theme::show_modern_dialog(
                            "שגיאה בבדיקת עדכונים",
                            "לא ניתן להתחבר ל-GitHub",
                            &format!("אירעה שגיאה בעת בדיקת העדכונים:\n{}", e),
                            "סגור",
                            None,
                            true,
                        );
                    }
                }
            }
        }
    });
}

unsafe fn save_and_close(hwnd: HWND) {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };

    if state_ptr.is_null() {
        unsafe {
            DestroyWindow(hwnd);
        }
        return;
    }

    let state = unsafe { &mut *state_ptr };

    let get_combo_vk = |id: usize, modifiers: u32, default_vk: u32| -> u32 {
        unsafe {
            let ctrl = GetDlgItem(hwnd, id as i32);
            let idx = SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as isize;
            if idx >= 0 {
                let valid: Vec<_> = ALL_KEYBOARD_KEYS
                    .iter()
                    .filter(|(_, vk)| !is_windows_conflict(modifiers, *vk))
                    .collect();
                if let Some((_, vk)) = valid.get(idx as usize) {
                    return *vk;
                }
            }
            default_vk
        }
    };

    let ui_lang = if state.is_english {
        "en".to_string()
    } else {
        "he".to_string()
    };

    let updated_config = AppConfig {
        hotkey_primary: HotkeyConfig {
            modifiers: state.pri_mods,
            vk: get_combo_vk(ID_CMB_PRI_KEY, state.pri_mods, state.pri_vk),
        },
        hotkey_secondary: HotkeyConfig {
            modifiers: state.sec_mods,
            vk: get_combo_vk(ID_CMB_SEC_KEY, state.sec_mods, state.sec_vk),
        },
        hotkey_ai_search: HotkeyConfig {
            modifiers: state.ai_mods,
            vk: get_combo_vk(ID_CMB_AI_KEY, state.ai_mods, state.ai_vk),
        },
        hotkey_translate: HotkeyConfig {
            modifiers: state.trans_mods,
            vk: get_combo_vk(ID_CMB_TRANS_KEY, state.trans_mods, state.trans_vk),
        },
        auto_switch_layout: state.auto_switch_layout,
        auto_select_line: state.auto_select_line,
        start_with_windows: state.start_with_windows,
        restore_clipboard: state.restore_clipboard,
        ui_language: ui_lang,
        last_welcome_version: state.last_welcome_version.clone(),
    };

    updated_config.save();
    (state.on_save)(updated_config);

    unsafe {
        DestroyWindow(hwnd);
    }
}
