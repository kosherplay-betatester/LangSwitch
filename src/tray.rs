use crate::clipboard;
use crate::config::AppConfig;
use crate::converter;
use crate::settings_gui;
use crate::updater::{self, UpdateCheckResult, CURRENT_VERSION};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, GetDC, ReleaseDC, SelectObject, SetBkMode,
    SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY,
    DT_CENTER, DT_SINGLELINE, DT_VCENTER, FW_BOLD, OUT_DEFAULT_PRECIS, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT,
};
use windows_sys::Win32::UI::Shell::{
    ShellExecuteW, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
    DestroyMenu, DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowLongPtrW,
    HICON, ICONINFO, IDYES, IMAGE_ICON, LR_DEFAULTCOLOR, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
    MB_YESNO, MF_DISABLED, MF_GRAYED, MF_SEPARATOR, MF_STRING, MessageBoxW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SetForegroundWindow, SetWindowLongPtrW, SM_CXSMICON,
    SM_CYSMICON, SW_SHOW, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
    WM_COMMAND, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP,
    WM_USER, WNDCLASSEXW, WS_OVERLAPPED, LoadIconW, LoadImageW,
};

pub const WM_TRAYICON: u32 = WM_USER + 100;
pub const WM_APP_RELOAD_CONFIG: u32 = WM_USER + 101;
pub const HOTKEY_PRIMARY_ID: i32 = 1001;
pub const HOTKEY_SECONDARY_ID: i32 = 1002;
pub const HOTKEY_AI_SEARCH_ID: i32 = 1003;
pub const HOTKEY_TRANSLATE_ID: i32 = 1004;

const ID_TRAY_TITLE: usize = 2000;
const ID_TRAY_SETTINGS: usize = 2001;
const ID_TRAY_PAUSE: usize = 2002;
const ID_TRAY_UPDATE: usize = 2003;
const ID_TRAY_ABOUT: usize = 2004;
const ID_TRAY_EXIT: usize = 2005;

pub struct TrayApp {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    config: Arc<Mutex<AppConfig>>,
    is_paused: Arc<AtomicBool>,
    icon_active: HICON,
    icon_paused: HICON,
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

unsafe fn create_tray_icon(paused: bool) -> HICON {
    let size = 16;
    let hdc_screen = unsafe { GetDC(ptr::null_mut()) };
    let hdc_mem = unsafe { CreateCompatibleDC(hdc_screen) };
    let hbm_color = unsafe { CreateCompatibleBitmap(hdc_screen, size, size) };
    let hbm_mask = unsafe { CreateBitmap(size, size, 1, 1, ptr::null()) };

    let old_bmp = unsafe { SelectObject(hdc_mem, hbm_color as *mut _) };

    // Background color: Vibrant Indigo for active, Slate for paused
    let bg_color = if paused {
        rgb(100, 116, 139)
    } else {
        rgb(99, 102, 241) // Vibrant Indigo #6366F1
    };

    let brush = unsafe { CreateSolidBrush(bg_color) };
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: size,
        bottom: size,
    };
    unsafe {
        FillRect(hdc_mem, &rect, brush);
        DeleteObject(brush as *mut _);
        SetBkMode(hdc_mem, TRANSPARENT as i32);
        SetTextColor(hdc_mem, rgb(255, 255, 255));
    }

    let font_name = to_wide("Segoe UI");
    let hfont = unsafe {
        CreateFontW(
            -11,
            0,
            0,
            0,
            FW_BOLD as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            DEFAULT_QUALITY as u32,
            DEFAULT_PITCH as u32,
            font_name.as_ptr(),
        )
    };

