# ⚡ LangSwitch

<div align="center">

**Lightweight, Blazing Fast Hebrew ↔ English Layout Text Switcher for Windows**  
*כלי קל משקל, יעיל ומהיר כברק להמרת טקסט שנכתב בשפה הפוכה (עברית ↔ אנגלית) עבור Windows*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0078D4.svg)]()
[![Rust](https://img.shields.io/badge/Language-Rust%202024-orange.svg)]()

[English](#-english) &nbsp;|&nbsp; [עברית](#-עברית)

</div>

---

## ⚡ Quick Install (Windows 10 / 11)

<div dir="rtl">

הורד והפעל ישירות את קובץ התוכנה:

</div>

👉 **[Download LangSwitch for Windows (langswitch.exe)](https://github.com/kosherplay-betatester/LangSwitch/releases/latest/download/langswitch.exe)**

<div dir="rtl">

או להורדה והפעלה ישירה באמצעות PowerShell:

</div>

```powershell
irm https://github.com/kosherplay-betatester/LangSwitch/releases/latest/download/langswitch.exe -OutFile "$env:TEMP\langswitch.exe"; & "$env:TEMP\langswitch.exe"
```

---

## 🌐 English

### Overview
**LangSwitch** is an ultra-fast, zero-polling, native Windows utility written in pure **Rust**. Inspired by the classic **LangOver**, it instantly converts accidentally mistyped text between Hebrew and English with a single keypress (default: `F10`), featuring a modern Light Blue ↔ Light Purple gradient UI and native Win32 integration for **Windows 10 & Windows 11**.

### ✨ Features
* **Primary Hotkey (`F10`)**:
  * Converts mistyped text **AND** automatically switches keyboard input layout directly!
  * **Smart Auto-Line Selection**: If no text is selected, it automatically selects the current line and converts it.
* **Secondary Hotkey (`Ctrl+F10`)**:
  * Converts mistyped text **ONLY** (without switching keyboard input layout).
* **🔍 Google AI Search Hotkey (`Shift+Win+G`)**:
  * Takes the highlighted text (or current line) and opens a new browser tab with Google's dedicated AI Mode search (`https://www.google.com/search?q={query}&udm=50`).
* **🌐 Google Quick Translation Hotkey (`Ctrl+Shift+T`)**:
  * Translates text instantly between Hebrew and English via Google in your default browser:
    * Hebrew text (e.g. `בית`) translates to English: opens query `בית באנגלית`.
    * English text (e.g. `house`) translates to Hebrew: opens query `house בעברית`.
    * Automatically displays Google's rich translation and dictionary card with audio and definitions at the top!
* **Direct Layout Switching & Multi-Layout Support**:
  * Directly switches to the target Hebrew or English layout via native Win32 messages — no Alt-key menu bar pauses.
  * Accurately distinguishes between duplicate variants (e.g., English US vs. UK or multiple Hebrew layouts) without random cycling.
* **Smart Letter Case Normalization**:
  * Typing uppercase or mixed English (`YexY` or `YEXY`) automatically converts to lowercase Hebrew (`טקסט`).
  * Converting Hebrew (`אקסא`) to English produces clean lowercase (`text`).
* **Windows Shortcut Conflict Prevention**:
  * In the Settings GUI, system combinations (such as `Win+Shift+S`, `Win+L`, `Alt+F4`, `Ctrl+C`) are dynamically filtered to prevent conflicting with Windows system shortcuts.
* **Modern Light Blue ↔ Light Purple Gradient UI**:
  * Custom double-buffered Win32 interface with gradient card borders, pill modifier selectors, and modern custom dialogs across the installer, welcome screen, and settings window.
* **Smart Installation with Conditional Admin Elevation**:
  * First-run setup creates Start Menu shortcuts and registers LangSwitch in Windows "Installed Apps" / Control Panel (`--uninstall` supported).
* **0.00% Idle CPU & Minimal RAM**:
  * Pure event-driven Win32 loop with zero background polling.

---

<div dir="rtl">

## 🇮🇱 עברית

### סקירה כללית
**LangSwitch** הוא כלי קל משקל ומהיר כברק שנכתב ב-**Rust** טהור עבור **Windows 10 / 11**. בהשראת התוכנה האהובה **LangOver**, הוא ממיר באופן מיידי טקסט שנכתב בטעות בשפה ההפוכה (עברית ↔ אנגלית) בלחיצת מקש אחת (ברירת מחדל: `F10`), בעיצוב מודרני ונקי בגווני תכלת-סגול (Light Blue ↔ Light Purple Gradient).

### ✨ תכונות עיקריות
* **המרה מהירה בלחיצת מקש ראשית (`F10`)**:
  * ממיר את הטקסט שנכתב בטעות **ומחליף ישירות את שפת המקלדת** ישירות דרך הודעות Win32 טבעיות – ללא השהיות תפריטים.
  * **בחירת שורה אוטומטית חכמה**: אם אין טקסט מסומן, האפליקציה בוחרת אוטומטית את השורה עליה עומד הסמן וממירה אותה.
* **מקש קיצור משני ייעודי (`Ctrl+F10`)**:
  * ממיר את הטקסט **בלבד** (ללא החלפת שפת המקלדת).
* **🔍 חיפוש ישיר ב-Google AI Mode (`Shift+Win+G`)**:
  * לוקח את הטקסט המסומן (או השורה הנוכחית) ופותח מיד טאב חדש בדפדפן עם שאילתת חיפוש במצב ה-AI הייעודי של גוגל (`https://www.google.com/search?q={שאילתה}&udm=50`).
* **🌐 תרגום מהיר בגוגל בלחיצת מקש (`Ctrl+Shift+T`)**:
  * מתרגם באופן מיידי טקסט בין עברית לאנגלית באמצעות גוגל בדפדפן:
    * טקסט בעברית (למשל `בית`): נפתח חיפוש גוגל עם השאילתה `בית באנגלית`.
    * טקסט באנגלית (למשל `house`): נפתח חיפוש גוגל עם השאילתה `house בעברית`.
    * מציג מיד בראש הדף את כרטיסיית התרגום הרשמית של גוגל, כולל הגייה קולית, פירושים ומילים נרדפות!
* **החלפת שפה ישירה ותמיכה בריבוי פריסות מקלדת**:
  * מזהה במדויק ופונה ישירות לפריסה המבוקשת (עברית או אנגלית) ללא דפדוף עיוור, גם אם מותקנות מספר פריסות אנגלית (למשל US ו-UK) או מספר פריסות עברית.
* **טיפול חכם באותיות גדולות (Case Normalization)**:
  * הקלדת אנגלית באותיות גדולות (`YexY` או `YEXY`) מומרת תחילה ל-lowercase (`yexy`) ואז לעברית (`טקסט`).
  * המרה מעברית (`אקסא`) לאנגלית מייצרת תמיד אותיות קטנות (`text`).
* **מניעת התנגשויות קיצורים ב-Windows**:
  * תפריט בחירת המקשים מסנן באופן דינמי קיצורי מערכת של Windows (כגון `Win+Shift+S` ל-Snipping Tool, `Win+L`, `Alt+F4` ועוד).
* **ממשק מודרני בגווני תכלת-סגול (Gradient UI)**:
  * חלון הגדרות, מסך פתיחה, מתקין וחלונות התראה בעיצוב מודרני עם מסגרות מדורגות (Light Blue ↔ Light Purple) ותמיכה דו-לשונית מלאה בעברית ובאנגלית.
* **התקנה חכמה ורישום מלא ב-Windows**:
  * בהפעלה ראשונה מוצעת התקנה מסודרת, יצירת קיצור דרך בתפריט התחל ורישום מלא ב"אפליקציות מותקנות" של Windows (`--uninstall`).
* **0.00% ניצול מעבד וצריכת זיכרון אפסית**:
  * מבוסס אירועים (Event-driven) ישירות מול מערכת ההפעלה, ללא לולאות polling ברקע.

</div>

---

## 🛠️ Build from Source / בנייה מקוד מקור

### Requirements / דרישות:
* **Rust 1.80+** (Edition 2024)
* **Target**: `x86_64-pc-windows-msvc`

### Compilation / קומפילציה:
```powershell
cargo build --release
```
הקובץ הבינארי ייווצר בנתיב `target/release/langswitch.exe`.

---

## 📄 License / רישיון
MIT License. Open source for the community.  
קוד פתוח תחת רישיון MIT.
