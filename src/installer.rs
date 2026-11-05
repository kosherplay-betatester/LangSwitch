use crate::config::set_startup_enabled;
use crate::ui_theme::{self, to_wide, DialogResult};
use crate::updater::CURRENT_VERSION;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_DWORD, REG_SZ, RegCloseKey, RegCreateKeyW, RegDeleteKeyW,
    RegSetValueExW,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetWindowThreadProcessId, PostMessageW, SW_SHOW, WM_CLOSE,
};

const UNINSTALL_KEY: &[u16] = &[
    'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16,
    'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16,
    's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16,
    'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16, 'C' as u16,
    'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16,
    'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16,
    'U' as u16, 'n' as u16, 'i' as u16, 'n' as u16, 's' as u16, 't' as u16, 'a' as u16,
    'l' as u16, 'l' as u16, '\\' as u16, 'L' as u16, 'a' as u16, 'n' as u16, 'g' as u16,
    'S' as u16, 'w' as u16, 'i' as u16, 't' as u16, 'c' as u16, 'h' as u16, 0,
];

pub fn get_default_install_dir() -> PathBuf {
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let mut p = PathBuf::from(pf);
        p.push("LangSwitch");
        p
    } else if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let mut p = PathBuf::from(local);
        p.push("Programs");
        p.push("LangSwitch");
        p
    } else {
        PathBuf::from("C:\\Program Files\\LangSwitch")
    }
}

pub fn get_start_menu_shortcut_path() -> Option<PathBuf> {
    if let Ok(app_data) = std::env::var("APPDATA") {
        let mut p = PathBuf::from(app_data);
        p.push("Microsoft");
        p.push("Windows");
        p.push("Start Menu");
        p.push("Programs");
        p.push("LangSwitch.lnk");
        Some(p)
    } else {
        None
    }
}

/// Checks if currently executing from the installed location.
pub fn is_running_from_install_dir() -> bool {
    if let Ok(current_exe) = std::env::current_exe() {
        let install_dir = get_default_install_dir();
        if let Some(parent) = current_exe.parent() {
            if parent == install_dir {
                return true;
            }
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let mut alt = PathBuf::from(local);
            alt.push("Programs");
            alt.push("LangSwitch");
            if let Some(parent) = current_exe.parent() {
                if parent == alt {
                    return true;
                }
            }
        }
    }
    false
}

/// Prompts the user on first run with a modern Light Blue <-> Light Purple gradient Installer Dialog.
pub fn check_and_prompt_first_run() {
    if is_running_from_install_dir() {
        return;
    }

    let config_path = crate::config::AppConfig::config_path();
    let already_prompted = if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            content.contains("\"first_run_prompted\": true")
        } else {
            false
        }
    } else {
        false
    };

    if already_prompted {
        return;
    }

    let answer = ui_theme::show_modern_dialog(
        &format!("⚡ התקנת LangSwitch v{}", CURRENT_VERSION),
        "המרת טקסט והחלפת שפה מהירה עבור Windows 10 / 11",
        "ברוכים הבאים ל-LangSwitch!\n\nהאם ברצונך להתקין את התוכנה במחשב?\n• העתקה לתיקיית התוכניות ויצירת קיצור דרך בתפריט ההתחלה\n• הפעלה אוטומטית שקטה ברקע עם עליית Windows\n• רישום מלא ברשימת האפליקציות המותקנות של Windows",
        "⚡ התקן עכשיו (מומלץ)",
        Some("הפעל ללא התקנה"),
        true,
    );

    if answer == DialogResult::Primary {
        install_app();
    } else {
        let cfg = crate::config::AppConfig::load();
        cfg.save();
        mark_first_run_prompted(&config_path);
    }
}

fn mark_first_run_prompted(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        if !content.contains("first_run_prompted") {
            let updated = content.replace("}", ",\n  \"first_run_prompted\": true\n}");
            let _ = fs::write(path, updated);
        }
    }
}

unsafe extern "system" {
    fn IsUserAnAdmin() -> i32;
}

pub fn is_running_as_admin() -> bool {
    unsafe { IsUserAnAdmin() != 0 }
}

