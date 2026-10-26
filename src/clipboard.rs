use std::ptr;
use std::thread;
use std::time::Duration;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    ActivateKeyboardLayout, GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_C, VK_CONTROL, VK_END, VK_HOME,
    VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
    VK_RWIN, VK_SHIFT, VK_V,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, PostMessageW};

pub const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;

const CF_UNICODETEXT: u32 = 13;

pub fn clear_clipboard() {
    unsafe {
        for _ in 0..5 {
            if OpenClipboard(ptr::null_mut()) != 0 {
                EmptyClipboard();
                CloseClipboard();
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
}



pub fn get_clipboard_text() -> Option<String> {
    unsafe {
        // Retry opening clipboard in case another process briefly locked it
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        if !opened {
            return None;
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return None;
        }

        let ptr = GlobalLock(handle) as *const u16;
        if ptr.is_null() {
            CloseClipboard();
            return None;
        }

        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(ptr, len);
        let text = String::from_utf16_lossy(slice);

        GlobalUnlock(handle);
        CloseClipboard();
        Some(text)
    }
}

pub fn set_clipboard_text(text: &str) -> bool {
    unsafe {
        let mut opened = false;
        for _ in 0..5 {
            if OpenClipboard(ptr::null_mut()) != 0 {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        if !opened {
            return false;
        }

        EmptyClipboard();

        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * std::mem::size_of::<u16>();

        let handle = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if handle.is_null() {
            CloseClipboard();
            return false;
        }

        let dest = GlobalLock(handle) as *mut u16;
        if dest.is_null() {
            CloseClipboard();
            return false;
        }

        ptr::copy_nonoverlapping(wide.as_ptr(), dest, wide.len());
        GlobalUnlock(handle);

        let success = !SetClipboardData(CF_UNICODETEXT, handle).is_null();
        CloseClipboard();
        success
    }
}

fn make_key_input(vk: u16, key_up: bool, extended: bool) -> INPUT {
    let mut flags = 0;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }
    if extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

/// Releases physical modifiers (Shift, Ctrl, Alt, Win) if currently held by user
pub fn release_physical_modifiers() {
    unsafe {
        let modifiers = [
            VK_SHIFT,
            VK_LSHIFT,
            VK_RSHIFT,
            VK_CONTROL,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_MENU,
            VK_LMENU,
            VK_RMENU,
            VK_LWIN,
            VK_RWIN,
        ];

        let mut inputs = Vec::new();
        for &vk in &modifiers {
            if (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 {
                inputs.push(make_key_input(vk, true, false));
            }
        }

        if !inputs.is_empty() {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
            thread::sleep(Duration::from_millis(3));
        }
    }
}

/// Send Ctrl + C cleanly
pub fn send_ctrl_c() {
    release_physical_modifiers();
    let inputs = [
        make_key_input(VK_CONTROL, false, false),
        make_key_input(VK_C, false, false),
        make_key_input(VK_C, true, false),
        make_key_input(VK_CONTROL, true, false),
    ];
    send_inputs(&inputs);
}

/// Send Ctrl + V cleanly
pub fn send_ctrl_v() {
    release_physical_modifiers();
    let inputs = [
        make_key_input(VK_CONTROL, false, false),
        make_key_input(VK_V, false, false),
        make_key_input(VK_V, true, false),
        make_key_input(VK_CONTROL, true, false),
    ];
    send_inputs(&inputs);
}

/// Select the entire current line by sending Home then Shift + End
#[allow(dead_code)]
pub fn select_current_line() {
    release_physical_modifiers();

    let inputs = [
        make_key_input(VK_HOME, false, true),
        make_key_input(VK_HOME, true, true),
        make_key_input(VK_SHIFT, false, false),
        make_key_input(VK_END, false, true),
        make_key_input(VK_END, true, true),
        make_key_input(VK_SHIFT, true, false),
    ];
    send_inputs(&inputs);
}

/// Selects the current line and sends Ctrl+C in a single atomic input sequence without delay
pub fn select_current_line_and_copy() {
    release_physical_modifiers();

    let inputs = [
        // Home key
        make_key_input(VK_HOME, false, true),
        make_key_input(VK_HOME, true, true),
        // Shift + End
        make_key_input(VK_SHIFT, false, false),
        make_key_input(VK_END, false, true),
        make_key_input(VK_END, true, true),
        make_key_input(VK_SHIFT, true, false),
        // Ctrl + C
        make_key_input(VK_CONTROL, false, false),
        make_key_input(VK_C, false, false),
        make_key_input(VK_C, true, false),
        make_key_input(VK_CONTROL, true, false),
    ];
    send_inputs(&inputs);
}

use std::sync::atomic::{AtomicUsize, Ordering};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL};
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
    Hebrew,
    English,
    #[allow(dead_code)]
    Toggle,
}

static LAST_USED_ENGLISH_HKL: AtomicUsize = AtomicUsize::new(0);
static LAST_USED_HEBREW_HKL: AtomicUsize = AtomicUsize::new(0);

const LANG_HEBREW: u16 = 0x000D;
const LANG_ENGLISH: u16 = 0x0009;

/// Returns all currently installed keyboard layout handles (HKL)
pub fn get_installed_layouts() -> Vec<HKL> {
    unsafe {
        let count = GetKeyboardLayoutList(0, ptr::null_mut());
        if count <= 0 {
            return Vec::new();
        }
        let mut buffer = vec![ptr::null_mut(); count as usize];
        let actual = GetKeyboardLayoutList(count, buffer.as_mut_ptr());
        if actual > 0 {
            buffer.truncate(actual as usize);
            buffer
        } else {
            Vec::new()
        }
    }
}

/// Switch keyboard layout instantly and accurately between Hebrew and English layouts.
/// Handles cases where multiple English (e.g. US, UK) or Hebrew layouts exist without random cycling.
pub fn switch_keyboard_layout(target: TargetLanguage) {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_null() {
            return;
        }

        let thread_id = GetWindowThreadProcessId(fg, ptr::null_mut());
        let current_hkl = GetKeyboardLayout(thread_id);
        let current_lang_id = (current_hkl as usize & 0xFFFF) as u16;
        let current_primary = current_lang_id & 0x03FF;

        // Remember last used variants
        if current_primary == LANG_ENGLISH {
            LAST_USED_ENGLISH_HKL.store(current_hkl as usize, Ordering::SeqCst);
        } else if current_primary == LANG_HEBREW {
            LAST_USED_HEBREW_HKL.store(current_hkl as usize, Ordering::SeqCst);
        }

        let installed = get_installed_layouts();
        if installed.is_empty() {
            return;
        }

        let target_primary = match target {
            TargetLanguage::Hebrew => LANG_HEBREW,
            TargetLanguage::English => LANG_ENGLISH,
            TargetLanguage::Toggle => {
                if current_primary == LANG_HEBREW {
                    LANG_ENGLISH
                } else {
                    LANG_HEBREW
                }
            }
        };

        // Find the best HKL for the target primary language
        let best_hkl: Option<HKL> = if target_primary == LANG_ENGLISH {
            let last = LAST_USED_ENGLISH_HKL.load(Ordering::SeqCst) as HKL;
            if !last.is_null() && installed.contains(&last) {
                Some(last)
            } else {
                // Prefer English US (0x0409) if present, else first English layout
                installed
                    .iter()
                    .copied()
                    .find(|&h| (h as usize & 0xFFFF) == 0x0409)
                    .or_else(|| {
                        installed
                            .iter()
                            .copied()
                            .find(|&h| ((h as usize & 0xFFFF) as u16 & 0x03FF) == LANG_ENGLISH)
                    })
            }
        } else {
            let last = LAST_USED_HEBREW_HKL.load(Ordering::SeqCst) as HKL;
            if !last.is_null() && installed.contains(&last) {
                Some(last)
            } else {
                // Prefer Standard Hebrew (0x040D) if present, else first Hebrew layout
                installed
                    .iter()
                    .copied()
                    .find(|&h| (h as usize & 0xFFFF) == 0x040D)
                    .or_else(|| {
                        installed
                            .iter()
                            .copied()
                            .find(|&h| ((h as usize & 0xFFFF) as u16 & 0x03FF) == LANG_HEBREW)
                    })
            }
        };

        if let Some(target_hkl) = best_hkl {
            if target_hkl != current_hkl {
                PostMessageW(fg, WM_INPUTLANGCHANGEREQUEST, 0, target_hkl as isize);
                ActivateKeyboardLayout(target_hkl, 0);
            }
        }
    }
}

