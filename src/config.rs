use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
#[cfg(windows)]
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};

#[cfg(not(windows))]
pub const MOD_ALT: u32 = 0x0001;
#[cfg(not(windows))]
pub const MOD_CONTROL: u32 = 0x0002;
#[cfg(not(windows))]
pub const MOD_SHIFT: u32 = 0x0004;
#[cfg(not(windows))]
pub const MOD_WIN: u32 = 0x0008;

// Standard and Extended function keys F1..F24
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
pub const VK_F13: u32 = 0x7C;
pub const VK_F14: u32 = 0x7D;
pub const VK_F15: u32 = 0x7E;
pub const VK_F16: u32 = 0x7F;
pub const VK_F17: u32 = 0x80;
pub const VK_F18: u32 = 0x81;
pub const VK_F19: u32 = 0x82;
pub const VK_F20: u32 = 0x83;
pub const VK_F21: u32 = 0x84;
pub const VK_F22: u32 = 0x85;
pub const VK_F23: u32 = 0x86;
pub const VK_F24: u32 = 0x87;

// Navigation & Editing keys
pub const VK_BACK: u32 = 0x08;
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_PAUSE: u32 = 0x13;
pub const VK_CAPITAL: u32 = 0x14; // Caps Lock
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_PRIOR: u32 = 0x21; // Page Up
pub const VK_NEXT: u32 = 0x22;  // Page Down
pub const VK_END: u32 = 0x23;
pub const VK_HOME: u32 = 0x24;
pub const VK_SNAPSHOT: u32 = 0x2C; // Print Screen
pub const VK_INSERT: u32 = 0x2D;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_SCROLL: u32 = 0x91;

// Numpad keys
pub const VK_NUMPAD0: u32 = 0x60;
pub const VK_NUMPAD1: u32 = 0x61;
pub const VK_NUMPAD2: u32 = 0x62;
pub const VK_NUMPAD3: u32 = 0x63;
pub const VK_NUMPAD4: u32 = 0x64;
pub const VK_NUMPAD5: u32 = 0x65;
pub const VK_NUMPAD6: u32 = 0x66;
pub const VK_NUMPAD7: u32 = 0x67;
pub const VK_NUMPAD8: u32 = 0x68;
pub const VK_NUMPAD9: u32 = 0x69;
pub const VK_MULTIPLY: u32 = 0x6A;
pub const VK_ADD: u32 = 0x6B;
pub const VK_SUBTRACT: u32 = 0x6D;
pub const VK_DECIMAL: u32 = 0x6E;
pub const VK_DIVIDE: u32 = 0x6F;

// OEM Punctuation keys
pub const VK_OEM_1: u32 = 0xBA; // ; :
pub const VK_OEM_PLUS: u32 = 0xBB; // = +
pub const VK_OEM_COMMA: u32 = 0xBC; // , <
pub const VK_OEM_MINUS: u32 = 0xBD; // - _
pub const VK_OEM_PERIOD: u32 = 0xBE; // . >
pub const VK_OEM_2: u32 = 0xBF; // / ?
pub const VK_OEM_3: u32 = 0xC0; // ` ~
pub const VK_OEM_4: u32 = 0xDB; // [ {
pub const VK_OEM_5: u32 = 0xDC; // \ |
pub const VK_OEM_6: u32 = 0xDD; // ] }
pub const VK_OEM_7: u32 = 0xDE; // ' "

