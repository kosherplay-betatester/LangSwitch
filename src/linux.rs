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

/// Clears clipboard text in X11 and Wayland so we never read stale data.
pub fn clear_clipboard(display: DisplayServer) {
    match display {
        DisplayServer::Wayland => {
            if command_exists("wl-copy") {
                let _ = Command::new("wl-copy").arg("--clear").output();
            }
        }
        DisplayServer::X11 => {
            if command_exists("xclip") {
                if let Ok(mut child) = Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(b"");
                    }
                    let _ = child.wait();
                }
            }
        }
        DisplayServer::Unknown => {}
    }
}

/// Simulates Ctrl+C with cleared modifiers.
pub fn send_copy(display: DisplayServer) {
    match display {
        DisplayServer::Wayland => {
            if command_exists("wtype") {
                let _ = Command::new("wtype")
                    .args(["-M", "ctrl", "-k", "c", "-m", "ctrl"])
                    .status();
            } else if command_exists("ydotool") {
                let _ = Command::new("ydotool")
                    .args(["key", "29:1", "46:1", "46:0", "29:0"])
                    .status();
            }
        }
        DisplayServer::X11 => {
            if command_exists("xdotool") {
                let _ = Command::new("xdotool")
                    .args(["key", "--clearmodifiers", "ctrl+c"])
                    .status();
            }
        }
        DisplayServer::Unknown => {}
    }
}

/// Automatically selects current line (Home + Shift+End + Ctrl+C) with brief key spacing.
pub fn select_current_line(display: DisplayServer) {
    match display {
        DisplayServer::Wayland => {
            if command_exists("wtype") {
                let _ = Command::new("wtype")
                    .args(["-k", "Home", "-M", "shift", "-k", "End", "-m", "shift", "-M", "ctrl", "-k", "c", "-m", "ctrl"])
                    .status();
            } else if command_exists("ydotool") {
                let _ = Command::new("ydotool")
                    .args(["key", "102:1", "102:0", "42:1", "107:1", "107:0", "42:0", "29:1", "46:1", "46:0", "29:0"])
                    .status();
            }
        }
        DisplayServer::X11 => {
            if command_exists("xdotool") {
                let _ = Command::new("xdotool").args(["key", "--clearmodifiers", "Home"]).status();
                thread::sleep(Duration::from_millis(20));
                let _ = Command::new("xdotool").args(["key", "--clearmodifiers", "shift+End"]).status();
                thread::sleep(Duration::from_millis(20));
                let _ = Command::new("xdotool").args(["key", "--clearmodifiers", "ctrl+c"]).status();
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
                let _ = Command::new("xdotool").args(["key", "--clearmodifiers", "ctrl+v"]).status();
            }
        }
        DisplayServer::Unknown => {}
    }
}

/// Switches keyboard layout dynamically without altering system layout tables.
pub fn switch_keyboard_layout(display: DisplayServer) {
    let de = detect_desktop_environment().to_lowercase();

    // 1. X11 desktops (Linux Mint Cinnamon, MATE, XFCE): safely simulate user's switch shortcut
    if display == DisplayServer::X11 && command_exists("xdotool") {
        let mut shortcut = "super+space";
        if let Ok(out) = Command::new("gsettings")
            .args(["get", "org.cinnamon.desktop.input-sources", "xkb-options"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
            if text.contains("alt_shift") {
                shortcut = "alt+shift";
            } else if text.contains("ctrl_shift") {
                shortcut = "ctrl+shift";
            } else if text.contains("super_space") || text.contains("win_space") {
                shortcut = "super+space";
            }
        }
        let _ = Command::new("xdotool")
            .args(["key", "--clearmodifiers", shortcut])
            .status();
        return;
    }

    // 2. GNOME Shell on Wayland (Fedora / Ubuntu)
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

    // 3. KDE Plasma fallback
    if de.contains("kde") || de.contains("plasma") {
        let _ = Command::new("qdbus")
            .args(["org.kde.keyboard", "/Layouts", "switchToNextLayout"])
            .output();
    }
}

/// Main conversion entry point for Linux.
pub fn perform_conversion(switch_layout_always: bool) {
    let display = detect_display_server();
    let config = AppConfig::load();

    // 1. Critical: Wait 150ms for physical hotkey (F10 / Ctrl+F10) to be released by user.
    // Without this delay, active apps (like Chrome / Telegram) receive stuck modifier events or menu focus!
    thread::sleep(Duration::from_millis(150));

    // 2. Clear clipboard first so we can detect freshly copied text
    clear_clipboard(display);

    // 3. Send Ctrl+C to copy whatever text is currently highlighted
    send_copy(display);
    thread::sleep(Duration::from_millis(120));

    let mut selected_text = get_clipboard_text(display);

    // 4. If nothing was highlighted and auto_select_line is enabled: select line & copy
    if selected_text.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) && config.auto_select_line {
        clear_clipboard(display);
        select_current_line(display);
        thread::sleep(Duration::from_millis(120));
        selected_text = get_clipboard_text(display);
    }

    // 5. Fallback to PRIMARY selection if clipboard is still empty
    if selected_text.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        if let Some(pri) = get_primary_selection(display) {
            if !pri.trim().is_empty() {
                selected_text = Some(pri);
            }
        }
    }

    let raw_text = match selected_text {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    // 6. Convert text using converter engine
    let (converted, _) = converter::convert_text(&raw_text);
    if converted == raw_text {
        return;
    }

    // 7. Save converted text to clipboard & paste
    if set_clipboard_text(&converted, display) {
        thread::sleep(Duration::from_millis(50));
        send_paste(display);
    }

    // 8. Safely toggle layout using user's configured shortcut without altering layout lists
    if switch_layout_always || config.auto_switch_layout {
        thread::sleep(Duration::from_millis(80));
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
const WELCOME_SCRIPT: &str = include_str!("../assets/welcome_linux.py");

/// Launches the interactive Welcome Screen GUI on Linux.
pub fn show_welcome_screen() {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let welcome_path = PathBuf::from(&home).join(".local/bin/langswitch-welcome");
    let _ = fs::write(&welcome_path, WELCOME_SCRIPT);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = fs::metadata(&welcome_path).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&welcome_path, perms);
        }
    }

    let _ = Command::new("python3")
        .arg(&welcome_path)
        .spawn();
}