/// Checks if another instance of LangSwitch is running, prompts the user with a modern dialog to close it.
pub fn check_and_close_running_instance() -> bool {
    let tray_class = to_wide("LangSwitchTrayClass");
    let hwnd = unsafe { FindWindowW(tray_class.as_ptr(), ptr::null()) };

    let my_pid = std::process::id();
    let mut target_pid: u32 = 0;

    if !hwnd.is_null() {
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut target_pid);
        }
    }

    if !hwnd.is_null() && target_pid != my_pid {
        let ans = ui_theme::show_modern_dialog(
            "LangSwitch פועלת ברקע",
            "נדרשת סגירה של המופע הפעיל לפני עדכון או התקנה",
            "נמצא כי LangSwitch כבר פועלת כעת במגש המערכת.\n\nהאם לסגור את הגרסה הפעילה ולהמשיך בהתקנת הגרסה החדשה כעת?",
            "סגור והמשך בהתקנה",
            Some("ביטול"),
            true,
        );

        if ans == DialogResult::Primary {
            unsafe {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
            if target_pid != 0 {
                let _ = Command::new("taskkill")
                    .args(["/PID", &target_pid.to_string(), "/F"])
                    .output();
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
            true
        } else {
            false
        }
    } else {
        true
    }
}

pub fn install_app() {
    if !check_and_close_running_instance() {
        return;
    }

    let current_exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => return,
    };

    let target_dir = get_default_install_dir();

    match try_install(&current_exe, &target_dir) {
        Ok(()) => return,
        Err(e) => {
            let is_permission_error = e.kind() == std::io::ErrorKind::PermissionDenied
                || e.raw_os_error() == Some(5);

            if is_permission_error && !is_running_as_admin() {
                let elevated = elevate_and_install(&current_exe);
                if elevated {
                    std::process::exit(0);
                }

                let ans = ui_theme::show_modern_dialog(
                    "התקנה בתיקיית המשתמש",
                    "לא אושרו הרשאות מנהל (Administrator)",
                    "האם ברצונך להתקין את LangSwitch בתיקיית המשתמש המקומית (AppData\\Programs) ללא צורך בהרשאות מנהל?",
                    "כן, התקן בתיקיית המשתמש",
                    Some("ביטול"),
                    true,
                );

                if ans == DialogResult::Primary {
                    if let Ok(local) = std::env::var("LOCALAPPDATA") {
                        let user_target_dir =
                            PathBuf::from(local).join("Programs").join("LangSwitch");
                        if try_install(&current_exe, &user_target_dir).is_ok() {
                            return;
                        }
                    }
                }
                return;
            }

            ui_theme::show_modern_dialog(
                "שגיאת התקנה",
                "לא ניתן להעתיק את קבצי התוכנה",
                &format!(
                    "אירעה שגיאה בעת העתקת קבצי ההתקנה:\n{}\n\nנתיב יעד: {}",
                    e,
                    target_dir.display()
                ),
                "סגור",
                None,
                true,
            );
        }
    }
}

pub fn try_install(source_exe: &Path, target_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target_dir)?;
    let target_exe = target_dir.join("langswitch.exe");

    if target_exe.exists() {
        let old_exe = target_dir.join("langswitch.exe.old");
        let _ = fs::remove_file(&old_exe);
        let _ = fs::rename(&target_exe, &old_exe);
    }

    fs::copy(source_exe, &target_exe)?;

    register_uninstall_entry(target_dir, &target_exe);
    create_start_menu_shortcut(&target_exe, target_dir);
    set_startup_enabled(true);

    ui_theme::show_modern_dialog(
        "ההתקנה הושלמה בהצלחה! 🎉",
        &format!("LangSwitch v{} מוכנה לעבודה", CURRENT_VERSION),
        "LangSwitch הותקנה בהצלחה במחשב!\n\n• נוסף קיצור דרך בתפריט ההתחלה (Start Menu)\n• התוכנה הוגדרה לפעול אוטומטית עם עליית Windows\n• התוכנה תופעל כעת ברקע באזור מגש המערכת (System Tray)",
        "מעולה, בוא נתחיל! 🚀",
        None,
        true,
    );

    let _ = Command::new(&target_exe).spawn();
    std::process::exit(0);
}

fn elevate_and_install(current_exe: &Path) -> bool {
    let exe_wide = to_wide(&current_exe.to_string_lossy());
    let op_wide = to_wide("runas");
    let param_wide = to_wide("--install-now");
    let res = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            op_wide.as_ptr(),
            exe_wide.as_ptr(),
            param_wide.as_ptr(),
            ptr::null(),
            SW_SHOW,
        )
    };
    (res as usize) > 32
}

