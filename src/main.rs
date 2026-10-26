#![windows_subsystem = "windows"]

mod clipboard;
mod config;
mod converter;
mod settings_gui;
mod tray;
mod updater;

use config::AppConfig;
use std::ptr;
use windows_sys::Win32::Foundation::{GetLastError, HANDLE, ERROR_ALREADY_EXISTS};
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
    unsafe {
        // Enforce single instance per user session
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
