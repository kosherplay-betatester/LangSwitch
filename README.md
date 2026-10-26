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

העתק והדבק את הפקודה הבאה בטרמינל להתקנה אוטומטית מלאה (כולל אייקון והגדרת מקש `F10` במערכת):

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
* **Instant Hotkey Conversion (`F10`)**:
  * Converts highlighted text (word, sentence, paragraph, or full text).
  * **Smart Auto-Line Selection**: If no text is selected, it automatically selects the current line (similar to mouse triple-click or `Home` + `Shift+End`) and converts it.
* **Smart Letter Case Normalization**:
  * Typing uppercase or mixed English (`YexY` or `YEXY`) automatically converts to lowercase Hebrew (`טקסט`).
  * Converting Hebrew (`אקסא`) to English produces clean lowercase (`text`).
* **Keyboard Layout Auto-Switching**:
  * Configurable option to switch input language automatically upon conversion (`Alt+Shift`).
  * **Dedicated Secondary Hotkey (`Shift+F10`)**: Always converts text **and** toggles the layout regardless of the global setting.
* **Windows Shortcut Conflict Prevention**:
  * In the Windows Settings GUI, dangerous combinations (such as `Win+Shift+S`, `Win+L`, `Alt+F4`, `Ctrl+C`) are dynamically filtered to prevent conflicting with Windows system shortcuts.
* **Smart Installation with Conditional Admin Elevation**:
  * First-run setup automatically creates Start Menu shortcuts and registers LangSwitch in Windows "Installed Apps" / Control Panel (`--uninstall` supported).
  * **Elevates as Admin only when strictly necessary** (e.g. copying into `Program Files`); otherwise installs cleanly without intrusive UAC prompts.
* **0.00% Idle CPU & Minimal RAM**:
  * Event-driven Win32 loop on Windows; lightweight CLI triggers or daemon on Linux. Zero polling loops.
* **Full Bilingual Support**:
  * Settings dialog and tray menus fully available in both **Hebrew** and **English**.

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
* **המרה מהירה בלחיצת מקש (`F10`)**:
  * המרת טקסט מסומן (מילה, משפט, פסקה או הכל).
  * **בחירת שורה אוטומטית חכמה**: אם אין טקסט מסומן, האפליקציה בוחרת אוטומטית את השורה עליה עומד הסמן וממירה אותה (בדומה ל-3 קליקים בעכבר או `Home` + `Shift+End`).
* **טיפול חכם באותיות גדולות (Case Normalization)**:
  * הקלדת אנגלית באותיות גדולות (`YexY` או `YEXY`) מומרת תחילה ל-lowercase (`yexy`) ואז לעברית (`טקסט`).
  * המרה מעברית (`אקסא`) לאנגלית מייצרת תמיד אותיות קטנות (`text`).
* **החלפת שפת מקלדת אוטומטית**:
  * אפשרות בהגדרות להחלפת שפת הקלט אוטומטית בעת המרה (`Alt+Shift`).
  * **מקש קיצור משני ייעודי (`Shift+F10`)**: ממיר את הטקסט **וגם מחליף את שפת המקלדת תמיד**, גם אם ההגדרה האוטומטית כבויה.
* **מניעת התנגשויות קיצורים ב-Windows**:
  * תפריט בחירת המקשים מסנן באופן דינמי קיצורי מערכת של Windows (כגון `Win+Shift+S` ל-Snipping Tool, `Win+L`, `Alt+F4` ועוד).
* **התקנה חכמה ובקשת הרשאות מנהל רק במידת הצורך**:
  * בהפעלה ראשונה מוצעת התקנה מסודרת, יצירת קיצור דרך בתפריט התחל ורישום מלא ב"אפליקציות מותקנות" / לוח הבקרה של Windows (`--uninstall`).
  * **בקשת הרשאות מנהל (UAC Administrator) מתבצעת רק אם יש צורך אמיתי** (כגון העתקה ל-`Program Files`), ללא חלונות אישור מיותרים.
* **0.00% ניצול מעבד וצריכת זיכרון אפסית**:
  * מבוסס אירועים (Event-driven) ישירות מול מערכת ההפעלה, ללא לולאות polling ברקע.
* **ממשק דו-לשוני מלא**:
  * חלון ההגדרות ותפריט מגש המערכת (Tray) זמינים בעברית ובאנגלית.

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
הרץ פקודה זו בטרמינל כדי להתקין את LangSwitch, להטמיע את האייקון בתפריט ולהגדיר אוטומטית את קיצורי הדרך `F10` ו-`Shift+F10` ב-GNOME וב-Cinnamon:

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
