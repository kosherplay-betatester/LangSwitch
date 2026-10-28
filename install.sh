#!/usr/bin/env bash
# ==============================================================================
# ⚡ LangSwitch - Ultra-friendly Universal Linux Installer
# Supports: Linux Mint, Fedora, Arch Linux, Ubuntu, Debian, Manjaro, openSUSE
# Display Servers: Wayland & X11
# ==============================================================================

# Terminal colors
BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BOLD}${CYAN}"
echo "============================================================"
echo "⚡ LangSwitch - התקנה מהירה וידידותית ללינוקס"
echo "   Ultra-friendly Installer for Linux (X11 & Wayland)"
echo "============================================================"
echo -e "${NC}"

# 1. Detect Architecture
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    echo -e "${RED}❌ שגיאה: כרגע נתמכת ארכיטקטורת x86_64 בלבד (Your arch: $ARCH).${NC}"
    exit 1
fi

# 2. Detect Display Server & Desktop Environment
IS_WAYLAND=false
if [ -n "$WAYLAND_DISPLAY" ]; then
    IS_WAYLAND=true
    DISPLAY_NAME="Wayland"
elif [ -n "$DISPLAY" ]; then
    DISPLAY_NAME="X11"
else
    DISPLAY_NAME="Headless/Unknown"
fi

DE="${XDG_CURRENT_DESKTOP:-$DESKTOP_SESSION}"
echo -e "${BLUE}ℹ️  סביבת שולחן עבודה:${NC} ${BOLD}$DE${NC} (${DISPLAY_NAME})"

# Self-healing: verify both English and Hebrew layouts exist with toggle
if command -v setxkbmap >/dev/null 2>&1; then
    CURRENT_LAYOUTS=$(setxkbmap -query 2>/dev/null | grep -E '^layout:' | awk '{print $2}')
    if [ -n "$CURRENT_LAYOUTS" ] && [[ "$CURRENT_LAYOUTS" != *"il"* ]]; then
        echo -e "${YELLOW}ℹ️  מוודא הגדרת שפות מקלדת: מוסיף עברית לצד אנגלית עם מעבר Win+Space...${NC}"
        setxkbmap -layout "$CURRENT_LAYOUTS,il" -option grp:win_space_toggle 2>/dev/null || true
    fi
fi

# 3. Detect Distro & Package Manager
DISTRO="Linux"
PKG_MGR=""
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO="$NAME"
fi
echo -e "${BLUE}ℹ️  הפצת לינוקס:${NC} ${BOLD}$DISTRO${NC}"

if command -v apt-get >/dev/null 2>&1; then
    PKG_MGR="apt"
elif command -v dnf >/dev/null 2>&1; then
    PKG_MGR="dnf"
elif command -v pacman >/dev/null 2>&1; then
    PKG_MGR="pacman"
elif command -v zypper >/dev/null 2>&1; then
    PKG_MGR="zypper"
fi

# 4. Create local user directories (NO ROOT REQUIRED)
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# 5. Download or copy binary
TARGET_BIN="$INSTALL_DIR/langswitch"
echo ""
echo -e "${BOLD}📥 מתקין את LangSwitch לתיקייה:${NC} $INSTALL_DIR"

if [ -f "./target/release/langswitch" ]; then
    cp "./target/release/langswitch" "$TARGET_BIN"
elif [ -f "./langswitch" ]; then
    cp "./langswitch" "$TARGET_BIN"
else
    DOWNLOAD_URL="https://github.com/kosherplay-betatester/LangSwitch/releases/latest/download/langswitch-linux-x86_64"
    echo -e "${CYAN}מוריד את הגרסה העדכנית ביותר מ-GitHub Releases...${NC}"
    curl -sSL -f "$DOWNLOAD_URL" -o "$TARGET_BIN" || {
        echo -e "${RED}שגיאה בהורדת הקובץ מ-GitHub. אנא בדוק חיבור לאינטרנט.${NC}"
        exit 1
    }
fi

chmod +x "$TARGET_BIN"
echo -e "${GREEN}✅ הקובץ הבינארי הותקן בהצלחה!${NC}"

