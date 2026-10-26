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

const ICON_SVG: &str = include_str!("../assets/icon.svg");

/// Installs standard desktop shortcut, embedded icon, and desktop environment shortcuts.
pub fn install_app_linux() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set")?;

    // 1. Install Icon to ~/.local/share/icons/hicolor/scalable/apps/langswitch.svg
    let icon_dir = PathBuf::from(&home).join(".local/share/icons/hicolor/scalable/apps");
    let _ = fs::create_dir_all(&icon_dir);
    let icon_path = icon_dir.join("langswitch.svg");
    if let Err(e) = fs::write(&icon_path, ICON_SVG) {
        eprintln!("Warning: Failed writing icon to {}: {}", icon_path.display(), e);
    } else {
        println!("✅ Embedded SVG icon installed to: {}", icon_path.display());
    }

    // 2. Install Desktop file to ~/.local/share/applications/langswitch.desktop
    let target_dir = PathBuf::from(&home).join(".local/share/applications");
    let _ = fs::create_dir_all(&target_dir);
    let desktop_file = target_dir.join("langswitch.desktop");

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe_str = exe.to_string_lossy().to_string();

    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=LangSwitch\n\
         GenericName=Text Layout Switcher\n\
         Comment=Instant Hebrew/English text conversion\n\
         Exec=\"{}\" --convert\n\
         Icon=langswitch\n\
         Terminal=false\n\
         Categories=Utility;TextTools;\n\
         Keywords=hebrew;english;layout;switch;langover;\n\
         X-KDE-Shortcuts=F10\n\
         Actions=SwitchLayout;\n\n\
         [Desktop Action SwitchLayout]\n\
         Name=Convert & Switch Keyboard Layout\n\
         Exec=\"{}\" --convert-and-switch\n",
        exe_str, exe_str
    );

    fs::write(&desktop_file, content).map_err(|e| e.to_string())?;
    println!("✅ Desktop entry successfully installed to: {}", desktop_file.display());

    // 3. Automatically configure desktop environment keyboard shortcuts
    auto_configure_desktop_shortcuts(&exe_str);

    // 4. Update desktop and icon caches if tools exist
    let _ = Command::new("update-desktop-database")
        .arg(target_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let _ = Command::new("gtk-update-icon-cache")
        .arg("-f")
        .arg(PathBuf::from(&home).join(".local/share/icons/hicolor"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    println!("\n🎉 LangSwitch הותקנה בהצלחה בלינוקס!");
    println!("   מקש קיצור ראשי:  F10       (המרה)");
    println!("   מקש קיצור משני: Ctrl+F10  (המרה + החלפת שפת מקלדת)");
    Ok(())
}

/// Automatically configures system shortcuts in GNOME (Fedora/Ubuntu) and Cinnamon (Linux Mint).
pub fn auto_configure_desktop_shortcuts(exe_path: &str) {
    let de = detect_desktop_environment().to_lowercase();

    // 1. GNOME (Fedora / Ubuntu / Debian)
    if de.contains("gnome") {
        println!("🔧 מגדיר אוטומטית מקשי קיצור ב-GNOME (Fedora / Ubuntu)...");
        let p1 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-convert/";
        let p2 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-switch/";

        let list_str = format!("['{}', '{}']", p1, p2);
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", &list_str])
            .status();

        // Binding 1: F10
        let _ = Command::new("gsettings")
            .args(["set", &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", p1), "name", "LangSwitch Convert"])
            .status();
        let cmd1 = format!("\"{}\" --convert", exe_path);
        let _ = Command::new("gsettings")
            .args(["set", &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", p1), "command", &cmd1])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", p1), "binding", "F10"])
            .status();

        // Binding 2: Ctrl+F10
        let _ = Command::new("gsettings")
            .args(["set", &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", p2), "name", "LangSwitch Convert & Switch"])
            .status();
        let cmd2 = format!("\"{}\" --convert-and-switch", exe_path);
        let _ = Command::new("gsettings")
            .args(["set", &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", p2), "command", &cmd2])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", &format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", p2), "binding", "<Control>F10"])
            .status();
        println!("✅ קיצורי הדרך F10 ו-Ctrl+F10 הוגדרו בהצלחה ב-GNOME!");
        return;
    }

    // 2. Cinnamon (Linux Mint)
    if de.contains("cinnamon") || de.contains("x-cinnamon") {
        println!("🔧 מגדיר אוטומטית מקשי קיצור ב-Linux Mint (Cinnamon)...");
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings", "custom-list", "['langswitch0', 'langswitch1']"])
            .status();

        let cmd1 = format!("\"{}\" --convert", exe_path);
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/langswitch0/", "name", "LangSwitch Convert"])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/langswitch0/", "command", &cmd1])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/langswitch0/", "binding", "['F10']"])
            .status();

        let cmd2 = format!("\"{}\" --convert-and-switch", exe_path);
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/langswitch1/", "name", "LangSwitch Convert & Switch"])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/langswitch1/", "command", &cmd2])
            .status();
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/langswitch1/", "binding", "['<Control>F10']"])
            .status();
        println!("✅ קיצורי הדרך F10 ו-Ctrl+F10 הוגדרו בהצלחה ב-Linux Mint Cinnamon!");
        return;
    }

    // 3. Other environments (KDE / Hyprland / Sway / i3)
    if de.contains("kde") || de.contains("plasma") {
        println!("ℹ️  ב-KDE Plasma: קיצור הדרך F10 מוטמע ישירות בקובץ ה-desktop שהותקן.");
    } else {
        println!("💡 ב-Hyprland / Sway / i3 הוסף להגדרות:");
        println!("   bind = , F10, exec, {} --convert", exe_path);
        println!("   bind = CONTROL, F10, exec, {} --convert-and-switch", exe_path);
    }
}
