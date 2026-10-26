#![windows_subsystem = "windows"]

mod clipboard;
mod config;
mod converter;
mod installer;
mod settings_gui;
mod tray;
mod updater;

use config::AppConfig;
use std::ptr;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

unsafe extern "system" {
    fn CreateMutexW(
        lpmutexattributes: *const std::ffi::c_void,
        binitialowner: i32,
        lpname: *const u16,
    ) -> HANDLE;
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 1. Check for command-line commands
    if args.iter().any(|a| a == "--uninstall" || a == "-u") {
        installer::handle_uninstall();
        return;
    }

    if args.iter().any(|a| a == "--install-now") {
        let current_exe = std::env::current_exe().unwrap();
        let target_dir = installer::get_default_install_dir();
        let _ = std::fs::create_dir_all(&target_dir);
        installer::do_install_copy(&current_exe, &target_dir);
        return;
    }

    // 2. Check for first run installation prompt
    installer::check_and_prompt_first_run();

    // 3. Enforce single instance per user session
    unsafe {
        let mutex_name = to_wide("Local\\LangSwitch_SingleInstance_Mutex");
        let mutex = CreateMutexW(ptr::null(), 1, mutex_name.as_ptr());

        if mutex.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            let msg = to_wide("LangSwitch כבר פועלת ברקע!\nתוכל למצוא את האייקון שלה באזור ההודעות (System Tray).");
            let title = to_wide("LangSwitch");
            MessageBoxW(ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            return;
        }

        let config = AppConfig::load();
        tray::run_tray_app(config);
    }
}
