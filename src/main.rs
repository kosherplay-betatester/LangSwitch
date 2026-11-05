#![windows_subsystem = "windows"]

mod clipboard;
mod config;
mod converter;
mod installer;
mod settings_gui;
mod tray;
mod ui_theme;
mod updater;
mod welcome_gui;

use config::AppConfig;
use std::ptr;
use ui_theme::to_wide;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};

unsafe extern "system" {
    fn CreateMutexW(
        lpmutexattributes: *const std::ffi::c_void,
        binitialowner: i32,
        lpname: *const u16,
    ) -> HANDLE;
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

    if args.iter().any(|a| a == "--settings") {
        let config = AppConfig::load();
        if let Some(handle) = settings_gui::show_settings_window(config, Box::new(|_| {})) {
            let _ = handle.join();
        }
        return;
    }

    // 2. Check for first run installation prompt
    installer::check_and_prompt_first_run();

    // 3. Enforce single instance per user session
    unsafe {
        let mutex_name = to_wide("Local\\LangSwitch_SingleInstance_Mutex");
        let mutex = CreateMutexW(ptr::null(), 1, mutex_name.as_ptr());

        if mutex.is_null() || GetLastError() == ERROR_ALREADY_EXISTS {
            ui_theme::show_modern_dialog(
                "⚡ LangSwitch כבר פועלת!",
                "התוכנה פועלת ברקע באזור מגש המערכת",
                "LangSwitch כבר פועלת ברקע במחשב שלך!\n\nתוכל למצוא את האייקון שלה באזור ההודעות (System Tray ליד השעון) וללחוץ עליו לפתיחת חלון ההגדרות.",
                "הבנתי, תודה!",
                None,
                true,
            );
            return;
        }

        let mut config = AppConfig::load();

        // First run after install or update only: show friendly welcome window
        if config.last_welcome_version.as_deref() != Some(updater::CURRENT_VERSION) {
            welcome_gui::show_welcome_window(&config);
            config.last_welcome_version = Some(updater::CURRENT_VERSION.to_string());
            config.save();
        }

        tray::run_tray_app(config);
    }
}