/// Comprehensive list of keyboard keys available for hotkeys
pub const ALL_KEYBOARD_KEYS: &[(&str, u32)] = &[
    // Recommended
    ("F10 (מומלץ / Recommended)", VK_F10),
    // Standard Function keys
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
    // Extended Function keys
    ("F13", VK_F13),
    ("F14", VK_F14),
    ("F15", VK_F15),
    ("F16", VK_F16),
    ("F17", VK_F17),
    ("F18", VK_F18),
    ("F19", VK_F19),
    ("F20", VK_F20),
    ("F21", VK_F21),
    ("F22", VK_F22),
    ("F23", VK_F23),
    ("F24", VK_F24),
    // Letters A-Z
    ("A", 0x41),
    ("B", 0x42),
    ("C", 0x43),
    ("D", 0x44),
    ("E", 0x45),
    ("F", 0x46),
    ("G", 0x47),
    ("H", 0x48),
    ("I", 0x49),
    ("J", 0x4A),
    ("K", 0x4B),
    ("L", 0x4C),
    ("M", 0x4D),
    ("N", 0x4E),
    ("O", 0x4F),
    ("P", 0x50),
    ("Q", 0x51),
    ("R", 0x52),
    ("S", 0x53),
    ("T", 0x54),
    ("U", 0x55),
    ("V", 0x56),
    ("W", 0x57),
    ("X", 0x58),
    ("Y", 0x59),
    ("Z", 0x5A),
    // Number row
    ("0", 0x30),
    ("1", 0x31),
    ("2", 0x32),
    ("3", 0x33),
    ("4", 0x34),
    ("5", 0x35),
    ("6", 0x36),
    ("7", 0x37),
    ("8", 0x38),
    ("9", 0x39),
    // Navigation / Editing
    ("Space", VK_SPACE),
    ("Tab", VK_TAB),
    ("Enter", VK_RETURN),
    ("Esc", VK_ESCAPE),
    ("Backspace", VK_BACK),
    ("Insert", VK_INSERT),
    ("Delete", VK_DELETE),
    ("Home", VK_HOME),
    ("End", VK_END),
    ("Page Up", VK_PRIOR),
    ("Page Down", VK_NEXT),
    ("Pause / Break", VK_PAUSE),
    ("Scroll Lock", VK_SCROLL),
    ("Caps Lock", VK_CAPITAL),
    ("Print Screen", VK_SNAPSHOT),
    // Punctuation / OEM
    ("` / ~", VK_OEM_3),
    ("- / _", VK_OEM_MINUS),
    ("= / +", VK_OEM_PLUS),
    ("[ / {", VK_OEM_4),
    ("] / }", VK_OEM_6),
    ("\\ / |", VK_OEM_5),
    ("; / :", VK_OEM_1),
    ("' / \"", VK_OEM_7),
    (", / <", VK_OEM_COMMA),
    (". / >", VK_OEM_PERIOD),
    ("/ / ?", VK_OEM_2),
    // Numpad
    ("Num 0", VK_NUMPAD0),
    ("Num 1", VK_NUMPAD1),
    ("Num 2", VK_NUMPAD2),
    ("Num 3", VK_NUMPAD3),
    ("Num 4", VK_NUMPAD4),
    ("Num 5", VK_NUMPAD5),
    ("Num 6", VK_NUMPAD6),
    ("Num 7", VK_NUMPAD7),
    ("Num 8", VK_NUMPAD8),
    ("Num 9", VK_NUMPAD9),
    ("Num +", VK_ADD),
    ("Num -", VK_SUBTRACT),
    ("Num *", VK_MULTIPLY),
    ("Num /", VK_DIVIDE),
    ("Num .", VK_DECIMAL),
];

