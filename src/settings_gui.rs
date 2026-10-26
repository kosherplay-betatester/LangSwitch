use crate::config::{
    is_windows_conflict, ALL_KEYBOARD_KEYS, AppConfig, HotkeyConfig, VK_F10,
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
    RegisterClassExW, SendMessageW, SetForegroundWindow, SetWindowLongPtrW, SetWindowTextW,
    ShowWindow, TranslateMessage, BM_GETCHECK, BM_SETCHECK, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON,
    BS_GROUPBOX, BS_PUSHBUTTON, CBS_DROPDOWNLIST, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, HMENU,
    IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO, SM_CXSCREEN,
    SM_CYSCREEN, SW_RESTORE, SW_SHOW, WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_SETFONT, WNDCLASSEXW,
    WS_CHILD, WS_EX_DLGMODALFRAME, WS_OVERLAPPED, WS_POPUP, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL,
};

static SETTINGS_HWND: AtomicPtr<std::ffi::c_void> = AtomicPtr::new(ptr::null_mut());

const BST_CHECKED: isize = 1;
const BST_UNCHECKED: isize = 0;
const COLOR_BTNFACE: u32 = 15;

// Control IDs
const ID_LBL_GRP_HOTKEYS: usize = 90;
const ID_LBL_PRI_TITLE: usize = 91;
const ID_LBL_SEC_TITLE: usize = 92;
const ID_LBL_GRP_OPTIONS: usize = 93;
const ID_LBL_GRP_UPDATES: usize = 94;
const ID_LBL_VERSION: usize = 95;
const ID_LBL_LANG: usize = 96;

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

