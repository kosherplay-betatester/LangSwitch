//! Linux platform implementation for LangSwitch.
//! Seamlessly supports both X11 (Linux Mint, Ubuntu X11, Debian)
//! and Wayland (Fedora GNOME, Arch Linux KDE Plasma 6 / Hyprland / Sway).

use crate::config::AppConfig;
use crate::converter;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

impl std::fmt::Display for DisplayServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayServer::Wayland => write!(f, "Wayland"),
            DisplayServer::X11 => write!(f, "X11"),
            DisplayServer::Unknown => write!(f, "Unknown (Headless / TTY)"),
        }
    }
}

pub fn detect_display_server() -> DisplayServer {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        DisplayServer::Wayland
    } else if std::env::var("DISPLAY").is_ok() {
        DisplayServer::X11
    } else {
        DisplayServer::Unknown
    }
}

pub fn detect_desktop_environment() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

pub fn detect_distro_name() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                return name.trim_matches('"').to_string();
            }
        }
    }
    "Linux".to_string()
}

pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Reads the PRIMARY selection (highlighted text) on Linux.
/// Both X11 and Wayland support PRIMARY selection natively.
pub fn get_primary_selection(display: DisplayServer) -> Option<String> {
    match display {
        DisplayServer::Wayland => {
            if let Ok(output) = Command::new("wl-paste").arg("--primary").output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            None
        }
        DisplayServer::X11 => {
            if let Ok(output) = Command::new("xclip")
                .args(["-o", "-selection", "primary"])
                .output()
            {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            if let Ok(output) = Command::new("xsel").args(["-o", "-p"]).output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            None
        }
        DisplayServer::Unknown => None,
    }
}

/// Reads the standard CLIPBOARD buffer.
pub fn get_clipboard_text(display: DisplayServer) -> Option<String> {
    match display {
        DisplayServer::Wayland => {
            if let Ok(output) = Command::new("wl-paste").arg("--no-newline").output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            None
        }
        DisplayServer::X11 => {
            if let Ok(output) = Command::new("xclip")
                .args(["-o", "-selection", "clipboard"])
                .output()
            {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            if let Ok(output) = Command::new("xsel").args(["-o", "-b"]).output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            None
        }
        DisplayServer::Unknown => None,
    }
}

/// Writes text into the standard CLIPBOARD buffer.
pub fn set_clipboard_text(text: &str, display: DisplayServer) -> bool {
    match display {
        DisplayServer::Wayland => {
            if let Ok(mut child) = Command::new("wl-copy")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }
            false
        }
        DisplayServer::X11 => {
            if let Ok(mut child) = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }
            if let Ok(mut child) = Command::new("xsel")
                .args(["-b", "-i"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                return child.wait().map(|s| s.success()).unwrap_or(false);
            }
            false
        }
        DisplayServer::Unknown => false,
    }
}

/// Automatically selects current line (Home + Shift+End + Ctrl+C) if no text was highlighted.
pub fn select_current_line(display: DisplayServer) {
    match display {
        DisplayServer::Wayland => {
            // Under Wayland: wtype or ydotool
            if command_exists("wtype") {
                let _ = Command::new("wtype")
                    .args(["-k", "Home", "-M", "shift", "-k", "End", "-m", "shift", "-M", "ctrl", "-k", "c", "-m", "ctrl"])
                    .status();
            } else if command_exists("ydotool") {
                // Home (102), Shift (42), End (107), Ctrl (29), C (46)
                let _ = Command::new("ydotool")
                    .args(["key", "102:1", "102:0", "42:1", "107:1", "107:0", "42:0", "29:1", "46:1", "46:0", "29:0"])
                    .status();
            }
        }
        DisplayServer::X11 => {
            if command_exists("xdotool") {
                let _ = Command::new("xdotool")
                    .args(["key", "Home", "shift+End", "ctrl+c"])
                    .status();
            }
        }
        DisplayServer::Unknown => {}
    }
}

/// Simulates pasting from clipboard (Ctrl+V).
pub fn send_paste(display: DisplayServer) {
    thread::sleep(Duration::from_millis(25));
    match display {
        DisplayServer::Wayland => {
            if command_exists("wtype") {
                let _ = Command::new("wtype")
                    .args(["-M", "ctrl", "-k", "v", "-m", "ctrl"])
                    .status();
            } else if command_exists("ydotool") {
                let _ = Command::new("ydotool")
                    .args(["key", "29:1", "47:1", "47:0", "29:0"])
                    .status();
            }
        }
        DisplayServer::X11 => {
            if command_exists("xdotool") {
                let _ = Command::new("xdotool").args(["key", "ctrl+v"]).status();
            }
        }
        DisplayServer::Unknown => {}
    }
}

/// Switches keyboard layout dynamically based on desktop environment.
pub fn switch_keyboard_layout(display: DisplayServer) {
    let de = detect_desktop_environment().to_lowercase();

    // 1. GNOME (Fedora / Ubuntu)
    if de.contains("gnome") {
        let gnome_script = "imports.ui.status.keyboard.getInputSourceManager()._switchInputSource();";
        let _ = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest", "org.gnome.Shell",
                "--object-path", "/org/gnome/Shell",
                "--method", "org.gnome.Shell.Eval",
                gnome_script,
            ])
            .output();
        return;
    }

    // 2. Cinnamon (Linux Mint)
    if de.contains("cinnamon") || de.contains("x-cinnamon") {
        // Toggle input source or simulate layout shortcut
        let _ = Command::new("cinnamon-dbus-command").args(["switch-layout"]).output();
        if display == DisplayServer::X11 && command_exists("xdotool") {
            let _ = Command::new("xdotool").args(["key", "alt+shift"]).output();
        }
        return;
    }

    // 3. KDE Plasma (Arch Linux, Fedora KDE)
    if de.contains("kde") || de.contains("plasma") {
        let _ = Command::new("qdbus")
            .args(["org.kde.keyboard", "/Layouts", "switchToNextLayout"])
            .output();
        return;
    }

    // 4. Generic X11 fallback (setxkbmap toggle or alt+shift)
    if display == DisplayServer::X11 && command_exists("xdotool") {
        let _ = Command::new("xdotool").args(["key", "alt+shift"]).output();
    }
}

