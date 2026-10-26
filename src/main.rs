#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod clipboard;
mod config;
mod converter;
#[cfg(windows)]
mod installer;
#[cfg(windows)]
mod settings_gui;
#[cfg(windows)]
mod welcome_gui;
#[cfg(windows)]
mod tray;
mod updater;

#[cfg(unix)]
pub mod linux;

#[cfg(windows)]
use config::AppConfig;
#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

#[cfg(windows)]
unsafe extern "system" {
    fn CreateMutexW(
        lpmutexattributes: *const std::ffi::c_void,
        binitialowner: i32,
        lpname: *const u16,
    ) -> HANDLE;
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    #[cfg(unix)]
    {
        use std::io::IsTerminal;

        if args.iter().any(|a| a == "--convert" || a == "-c") {
            linux::perform_conversion(false);
            return;
        }

        if args.iter().any(|a| a == "--convert-and-switch" || a == "-s") {
            linux::perform_conversion(true);
            return;
        }

        if args.iter().any(|a| a == "--status") {
            linux::print_status();
            return;
        }

        if args.iter().any(|a| a == "--install" || a == "--install-desktop" || a == "-i") {
            if let Err(e) = linux::install_app_linux() {
                eprintln!("Error installing LangSwitch: {}", e);
            }
            return;
        }

        if args.iter().any(|a| a == "--help" || a == "-h") {
            println!("⚡ LangSwitch v{}", updater::CURRENT_VERSION);
            println!("Hebrew/English Text Layout Switcher for Linux (X11 & Wayland)");
            println!("\nCommands:");
            println!("  --convert, -c             Convert selected text or current line");
            println!("  --convert-and-switch, -s  Convert text and toggle keyboard layout");
            println!("  --status                  Diagnose X11/Wayland display server and tools");
            println!("  --install-desktop         Install .desktop entry in app menu");
            println!("  --help, -h                Show this help message");
            return;
        }

        // If invoked from a terminal without arguments, show diagnostics
        if std::io::stdout().is_terminal() {
            println!("⚡ LangSwitch v{}", updater::CURRENT_VERSION);
            linux::print_status();
        } else {
            // Invoked from desktop shortcut or hotkey
            linux::perform_conversion(false);
        }
        return;
    }

    #[cfg(windows)]
    {
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
}