const CB_ADDSTRING: u32 = 0x0143;
const CB_SETCURSEL: u32 = 0x014E;
const CB_GETCURSEL: u32 = 0x0147;
const CB_RESETCONTENT: u32 = 0x014B;

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

        let win_width = 620;
        let win_height = 575;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = (screen_w - win_width) / 2;
        let pos_y = (screen_h - win_height) / 2;

        let title = if current_config.ui_language == "en" {
            to_wide("LangSwitch - Settings")
        } else {
            to_wide("LangSwitch - הגדרות")
        };

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

        let is_en = current_config.ui_language == "en";
        init_controls(hwnd, hinstance, hfont, &current_config, is_en);

        let state = Box::new(SettingsState {
            on_save,
            hfont,
            pri_vk: current_config.hotkey_primary.vk,
            sec_vk: current_config.hotkey_secondary.vk,
            is_english: is_en,
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
    pri_vk: u32,
    sec_vk: u32,
    is_english: bool,
}

unsafe fn init_controls(
    hwnd: HWND,
    hinstance: windows_sys::Win32::Foundation::HMODULE,
    hfont: HFONT,
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
            if !hfont.is_null() {
                SendMessageW(ctrl, WM_SETFONT, hfont as WPARAM, 1);
            }
            ctrl
        }
    };

    // --- Language Selector at Top Right ---
    create_ctrl(
        WC_STATICW,
        if is_en { "Language / שפה:" } else { "שפה / Language:" },
        0,
        340,
        14,
        130,
        22,
        ID_LBL_LANG,
    );
    let cmb_lang = create_ctrl(
        WC_COMBOBOXW,
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP | WS_VSCROLL,
        470,
        12,
        115,
        120,
        ID_CMB_LANG,
    );
    let lang_he = to_wide("עברית (Hebrew)");
    let lang_en = to_wide("English");
    unsafe {
        SendMessageW(cmb_lang, CB_ADDSTRING, 0, lang_he.as_ptr() as LPARAM);
        SendMessageW(cmb_lang, CB_ADDSTRING, 0, lang_en.as_ptr() as LPARAM);
        SendMessageW(cmb_lang, CB_SETCURSEL, if is_en { 1 } else { 0 }, 0);
    }

    // --- Group 1: Hotkeys ---
    create_ctrl(
        WC_BUTTONW,
        if is_en { " Hotkeys " } else { " מקשי קיצור (Hotkeys) " },
        BS_GROUPBOX as u32,
        15,
        36,
        575,
        165,
        ID_LBL_GRP_HOTKEYS,
    );

    // Primary hotkey
    create_ctrl(
        WC_STATICW,
        if is_en {
            "Primary Hotkey (Hebrew <-> English Conversion):"
        } else {
            "מקש ראשי (המרה בין עברית לאנגלית):"
        },
        0,
        30,
        60,
        350,
        20,
        ID_LBL_PRI_TITLE,
    );

    let chk_pri_ctrl = create_ctrl(WC_BUTTONW, "Ctrl", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 30, 85, 55, 20, ID_CHK_PRI_CTRL);
    let chk_pri_alt = create_ctrl(WC_BUTTONW, "Alt", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 90, 85, 50, 20, ID_CHK_PRI_ALT);
    let chk_pri_shift = create_ctrl(WC_BUTTONW, "Shift", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 145, 85, 58, 20, ID_CHK_PRI_SHIFT);
    let chk_pri_win = create_ctrl(WC_BUTTONW, "Win", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 208, 85, 52, 20, ID_CHK_PRI_WIN);

    // Primary ComboBox WITH WS_VSCROLL for mouse wheel scrolling!
    let _cmb_pri_key = create_ctrl(
        WC_COMBOBOXW,
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP | WS_VSCROLL,
        275,
        83,
        290,
        380,
        ID_CMB_PRI_KEY,
    );

    // Secondary hotkey
    create_ctrl(
        WC_STATICW,
        if is_en {
            "Secondary Hotkey (Convert + Always Switch Layout):"
        } else {
            "מקש משני (המרה + החלפת שפת מקלדת תמיד):"
        },
        0,
        30,
        118,
        350,
        20,
        ID_LBL_SEC_TITLE,
    );

    let chk_sec_ctrl = create_ctrl(WC_BUTTONW, "Ctrl", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 30, 142, 55, 20, ID_CHK_SEC_CTRL);
    let chk_sec_alt = create_ctrl(WC_BUTTONW, "Alt", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 90, 142, 50, 20, ID_CHK_SEC_ALT);
    let chk_sec_shift = create_ctrl(WC_BUTTONW, "Shift", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 145, 142, 58, 20, ID_CHK_SEC_SHIFT);
    let chk_sec_win = create_ctrl(WC_BUTTONW, "Win", BS_AUTOCHECKBOX as u32 | WS_TABSTOP, 208, 142, 52, 20, ID_CHK_SEC_WIN);

    // Secondary ComboBox WITH WS_VSCROLL for mouse wheel scrolling!
    let _cmb_sec_key = create_ctrl(
        WC_COMBOBOXW,
        "",
        CBS_DROPDOWNLIST as u32 | WS_TABSTOP | WS_VSCROLL,
        275,
        140,
        290,
        380,
        ID_CMB_SEC_KEY,
    );

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
    }

    // --- Group 2: Behavior Options ---
    create_ctrl(
        WC_BUTTONW,
        if is_en { " Options & Behavior " } else { " אפשרויות והתנהגות " },
        BS_GROUPBOX as u32,
        15,
        210,
        575,
        145,
        ID_LBL_GRP_OPTIONS,
    );

    let chk_switch = create_ctrl(
        WC_BUTTONW,
        if is_en {
            "Switch keyboard layout automatically on conversion (Primary key)"
        } else {
            "החלף שפת מקלדת אוטומטית בהמרה (עבור המקש הראשי)"
        },
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        235,
        540,
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
        if is_en {
            "Auto-select current line if no text is selected"
        } else {
            "סמן אוטומטית את השורה הנוכחית אם לא סומן טקסט"
        },
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        262,
        540,
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
        if is_en {
            "Start automatically on Windows login"
        } else {
            "הפעל אוטומטית עם כניסה למערכת Windows"
        },
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        289,
        540,
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
        if is_en {
            "Restore original clipboard contents after conversion"
        } else {
            "שחזר לוח גזירים (Clipboard) מקורי לאחר ההמרה"
        },
        BS_AUTOCHECKBOX as u32 | WS_TABSTOP,
        30,
        316,
        540,
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
        if is_en { " Updates & Info " } else { " עדכונים ומידע " },
        BS_GROUPBOX as u32,
        15,
        365,
        575,
        70,
        ID_LBL_GRP_UPDATES,
    );

    let ver_label = if is_en {
        format!("Current Version: v{}", CURRENT_VERSION)
    } else {
        format!("גרסה נוכחית: v{}", CURRENT_VERSION)
    };
    create_ctrl(WC_STATICW, &ver_label, 0, 30, 395, 230, 20, ID_LBL_VERSION);

    create_ctrl(
        WC_BUTTONW,
        if is_en { "Check for Updates" } else { "בדוק עדכונים מ-GitHub" },
        BS_PUSHBUTTON as u32 | WS_TABSTOP,
        390,
        388,
        185,
        30,
        ID_BTN_CHECK_UPDATE,
    );

    // --- Bottom Action Buttons ---
    create_ctrl(
        WC_BUTTONW,
        if is_en { "Restore Defaults" } else { "איפוס ברירת מחדל" },
        BS_PUSHBUTTON as u32 | WS_TABSTOP,
        15,
        485,
        160,
        34,
        ID_BTN_RESTORE_DEFAULTS,
    );

    create_ctrl(
        WC_BUTTONW,
        if is_en { "Save & Apply" } else { "שמור והחל" },
        BS_DEFPUSHBUTTON as u32 | WS_TABSTOP,
        330,
        485,
        130,
        34,
        ID_BTN_SAVE,
    );

    create_ctrl(
        WC_BUTTONW,
        if is_en { "Cancel" } else { "ביטול" },
        BS_PUSHBUTTON as u32 | WS_TABSTOP,
        470,
        485,
        115,
        34,
        ID_BTN_CANCEL,
    );
}