/// Main conversion entry point for Linux.
pub fn perform_conversion(switch_layout_always: bool) {
    let display = detect_display_server();
    let config = AppConfig::load();

    // 1. Try reading PRIMARY selection (text already selected/highlighted by user)
    let mut selected_text = get_primary_selection(display);

    // 2. If no text was highlighted, and auto_select_line is enabled, select line & copy
    if selected_text.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) && config.auto_select_line {
        select_current_line(display);
        thread::sleep(Duration::from_millis(35));
        selected_text = get_clipboard_text(display);
    }

    let raw_text = match selected_text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    // 3. Convert text using converter engine
    let (converted, _) = converter::convert_text(&raw_text);
    if converted == raw_text {
        return;
    }

    // 4. Save to clipboard & paste
    if set_clipboard_text(&converted, display) {
        send_paste(display);
    }

    // 5. Switch layout if secondary hotkey or auto_switch_layout enabled
    if switch_layout_always || config.auto_switch_layout {
        thread::sleep(Duration::from_millis(35));
        switch_keyboard_layout(display);
    }
}

/// Print comprehensive diagnostic status for Linux users.
pub fn print_status() {
    let display = detect_display_server();
    let distro = detect_distro_name();
    let de = detect_desktop_environment();

    println!("==================================================");
    println!("⚡ LangSwitch - Linux Environment Diagnostics");
    println!("==================================================");
    println!("Operating System:     {}", distro);
    println!("Desktop Environment:  {}", de);
    println!("Display Server:       {}", display);
    println!("--------------------------------------------------");
    println!("Toolchain Status:");

    match display {
        DisplayServer::Wayland => {
            let has_wl = command_exists("wl-paste") && command_exists("wl-copy");
            let has_wtype = command_exists("wtype");
            let has_ydotool = command_exists("ydotool");

            println!("  • wl-clipboard (wl-copy, wl-paste): {}", if has_wl { "✅ Installed" } else { "❌ Missing" });
            println!("  • wtype (simulated keystrokes):     {}", if has_wtype { "✅ Installed" } else { "⚠️ Optional (or ydotool)" });
            println!("  • ydotool (universal uinput keys):  {}", if has_ydotool { "✅ Installed" } else { "⚠️ Optional" });

            if !has_wl {
                println!("\n💡 Recommendation for Wayland:");
                println!("  Install wl-clipboard for instant clipboard & primary selection support:");
                if distro.contains("Fedora") {
                    println!("    sudo dnf install wl-clipboard wtype");
                } else if distro.contains("Arch") {
                    println!("    sudo pacman -S wl-clipboard wtype");
                } else {
                    println!("    sudo apt install wl-clipboard wtype");
                }
            }
        }
        DisplayServer::X11 => {
            let has_xclip = command_exists("xclip");
            let has_xsel = command_exists("xsel");
            let has_xdotool = command_exists("xdotool");

            println!("  • xclip:   {}", if has_xclip { "✅ Installed" } else { "❌ Missing" });
            println!("  • xsel:    {}", if has_xsel { "✅ Installed" } else { "⚠️ Optional" });
            println!("  • xdotool: {}", if has_xdotool { "✅ Installed" } else { "❌ Missing" });

            if !has_xclip || !has_xdotool {
                println!("\n💡 Recommendation for X11 (Linux Mint / Debian / Ubuntu):");
                println!("  Install xclip and xdotool:");
                if distro.contains("Mint") || distro.contains("Ubuntu") || distro.contains("Debian") {
                    println!("    sudo apt install xclip xdotool");
                } else if distro.contains("Arch") {
                    println!("    sudo pacman -S xclip xdotool");
                } else if distro.contains("Fedora") {
                    println!("    sudo dnf install xclip xdotool");
                }
            }
        }
        DisplayServer::Unknown => {
            println!("  ⚠️ Warning: No graphical display server (WAYLAND_DISPLAY or DISPLAY) detected.");
        }
    }

    println!("==================================================");
    println!("Usage Commands:");
    println!("  langswitch --convert             Convert selected text or current line");
    println!("  langswitch --convert-and-switch  Convert text and toggle keyboard layout");
    println!("  langswitch --install-desktop     Install .desktop shortcut in app menu");
    println!("  langswitch --status              Show this diagnostics page");
    println!("==================================================");
}

/// Installs standard desktop shortcut to ~/.local/share/applications/langswitch.desktop
pub fn install_desktop() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;
    let target_dir = PathBuf::from(home).join(".local/share/applications");
    let _ = fs::create_dir_all(&target_dir);
    let desktop_file = target_dir.join("langswitch.desktop");

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=LangSwitch\n\
         Comment=Hebrew/English Text Layout Switcher\n\
         Exec=\"{}\" --convert\n\
         Terminal=false\n\
         Categories=Utility;\n\
         Keywords=hebrew;english;layout;switch;langover;\n",
        exe.to_string_lossy()
    );

    fs::write(&desktop_file, content).map_err(|e| e.to_string())?;
    println!("✅ Desktop entry successfully installed to: {}", desktop_file.display());
    Ok(())
}