/// Checks if a given modifier combination + key code conflicts with Windows system shortcuts.
pub fn is_windows_conflict(modifiers: u32, vk: u32) -> bool {
    let has_ctrl = modifiers & MOD_CONTROL != 0;
    let has_alt = modifiers & MOD_ALT != 0;
    let has_shift = modifiers & MOD_SHIFT != 0;
    let has_win = modifiers & MOD_WIN != 0;

    // 1. Windows Key Combinations
    if has_win {
        // Win + Shift combinations
        if has_shift && !has_ctrl && !has_alt {
            match vk {
                0x53 => return true, // 'S' -> Windows Snipping Tool (User's explicit example!)
                0x43 => return true, // 'C' -> Copilot / Teams
                0x4D => return true, // 'M' -> Restore all minimized windows
                0x30..=0x39 => return true, // 0..9 -> New instance of taskbar app
                _ => {}
            }
        }

        // Win + Ctrl combinations
        if has_ctrl && !has_shift && !has_alt {
            match vk {
                VK_RETURN => return true, // Narrator
                0x44 => return true,      // 'D' -> Add virtual desktop
                VK_F4 => return true,     // Close virtual desktop
                _ => {}
            }
        }

        // Win + Ctrl + Shift + B (restart GPU driver)
        if has_ctrl && has_shift && !has_alt && vk == 0x42 {
            return true;
        }

        // Win alone (no other modifiers)
        if !has_ctrl && !has_alt && !has_shift {
            match vk {
                0x41 => return true, // 'A' -> Action Center / Quick Settings
                0x42 => return true, // 'B' -> System Tray focus
                0x43 => return true, // 'C' -> Copilot / Cortana
                0x44 => return true, // 'D' -> Show Desktop
                0x45 => return true, // 'E' -> File Explorer
                0x47 => return true, // 'G' -> Xbox Game Bar
                0x48 => return true, // 'H' -> Voice Typing / Dictation
                0x49 => return true, // 'I' -> Windows Settings
                0x4B => return true, // 'K' -> Cast / Connect
                0x4C => return true, // 'L' -> Lock Workstation
                0x4D => return true, // 'M' -> Minimize All Windows
                0x4E => return true, // 'N' -> Notifications / Calendar
                0x50 => return true, // 'P' -> Project / Presentation
                0x52 => return true, // 'R' -> Run dialog
                0x53 => return true, // 'S' -> Windows Search
                0x54 => return true, // 'T' -> Cycle taskbar apps
                0x55 => return true, // 'U' -> Ease of Access / Accessibility
                0x56 => return true, // 'V' -> Clipboard History
                0x57 => return true, // 'W' -> Widgets
                0x58 => return true, // 'X' -> Quick Link / Start context menu
                0x5A => return true, // 'Z' -> Snap Layouts
                0x30..=0x39 => return true, // 0..9 -> Pinned taskbar apps
                VK_TAB => return true,      // Task View
                VK_SPACE => return true,    // Switch Input Language
                VK_RETURN => return true,   // Narrator
                VK_PAUSE => return true,    // System Properties
                VK_OEM_PERIOD | VK_OEM_1 => return true, // Emoji Picker (. or ;)
                _ => {}
            }
        }
    }

    // 2. Alt Combinations
    if has_alt && !has_win {
        // Alt alone
        if !has_ctrl && !has_shift {
            match vk {
                VK_TAB => return true,    // Task Switcher (Alt + Tab)
                VK_F4 => return true,     // Close active window (Alt + F4)
                VK_SPACE => return true,  // Window System Menu (Alt + Space)
                VK_ESCAPE => return true, // Direct Window Switch (Alt + Esc)
                VK_RETURN => return true, // Properties / Fullscreen (Alt + Enter)
                _ => {}
            }
        }
    }

    // 3. Ctrl Combinations
    if has_ctrl && !has_win && !has_alt {
        // Ctrl + Shift
        if has_shift {
            match vk {
                VK_ESCAPE => return true, // Task Manager (Ctrl + Shift + Esc)
                0x4E => return true,      // 'N' -> New incognito window / new folder
                0x54 => return true,      // 'T' -> Reopen closed tab
                _ => {}
            }
        }
        // Ctrl alone (Avoid hijacking standard text editor shortcuts across all apps)
        if !has_shift {
            match vk {
                VK_ESCAPE => return true, // Start menu (Ctrl + Esc)
                0x41 => return true,      // 'A' (Select All)
                0x43 => return true,      // 'C' (Copy)
                0x56 => return true,      // 'V' (Paste)
                0x58 => return true,      // 'X' (Cut)
                0x5A => return true,      // 'Z' (Undo)
                0x59 => return true,      // 'Y' (Redo)
                0x53 => return true,      // 'S' (Save)
                0x4F => return true,      // 'O' (Open)
                0x4E => return true,      // 'N' (New)
                0x50 => return true,      // 'P' (Print)
                0x46 => return true,      // 'F' (Find)
                0x57 => return true,      // 'W' (Close tab)
                _ => {}
            }
        }
    }

    false
}

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
            0x70..=0x87 => format!("F{}", self.vk - 0x70 + 1),
            0x41..=0x5A => ((self.vk as u8) as char).to_string(),
            0x30..=0x39 => ((self.vk as u8) as char).to_string(),
            0x60..=0x69 => format!("Num {}", self.vk - 0x60),
            0x6A => "Num *".to_string(),
            0x6B => "Num +".to_string(),
            0x6D => "Num -".to_string(),
            0x6E => "Num .".to_string(),
            0x6F => "Num /".to_string(),
            VK_SPACE => "Space".to_string(),
            VK_TAB => "Tab".to_string(),
            VK_RETURN => "Enter".to_string(),
            VK_ESCAPE => "Esc".to_string(),
            VK_BACK => "Backspace".to_string(),
            VK_INSERT => "Insert".to_string(),
            VK_DELETE => "Delete".to_string(),
            VK_HOME => "Home".to_string(),
            VK_END => "End".to_string(),
            VK_PRIOR => "Page Up".to_string(),
            VK_NEXT => "Page Down".to_string(),
            VK_PAUSE => "Pause".to_string(),
            VK_CAPITAL => "Caps Lock".to_string(),
            VK_SCROLL => "Scroll Lock".to_string(),
            VK_SNAPSHOT => "Print Screen".to_string(),
            VK_OEM_1 => ";".to_string(),
            VK_OEM_PLUS => "=".to_string(),
            VK_OEM_COMMA => ",".to_string(),
            VK_OEM_MINUS => "-".to_string(),
            VK_OEM_PERIOD => ".".to_string(),
            VK_OEM_2 => "/".to_string(),
            VK_OEM_3 => "`".to_string(),
            VK_OEM_4 => "[".to_string(),
            VK_OEM_5 => "\\".to_string(),
            VK_OEM_6 => "]".to_string(),
            VK_OEM_7 => "'".to_string(),
            _ => format!("Key(0x{:X})", self.vk),
        };

        parts.push(&key_name);
        parts.join(" + ")
    }
}

