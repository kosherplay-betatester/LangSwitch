use std::ptr;
use std::thread;
use std::time::Duration;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
    SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, VK_C, VK_CONTROL, VK_END, VK_HOME, VK_MENU, VK_SHIFT, VK_V,
};

const CF_UNICODETEXT: u32 = 13;

pub fn get_clipboard_seq() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
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

/// Send Ctrl + C
pub fn send_ctrl_c() {
    let inputs = [
        make_key_input(VK_CONTROL, false, false),
        make_key_input(VK_C, false, false),
        make_key_input(VK_C, true, false),
        make_key_input(VK_CONTROL, true, false),
    ];
    send_inputs(&inputs);
}

/// Send Ctrl + V
pub fn send_ctrl_v() {
    let inputs = [
        make_key_input(VK_CONTROL, false, false),
        make_key_input(VK_V, false, false),
        make_key_input(VK_V, true, false),
        make_key_input(VK_CONTROL, true, false),
    ];
    send_inputs(&inputs);
}

/// Select the entire current line by sending Home then Shift + End
pub fn select_current_line() {
    let inputs_home = [
        make_key_input(VK_HOME, false, true),
        make_key_input(VK_HOME, true, true),
    ];
    send_inputs(&inputs_home);
    thread::sleep(Duration::from_millis(10));

    let inputs_shift_end = [
        make_key_input(VK_SHIFT, false, false),
        make_key_input(VK_END, false, true),
        make_key_input(VK_END, true, true),
        make_key_input(VK_SHIFT, true, false),
    ];
    send_inputs(&inputs_shift_end);
    thread::sleep(Duration::from_millis(10));
}

/// Switch keyboard layout (Alt + Shift)
pub fn switch_keyboard_layout() {
    let inputs = [
        make_key_input(VK_MENU, false, false),
        make_key_input(VK_SHIFT, false, false),
        make_key_input(VK_SHIFT, true, false),
        make_key_input(VK_MENU, true, false),
    ];
    send_inputs(&inputs);
}
