# ⚡ LangSwitch

<div align="center">

**Lightweight, Blazing Fast Hebrew ↔ English Layout Text Switcher**  
*כלי קל משקל, יעיל ומהיר כברק להמרת טקסט שנכתב בשפה הפוכה (עברית ↔ אנגלית)*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20(X11%20%26%20Wayland)-success.svg)]()
[![Rust](https://img.shields.io/badge/Language-Rust%202024-orange.svg)]()

[English](#-english) &nbsp;|&nbsp; [עברית](#-עברית)

</div>

---

## ⚡ Quick Install

### 🐧 Linux (Linux Mint, Fedora, Arch, Ubuntu, Debian)

<div dir="rtl">

העתק והדבק את הפקודה הבאה בטרמינל להתקנה אוטומטית מלאה (כולל אייקון והגדרת קיצורי `F10` ו-`Ctrl+F10` במערכת):

</div>

```bash
curl -sSL https://raw.githubusercontent.com/kosherplay-betatester/LangSwitch/main/install.sh | bash
```

---

### 🪟 Windows (Windows 10 / 11)

<div dir="rtl">

הורד והפעל ישירות את קובץ התוכנה:

</div>

👉 **[Download LangSwitch for Windows (langswitch.exe)](https://github.com/kosherplay-betatester/LangSwitch/releases/latest/download/langswitch.exe)**

<div dir="rtl">

או באמצעות PowerShell:

</div>

```powershell
irm https://github.com/kosherplay-betatester/LangSwitch/releases/latest/download/langswitch.exe -OutFile "$env:TEMP\langswitch.exe"; & "$env:TEMP\langswitch.exe"
```

---

## 🌐 English

### Overview
**LangSwitch** is an ultra-fast, zero-polling, native utility written in pure **Rust**. Inspired by the classic **LangOver**, it instantly converts accidentally mistyped text between Hebrew and English with a single keypress (default: `F10`), with full support for both **Windows** (10/11) and **Linux** (**X11 & Wayland** across Linux Mint, Fedora, Arch Linux, Ubuntu, etc.).

### ✨ Features
* **Primary Hotkey (`F10`)**:
  * Converts mistyped text **AND** automatically switches keyboard input layout directly!
  * **Smart Auto-Line Selection**: If no text is selected, it automatically selects the current line (using `Home` + `Shift+End`) and converts it.
* **Secondary Hotkey (`Ctrl+F10`)**:
  * Converts mistyped text **ONLY** (without switching keyboard input layout).
* **🔍 Google AI Search Hotkey (`Shift+Win+G`)**:
  * Highlights or takes the current text and opens a new browser tab with Google's dedicated AI Mode search (`https://www.google.com/search?q={query}&udm=50`).
* **Direct Layout Switching & Multi-Layout Support**:
  * Directly switches to the target Hebrew or English layout in 0.001ms via native Win32 messages — no Alt-key menu bar pauses.
  * Accurately distinguishes between duplicate variants (e.g., English US vs. UK or multiple Hebrew layouts) without random cycling.
* **Smart Letter Case Normalization**:
  * Typing uppercase or mixed English (`YexY` or `YEXY`) automatically converts to lowercase Hebrew (`טקסט`).
  * Converting Hebrew (`אקסא`) to English produces clean lowercase (`text`).
* **Windows Shortcut Conflict Prevention**:
  * In the Windows Settings GUI, dangerous combinations (such as `Win+Shift+S`, `Win+L`, `Alt+F4`, `Ctrl+C`) are dynamically filtered to prevent conflicting with Windows system shortcuts.
* **Smart Installation with Conditional Admin Elevation**:
  * First-run setup automatically creates Start Menu shortcuts and registers LangSwitch in Windows "Installed Apps" / Control Panel (`--uninstall` supported).
  * Gracefully prompts to close any running instance before install or update.
* **0.00% Idle CPU & Minimal RAM**:
  * Event-driven Win32 loop on Windows; lightweight CLI triggers or daemon on Linux. Zero polling loops.
* **Full Bilingual Support & Modern UI**:
  * Clean, eye-friendly soft-slate settings interface and tray menus available in both **Hebrew** and **English**.

### 🐧 Linux Compatibility (X11 & Wayland)
LangSwitch works out-of-the-box on all major Linux distributions:
* **Linux Mint** (Cinnamon / MATE / XFCE - X11)
* **Fedora** (GNOME - Wayland)
* **Arch Linux** (KDE Plasma 6, Hyprland, Sway, i3)
* **Ubuntu / Debian** (GNOME Wayland & X11)

#### 🚀 Automated Installation:
```bash
curl -sSL https://raw.githubusercontent.com/kosherplay-betatester/LangSwitch/main/install.sh | bash
```

#### Linux CLI Commands:
```bash
# Convert selected text or current line
langswitch --convert

# Convert text and switch keyboard layout
langswitch --convert-and-switch

# Diagnose environment (Display server, installed clipboard/keystroke tools)
langswitch --status

# Automatically install desktop icon and configure system shortcuts
langswitch --install
```

#### Recommended Tools:
* **Wayland**: `wl-clipboard` (for clipboard) and `wtype` or `ydotool` (for simulated keystrokes).
* **X11**: `xclip` and `xdotool`.

---

<div dir="rtl">

## 🇮🇱 עברית

### סקירה כללית
**LangSwitch** הוא כלי קל משקל ומהיר כברק שנכתב ב-**Rust** טהור. בהשראת התוכנה האהובה **LangOver**, הוא ממיר באופן מיידי טקסט שנכתב בטעות בשפה ההפוכה (עברית ↔ אנגלית) בלחיצת מקש אחת (ברירת מחדל: `F10`), עם תמיכה מלאה ב-**Windows** (10/11) ו-**Linux** (תמיכה מלאה הן ב-**X11** והן ב-**Wayland** ב-Linux Mint, Fedora, Arch Linux, Ubuntu ועוד).

### ✨ תכונות עיקריות
* **המרה מהירה בלחיצת מקש ראשית (`F10`)**:
  * ממיר את הטקסט שנכתב בטעות **ומחליף ישירות את שפת המקלדת** במהירות הבזק (0.001ms) ישירות דרך הודעות Win32 טבעיות – ללא השהיות תפריטים.
  * **בחירת שורה אוטומטית חכמה**: אם אין טקסט מסומן, האפליקציה בוחרת אוטומטית את השורה עליה עומד הסמן וממירה אותה (באמצעות `Home` + `Shift+End`).
* **מקש קיצור משני ייעודי (`Ctrl+F10`)**:
  * ממיר את הטקסט **בלבד** (ללא החלפת שפת המקלדת).
* **🔍 חיפוש ישיר ב-Google AI Overviews (`Shift+Win+G`)**:
  * מעתיק את הטקסט המסומן (או השורה הנוכחית) ופותח מיד טאב חדש בדפדפן המוגדר כברירת מחדל עם שאילתת חיפוש במצב ה-AI הייעודי של גוגל (`https://www.google.com/search?q={שאילתה}&udm=50`).
* **החלפת שפה ישירה ותמיכה בריבוי פריסות מקלדת**:
  * מזהה במדויק ופונה ישירות לפריסה המבוקשת (עברית או אנגלית) ללא דפדוף עיוור, גם אם מותקנות מספר פריסות אנגלית (למשל US ו-UK) או מספר פריסות עברית.
* **טיפול חכם באותיות גדולות (Case Normalization)**:
  * הקלדת אנגלית באותיות גדולות (`YexY` או `YEXY`) מומרת תחילה ל-lowercase (`yexy`) ואז לעברית (`טקסט`).
  * המרה מעברית (`אקסא`) לאנגלית מייצרת תמיד אותיות קטנות (`text`).
* **מניעת התנגשויות קיצורים ב-Windows**:
  * תפריט בחירת המקשים מסנן באופן דינמי קיצורי מערכת של Windows (כגון `Win+Shift+S` ל-Snipping Tool, `Win+L`, `Alt+F4` ועוד).
* **ממשק הגדרות מודרני בעיצוב נעים ורך**:
  * חלון הגדרות מודרני בצבעוניות Slate אלגנטית, מותאם לעיניים ומאפשר התאמה מלאה של כל שלושת קיצורי הדרך.
* **התקנה חכמה ובקשת הרשאות מנהל רק במידת הצורך**:
  * בהפעלה ראשונה מוצעת התקנה מסודרת, יצירת קיצור דרך בתפריט התחל ורישום מלא ב"אפליקציות מותקנות" / לוח הבקרה של Windows (`--uninstall`).
  * **בקשת הרשאות מנהל (UAC Administrator) מתבצעת רק אם יש צורך אמיתי** (כגון העתקה ל-`Program Files`), ללא חלונות אישור מיותרים.
* **0.00% ניצול מעבד וצריכת זיכרון אפסית**:
  * מבוסס אירועים (Event-driven) ישירות מול מערכת ההפעלה, ללא לולאות polling ברקע.
* **ממשק דו-לשוני מלא**:
  * חלון ההגדרות, חלון הפתיחה ותפריט מגש המערכת (Tray) זמינים בעברית ובאנגלית.

### 🐧 תמיכה בלינוקס (X11 ו-Wayland)
LangSwitch פועלת באופן חלק בכל הפצות הלינוקס הפופולריות:
* **Linux Mint** (Cinnamon / MATE / XFCE - X11)
* **Fedora** (GNOME - Wayland)
* **Arch Linux** (KDE Plasma 6, Hyprland, Sway, i3)
* **Ubuntu / Debian** (GNOME Wayland & X11)

#### מדוע בלינוקס אין צורך בסינון קיצורים קשיח מראש?
בניגוד ל-Windows שבה קיצורי הדרך אחידים וקבועים בכל המחשבים, בלינוקס לכל הפצה ולכל סביבת שולחן עבודה (GNOME, Cinnamon, KDE, Hyprland) יש קיצורי מערכת שונים לחלוטין הניתנים לשינוי מלא על ידי המשתמש.  
מכיוון שסביבות הלינוקס מנהלות את הקיצורים באופן עצמאי, **מערכת ההפעלה עצמה מזהירה ומתריעה בזמן אמת** אם נבחר מקש קיצור שכבר תפוס על ידי יישום אחר.

#### 🚀 התקנה אוטומטית וידידותית בפקודה אחת:
הרץ פקודה זו בטרמינל כדי להתקין את LangSwitch, להטמיע את האייקון בתפריט ולהגדיר אוטומטית את קיצורי הדרך `F10` ו-`Ctrl+F10` ב-GNOME וב-Cinnamon:

<div dir="ltr">

```bash
curl -sSL https://raw.githubusercontent.com/kosherplay-betatester/LangSwitch/main/install.sh | bash
```

</div>

#### פקודות שימוש בלינוקס:

<div dir="ltr">

```bash
# Convert selected text or current line
langswitch --convert

# Convert text and switch keyboard layout
langswitch --convert-and-switch

# Check environment diagnostics
langswitch --status

# Install desktop icon and configure shortcuts
langswitch --install
```

</div>

</div>

---

## 🛠️ Build from Source / בנייה מקוד מקור

### Requirements / דרישות:
* **Rust 1.80+** (Edition 2024)
* **Windows**: `x86_64-pc-windows-msvc`
* **Linux**: `x86_64-unknown-linux-gnu`

### Compilation / קומפילציה:
```bash
cargo build --release
```
הקובץ הבינארי ייווצר בנתיב `target/release/langswitch.exe` (ב-Windows) או `target/release/langswitch` (ב-Linux).

---

## 📄 License / רישיון
MIT License. Open source for the community.  
קוד פתוח תחת רישיון MIT.