fn default_ui_language() -> String {
    "he".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkey_primary: HotkeyConfig,
    pub hotkey_secondary: HotkeyConfig,
    pub auto_switch_layout: bool,
    pub auto_select_line: bool,
    pub start_with_windows: bool,
    pub restore_clipboard: bool,
    #[serde(default = "default_ui_language")]
    pub ui_language: String, // "he" or "en"
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
            ui_language: "he".to_string(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        #[cfg(windows)]
        {
            if let Ok(app_data) = std::env::var("APPDATA") {
                let mut p = PathBuf::from(app_data);
                p.push("LangSwitch");
                p.push("config.json");
                return p;
            }
        }
        #[cfg(not(windows))]
        {
            if let Ok(home) = std::env::var("HOME") {
                let mut p = PathBuf::from(home);
                p.push(".config");
                p.push("langswitch");
                p.push("config.json");
                return p;
            }
        }
        PathBuf::from("config.json")
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

#[cfg(windows)]
const RUN_KEY: &[u16] = &[
    'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16,
    'e' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16,
    's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16,
    'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16, 'C' as u16,
    'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16,
    'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16,
    'R' as u16, 'u' as u16, 'n' as u16, 0,
];

#[cfg(windows)]
const APP_NAME: &[u16] = &[
    'L' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'S' as u16, 'w' as u16, 'i' as u16,
    't' as u16, 'c' as u16, 'h' as u16, 0,
];

#[cfg(windows)]
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

#[cfg(not(windows))]
pub fn is_startup_enabled() -> bool {
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config/autostart/langswitch.desktop");
        p.exists()
    } else {
        false
    }
}

#[cfg(windows)]
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

#[cfg(not(windows))]
pub fn set_startup_enabled(enable: bool) -> bool {
    if let Ok(home) = std::env::var("HOME") {
        let autostart_dir = PathBuf::from(home).join(".config/autostart");
        let desktop_file = autostart_dir.join("langswitch.desktop");
        if enable {
            let _ = fs::create_dir_all(&autostart_dir);
            let content = "[Desktop Entry]\nType=Application\nName=LangSwitch\nExec=langswitch --convert\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n";
            fs::write(desktop_file, content).is_ok()
        } else {
            let _ = fs::remove_file(desktop_file);
            true
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snipping_tool_conflict() {
        // Shift + Win + S must be a conflict
        assert!(is_windows_conflict(MOD_SHIFT | MOD_WIN, 0x53));
    }

    #[test]
    fn test_win_search_conflict() {
        // Win + S must be a conflict
        assert!(is_windows_conflict(MOD_WIN, 0x53));
    }

    #[test]
    fn test_safe_hotkeys() {
        // Shift + Win + F10 is safe!
        assert!(!is_windows_conflict(MOD_SHIFT | MOD_WIN, VK_F10));
        // F10 alone is safe!
        assert!(!is_windows_conflict(0, VK_F10));
        // Shift + F10 is safe!
        assert!(!is_windows_conflict(MOD_SHIFT, VK_F10));
        // Ctrl + Shift + F10 is safe!
        assert!(!is_windows_conflict(MOD_CONTROL | MOD_SHIFT, VK_F10));
    }

    #[test]
    fn test_all_keys_available_without_modifiers() {
        // Key 'A' without modifier is NOT a Windows shortcut conflict
        assert!(!is_windows_conflict(0, 0x41));
    }

    #[test]
    fn test_alt_f4_conflict() {
        assert!(is_windows_conflict(MOD_ALT, VK_F4));
    }
}