pub fn do_install_copy(source_exe: &Path, target_dir: &Path) {
    if !check_and_close_running_instance() {
        return;
    }
    if let Err(e) = try_install(source_exe, target_dir) {
        ui_theme::show_modern_dialog(
            "שגיאת התקנה",
            "העתקת הקבצים נכשלה",
            &format!("שגיאה בהעתקת קבצי ההתקנה:\n{}", e),
            "סגור",
            None,
            true,
        );
    }
}

fn register_uninstall_entry(install_dir: &Path, exe_path: &Path) {
    unsafe {
        let mut key = std::ptr::null_mut();
        let res = RegCreateKeyW(HKEY_CURRENT_USER, UNINSTALL_KEY.as_ptr(), &mut key);

        if res != 0 {
            return;
        }

        let set_sz = |name: &str, val: &str| {
            let wname = to_wide(name);
            let wval = to_wide(val);
            RegSetValueExW(
                key,
                wname.as_ptr(),
                0,
                REG_SZ,
                wval.as_ptr() as *const u8,
                (wval.len() * 2) as u32,
            );
        };

        let set_dword = |name: &str, val: u32| {
            let wname = to_wide(name);
            RegSetValueExW(
                key,
                wname.as_ptr(),
                0,
                REG_DWORD,
                &val as *const u32 as *const u8,
                4,
            );
        };

        set_sz("DisplayName", "LangSwitch");
        set_sz("DisplayVersion", CURRENT_VERSION);
        set_sz("Publisher", "kosherplay-betatester");
        set_sz("InstallLocation", &install_dir.to_string_lossy());
        set_sz(
            "UninstallString",
            &format!("\"{}\" --uninstall", exe_path.to_string_lossy()),
        );
        set_sz("DisplayIcon", &format!("\"{}\",0", exe_path.to_string_lossy()));
        set_sz("HelpLink", "https://github.com/kosherplay-betatester/LangSwitch");
        set_sz("URLInfoAbout", "https://github.com/kosherplay-betatester/LangSwitch");
        set_dword("NoModify", 1);
        set_dword("NoRepair", 1);
        set_dword("EstimatedSize", 2800);

        RegCloseKey(key);
    }
}

fn create_start_menu_shortcut(exe_path: &Path, work_dir: &Path) {
    if let Some(shortcut_path) = get_start_menu_shortcut_path() {
        if let Some(parent) = shortcut_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let cmd = format!(
            "$s=(New-Object -COM WScript.Shell).CreateShortcut('{}'); \
             $s.TargetPath='{}'; \
             $s.WorkingDirectory='{}'; \
             $s.Description='LangSwitch - Fast Hebrew/English Text Converter'; \
             $s.Save()",
            shortcut_path.to_string_lossy(),
            exe_path.to_string_lossy(),
            work_dir.to_string_lossy()
        );

        let _ = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &cmd])
            .output();
    }
}

/// Uninstalls LangSwitch when invoked with `--uninstall`
pub fn handle_uninstall() {
    let answer = ui_theme::show_modern_dialog(
        "הסרת התקנת LangSwitch",
        "הסרת התוכנה וקיצורי הדרך מהמחשב",
        "האם אתה בטוח שברצונך להסיר את LangSwitch מהמחשב?\n\nפעולה זו תסיר את קבצי התוכנה, קיצור הדרך בתפריט ההתחלה והגדרות ההפעלה האוטומטית.",
        "כן, הסר את התוכנה",
        Some("ביטול"),
        true,
    );

    if answer != DialogResult::Primary {
        std::process::exit(0);
    }

    set_startup_enabled(false);

    unsafe {
        RegDeleteKeyW(HKEY_CURRENT_USER, UNINSTALL_KEY.as_ptr());
    }

    if let Some(shortcut) = get_start_menu_shortcut_path() {
        let _ = fs::remove_file(shortcut);
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(install_dir) = current_exe.parent() {
            let dir_str = install_dir.to_string_lossy();
            let cleanup_cmd = format!(
                "ping 127.0.0.1 -n 2 > nul & rmdir /s /q \"{}\"",
                dir_str
            );
            let _ = Command::new("cmd").args(["/C", &cleanup_cmd]).spawn();
        }
    }

    ui_theme::show_modern_dialog(
        "ההסרה הושלמה",
        "LangSwitch הוסרה מהמחשב",
        "LangSwitch הוסרה בהצלחה מהמחשב שלך.\nתודה שהשתמשת ב-LangSwitch!",
        "סגור",
        None,
        true,
    );

    std::process::exit(0);
}
