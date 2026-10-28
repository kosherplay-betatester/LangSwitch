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

# 7. Run in-app automated desktop & shortcut setup
echo ""
echo -e "${BOLD}⚙️  מגדיר קיצורי דרך ותפריט מערכת...${NC}"
"$TARGET_BIN" --install || true

if command -v gsettings >/dev/null 2>&1; then
    # Disable GTK F10 menubar accelerator so F10 works globally without conflict
    gsettings set org.cinnamon.desktop.interface menubar-accel "" 2>/dev/null || true
    gsettings set org.gnome.desktop.interface menubar-accel "" 2>/dev/null || true
    gsettings set org.mate.interface menubar-accel "" 2>/dev/null || true

    # Direct setup for Cinnamon if detected
    if [[ "$DE" =~ [Cc]innamon ]]; then
        CINNAMON_LIST=$(gsettings get org.cinnamon.desktop.keybindings custom-list 2>/dev/null || echo "[]")
        if [[ "$CINNAMON_LIST" != *"custom0"* ]]; then
            if [[ "$CINNAMON_LIST" == "[]" || "$CINNAMON_LIST" == "@as []" ]]; then
                NEW_CINNAMON="['custom0', 'custom1', 'langswitch0', 'langswitch1']"
            else
                NEW_CINNAMON=$(echo "$CINNAMON_LIST" | sed "s/]/, 'custom0', 'custom1', 'langswitch0', 'langswitch1']/")
            fi
            gsettings set org.cinnamon.desktop.keybindings custom-list "$NEW_CINNAMON" 2>/dev/null || true
        fi

        # Register both standard Cinnamon custom0/custom1 and fallback langswitch0/langswitch1
        for slot in custom0 langswitch0; do
            gsettings set org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/$slot/ name "LangSwitch Convert" 2>/dev/null || true
            gsettings set org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/$slot/ command "$TARGET_BIN --convert" 2>/dev/null || true
            gsettings set org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/$slot/ binding "['F10']" 2>/dev/null || true
        done

        for slot in custom1 langswitch1; do
            gsettings set org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/$slot/ name "LangSwitch Convert & Switch" 2>/dev/null || true
            gsettings set org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/$slot/ command "$TARGET_BIN --convert-and-switch" 2>/dev/null || true
            gsettings set org.cinnamon.desktop.keybindings.custom-keybinding:/org/cinnamon/desktop/keybindings/custom-keybindings/$slot/ binding "['<Control>F10']" 2>/dev/null || true
        done

        # Also register in GNOME media-keys (recognized by older Cinnamon & MATE)
        P1="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-convert/"
        P2="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/langswitch-switch/"
        gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['$P1', '$P2']" 2>/dev/null || true
        gsettings set "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$P1" name "LangSwitch Convert" 2>/dev/null || true
        gsettings set "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$P1" command "$TARGET_BIN --convert" 2>/dev/null || true
        gsettings set "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$P1" binding "F10" 2>/dev/null || true
        gsettings set "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$P2" name "LangSwitch Convert & Switch" 2>/dev/null || true
        gsettings set "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$P2" command "$TARGET_BIN --convert-and-switch" 2>/dev/null || true
        gsettings set "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$P2" binding "<Control>F10" 2>/dev/null || true
    fi

    # MATE setup
    if [[ "$DE" =~ [Mm][Aa][Tt][Ee] ]]; then
        MP1="/org/mate/settings-daemon/plugins/media-keys/custom-keybindings/custom0/"
        MP2="/org/mate/settings-daemon/plugins/media-keys/custom-keybindings/custom1/"
        gsettings set org.mate.SettingsDaemon.plugins.media-keys custom-keybindings "['$MP1', '$MP2']" 2>/dev/null || true
        gsettings set "org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:$MP1" name "LangSwitch Convert" 2>/dev/null || true
        gsettings set "org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:$MP1" command "$TARGET_BIN --convert" 2>/dev/null || true
        gsettings set "org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:$MP1" binding "F10" 2>/dev/null || true
        gsettings set "org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:$MP2" name "LangSwitch Convert & Switch" 2>/dev/null || true
        gsettings set "org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:$MP2" command "$TARGET_BIN --convert-and-switch" 2>/dev/null || true
        gsettings set "org.mate.SettingsDaemon.plugins.media-keys.custom-keybinding:$MP2" binding "<Control>F10" 2>/dev/null || true
    fi
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
                sudo </dev/tty apt-get update -qq && sudo </dev/tty apt-get install -y "${PACKAGES_TO_INSTALL[@]}" && INSTALLED=true || true
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
                echo -e "${BOLD}${CYAN}   sudo apt update && sudo apt install -y ${STILL_MISSING[*]}${NC}"
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
echo -e "  • ${BOLD}F10${NC}       - המרת טקסט מסומן או שורה נוכחית (עברית ↔ אנגלית)"
echo -e "  • ${BOLD}Ctrl+F10${NC}  - המרת טקסט והחלפת שפת קלט המקלדת"
echo ""
echo -e "בדיקת תקינות המערכת (Diagnostics):"
"$TARGET_BIN" --status || true
echo ""
echo -e "לפרטים והגדרות נוספות: ${CYAN}langswitch --status${NC}"
echo "תהנה מ-LangSwitch! ⚡"

# 11. Launch the Welcome Screen GUI immediately!
if [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ]; then
    echo ""
    echo -e "${CYAN}🚀 פותח את מסך הפתיחה (Welcome Screen)...${NC}"
    if command -v python3 >/dev/null 2>&1 && [ -f "$WELCOME_BIN" ]; then
        python3 "$WELCOME_BIN" >/dev/null 2>&1 &
    else
        "$TARGET_BIN" --welcome >/dev/null 2>&1 &
    fi
fi