    let old_font = unsafe { SelectObject(hdc_mem, hfont as *mut _) };
    let text = to_wide("Aא");
    unsafe {
        DrawTextW(
            hdc_mem,
            text.as_ptr(),
            -1,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(hdc_mem, old_font);
        SelectObject(hdc_mem, old_bmp);
        DeleteObject(hfont as *mut _);
        DeleteDC(hdc_mem);
        ReleaseDC(ptr::null_mut(), hdc_screen);
    }

    let icon_info = ICONINFO {
        fIcon: 1,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };

    let hicon = unsafe { CreateIconIndirect(&icon_info) };
    unsafe {
        DeleteObject(hbm_color as *mut _);
        DeleteObject(hbm_mask as *mut _);
    }
    hicon
}

pub fn run_tray_app(initial_config: AppConfig) {
    unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = to_wide("LangSwitchTrayClass");

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(tray_wnd_proc),
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

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wide("LangSwitchHost").as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            ptr::null(),
        );

        let sm_cx = GetSystemMetrics(SM_CXSMICON);
        let sm_cy = GetSystemMetrics(SM_CYSMICON);

        let icon_active = {
            let h = LoadImageW(
                hinstance,
                1 as *const u16,
                IMAGE_ICON,
                sm_cx,
                sm_cy,
                LR_DEFAULTCOLOR,
            );
            if !h.is_null() {
                h as HICON
            } else {
                let fallback = LoadIconW(hinstance, 1 as *const u16);
                if !fallback.is_null() {
                    fallback
                } else {
                    create_tray_icon(false)
                }
            }
        };

        let icon_paused = {
            let h = LoadImageW(
                hinstance,
                2 as *const u16,
                IMAGE_ICON,
                sm_cx,
                sm_cy,
                LR_DEFAULTCOLOR,
            );
            if !h.is_null() {
                h as HICON
            } else {
                create_tray_icon(true)
            }
        };

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        nid.hIcon = icon_active;

        let tip = format!(
            "LangSwitch v{}\nראשי: {}\nמשני: {}\nחיפוש AI: {}\nתרגום: {}",
            CURRENT_VERSION,
            initial_config.hotkey_primary.display_string(),
            initial_config.hotkey_secondary.display_string(),
            initial_config.hotkey_ai_search.display_string(),
            initial_config.hotkey_translate.display_string(),
        );
        let wide_tip = to_wide(&tip);
        let copy_len = wide_tip.len().min(nid.szTip.len());
        nid.szTip[..copy_len].copy_from_slice(&wide_tip[..copy_len]);

        Shell_NotifyIconW(NIM_ADD, &nid);

        let config_arc = Arc::new(Mutex::new(initial_config));
        let is_paused_arc = Arc::new(AtomicBool::new(false));

        register_configured_hotkeys(hwnd, &config_arc.lock().unwrap());

        let app_state = Box::new(TrayApp {
            hwnd,
            nid,
            config: config_arc,
            is_paused: is_paused_arc,
            icon_active,
            icon_paused,
        });

        SetWindowLongPtrW(
            hwnd,
            windows_sys::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
            Box::into_raw(app_state) as isize,
        );

