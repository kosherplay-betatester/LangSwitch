use crate::config::{
    AppConfig, HotkeyConfig, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6,
    VK_F7, VK_F8, VK_F9,
};
use crate::updater::{self, UpdateCheckResult, CURRENT_VERSION};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, DeleteObject, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH,
    DEFAULT_QUALITY, FW_NORMAL, HBRUSH, HFONT, OUT_DEFAULT_PRECIS,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{WC_BUTTONW, WC_COMBOBOXW, WC_STATICW};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetDlgItem, GetMessageW,
    GetSystemMetrics, GetWindowLongPtrW, IsWindow, MessageBoxW, PostQuitMessage,
    RegisterClassExW, SendMessageW, SetForegroundWindow, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, BS_GROUPBOX,
    BS_PUSHBUTTON, CBS_DROPDOWNLIST, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HMENU, IDYES,
    MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MB_YESNO, SM_CXSCREEN, SM_CYSCREEN, SW_RESTORE,
    SW_SHOW, WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_SETFONT, WNDCLASSEXW, WS_CHILD,
    WS_EX_DLGMODALFRAME, WS_OVERLAPPED, WS_POPUP, WS_TABSTOP, WS_VISIBLE,
};

static SETTINGS_HWND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(ptr::null_mut());

const BST_CHECKED: isize = 1;
const COLOR_BTNFACE: u32 = 15;

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

const AVAILABLE_KEYS: &[(&str, u32)] = &[
    ("F10 (מומלץ)", VK_F10),
    ("F1", VK_F1),
    ("F2", VK_F2),
    ("F3", VK_F3),
    ("F4", VK_F4),
    ("F5", VK_F5),
    ("F6", VK_F6),
    ("F7", VK_F7),
    ("F8", VK_F8),
    ("F9", VK_F9),
    ("F11", VK_F11),
    ("F12", VK_F12),
    ("Space", 0x20),
    ("Insert", 0x2D),
    ("Pause/Break", 0x13),
    ("Q", 0x51),
    ("W", 0x57),
    ("E", 0x45),
    ("R", 0x52),
    ("T", 0x54),
    ("Y", 0x59),
    ("Z", 0x5A),
    ("X", 0x58),
    ("C", 0x43),
    ("V", 0x56),
];

const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn show_settings_window(
    current_config: AppConfig,
    on_save: Box<dyn Fn(AppConfig) + Send + 'static>,
) {
    let existing = SETTINGS_HWND.load(Ordering::SeqCst) as HWND;
    if !existing.is_null() && unsafe { IsWindow(existing) } != 0 {
        unsafe {
            ShowWindow(existing, SW_RESTORE);
            SetForegroundWindow(existing);
        }
        return;
    }

    std::thread::spawn(move || unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = to_wide("LangSwitchSettingsWindow");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: (COLOR_BTNFACE + 1) as HBRUSH,
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: ptr::null_mut(),
        };

        RegisterClassExW(&wnd_class);

        let win_width = 540;
        let win_height = 510;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_width) / 2;
        let pos_y = (screen_h - win_height) / 2;

        let title = to_wide("LangSwitch - הגדרות");

        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE | WS_OVERLAPPED | 0x00080000 | 0x00040000,
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

        // Segoe UI font
        let font_name = to_wide("Segoe UI");
        let hfont = CreateFontW(
            -13,
            0,
            0,
            0,
            FW_NORMAL as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32,
            font_name.as_ptr(),
        );

        init_controls(hwnd, hinstance, hfont, &current_config);

        let state = Box::new(SettingsState {
            on_save,
            hfont,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        SETTINGS_HWND.store(ptr::null_mut(), Ordering::SeqCst);
    });
}

struct SettingsState {
    on_save: Box<dyn Fn(AppConfig) + Send + 'static>,
    hfont: HFONT,
}

