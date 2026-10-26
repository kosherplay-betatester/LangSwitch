use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};

pub const VK_F1: u32 = 0x70;
pub const VK_F2: u32 = 0x71;
pub const VK_F3: u32 = 0x72;
pub const VK_F4: u32 = 0x73;
pub const VK_F5: u32 = 0x74;
pub const VK_F6: u32 = 0x75;
pub const VK_F7: u32 = 0x76;
pub const VK_F8: u32 = 0x77;
pub const VK_F9: u32 = 0x78;
pub const VK_F10: u32 = 0x79;
pub const VK_F11: u32 = 0x7A;
pub const VK_F12: u32 = 0x7B;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub modifiers: u32,
    pub vk: u32,
}

impl HotkeyConfig {
    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers & MOD_CONTROL != 0 {
            parts.push("Ctrl");
        }
        if self.modifiers & MOD_ALT != 0 {
            parts.push("Alt");
        }
        if self.modifiers & MOD_SHIFT != 0 {
            parts.push("Shift");
        }
        if self.modifiers & MOD_WIN != 0 {
            parts.push("Win");
        }

        let key_name = match self.vk {
            0x70..=0x7B => format!("F{}", self.vk - 0x70 + 1),
            0x41..=0x5A => ((self.vk as u8) as char).to_string(),
            0x30..=0x39 => ((self.vk as u8) as char).to_string(),
            0x20 => "Space".to_string(),
            0x13 => "Pause".to_string(),
            0x2D => "Insert".to_string(),
            _ => format!("Key(0x{:X})", self.vk),
        };

        parts.push(&key_name);
        parts.join(" + ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey_primary: HotkeyConfig,
    pub hotkey_secondary: HotkeyConfig,
    pub auto_switch_layout: bool,
    pub auto_select_line: bool,
    pub start_with_windows: bool,
    pub restore_clipboard: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // Default: F10
            hotkey_primary: HotkeyConfig {
                modifiers: 0,
                vk: VK_F10,
            },
            // Default: Shift + F10
            hotkey_secondary: HotkeyConfig {
                modifiers: MOD_SHIFT,
                vk: VK_F10,
            },
            auto_switch_layout: true,
            auto_select_line: true,
            start_with_windows: false,
            restore_clipboard: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(app_data) = std::env::var("APPDATA") {
            let mut p = PathBuf::from(app_data);
            p.push("LangSwitch");
            p.push("config.json");
            p
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut config) = serde_json::from_str::<AppConfig>(&content) {
                    config.start_with_windows = is_startup_enabled();
                    return config;
                }
            }
        }
        let mut def = Self::default();
        def.start_with_windows = is_startup_enabled();
        def.save();
        def
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
        let _ = set_startup_enabled(self.start_with_windows);
    }
}

const RUN_KEY: &[u16] = &[
    'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16,
    'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16,
    's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16,
    'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16, 'C' as u16,
    'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16,
    'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16,
    'R' as u16, 'u' as u16, 'n' as u16, 0,
];

const APP_NAME: &[u16] = &[
    'L' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'S' as u16, 'w' as u16, 'i' as u16,
    't' as u16, 'c' as u16, 'h' as u16, 0,
];

pub fn is_startup_enabled() -> bool {
    unsafe {
        let mut key = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut key,
        ) != 0
        {
            return false;
        }

        let mut data_type = 0;
        let mut data_size = 0;
        let res = RegQueryValueExW(
            key,
            APP_NAME.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            std::ptr::null_mut(),
            &mut data_size,
        );

        RegCloseKey(key);
        res == 0
    }
}

pub fn set_startup_enabled(enable: bool) -> bool {
    unsafe {
        let mut key = std::ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY.as_ptr(),
            0,
            KEY_SET_VALUE,
            &mut key,
        ) != 0
        {
            return false;
        }

        let success = if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let path_str = format!("\"{}\"", exe_path.to_string_lossy());
                let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                RegSetValueExW(
                    key,
                    APP_NAME.as_ptr(),
                    0,
                    REG_SZ,
                    wide.as_ptr() as *const u8,
                    (wide.len() * 2) as u32,
                ) == 0
            } else {
                false
            }
        } else {
            RegDeleteValueW(key, APP_NAME.as_ptr()) == 0
        };

        RegCloseKey(key);
        success
    }
}