        // Windows Message Loop
        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn register_configured_hotkeys(hwnd: HWND, config: &AppConfig) {
    unsafe {
        UnregisterHotKey(hwnd, HOTKEY_PRIMARY_ID);
        UnregisterHotKey(hwnd, HOTKEY_SECONDARY_ID);
        UnregisterHotKey(hwnd, HOTKEY_AI_SEARCH_ID);
        UnregisterHotKey(hwnd, HOTKEY_TRANSLATE_ID);

        RegisterHotKey(
            hwnd,
            HOTKEY_PRIMARY_ID,
            config.hotkey_primary.modifiers | MOD_NOREPEAT,
            config.hotkey_primary.vk,
        );

        RegisterHotKey(
            hwnd,
            HOTKEY_SECONDARY_ID,
            config.hotkey_secondary.modifiers | MOD_NOREPEAT,
            config.hotkey_secondary.vk,
        );

        RegisterHotKey(
            hwnd,
            HOTKEY_AI_SEARCH_ID,
            config.hotkey_ai_search.modifiers | MOD_NOREPEAT,
            config.hotkey_ai_search.vk,
        );

        RegisterHotKey(
            hwnd,
            HOTKEY_TRANSLATE_ID,
            config.hotkey_translate.modifiers | MOD_NOREPEAT,
            config.hotkey_translate.vk,
        );
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state_ptr = unsafe {
        GetWindowLongPtrW(
            hwnd,
            windows_sys::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
        ) as *mut TrayApp
    };

    match msg {
        WM_HOTKEY => {
            if !state_ptr.is_null() {
                let app = unsafe { &mut *state_ptr };
                if !app.is_paused.load(Ordering::SeqCst) {
                    let hotkey_id = wparam as i32;
                    if hotkey_id == HOTKEY_TRANSLATE_ID {
                        let (auto_select, restore_clip) = {
                            let cfg = app.config.lock().unwrap();
                            (cfg.auto_select_line, cfg.restore_clipboard)
                        };

                        thread::spawn(move || {
                            perform_google_translation(auto_select, restore_clip);
                        });
                    } else if hotkey_id == HOTKEY_AI_SEARCH_ID {
                        let (auto_select, restore_clip) = {
                            let cfg = app.config.lock().unwrap();
                            (cfg.auto_select_line, cfg.restore_clipboard)
                        };

                        thread::spawn(move || {
                            perform_google_ai_search(auto_select, restore_clip);
                        });
                    } else {
                        let (auto_select, switch_layout, restore_clip) = {
                            let cfg = app.config.lock().unwrap();
                            (
                                cfg.auto_select_line,
                                if hotkey_id == HOTKEY_PRIMARY_ID {
                                    // Primary hotkey: convert text AND switch keyboard layout
                                    cfg.auto_switch_layout
                                } else {
                                    // Secondary hotkey: convert text ONLY (no layout switch)
                                    false
                                },
                                cfg.restore_clipboard,
                            )
                        };

                        thread::spawn(move || {
                            perform_conversion(auto_select, switch_layout, restore_clip);
                        });
                    }
                }
            }
            return 0;
        }
        WM_APP_RELOAD_CONFIG => {
            if !state_ptr.is_null() {
                let app = unsafe { &mut *state_ptr };
                let cfg = app.config.lock().unwrap().clone();
                if !app.is_paused.load(Ordering::SeqCst) {
                    unsafe {
                        register_configured_hotkeys(hwnd, &cfg);
                    }
                }
                unsafe {
                    let tip = format!(
                        "LangSwitch v{}\nראשי: {}\nמשני: {}\nחיפוש AI: {}\nתרגום: {}",
                        CURRENT_VERSION,
                        cfg.hotkey_primary.display_string(),
                        cfg.hotkey_secondary.display_string(),
                        cfg.hotkey_ai_search.display_string(),
                        cfg.hotkey_translate.display_string(),
                    );
                    let tip_wide = to_wide(&tip);
                    let len = tip_wide.len().min(app.nid.szTip.len());
                    app.nid.szTip[..len].copy_from_slice(&tip_wide[..len]);
                    Shell_NotifyIconW(NIM_MODIFY, &app.nid);
                }
            }
            return 0;
        }
        WM_TRAYICON => {
            if !state_ptr.is_null() {
                let app = unsafe { &mut *state_ptr };
                let event = lparam as u32;
                if event == WM_RBUTTONUP {
                    unsafe {
                        show_tray_menu(hwnd, app);
                    }
                } else if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                    open_settings_window(app);
                }
            }
            return 0;
        }
        WM_COMMAND => {
            if !state_ptr.is_null() {
                let app = unsafe { &mut *state_ptr };
                let cmd_id = (wparam & 0xFFFF) as usize;
                match cmd_id {
                    ID_TRAY_SETTINGS => {
                        open_settings_window(app);
                    }
                    ID_TRAY_PAUSE => unsafe {
                        toggle_pause(app);
                    },
                    ID_TRAY_UPDATE => {
                        let target_hwnd = hwnd as usize;
                        thread::spawn(move || {
                            let res = updater::check_for_updates();
                            let h = target_hwnd as HWND;
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
                                            h,
                                            wmsg.as_ptr(),
                                            wtitle.as_ptr(),
                                            MB_YESNO | MB_ICONINFORMATION,
                                        );
                                        if answer == IDYES {
                                            let wurl = to_wide(&url);
                                            let wopen = to_wide("open");
                                            ShellExecuteW(
                                                h,
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
                                            h,
                                            wmsg.as_ptr(),
                                            wtitle.as_ptr(),
                                            MB_OK | MB_ICONINFORMATION,
                                        );
                                    }
                                    UpdateCheckResult::NoReleasesFound => {
                                        let msg =
                                            "לא נמצאו שחרורים (Releases) ב-GitHub עבור פרויקט זה עדיין.";
                                        let wmsg = to_wide(msg);
                                        let wtitle = to_wide("בדיקת עדכונים");
                                        MessageBoxW(
                                            h,
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
                                            h,
                                            wmsg.as_ptr(),
                                            wtitle.as_ptr(),
                                            MB_OK | MB_ICONERROR,
                                        );
                                    }
                                }
                            }
                        });
                    }
                    ID_TRAY_ABOUT => {
                        let msg = format!(
                            "LangSwitch v{}\n\nהמרה מהירה בין עברית לאנגלית\nבהשראת LangOver - יעיל, קל משקל ומהיר כברק!\n\nGitHub:\ngithub.com/kosherplay-betatester/LangSwitch",
                            CURRENT_VERSION
                        );
                        let wmsg = to_wide(&msg);
                        let wtitle = to_wide("אודות LangSwitch");
                        unsafe {
                            MessageBoxW(
                                hwnd,
                                wmsg.as_ptr(),
                                wtitle.as_ptr(),
                                MB_OK | MB_ICONINFORMATION,
                            );
                        }
                    }
                    ID_TRAY_EXIT => unsafe {
                        Shell_NotifyIconW(NIM_DELETE, &app.nid);
                        UnregisterHotKey(hwnd, HOTKEY_PRIMARY_ID);
                        UnregisterHotKey(hwnd, HOTKEY_SECONDARY_ID);
                        UnregisterHotKey(hwnd, HOTKEY_AI_SEARCH_ID);
                        UnregisterHotKey(hwnd, HOTKEY_TRANSLATE_ID);
                        PostQuitMessage(0);
                    },
                    _ => {}
                }
            }
            return 0;
        }
        WM_DESTROY => {
            if !state_ptr.is_null() {
                let app = unsafe { Box::from_raw(state_ptr) };
                unsafe {
                    Shell_NotifyIconW(NIM_DELETE, &app.nid);
                    UnregisterHotKey(hwnd, HOTKEY_PRIMARY_ID);
                    UnregisterHotKey(hwnd, HOTKEY_SECONDARY_ID);
                    UnregisterHotKey(hwnd, HOTKEY_AI_SEARCH_ID);
                    UnregisterHotKey(hwnd, HOTKEY_TRANSLATE_ID);
                    DeleteObject(app.icon_active as *mut _);
                    DeleteObject(app.icon_paused as *mut _);
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

unsafe fn show_tray_menu(hwnd: HWND, app: &TrayApp) {
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut pt);
    }

    let is_english = {
        let cfg = app.config.lock().unwrap();
        cfg.ui_language == "en"
    };

    let hmenu = unsafe { CreatePopupMenu() };

    let title_text = format!("LangSwitch v{}", CURRENT_VERSION);
    let title_wide = to_wide(&title_text);
    unsafe {
        AppendMenuW(
            hmenu,
            (MF_STRING | MF_DISABLED | MF_GRAYED) as u32,
            ID_TRAY_TITLE,
            title_wide.as_ptr(),
        );
        AppendMenuW(hmenu, MF_SEPARATOR as u32, 0, ptr::null());
    }

    let settings_text = to_wide(if is_english { "Settings..." } else { "הגדרות..." });
    unsafe {
        AppendMenuW(
            hmenu,
            MF_STRING as u32,
            ID_TRAY_SETTINGS,
            settings_text.as_ptr(),
        );
    }

    let is_paused = app.is_paused.load(Ordering::SeqCst);
    let pause_text = if is_paused {
        to_wide(if is_english { "Resume" } else { "המשך פעילות" })
    } else {
        to_wide(if is_english { "Pause" } else { "השהה" })
    };
    unsafe {
        AppendMenuW(hmenu, MF_STRING as u32, ID_TRAY_PAUSE, pause_text.as_ptr());
    }

    let update_text = to_wide(if is_english { "Check for Updates..." } else { "בדוק עדכונים..." });
    unsafe {
        AppendMenuW(hmenu, MF_STRING as u32, ID_TRAY_UPDATE, update_text.as_ptr());
    }

    let about_text = to_wide(if is_english { "About..." } else { "אודות..." });
    unsafe {
        AppendMenuW(hmenu, MF_STRING as u32, ID_TRAY_ABOUT, about_text.as_ptr());
        AppendMenuW(hmenu, MF_SEPARATOR as u32, 0, ptr::null());
    }

    let exit_text = to_wide(if is_english { "Exit" } else { "יציאה" });
    unsafe {
        AppendMenuW(hmenu, MF_STRING as u32, ID_TRAY_EXIT, exit_text.as_ptr());

        // Bring window to foreground before showing popup menu
        SetForegroundWindow(hwnd);
        TrackPopupMenu(
            hmenu,
            (TPM_RIGHTBUTTON | TPM_BOTTOMALIGN) as u32,
            pt.x,
            pt.y,
            0,
            hwnd,
            ptr::null(),
        );
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(hmenu);
    }
}

unsafe fn toggle_pause(app: &mut TrayApp) {
    let currently_paused = app.is_paused.load(Ordering::SeqCst);
    let new_state = !currently_paused;
    app.is_paused.store(new_state, Ordering::SeqCst);

    if new_state {
        // Paused: Completely unregister hotkeys so zero keyboard interception occurs
        unsafe {
            UnregisterHotKey(app.hwnd, HOTKEY_PRIMARY_ID);
            UnregisterHotKey(app.hwnd, HOTKEY_SECONDARY_ID);

            app.nid.hIcon = app.icon_paused;
            let tip = to_wide("LangSwitch (מושהה / Paused)");
            let len = tip.len().min(app.nid.szTip.len());
            app.nid.szTip[..len].copy_from_slice(&tip[..len]);
            Shell_NotifyIconW(NIM_MODIFY, &app.nid);
        }
    } else {
        // Resumed: Re-register hotkeys
        let cfg = app.config.lock().unwrap();
        unsafe {
            register_configured_hotkeys(app.hwnd, &cfg);

            app.nid.hIcon = app.icon_active;
            let tip = format!(
                "LangSwitch - פעיל\nמקש ראשי: {}",
                cfg.hotkey_primary.display_string()
            );
            let tip_wide = to_wide(&tip);
            let len = tip_wide.len().min(app.nid.szTip.len());
            app.nid.szTip[..len].copy_from_slice(&tip_wide[..len]);
            Shell_NotifyIconW(NIM_MODIFY, &app.nid);
        }
    }
}

fn open_settings_window(app: &TrayApp) {
    let current_cfg = app.config.lock().unwrap().clone();
    let config_arc = Arc::clone(&app.config);
    let hwnd_val = app.hwnd as usize;

    settings_gui::show_settings_window(
        current_cfg,
        Box::new(move |new_cfg| {
            {
                let mut guard = config_arc.lock().unwrap();
                *guard = new_cfg;
            }

            // Post message to the main thread that owns hwnd so RegisterHotKey succeeds 100%!
            unsafe {
                let hwnd = hwnd_val as HWND;
                PostMessageW(hwnd, WM_APP_RELOAD_CONFIG, 0, 0);
            }
        }),
    );
}

pub fn perform_conversion(auto_select_line: bool, switch_layout: bool, restore_clipboard: bool) {
    let original_clipboard = if restore_clipboard {
        clipboard::get_clipboard_text()
    } else {
        None
    };

    // 1. Ensure any physically held modifiers (Shift, Alt, Win, Ctrl) are released
    clipboard::release_physical_modifiers();

    // 2. Clear clipboard first so we can detect incoming copied text with zero false-positives
    clipboard::clear_clipboard();

    // 3. Try copying already-selected text
    clipboard::send_ctrl_c();

    let mut selected_text = None;
    // Rapid check: if text was already selected, it arrives in 2-4ms
    for _ in 0..4 {
        thread::sleep(Duration::from_millis(2));
        if let Some(t) = clipboard::get_clipboard_text() {
            if !t.is_empty() {
                selected_text = Some(t);
                break;
            }
        }
    }

    // 4. If nothing was selected and auto_select_line is enabled, select line & copy in a single atomic input sequence!
    if selected_text.is_none() && auto_select_line {
        clipboard::clear_clipboard();
        clipboard::select_current_line_and_copy();

        for _ in 0..5 {
            thread::sleep(Duration::from_millis(2));
            if let Some(t) = clipboard::get_clipboard_text() {
                if !t.is_empty() {
                    selected_text = Some(t);
                    break;
                }
            }
        }
    }

    // 5. Read text to convert
    let text = match selected_text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    // 6. Convert text in memory
    let (converted, direction) = converter::convert_text(&text);
    if converted == text {
        return;
    }

    // 7. Put converted text into clipboard
    clipboard::set_clipboard_text(&converted);
    thread::sleep(Duration::from_millis(2));

    // 8. Ensure modifiers are released and paste converted text immediately!
    clipboard::release_physical_modifiers();
    clipboard::send_ctrl_v();

    // 9. Switch layout if requested (direct Win32 message, 0ms overhead, no Alt key)
    if switch_layout {
        let target = match direction {
            converter::ConversionDirection::EnglishToHebrew => clipboard::TargetLanguage::Hebrew,
            converter::ConversionDirection::HebrewToEnglish => clipboard::TargetLanguage::English,
        };
        clipboard::switch_keyboard_layout(target);
    }

    // 10. Restore original clipboard if requested
    if let Some(orig) = original_clipboard {
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            clipboard::set_clipboard_text(&orig);
        });
    }
}