unsafe fn init_controls(
    hwnd: HWND,
    hinstance: windows_sys::Win32::Foundation::HMODULE,
    hfont: HFONT,
    config: &AppConfig,
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
            if !hfont.is_null() {
                SendMessageW(ctrl, WM_SETFONT, hfont as WPARAM, 1);
            }
            ctrl
        }
    };

    // --- Group 1: Hotkeys ---
    create_ctrl(
        WC_BUTTONW,
        " מקשי קיצור (Hotkeys) ",
        BS_GROUPBOX as u32,
        15,
        10,
        495,
        170,
        0,
    );

    // Primary hotkey
    create_ctrl(
        WC_STATICW,
        "מקש ראשי (המרה בין עברית לאנגלית):",
        0,
        30,
        35,
        280,
        20,
        0,
    );

    let chk_pri_ctrl = create_ctrl(
        WC_BUTTONW,
        "Ctrl",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        60,
        50,
        20,
        ID_CHK_PRI_CTRL,
    );
    let chk_pri_alt = create_ctrl(
        WC_BUTTONW,
        "Alt",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        85,
        60,
        50,
        20,
        ID_CHK_PRI_ALT,
    );
    let chk_pri_shift = create_ctrl(
        WC_BUTTONW,
        "Shift",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        140,
        60,
        55,
        20,
        ID_CHK_PRI_SHIFT,
    );
    let chk_pri_win = create_ctrl(
        WC_BUTTONW,
        "Win",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        200,
        60,
        50,
        20,
        ID_CHK_PRI_WIN,
    );
    let cmb_pri_key = create_ctrl(
        WC_COMBOBOXW,
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP,
        260,
        58,
        140,
        250,
        ID_CMB_PRI_KEY,
    );

    // Secondary hotkey
    create_ctrl(
        WC_STATICW,
        "מקש משני (המרה + החלפת שפת מקלדת תמיד):",
        0,
        30,
        95,
        320,
        20,
        0,
    );

    let chk_sec_ctrl = create_ctrl(
        WC_BUTTONW,
        "Ctrl",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        120,
        50,
        20,
        ID_CHK_SEC_CTRL,
    );
    let chk_sec_alt = create_ctrl(
        WC_BUTTONW,
        "Alt",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        85,
        120,
        50,
        20,
        ID_CHK_SEC_ALT,
    );
    let chk_sec_shift = create_ctrl(
        WC_BUTTONW,
        "Shift",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        140,
        120,
        55,
        20,
        ID_CHK_SEC_SHIFT,
    );
    let chk_sec_win = create_ctrl(
        WC_BUTTONW,
        "Win",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        200,
        120,
        50,
        20,
        ID_CHK_SEC_WIN,
    );
    let cmb_sec_key = create_ctrl(
        WC_COMBOBOXW,
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP,
        260,
        118,
        140,
        250,
        ID_CMB_SEC_KEY,
    );

    // Populate comboboxes
    for (name, _) in AVAILABLE_KEYS {
        let wname = to_wide(name);
        unsafe {
            SendMessageW(cmb_pri_key, CB_ADDSTRING, 0, wname.as_ptr() as LPARAM);
            SendMessageW(cmb_sec_key, CB_ADDSTRING, 0, wname.as_ptr() as LPARAM);
        }
    }

    // Set combo selections
    let pri_idx = AVAILABLE_KEYS
        .iter()
        .position(|(_, vk)| *vk == config.hotkey_primary.vk)
        .unwrap_or(0);
    unsafe {
        SendMessageW(cmb_pri_key, CB_SETCURSEL, pri_idx, 0);
    }

    let sec_idx = AVAILABLE_KEYS
        .iter()
        .position(|(_, vk)| *vk == config.hotkey_secondary.vk)
        .unwrap_or(0);
    unsafe {
        SendMessageW(cmb_sec_key, CB_SETCURSEL, sec_idx, 0);
    }

    // Set primary modifier checkboxes
    unsafe {
        if config.hotkey_primary.modifiers & MOD_CONTROL != 0 {
            SendMessageW(chk_pri_ctrl, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
        if config.hotkey_primary.modifiers & MOD_ALT != 0 {
            SendMessageW(chk_pri_alt, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
        if config.hotkey_primary.modifiers & MOD_SHIFT != 0 {
            SendMessageW(chk_pri_shift, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
        if config.hotkey_primary.modifiers & MOD_WIN != 0 {
            SendMessageW(chk_pri_win, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }

        // Set secondary modifier checkboxes
        if config.hotkey_secondary.modifiers & MOD_CONTROL != 0 {
            SendMessageW(chk_sec_ctrl, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
        if config.hotkey_secondary.modifiers & MOD_ALT != 0 {
            SendMessageW(chk_sec_alt, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
        if config.hotkey_secondary.modifiers & MOD_SHIFT != 0 {
            SendMessageW(chk_sec_shift, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
        if config.hotkey_secondary.modifiers & MOD_WIN != 0 {
            SendMessageW(chk_sec_win, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
    }

    // --- Group 2: Behavior Options ---
    create_ctrl(
        WC_BUTTONW,
        " אפשרויות והתנהגות ",
        BS_GROUPBOX as u32,
        15,
        190,
        495,
        140,
        0,
    );

    let chk_switch = create_ctrl(
        WC_BUTTONW,
        "החלף שפת מקלדת אוטומטית בהמרה (עבור המקש הראשי)",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        215,
        460,
        22,
        ID_CHK_AUTO_SWITCH,
    );
    if config.auto_switch_layout {
        unsafe {
            SendMessageW(chk_switch, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
    }

    let chk_select = create_ctrl(
        WC_BUTTONW,
        "סמן אוטומטית את השורה הנוכחית אם לא סומן טקסט",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        242,
        460,
        22,
        ID_CHK_AUTO_SELECT_LINE,
    );
    if config.auto_select_line {
        unsafe {
            SendMessageW(chk_select, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
    }

    let chk_start = create_ctrl(
        WC_BUTTONW,
        "הפעל אוטומטית עם כניסה ל-Windows",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        269,
        460,
        22,
        ID_CHK_START_WINDOWS,
    );
    if config.start_with_windows {
        unsafe {
            SendMessageW(chk_start, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
    }

    let chk_clip = create_ctrl(
        WC_BUTTONW,
        "שחזר לוח גזירים (Clipboard) מקורי לאחר ההמרה",
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        296,
        460,
        22,
        ID_CHK_RESTORE_CLIPBOARD,
    );
    if config.restore_clipboard {
        unsafe {
            SendMessageW(chk_clip, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        }
    }

    // --- Group 3: Update and Info ---
    create_ctrl(
        WC_BUTTONW,
        " עדכונים ",
        BS_GROUPBOX as u32,
        15,
        340,
        495,
        65,
        0,
    );

    let ver_label = format!("גרסה נוכחית: v{}", CURRENT_VERSION);
    create_ctrl(WC_STATICW, &ver_label, 0, 30, 368, 180, 20, 0);

    create_ctrl(
        WC_BUTTONW,
        "בדוק עדכונים מ-GitHub",
        BS_PUSHBUTTON as u32 | WS_TABSTOP,
        320,
        363,
        170,
        28,
        ID_BTN_CHECK_UPDATE,
    );

    // --- Bottom Action Buttons ---
    create_ctrl(
        WC_BUTTONW,
        "שמור והחל",
        BS_DEFPUSHBUTTON as u32 | WS_TABSTOP,
        290,
        425,
        110,
        32,
        ID_BTN_SAVE,
    );

    create_ctrl(
        WC_BUTTONW,
        "ביטול",
        BS_PUSHBUTTON as u32 | WS_TABSTOP,
        410,
        425,
        100,
        32,
        ID_BTN_CANCEL,
    );
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_COMMAND => {
            let ctrl_id = (wparam & 0xFFFF) as usize;
            match ctrl_id {
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
        WM_CLOSE => unsafe {
            DestroyWindow(hwnd);
            return 0;
        },
        WM_DESTROY => unsafe {
            let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState;
            if !state_ptr.is_null() {
                let state = Box::from_raw(state_ptr);
                if !state.hfont.is_null() {
                    DeleteObject(state.hfont as *mut _);
                }
            }
            PostQuitMessage(0);
            return 0;
        },
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe fn trigger_check_update(hwnd: HWND) {
    let hwnd_val = hwnd as usize;
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
                    let msg = format!(
                        "גרסה חדשה זמינה: {}\n\nשינויים:\n{}\n\nהאם ברצונך לפתוח את דף ההורדה ב-GitHub?",
                        version, notes
                    );
                    let wmsg = to_wide(&msg);
                    let wtitle = to_wide("עדכון חדש זמין!");
                    let answer = MessageBoxW(
                        target_hwnd,
                        wmsg.as_ptr(),
                        wtitle.as_ptr(),
                        MB_YESNO | MB_ICONINFORMATION,
                    );
                    if answer == IDYES {
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
                    let msg = format!(
                        "התוכנה מעודכנת!\nאתה משתמש בגרסה האחרונה (v{}).",
                        CURRENT_VERSION
                    );
                    let wmsg = to_wide(&msg);
                    let wtitle = to_wide("בדיקת עדכונים");
                    MessageBoxW(
                        target_hwnd,
                        wmsg.as_ptr(),
                        wtitle.as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                UpdateCheckResult::NoReleasesFound => {
                    let msg = "לא נמצאו שחרורים (Releases) ב-GitHub עבור פרויקט זה עדיין.";
                    let wmsg = to_wide(msg);
                    let wtitle = to_wide("בדיקת עדכונים");
                    MessageBoxW(
                        target_hwnd,
                        wmsg.as_ptr(),
                        wtitle.as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                UpdateCheckResult::Error(e) => {
                    let msg = format!("אירעה שגיאה בבדיקת העדכון:\n{}", e);
                    let wmsg = to_wide(&msg);
                    let wtitle = to_wide("שגיאת עדכון");
                    MessageBoxW(
                        target_hwnd,
                        wmsg.as_ptr(),
                        wtitle.as_ptr(),
                        MB_OK | MB_ICONERROR,
                    );
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

    let is_checked = |id: usize| -> bool {
        unsafe {
            let ctrl = GetDlgItem(hwnd, id as i32);
            SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED
        }
    };

    let get_combo_vk = |id: usize| -> u32 {
        unsafe {
            let ctrl = GetDlgItem(hwnd, id as i32);
            let idx = SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as usize;
            AVAILABLE_KEYS.get(idx).map(|(_, vk)| *vk).unwrap_or(VK_F10)
        }
    };

    let mut pri_mod = 0;
    if is_checked(ID_CHK_PRI_CTRL) {
        pri_mod |= MOD_CONTROL;
    }
    if is_checked(ID_CHK_PRI_ALT) {
        pri_mod |= MOD_ALT;
    }
    if is_checked(ID_CHK_PRI_SHIFT) {
        pri_mod |= MOD_SHIFT;
    }
    if is_checked(ID_CHK_PRI_WIN) {
        pri_mod |= MOD_WIN;
    }

    let mut sec_mod = 0;
    if is_checked(ID_CHK_SEC_CTRL) {
        sec_mod |= MOD_CONTROL;
    }
    if is_checked(ID_CHK_SEC_ALT) {
        sec_mod |= MOD_ALT;
    }
    if is_checked(ID_CHK_SEC_SHIFT) {
        sec_mod |= MOD_SHIFT;
    }
    if is_checked(ID_CHK_SEC_WIN) {
        sec_mod |= MOD_WIN;
    }

    let updated_config = AppConfig {
        hotkey_primary: HotkeyConfig {
            modifiers: pri_mod,
            vk: get_combo_vk(ID_CMB_PRI_KEY),
        },
        hotkey_secondary: HotkeyConfig {
            modifiers: sec_mod,
            vk: get_combo_vk(ID_CMB_SEC_KEY),
        },
        auto_switch_layout: is_checked(ID_CHK_AUTO_SWITCH),
        auto_select_line: is_checked(ID_CHK_AUTO_SELECT_LINE),
        start_with_windows: is_checked(ID_CHK_START_WINDOWS),
        restore_clipboard: is_checked(ID_CHK_RESTORE_CLIPBOARD),
    };

    updated_config.save();
    (state.on_save)(updated_config);

    unsafe {
        DestroyWindow(hwnd);
    }
}