/// Populates a combobox with all keys that do NOT conflict with Windows shortcuts for the given modifier set.
unsafe fn populate_combobox(hwnd: HWND, cmb_id: usize, modifiers: u32, desired_vk: u32) {
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
    for (i, (name, vk)) in valid_keys.iter().enumerate() {
        let wname = to_wide(name);
        unsafe {
            SendMessageW(cmb, CB_ADDSTRING, 0, wname.as_ptr() as LPARAM);
        }
        if *vk == desired_vk {
            selected_idx = i;
        }
    }

    unsafe {
        SendMessageW(cmb, CB_SETCURSEL, selected_idx, 0);
    }
}

/// Reads the modifier checkboxes and updates the combobox filtering in real-time.
unsafe fn refresh_hotkey_combobox(hwnd: HWND, is_primary: bool) {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };

    let is_checked = |id: usize| -> bool {
        unsafe {
            let ctrl = GetDlgItem(hwnd, id as i32);
            SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED
        }
    };

    if is_primary {
        let mut modifiers = 0;
        if is_checked(ID_CHK_PRI_CTRL) {
            modifiers |= MOD_CONTROL;
        }
        if is_checked(ID_CHK_PRI_ALT) {
            modifiers |= MOD_ALT;
        }
        if is_checked(ID_CHK_PRI_SHIFT) {
            modifiers |= MOD_SHIFT;
        }
        if is_checked(ID_CHK_PRI_WIN) {
            modifiers |= MOD_WIN;
        }

        let current_vk = if !state_ptr.is_null() {
            unsafe { (*state_ptr).pri_vk }
        } else {
            VK_F10
        };

        unsafe {
            populate_combobox(hwnd, ID_CMB_PRI_KEY, modifiers, current_vk);
        }
    } else {
        let mut modifiers = 0;
        if is_checked(ID_CHK_SEC_CTRL) {
            modifiers |= MOD_CONTROL;
        }
        if is_checked(ID_CHK_SEC_ALT) {
            modifiers |= MOD_ALT;
        }
        if is_checked(ID_CHK_SEC_SHIFT) {
            modifiers |= MOD_SHIFT;
        }
        if is_checked(ID_CHK_SEC_WIN) {
            modifiers |= MOD_WIN;
        }

        let current_vk = if !state_ptr.is_null() {
            unsafe { (*state_ptr).sec_vk }
        } else {
            VK_F10
        };

        unsafe {
            populate_combobox(hwnd, ID_CMB_SEC_KEY, modifiers, current_vk);
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
        WM_COMMAND => {
            let ctrl_id = (wparam & 0xFFFF) as usize;
            let notif = ((wparam >> 16) & 0xFFFF) as u32;

            // Handle combobox selection changes
            if notif == 1 {
                // CBN_SELCHANGE
                let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
                if !state_ptr.is_null() {
                    let state = unsafe { &mut *state_ptr };
                    if ctrl_id == ID_CMB_PRI_KEY {
                        let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_PRI_KEY as i32) };
                        let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as usize };
                        let modifiers = unsafe { get_modifiers(hwnd, true) };
                        let valid: Vec<_> = ALL_KEYBOARD_KEYS
                            .iter()
                            .filter(|(_, vk)| !is_windows_conflict(modifiers, *vk))
                            .collect();
                        if let Some((_, vk)) = valid.get(idx) {
                            state.pri_vk = *vk;
                        }
                    } else if ctrl_id == ID_CMB_SEC_KEY {
                        let ctrl = unsafe { GetDlgItem(hwnd, ID_CMB_SEC_KEY as i32) };
                        let idx = unsafe { SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as usize };
                        let modifiers = unsafe { get_modifiers(hwnd, false) };
                        let valid: Vec<_> = ALL_KEYBOARD_KEYS
                            .iter()
                            .filter(|(_, vk)| !is_windows_conflict(modifiers, *vk))
                            .collect();
                        if let Some((_, vk)) = valid.get(idx) {
                            state.sec_vk = *vk;
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
            }

            match ctrl_id {
                // Modifier checkboxes toggled -> refresh combobox in real time
                ID_CHK_PRI_CTRL | ID_CHK_PRI_ALT | ID_CHK_PRI_SHIFT | ID_CHK_PRI_WIN => unsafe {
                    refresh_hotkey_combobox(hwnd, true);
                    return 0;
                },
                ID_CHK_SEC_CTRL | ID_CHK_SEC_ALT | ID_CHK_SEC_SHIFT | ID_CHK_SEC_WIN => unsafe {
                    refresh_hotkey_combobox(hwnd, false);
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

unsafe fn update_ui_language(hwnd: HWND, is_en: bool) {
    let set_text = |id: usize, text: &str| unsafe {
        let ctrl = GetDlgItem(hwnd, id as i32);
        let w = to_wide(text);
        SetWindowTextW(ctrl, w.as_ptr());
    };

    let title = if is_en { "LangSwitch - Settings" } else { "LangSwitch - הגדרות" };
    let w_title = to_wide(title);
    unsafe {
        SetWindowTextW(hwnd, w_title.as_ptr());
    }

    set_text(ID_LBL_LANG, if is_en { "Language / שפה:" } else { "שפה / Language:" });
    set_text(ID_LBL_GRP_HOTKEYS, if is_en { " Hotkeys " } else { " מקשי קיצור (Hotkeys) " });
    set_text(ID_LBL_PRI_TITLE, if is_en { "Primary Hotkey (Hebrew <-> English Conversion):" } else { "מקש ראשי (המרה בין עברית לאנגלית):" });
    set_text(ID_LBL_SEC_TITLE, if is_en { "Secondary Hotkey (Convert + Always Switch Layout):" } else { "מקש משני (המרה + החלפת שפת מקלדת תמיד):" });
    set_text(ID_LBL_GRP_OPTIONS, if is_en { " Options & Behavior " } else { " אפשרויות והתנהגות " });
    set_text(ID_CHK_AUTO_SWITCH, if is_en { "Switch keyboard layout automatically on conversion (Primary key)" } else { "החלף שפת מקלדת אוטומטית בהמרה (עבור המקש הראשי)" });
    set_text(ID_CHK_AUTO_SELECT_LINE, if is_en { "Auto-select current line if no text is selected" } else { "סמן אוטומטית את השורה הנוכחית אם לא סומן טקסט" });
    set_text(ID_CHK_START_WINDOWS, if is_en { "Start automatically on Windows login" } else { "הפעל אוטומטית עם כניסה למערכת Windows" });
    set_text(ID_CHK_RESTORE_CLIPBOARD, if is_en { "Restore original clipboard contents after conversion" } else { "שחזר לוח גזירים (Clipboard) מקורי לאחר ההמרה" });
    set_text(ID_LBL_GRP_UPDATES, if is_en { " Updates & Info " } else { " עדכונים ומידע " });
    set_text(ID_LBL_VERSION, &if is_en { format!("Current Version: v{}", CURRENT_VERSION) } else { format!("גרסה נוכחית: v{}", CURRENT_VERSION) });
    set_text(ID_BTN_CHECK_UPDATE, if is_en { "Check for Updates" } else { "בדוק עדכונים מ-GitHub" });
    set_text(ID_BTN_RESTORE_DEFAULTS, if is_en { "Restore Defaults" } else { "איפוס ברירת מחדל" });
    set_text(ID_BTN_SAVE, if is_en { "Save & Apply" } else { "שמור והחל" });
    set_text(ID_BTN_CANCEL, if is_en { "Cancel" } else { "ביטול" });
}

unsafe fn handle_restore_defaults(hwnd: HWND) {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsState };
    let is_en = if !state_ptr.is_null() {
        unsafe { (*state_ptr).is_english }
    } else {
        false
    };

    let msg = if is_en {
        "Are you sure you want to reset all settings to default values?"
    } else {
        "האם אתה בטוח שברצונך לאפס את כל ההגדרות לברירת המחדל?"
    };
    let title = if is_en { "Restore Defaults" } else { "איפוס לברירת מחדל" };

    let wmsg = to_wide(msg);
    let wtitle = to_wide(title);
    let answer = unsafe {
        MessageBoxW(hwnd, wmsg.as_ptr(), wtitle.as_ptr(), MB_YESNO | MB_ICONQUESTION)
    };

    if answer != IDYES {
        return;
    }

    // Reset controls to defaults
    let set_check = |id: usize, checked: bool| unsafe {
        let ctrl = GetDlgItem(hwnd, id as i32);
        SendMessageW(ctrl, BM_SETCHECK, if checked { BST_CHECKED } else { BST_UNCHECKED } as WPARAM, 0);
    };

    // Primary: F10, no modifiers
    set_check(ID_CHK_PRI_CTRL, false);
    set_check(ID_CHK_PRI_ALT, false);
    set_check(ID_CHK_PRI_SHIFT, false);
    set_check(ID_CHK_PRI_WIN, false);

    // Secondary: Shift + F10
    set_check(ID_CHK_SEC_CTRL, false);
    set_check(ID_CHK_SEC_ALT, false);
    set_check(ID_CHK_SEC_SHIFT, true);
    set_check(ID_CHK_SEC_WIN, false);

    // Options defaults
    set_check(ID_CHK_AUTO_SWITCH, true);
    set_check(ID_CHK_AUTO_SELECT_LINE, true);
    set_check(ID_CHK_START_WINDOWS, false);
    set_check(ID_CHK_RESTORE_CLIPBOARD, false);

    if !state_ptr.is_null() {
        unsafe {
            (*state_ptr).pri_vk = VK_F10;
            (*state_ptr).sec_vk = VK_F10;
        }
    }

    unsafe {
        populate_combobox(hwnd, ID_CMB_PRI_KEY, 0, VK_F10);
        populate_combobox(hwnd, ID_CMB_SEC_KEY, MOD_SHIFT, VK_F10);
    }
}

unsafe fn get_modifiers(hwnd: HWND, is_primary: bool) -> u32 {
    let is_checked = |id: usize| -> bool {
        unsafe {
            let ctrl = GetDlgItem(hwnd, id as i32);
            SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED
        }
    };

    let mut mod_mask = 0;
    if is_primary {
        if is_checked(ID_CHK_PRI_CTRL) {
            mod_mask |= MOD_CONTROL;
        }
        if is_checked(ID_CHK_PRI_ALT) {
            mod_mask |= MOD_ALT;
        }
        if is_checked(ID_CHK_PRI_SHIFT) {
            mod_mask |= MOD_SHIFT;
        }
        if is_checked(ID_CHK_PRI_WIN) {
            mod_mask |= MOD_WIN;
        }
    } else {
        if is_checked(ID_CHK_SEC_CTRL) {
            mod_mask |= MOD_CONTROL;
        }
        if is_checked(ID_CHK_SEC_ALT) {
            mod_mask |= MOD_ALT;
        }
        if is_checked(ID_CHK_SEC_SHIFT) {
            mod_mask |= MOD_SHIFT;
        }
        if is_checked(ID_CHK_SEC_WIN) {
            mod_mask |= MOD_WIN;
        }
    }
    mod_mask
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
                    let msg = if is_en {
                        format!(
                            "A new version is available: {}\n\nRelease notes:\n{}\n\nWould you like to open the GitHub download page?",
                            version, notes
                        )
                    } else {
                        format!(
                            "גרסה חדשה זמינה: {}\n\nשינויים:\n{}\n\nהאם ברצונך לפתוח את דף ההורדה ב-GitHub?",
                            version, notes
                        )
                    };
                    let wmsg = to_wide(&msg);
                    let wtitle = if is_en { to_wide("Update Available!") } else { to_wide("עדכון חדש זמין!") };
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
                    let msg = if is_en {
                        format!("LangSwitch is up to date!\nYou are running the latest version (v{}).", CURRENT_VERSION)
                    } else {
                        format!("התוכנה מעודכנת!\nאתה משתמש בגרסה האחרונה (v{}).", CURRENT_VERSION)
                    };
                    let wmsg = to_wide(&msg);
                    let wtitle = if is_en { to_wide("Update Check") } else { to_wide("בדיקת עדכונים") };
                    MessageBoxW(
                        target_hwnd,
                        wmsg.as_ptr(),
                        wtitle.as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                UpdateCheckResult::NoReleasesFound => {
                    let msg = if is_en {
                        "No releases found on GitHub yet."
                    } else {
                        "לא נמצאו שחרורים (Releases) ב-GitHub עבור פרויקט זה עדיין."
                    };
                    let wmsg = to_wide(msg);
                    let wtitle = if is_en { to_wide("Update Check") } else { to_wide("בדיקת עדכונים") };
                    MessageBoxW(
                        target_hwnd,
                        wmsg.as_ptr(),
                        wtitle.as_ptr(),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
                UpdateCheckResult::Error(e) => {
                    let msg = if is_en {
                        format!("An error occurred while checking for updates:\n{}", e)
                    } else {
                        format!("אירעה שגיאה בבדיקת העדכון:\n{}", e)
                    };
                    let wmsg = to_wide(&msg);
                    let wtitle = if is_en { to_wide("Update Error") } else { to_wide("שגיאת עדכון") };
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

    let pri_mod = unsafe { get_modifiers(hwnd, true) };
    let sec_mod = unsafe { get_modifiers(hwnd, false) };

    let get_combo_vk = |id: usize, modifiers: u32, default_vk: u32| -> u32 {
        unsafe {
            let ctrl = GetDlgItem(hwnd, id as i32);
            let idx = SendMessageW(ctrl, CB_GETCURSEL, 0, 0) as usize;
            let valid: Vec<_> = ALL_KEYBOARD_KEYS
                .iter()
                .filter(|(_, vk)| !is_windows_conflict(modifiers, *vk))
                .collect();
            valid.get(idx).map(|(_, vk)| *vk).unwrap_or(default_vk)
        }
    };

    let ui_lang = if state.is_english { "en".to_string() } else { "he".to_string() };

    let updated_config = AppConfig {
        hotkey_primary: HotkeyConfig {
            modifiers: pri_mod,
            vk: get_combo_vk(ID_CMB_PRI_KEY, pri_mod, state.pri_vk),
        },
        hotkey_secondary: HotkeyConfig {
            modifiers: sec_mod,
            vk: get_combo_vk(ID_CMB_SEC_KEY, sec_mod, state.sec_vk),
        },
        auto_switch_layout: is_checked(ID_CHK_AUTO_SWITCH),
        auto_select_line: is_checked(ID_CHK_AUTO_SELECT_LINE),
        start_with_windows: is_checked(ID_CHK_START_WINDOWS),
        restore_clipboard: is_checked(ID_CHK_RESTORE_CLIPBOARD),
        ui_language: ui_lang,
    };

    updated_config.save();
    (state.on_save)(updated_config);

    unsafe {
        DestroyWindow(hwnd);
    }
}