# 6. Install Icon & Welcome Screen Script
if [ -f "./assets/icon.svg" ]; then
    cp "./assets/icon.svg" "$ICON_DIR/langswitch.svg"
else
    curl -sSL "https://raw.githubusercontent.com/kosherplay-betatester/LangSwitch/main/assets/icon.svg" -o "$ICON_DIR/langswitch.svg" 2>/dev/null || true
fi

WELCOME_BIN="$INSTALL_DIR/langswitch-welcome"
if [ -f "./assets/welcome_linux.py" ]; then
    cp "./assets/welcome_linux.py" "$WELCOME_BIN"
else
    curl -sSL "https://raw.githubusercontent.com/kosherplay-betatester/LangSwitch/main/assets/welcome_linux.py" -o "$WELCOME_BIN" 2>/dev/null || true
fi
chmod +x "$WELCOME_BIN" 2>/dev/null || true

AUTOSTART_DIR="$HOME/.config/autostart"
mkdir -p "$AUTOSTART_DIR"
cat << EOF > "$AUTOSTART_DIR/langswitch.desktop"
[Desktop Entry]
Type=Application
Name=LangSwitch
Comment=Instant Hebrew/English text conversion system tray
Exec=python3 $WELCOME_BIN --tray
Icon=langswitch
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
EOF

# 7. Run in-app automated desktop & shortcut setup
echo ""
echo -e "${BOLD}⚙️  מגדיר קיצורי דרך ותפריט מערכת...${NC}"
"$TARGET_BIN" --install || true

if command -v gsettings >/dev/null 2>&1; then
    # Disable GTK F10 menubar accelerator so F10 works globally without conflict
    gsettings set org.cinnamon.desktop.interface menubar-accel "" 2>/dev/null || true
    gsettings set org.gnome.desktop.interface menubar-accel "" 2>/dev/null || true
    gsettings set org.mate.interface menubar-accel "" 2>/dev/null || true

    # Robust multi-desktop keybinding registration via Python
    python3 - << 'EOF' 2>/dev/null || true
import subprocess, ast, os

target_bin = os.path.expanduser("~/.local/bin/langswitch")

# A. Linux Mint Cinnamon Custom Keybindings
try:
    raw = subprocess.check_output(["gsettings", "get", "org.cinnamon.desktop.keybindings", "custom-list"], stderr=subprocess.DEVNULL).decode().strip()
    if raw.startswith("@as"):
        raw = raw[3:].strip()
    lst = ast.literal_eval(raw) if raw and raw != "[]" else []

    slot1, slot2 = None, None
    for s in lst:
        p = f"org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{s}/"
        try:
            cmd = subprocess.check_output(["gsettings", "get", p, "command"], stderr=subprocess.DEVNULL).decode()
            if "langswitch --convert-and-switch" in cmd:
                slot2 = s
            elif "langswitch --convert" in cmd:
                slot1 = s
        except Exception:
            pass

    if not slot1:
        idx = 0
        while f"custom{idx}" in lst:
            idx += 1
        slot1 = f"custom{idx}"
        lst.append(slot1)

    if not slot2:
        idx = 0
        while f"custom{idx}" in lst:
            idx += 1
        slot2 = f"custom{idx}"
        lst.append(slot2)

    subprocess.run(["gsettings", "set", "org.cinnamon.desktop.keybindings", "custom-list", str(lst)], stderr=subprocess.DEVNULL)

    p1 = f"org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{slot1}/"
    subprocess.run(["gsettings", "set", p1, "name", "LangSwitch Convert & Switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p1, "command", f"{target_bin} --convert-and-switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p1, "binding", "['F10']"], stderr=subprocess.DEVNULL)

    p2 = f"org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/{slot2}/"
    subprocess.run(["gsettings", "set", p2, "name", "LangSwitch Convert Only"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p2, "command", f"{target_bin} --convert-only"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", p2, "binding", "['<Control>F10']"], stderr=subprocess.DEVNULL)
except Exception:
    pass

# B. GNOME / MATE media-keys
try:
    p1 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-convert/"
    p2 = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-switch/"
    raw = subprocess.check_output(["gsettings", "get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"], stderr=subprocess.DEVNULL).decode().strip()
    if raw.startswith("@as"):
        raw = raw[3:].strip()
    lst = ast.literal_eval(raw) if raw and raw != "[]" else []
    if p1 not in lst: lst.append(p1)
    if p2 not in lst: lst.append(p2)
    subprocess.run(["gsettings", "set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", str(lst)], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p1}", "name", "LangSwitch Convert & Switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p1}", "command", f"{target_bin} --convert-and-switch"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p1}", "binding", "F10"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p2}", "name", "LangSwitch Convert Only"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p2}", "command", f"{target_bin} --convert-only"], stderr=subprocess.DEVNULL)
    subprocess.run(["gsettings", "set", f"org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{p2}", "binding", "<Control>F10"], stderr=subprocess.DEVNULL)