/// Installs standard desktop shortcut, embedded icon, welcome GUI and desktop environment shortcuts.
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

    // 2. Install Welcome GUI script to ~/.local/bin/langswitch-welcome
    let bin_dir = PathBuf::from(&home).join(".local/bin");
    let _ = fs::create_dir_all(&bin_dir);
    let welcome_path = bin_dir.join("langswitch-welcome");
    let _ = fs::write(&welcome_path, WELCOME_SCRIPT);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = fs::metadata(&welcome_path).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&welcome_path, perms);
        }
    }

    // 3. Install Desktop file to ~/.local/share/applications/langswitch.desktop
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
         Exec=\"{}\" --welcome\n\
         Icon=langswitch\n\
         Terminal=false\n\
         Categories=Utility;TextTools;\n\
         Keywords=hebrew;english;layout;switch;langover;\n\
         X-KDE-Shortcuts=F10\n\
         Actions=Convert;SwitchLayout;\n\n\
         [Desktop Action Convert]\n\
         Name=Convert Text (F10)\n\
         Exec=\"{}\" --convert\n\n\
         [Desktop Action SwitchLayout]\n\
         Name=Convert & Switch Keyboard Layout (Ctrl+F10)\n\
         Exec=\"{}\" --convert-and-switch\n",
        exe_str, exe_str, exe_str
    );

    fs::write(&desktop_file, content).map_err(|e| e.to_string())?;
    println!("✅ Desktop entry successfully installed to: {}", desktop_file.display());

    // 4. Automatically configure desktop environment keyboard shortcuts
    auto_configure_desktop_shortcuts(&exe_str);

    // 5. Update desktop and icon caches if tools exist
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

        // Disable GTK F10 menu capture so F10 works globally without conflict
        let _ = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.interface", "menubar-accel", ""])
            .status();

        let p1 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-convert/";
        let p2 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-switch/";

        let mut bindings = vec![format!("'{}'", p1), format!("'{}'", p2)];
        if let Ok(output) = Command::new("gsettings")
            .args(["get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"])
            .output()
        {
            if output.status.success() {
                let current = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let cleaned = current.trim_start_matches("@as").trim();
                if cleaned.starts_with('[') && cleaned.ends_with(']') {
                    let inner = &cleaned[1..cleaned.len() - 1];
                    for item in inner.split(',') {
                        let trimmed = item.trim();
                        if !trimmed.is_empty() && !trimmed.contains("langswitch-") {
                            bindings.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        let list_str = format!("[{}]", bindings.join(", "));
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

        // Disable GTK F10 menu capture so F10 works globally without conflict
        let _ = Command::new("gsettings")
            .args(["set", "org.cinnamon.desktop.interface", "menubar-accel", ""])
            .status();

        let py_script = format!(
            r#"import subprocess, ast
target_bin = "{}"
try:
    raw = subprocess.check_output(["gsettings", "get", "org.cinnamon.desktop.keybindings", "custom-list"], stderr=subprocess.DEVNULL).decode().strip()
    if raw.startswith("@as"): raw = raw[3:].strip()
    lst = ast.literal_eval(raw) if raw and raw != "[]" else []
    slot1, slot2 = None, None
    for s in lst:
        p = f"org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{{s}}/"
        try:
            cmd = subprocess.check_output(["gsettings", "get", p, "command"], stderr=subprocess.DEVNULL).decode()
            if "langswitch --convert-and-switch" in cmd: slot2 = s
            elif "langswitch --convert" in cmd: slot1 = s
        except Exception: pass
    if not slot1:
        idx = 0
        while f"custom{{idx}}" in lst: idx += 1
        slot1 = f"custom{{idx}}"
        lst.append(slot1)
    if not slot2:
        idx = 0
        while f"custom{{idx}}" in lst: idx += 1
        slot2 = f"custom{{idx}}"
        lst.append(slot2)
    subprocess.run(["gsettings", "set", "org.cinnamon.desktop.keybindings", "custom-list", str(lst)], stderr=subprocess.DEVNULL)
    p1 = f"org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{{slot1}}/"
    subprocess.run(["gsettings", "set", p1, "name", "LangSwitch Convert"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p1, "command", f"{{target_bin}} --convert"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p1, "binding", "['F10']"], stderr=subprocess.DEVNULL)
    p2 = f"org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{{slot2}}/"
    subprocess.run(["gsettings", "set", p2, "name", "LangSwitch Convert & Switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p2, "command", f"{{target_bin}} --convert-and-switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p2, "binding", "['<Control>F10']"], stderr=subprocess.DEVNULL)
except Exception: pass

try:
    p1 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-convert/"
    p2 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-switch/"
    raw = subprocess.check_output(["gsettings", "get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"], stderr=subprocess.DEVNULL).decode().strip()
    if raw.startswith("@as"): raw = raw[3:].strip()
    lst = ast.literal_eval(raw) if raw and raw != "[]" else []
    if p1 not in lst: lst.append(p1)
    if p2 not in lst: lst.append(p2)
    subprocess.run(["gsettings", "set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", str(lst)], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{{p1}}", "name", "LangSwitch Convert"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{{p1}}", "command", f"{{target_bin}} --convert"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{{p1}}", "binding", "F10"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{{p2}}", "name", "LangSwitch Convert & Switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{{p2}}", "command", f"{{target_bin}} --convert-and-switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{{p2}}", "binding", "<Control>F10"], stderr=subprocess.DEVNULL)
except Exception: pass
"#,
            exe_path
        );
        let _ = Command::new("python3").args(["-c", &py_script]).status();

        println!("✅ קיצורי הדרך F10 ו-Ctrl+F10 הוגדרו בהצלחה ב-Linux Mint Cinnamon!");
        return;
    }

    // 3. MATE (Linux Mint MATE, Ubuntu MATE)
    if de.contains("mate") {
        println!("🔧 מגדיר אוטומטית מקשי קיצור ב-MATE...");
        let _ = Command::new("gsettings")
            .args(["set", "org.mate.interface", "menubar-accel", ""])
            .status();

        let p1 = "/org/mate/settings-daemon/plugins/media-keys/custom-keybindings/custom0/";
        let p2 = "/org/mate/settings-daemon/plugins/media-keys/custom-keybindings/custom1/";
        let list_str = format!("['{}', '{}']", p1, p2);
        let _ = Command::new("gsettings")
            .args(["set", "org.mate.SettingsDaemon.plugins.media-keys", "custom-keybindings", &list_str])
            .status();

        let cmd1 = format!("\"{}\" --convert", exe_path);
        let cmd2 = format!("\"{}\" --convert-and-switch", exe_path);
        let _ = Command::new("gsettings").args(["set", &format!("org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:{}", p1), "name", "LangSwitch Convert"]).status();
        let _ = Command::new("gsettings").args(["set", &format!("org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:{}", p1), "command", &cmd1]).status();
        let _ = Command::new("gsettings").args(["set", &format!("org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:{}", p1), "binding", "F10"]).status();

        let _ = Command::new("gsettings").args(["set", &format!("org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:{}", p2), "name", "LangSwitch Convert & Switch"]).status();
        let _ = Command::new("gsettings").args(["set", &format!("org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:{}", p2), "command", &cmd2]).status();
        let _ = Command::new("gsettings").args(["set", &format!("org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:{}", p2), "binding", "<Control>F10"]).status();
        println!("✅ קיצורי הדרך F10 ו-Ctrl+F10 הוגדרו בהצלחה ב-MATE!");
        return;
    }

    // 4. XFCE (Linux Mint XFCE, Xubuntu, Manjaro XFCE)
    if de.contains("xfce") {
        println!("🔧 מגדיר אוטומטית מקשי קיצור ב-XFCE...");
        let cmd1 = format!("\"{}\" --convert", exe_path);
        let cmd2 = format!("\"{}\" --convert-and-switch", exe_path);
        let _ = Command::new("xfconf-query")
            .args(["-c", "xfce4-keyboard-shortcuts", "-p", "/commands/custom/<F10>", "-n", "-t", "string", "-s", &cmd1])
            .status();
        let _ = Command::new("xfconf-query")
            .args(["-c", "xfce4-keyboard-shortcuts", "-p", "/commands/custom/<Primary><F10>", "-n", "-t", "string", "-s", &cmd2])
            .status();
        println!("✅ קיצורי הדרך F10 ו-Ctrl+F10 הוגדרו בהצלחה ב-XFCE!");
        return;
    }

    // 5. Other environments (KDE / Hyprland / Sway / i3)
    if de.contains("kde") || de.contains("plasma") {
        println!("ℹ️  ב-KDE Plasma: קיצור הדרך F10 מוטמע ישירות בקובץ ה-desktop שהותקן.");
    } else {
        println!("💡 ב-Hyprland / Sway / i3 הוסף להגדרות:");
        println!("   bind = , F10, exec, {} --convert", exe_path);
        println!("   bind = CONTROL, F10, exec, {} --convert-and-switch", exe_path);
    }
}
