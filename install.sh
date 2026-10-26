#!/usr/bin/env bash
# ==============================================================================
# ⚡ LangSwitch - Ultra-friendly Universal Linux Installer
# Supports: Linux Mint, Fedora, Arch Linux, Ubuntu, Debian, Manjaro, openSUSE
# Display Servers: Wayland & X11
# ==============================================================================

set -e

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

# 4. Check & Install Helper Tools
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
    echo -e "${YELLOW}⚠️  נמצאו חבילות מערכת מומלצות שיש להתקין:${NC} ${PACKAGES_TO_INSTALL[*]}"
    echo -e "${CYAN}מתקין חבילות באמצעות $PKG_MGR... (ייתכן שתתבקש להזין סיסמת sudo)${NC}"

    case "$PKG_MGR" in
        apt)
            sudo apt-get update -qq && sudo apt-get install -y -qq "${PACKAGES_TO_INSTALL[@]}"
            ;;
        dnf)
            sudo dnf install -y -q "${PACKAGES_TO_INSTALL[@]}"
            ;;
        pacman)
            sudo pacman -S --needed --noconfirm "${PACKAGES_TO_INSTALL[@]}"
            ;;
        zypper)
            sudo zypper install -y "${PACKAGES_TO_INSTALL[@]}"
            ;;
        *)
            echo -e "${YELLOW}אנא התקן ידנית את החבילות:${NC} ${PACKAGES_TO_INSTALL[*]}"
            ;;
    esac
else
    echo -e "${GREEN}✅ כל רכיבי המערכת הנדרשים מותקנים!${NC}"
fi

# 5. Create local directories
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$INSTALL_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# 6. Download or copy binary
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
        echo -e "${RED}שגיאה בהורדת הקובץ מ-GitHub. מנסה גרסת גיבוי...${NC}"
        exit 1
    }
fi

chmod +x "$TARGET_BIN"
echo -e "${GREEN}✅ הקובץ הבינארי הותקן בהצלחה!${NC}"

# 7. Install Icon
if [ -f "./assets/icon.svg" ]; then
    cp "./assets/icon.svg" "$ICON_DIR/langswitch.svg"
else
    curl -sSL "https://raw.githubusercontent.com/kosherplay-betatester/LangSwitch/main/assets/icon.svg" -o "$ICON_DIR/langswitch.svg" 2>/dev/null || true
fi

# 8. Run in-app automated desktop & shortcut setup
echo ""
echo -e "${BOLD}⚙️  מגדיר קיצורי דרך ותפריט מערכת...${NC}"
"$TARGET_BIN" --install || true

# 9. Ensure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo -e "${YELLOW}💡 טיפ: התיקייה ~/.local/bin אינה מוגדרת כרגע ב-PATH שלך.${NC}"
    echo "   הוספנו תמיכה עבורך לקבצי הפרופיל (~/.bashrc, ~/.zshrc)."
    if [ -f "$HOME/.bashrc" ]; then
        grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.bashrc" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    fi
    if [ -f "$HOME/.zshrc" ]; then
        grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.zshrc" || echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
    fi
fi

echo ""
echo -e "${BOLD}${GREEN}"
echo "============================================================"
echo "🎉 ההתקנה הושלמה בהצלחה! (Installation Successful!)"
echo "============================================================"
echo -e "${NC}"
echo -e "מקשי קיצור ברירת מחדל:"
echo -e "  • ${BOLD}F10${NC}       - המרת טקסט מסומן או שורה נוכחית (עברית ↔ אנגלית)"
echo -e "  • ${BOLD}Ctrl+F10${NC}  - המרת טקסט והחלפת שפת קלט המקלדת"
echo ""
echo -e "לפרטים והגדרות נוספות: ${CYAN}langswitch --status${NC}"
echo "תהנה מ-LangSwitch! ⚡"