except Exception:
    pass
EOF
fi

# XFCE setup
if command -v xfconf-query >/dev/null 2>&1; then
    xfconf-query -c xfce4-keyboard-shortcuts -p "/commands/custom/<F10>" -n -t string -s "$TARGET_BIN --convert" 2>/dev/null || true
    xfconf-query -c xfce4-keyboard-shortcuts -p "/commands/custom/<Primary><F10>" -n -t string -s "$TARGET_BIN --convert-and-switch" 2>/dev/null || true
fi

# KDE Plasma setup
if command -v kwriteconfig6 >/dev/null 2>&1; then
    kwriteconfig6 --file kglobalshortcutsrc --group langswitch.desktop --key _launch "F10,none,LangSwitch Convert" 2>/dev/null || true
elif command -v kwriteconfig5 >/dev/null 2>&1; then
    kwriteconfig5 --file kglobalshortcutsrc --group langswitch.desktop --key _launch "F10,none,LangSwitch Convert" 2>/dev/null || true
fi

# 8. Check & Install Helper Tools (Clipboard & Keystroke Simulation)
echo ""
echo -e "${BOLD}🔍 בודק רכיבי מערכת נדרשים (Clipboard & Simulation tools)...${NC}"

PACKAGES_TO_INSTALL=()
if [ "$IS_WAYLAND" = true ]; then
    if ! command -v wl-copy >/dev/null 2>&1; then
        PACKAGES_TO_INSTALL+=("wl-clipboard")
    fi
    if ! command -v wtype >/dev/null 2>&1 && ! command -v ydotool >/dev/null 2>&1; then
        PACKAGES_TO_INSTALL+=("wtype")
    fi
else
    if ! command -v xclip >/dev/null 2>&1; then
        PACKAGES_TO_INSTALL+=("xclip")
    fi
    if ! command -v xdotool >/dev/null 2>&1; then
        PACKAGES_TO_INSTALL+=("xdotool")
    fi
fi

