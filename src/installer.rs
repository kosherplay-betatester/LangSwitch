use crate::config::set_startup_enabled;
use crate::updater::CURRENT_VERSION;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_DWORD, REG_SZ, RegCloseKey, RegCreateKeyW,
    RegDeleteKeyW, RegSetValueExW,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO, SW_SHOW,
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

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn get_default_install_dir() -> PathBuf {
    // Check if ProgramFiles is available, otherwise LocalAppData\Programs
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
        // Also check LocalAppData\Programs\LangSwitch in case installed per-user
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

/// Prompts the user on first run to install the application.
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

    // Ask user if they wish to install to Program Files
    let msg = to_wide(
        "ברוכים הבאים ל-LangSwitch! ⚡\n\n\
        האם ברצונך להתקין את LangSwitch במחשב?\n\n\
        ההתקנה תעתיק את התוכנה לתיקיית התוכניות, תיצור קיצור דרך בתפריט התחל,\n\
        ותרשום את האפליקציה בלוח הבקרה ורשימת היישומים להסרה של Windows.",
    );
    let title = to_wide("התקנת LangSwitch");

    let answer = unsafe {
        MessageBoxW(
            ptr::null_mut(),
            msg.as_ptr(),
            title.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };

    if answer == IDYES {
        install_app();
    } else {
        // Record that user was prompted so we don't nag on portable usage
        let cfg = crate::config::AppConfig::load();
        cfg.save();
        mark_first_run_prompted(&config_path);
    }
}

fn mark_first_run_prompted(path: &Path) {
    if let Ok(content) = fs::read_to_string(path) {
        if !content.contains("first_run_prompted") {
            let updated = content.replace(
                "}",
                ",\n  \"first_run_prompted\": true\n}",
            );
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

pub fn install_app() {
    let current_exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => return,
    };

    let target_dir = get_default_install_dir();

    // 1. Try to install directly first without requesting admin permissions
    match try_install(&current_exe, &target_dir) {
        Ok(()) => {
            // Succeeded directly! No admin elevation was needed.
            return;
        }
        Err(e) => {
            // 2. ONLY if really needed (PermissionDenied / OS error 5), request admin elevation
            let is_permission_error = e.kind() == std::io::ErrorKind::PermissionDenied
                || e.raw_os_error() == Some(5);

            if is_permission_error && !is_running_as_admin() {
                let elevated = elevate_and_install(&current_exe);
                if elevated {
                    std::process::exit(0);
                }

                // If user denied UAC elevation, offer fallback to LocalAppData (no admin needed)
                let msg = to_wide(
                    "לא אושרו הרשאות מנהל (Administrator) להתקנה ב-Program Files.\n\n\
                    האם ברצונך להתקין את האפליקציה בתיקיית המשתמש המקומית (AppData\\Programs)\n\
                    ללא צורך בהרשאות מנהל?",
                );
                let title = to_wide("הרשאות מנהל נדחו");
                let ans = unsafe {
                    MessageBoxW(
                        ptr::null_mut(),
                        msg.as_ptr(),
                        title.as_ptr(),
                        MB_YESNO | MB_ICONQUESTION,
                    )
                };

                if ans == IDYES {
                    if let Ok(local) = std::env::var("LOCALAPPDATA") {
                        let user_target_dir = PathBuf::from(local).join("Programs").join("LangSwitch");
                        if try_install(&current_exe, &user_target_dir).is_ok() {
                            return;
                        }
                    }
                }
                return;
            }

            // Other error
            let msg = to_wide(&format!("שגיאה בהעתקת קבצי ההתקנה:\n{}\n\nנתיב יעד: {}", e, target_dir.display()));
            let title = to_wide("שגיאת התקנה");
            unsafe {
                MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR);
            }
        }
    }
}

pub fn try_install(source_exe: &Path, target_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target_dir)?;
    let target_exe = target_dir.join("langswitch.exe");

    // If existing executable is running or locked, try renaming to .old first
    if target_exe.exists() {
        let old_exe = target_dir.join("langswitch.exe.old");
        let _ = fs::remove_file(&old_exe);
        let _ = fs::rename(&target_exe, &old_exe);
    }

    // Copy executable
    fs::copy(source_exe, &target_exe)?;

    // Register in Windows "Programs and Features" / "Installed Apps"
    register_uninstall_entry(target_dir, &target_exe);

    // Create Start Menu shortcut
    create_start_menu_shortcut(&target_exe, target_dir);

    // Automatically enable Run on Windows startup
    set_startup_enabled(true);

    let msg = to_wide(
        "LangSwitch הותקנה בהצלחה במחשב! ⚡\n\n\
        - נוסף קיצור דרך בתפריט התחל.\n\
        - התוכנה מופיעה כעת ברשימת האפליקציות של Windows (לוח הבקרה / הגדרות).\n\
        - התוכנה תופעל כעת מהתיקייה המותקנת.",
    );
    let title = to_wide("ההתקנה הושלמה בהצלחה");
    unsafe {
        MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
    }

    // Launch installed app and exit current instance
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
    // ShellExecute returns > 32 on success (UAC approved)
    (res as usize) > 32
}

pub fn do_install_copy(source_exe: &Path, target_dir: &Path) {
    if let Err(e) = try_install(source_exe, target_dir) {
        let msg = to_wide(&format!("שגיאה בהעתקת קבצי ההתקנה:\n{}", e));
        let title = to_wide("שגיאת התקנה");
        unsafe {
            MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | windows_sys::Win32::UI::WindowsAndMessaging::MB_ICONERROR);
        }
    }
}

fn register_uninstall_entry(install_dir: &Path, exe_path: &Path) {
    unsafe {
        let mut key = std::ptr::null_mut();
        let res = RegCreateKeyW(
            HKEY_CURRENT_USER,
            UNINSTALL_KEY.as_ptr(),
            &mut key,
        );

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
        set_dword("EstimatedSize", 2600); // ~2.6MB

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
    let msg = to_wide("האם אתה בטוח שברצונך להסיר את LangSwitch מהמחשב?");
    let title = to_wide("הסרת LangSwitch");

    let answer = unsafe {
        MessageBoxW(
            ptr::null_mut(),
            msg.as_ptr(),
            title.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };

    if answer != IDYES {
        std::process::exit(0);
    }

    // 1. Remove autostart registry entry
    set_startup_enabled(false);

    // 2. Remove Uninstall registry entry
    unsafe {
        RegDeleteKeyW(HKEY_CURRENT_USER, UNINSTALL_KEY.as_ptr());
    }

    // 3. Remove Start Menu shortcut
    if let Some(shortcut) = get_start_menu_shortcut_path() {
        let _ = fs::remove_file(shortcut);
    }

    // 4. Remove installation folder via detached cmd script after exe exits
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(install_dir) = current_exe.parent() {
            let dir_str = install_dir.to_string_lossy();
            let cleanup_cmd = format!(
                "ping 127.0.0.1 -n 2 > nul & rmdir /s /q \"{}\"",
                dir_str
            );
            let _ = Command::new("cmd")
                .args(["/C", &cleanup_cmd])
                .spawn();
        }
    }

    let done_msg = to_wide("LangSwitch הוסרה בהצלחה מהמחשב.");
    let done_title = to_wide("ההסרה הושלמה");
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            done_msg.as_ptr(),
            done_title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }

    std::process::exit(0);
}