/// Standard URL query string encoder (alphanumeric, -, _, ., ~ unchanged; spaces become '+'; other bytes %XX)
pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => {
                encoded.push('+');
            }
            other => {
                encoded.push_str(&format!("%{:02X}", other));
            }
        }
    }
    encoded
}

/// Copies selected text (or current line if auto_select_line is enabled) and opens Google AI Search (&udm=50) in a new browser tab
pub fn perform_google_ai_search(auto_select_line: bool, restore_clipboard: bool) {
    let original_clipboard = if restore_clipboard {
        clipboard::get_clipboard_text()
    } else {
        None
    };

    // 1. Ensure any physically held modifiers (Shift, Win, Ctrl, Alt) are released
    clipboard::release_physical_modifiers();

    // 2. Clear clipboard first
    clipboard::clear_clipboard();

    // 3. Try copying already-selected text
    clipboard::send_ctrl_c();

    let mut query_text = None;
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(2));
        if let Some(t) = clipboard::get_clipboard_text() {
            let trimmed = t.trim().to_string();
            if !trimmed.is_empty() {
                query_text = Some(trimmed);
                break;
            }
        }
    }

    // 4. If nothing was selected and auto_select_line is enabled, select current line
    if query_text.is_none() && auto_select_line {
        clipboard::clear_clipboard();
        clipboard::select_current_line_and_copy();

        for _ in 0..6 {
            thread::sleep(Duration::from_millis(2));
            if let Some(t) = clipboard::get_clipboard_text() {
                let trimmed = t.trim().to_string();
                if !trimmed.is_empty() {
                    query_text = Some(trimmed);
                    break;
                }
            }
        }
    }

    // 5. Restore clipboard if requested
    if let Some(orig) = original_clipboard {
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            clipboard::set_clipboard_text(&orig);
        });
    }

    // 6. If we have a query, open Google AI Search with &udm=50 in the default browser!
    if let Some(query) = query_text {
        let encoded = url_encode(&query);
        let search_url = format!("https://www.google.com/search?q={}&udm=50", encoded);
        let wide_url = to_wide(&search_url);
        let wide_open = to_wide("open");

        unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                wide_open.as_ptr(),
                wide_url.as_ptr(),
                ptr::null(),
                ptr::null(),
                SW_SHOW,
            );
        }
    }
}