if [ ${#PACKAGES_TO_INSTALL[@]} -ne 0 ]; then
    echo -e "${YELLOW}⚠️  נמצאו חבילות מערכת נדרשות שיש להתקין:${NC} ${PACKAGES_TO_INSTALL[*]}"
    echo -e "${CYAN}מנסה להתקין חבילות באמצעות $PKG_MGR...${NC}"

    INSTALLED=false
    if [ -e /dev/tty ]; then
        case "$PKG_MGR" in
            apt)
                (sudo </dev/tty apt-get update -qq 2>/dev/null || true)
                sudo </dev/tty apt-get install -y "${PACKAGES_TO_INSTALL[@]}" && INSTALLED=true || true
                ;;
            dnf)
                sudo </dev/tty dnf install -y "${PACKAGES_TO_INSTALL[@]}" && INSTALLED=true || true
                ;;
            pacman)
                sudo </dev/tty pacman -S --needed --noconfirm "${PACKAGES_TO_INSTALL[@]}" && INSTALLED=true || true
                ;;
            zypper)
                sudo </dev/tty zypper install -y "${PACKAGES_TO_INSTALL[@]}" && INSTALLED=true || true
                ;;
        esac
    fi

    # Verify if installed
    STILL_MISSING=()
    for pkg in "${PACKAGES_TO_INSTALL[@]}"; do
        if [ "$pkg" = "xclip" ] && ! command -v xclip >/dev/null 2>&1; then STILL_MISSING+=("xclip"); fi
        if [ "$pkg" = "xdotool" ] && ! command -v xdotool >/dev/null 2>&1; then STILL_MISSING+=("xdotool"); fi
        if [ "$pkg" = "wl-clipboard" ] && ! command -v wl-copy >/dev/null 2>&1; then STILL_MISSING+=("wl-clipboard"); fi
        if [ "$pkg" = "wtype" ] && ! command -v wtype >/dev/null 2>&1; then STILL_MISSING+=("wtype"); fi
    done

    if [ ${#STILL_MISSING[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ כל רכיבי המערכת הותקנו בהצלחה!${NC}"
    else
        echo ""
        echo -e "${YELLOW}============================================================${NC}"
        echo -e "${YELLOW}⚠️  שים לב: כדי שהמרת הטקסט עם מקש F10 תעבוד, הרץ בטרמינל:${NC}"
        case "$PKG_MGR" in
            apt)
                echo -e "${BOLD}${CYAN}   sudo apt install -y ${STILL_MISSING[*]}${NC}"
                ;;
            dnf)
                echo -e "${BOLD}${CYAN}   sudo dnf install -y ${STILL_MISSING[*]}${NC}"
                ;;
            pacman)
                echo -e "${BOLD}${CYAN}   sudo pacman -S ${STILL_MISSING[*]}${NC}"
                ;;
            *)
                echo -e "${BOLD}${CYAN}   התקן ידנית: ${STILL_MISSING[*]}${NC}"
                ;;
        esac
        echo -e "${YELLOW}============================================================${NC}"
    fi
else
    echo -e "${GREEN}✅ כל רכיבי המערכת הנדרשים מותקנים!${NC}"
fi

# 9. Ensure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    if [ -f "$HOME/.bashrc" ]; then
        grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bashrc" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    fi
    if [ -f "$HOME/.profile" ]; then
        grep -q 'PATH="$HOME/.local/bin:$PATH"' "$HOME/.profile" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.profile"
    fi
fi

# 10. Send desktop notification if available
if command -v notify-send >/dev/null 2>&1; then
    notify-send -i "$ICON_DIR/langswitch.svg" "⚡ LangSwitch" "ההתקנה הושלמה בהצלחה! מקש F10 מוכן להמרה." 2>/dev/null || true
fi

echo ""
echo -e "${BOLD}${GREEN}"
echo "============================================================"
echo "🎉 ההתקנה הושלמה בהצלחה! (Installation Successful!)"
echo "============================================================"
echo -e "${NC}"
echo -e "מקשי קיצור שהוגדרו במערכת:"
echo -e "  • ${BOLD}F10${NC}       - המרת טקסט והחלפת שפת מקלדת אוטומטית (עברית ↔ אנגלית)"
echo -e "  • ${BOLD}Ctrl+F10${NC}  - המרת טקסט בלבד (ללא החלפת מקלדת)"
echo ""
echo -e "בדיקת תקינות המערכת (Diagnostics):"
"$TARGET_BIN" --status || true
echo ""
echo -e "לפרטים והגדרות נוספות: ${CYAN}langswitch --status${NC}"
echo -e "יומן פעולות דיאגנוסטיקה: ${YELLOW}cat /tmp/langswitch.log${NC}"
echo "תהנה מ-LangSwitch! ⚡"

# 11. Launch the Welcome Screen GUI immediately!
if [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ]; then
    echo ""
    echo -e "${CYAN}🚀 פותח את מסך הפתיחה (Welcome Screen)...${NC}"
    pkill -f langswitch-welcome 2>/dev/null || true
    pkill -f "welcome_linux" 2>/dev/null || true
    sleep 0.5
    if command -v python3 >/dev/null 2>&1 && [ -f "$WELCOME_BIN" ]; then
        python3 "$WELCOME_BIN" >/dev/null 2>&1 &
    else
        "$TARGET_BIN" --welcome >/dev/null 2>&1 &
    fi
fi