/// Builds the Google translation search query based on input text language:
/// - If text contains Hebrew letters (or Hebrew >= Latin): translates to English -> query is "{text} באנגלית"
/// - If text is in Latin/English: translates to Hebrew -> query is "{text} בעברית"
pub fn build_translation_query(text: &str) -> String {
    let trimmed = text.trim();
    let hebrew_count = trimmed
        .chars()
        .filter(|c| ('\u{05D0}'..='\u{05EA}').contains(c))
        .count();
    let latin_count = trimmed.chars().filter(|c| c.is_ascii_alphabetic()).count();

    let suffix = if hebrew_count > 0 && hebrew_count >= latin_count {
        "באנגלית"
    } else if latin_count > 0 {
        "בעברית"
    } else if hebrew_count > 0 {
        "באנגלית"
    } else {
        "בעברית"
    };

    format!("{} {}", trimmed, suffix)
}

/// Copies selected text (or current line if auto_select_line is enabled) and opens Google Quick Translation
/// (e.g. "בית באנגלית" or "house בעברית") in a new browser tab
pub fn perform_google_translation(auto_select_line: bool, restore_clipboard: bool) {
    let original_clipboard = if restore_clipboard {
        clipboard::get_clipboard_text()
    } else {
        None
    };

    // 1. Ensure any physically held modifiers are released
    clipboard::release_physical_modifiers();

    // 2. Clear clipboard first
    clipboard::clear_clipboard();

    // 3. Try copying already-selected text
    clipboard::send_ctrl_c();

    let mut query_text = None;
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(2));
        if let Some(t) = clipboard::get_clipboard_text() {
            let trimmed = t.trim().to_string();
            if !trimmed.is_empty() {
                query_text = Some(trimmed);
                break;
            }
        }
    }

    // 4. If nothing was selected and auto_select_line is enabled, select current line
    if query_text.is_none() && auto_select_line {
        clipboard::clear_clipboard();
        clipboard::select_current_line_and_copy();

        for _ in 0..6 {
            thread::sleep(Duration::from_millis(2));
            if let Some(t) = clipboard::get_clipboard_text() {
                let trimmed = t.trim().to_string();
                if !trimmed.is_empty() {
                    query_text = Some(trimmed);
                    break;
                }
            }
        }
    }

    // 5. Restore clipboard if requested
    if let Some(orig) = original_clipboard {
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            clipboard::set_clipboard_text(&orig);
        });
    }

    // 6. If we have a query, open Google Quick Translation in the default browser!
    if let Some(text) = query_text {
        let query = build_translation_query(&text);
        let encoded = url_encode(&query);
        let search_url = format!("https://www.google.com/search?q={}", encoded);
        let wide_url = to_wide(&search_url);
        let wide_open = to_wide("open");

        unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                wide_open.as_ptr(),
                wide_url.as_ptr(),
                ptr::null(),
                ptr::null(),
                SW_SHOW,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode_english() {
        assert_eq!(url_encode("what is oxygen"), "what+is+oxygen");
    }

    #[test]
    fn test_url_encode_hebrew() {
        let enc = url_encode("מה זה חמצן");
        assert!(enc.contains('+'));
        assert!(enc.contains('%'));
    }

    #[test]
    fn test_build_translation_query_hebrew() {
        assert_eq!(build_translation_query("בית"), "בית באנגלית");
        assert_eq!(build_translation_query(" שלום עולם "), "שלום עולם באנגלית");
    }

    #[test]
    fn test_build_translation_query_english() {
        assert_eq!(build_translation_query("house"), "house בעברית");
        assert_eq!(
            build_translation_query("what is oxygen"),
            "what is oxygen בעברית"
        );
    }
}

