#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
LangSwitch - Linux Welcome Screen, Interactive Playground & Resident System Tray
Compatible with Python 3.6+ and GTK 3.0 (Linux Mint, Ubuntu, Fedora, Arch, Debian)
with automatic fallbacks to Tkinter and Zenity.
"""

import os
import sys
import subprocess
import threading

EN_KEYS = "qwertyuiop[]asdfghjkl;'zxcvbnm,./`"
HE_KEYS = "/'קראטוןםפ][שדגכעיחלךף,זסבהנמצתץ.;"

def convert_text(input_str: str):
    """Fast in-memory conversion matching LangSwitch core engine (returns converted, direction)"""
    res = []
    he_count = sum(1 for c in input_str if '\u05d0' <= c <= '\u05ea')
    en_count = sum(1 for c in input_str if c.isascii() and c.isalpha())
    direction = "en" if he_count > en_count else "he"

    for c in input_str:
        if c in EN_KEYS:
            idx = EN_KEYS.index(c)
            res.append(HE_KEYS[idx] if idx < len(HE_KEYS) else c)
        elif c in HE_KEYS:
            idx = HE_KEYS.index(c)
            res.append(EN_KEYS[idx] if idx < len(EN_KEYS) else c)
        elif c in 'QWERTYUIOPASDFGHJKLZXCVBNM':
            idx = EN_KEYS.index(c.lower())
            res.append(HE_KEYS[idx] if idx < len(HE_KEYS) else c)
        else:
            res.append(c)
    return "".join(res), direction

def switch_system_layout():
    """Safely toggles keyboard layout using the user's configured shortcut without altering layout lists"""
    shortcut = "super+space"
    # 1. Detect configured shortcut from Cinnamon xkb-options
    try:
        out = subprocess.check_output(
            ["gsettings", "get", "org.cinnamon.desktop.input-sources", "xkb-options"],
            stderr=subprocess.DEVNULL,
            timeout=1
        ).decode().lower()
        if "alt_shift" in out:
            shortcut = "alt+shift"
        elif "ctrl_shift" in out:
            shortcut = "ctrl+shift"
        elif "super_space" in out or "win_space" in out:
            shortcut = "super+space"
    except Exception:
        pass

    # 2. Simulate the user's layout switch shortcut
    try:
        subprocess.run(["xdotool", "key", "--clearmodifiers", shortcut], stderr=subprocess.DEVNULL)
    except Exception:
        pass

def open_system_keyboard_settings():
    """Opens the desktop environment's keyboard shortcuts settings window"""
    for cmd in ["cinnamon-settings keyboard", "gnome-control-center keyboard", "xfce4-keyboard-settings", "kcmshell5 kcm_keys"]:
        parts = cmd.split()
        try:
            subprocess.Popen(parts, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            return
        except Exception:
            continue

def trigger_langswitch_convert(switch_layout=False):
    """Runs the compiled binary to perform global conversion"""
    bin_path = os.path.expanduser("~/.local/bin/langswitch")
    arg = "--convert-and-switch" if switch_layout else "--convert"
    if os.path.exists(bin_path):
        subprocess.Popen([bin_path, arg])
    else:
        subprocess.Popen(["langswitch", arg])

def run_gtk():
    import gi
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk, Gdk, GLib

    class WelcomeWindow(Gtk.Window):
        def __init__(self, start_minimized=False):
            super().__init__(title="⚡ LangSwitch - ברוכים הבאים!")
            self.set_default_size(530, 680)
            self.set_position(Gtk.WindowPosition.CENTER)
            self.set_resizable(False)

            # Try loading embedded app icon
            self.icon_path = os.path.expanduser("~/.local/share/icons/hicolor/scalable/apps/langswitch.svg")
            if os.path.exists(self.icon_path):
                try:
                    self.set_icon_from_file(self.icon_path)
                except Exception:
                    pass

            # Modern stylesheet
            css_provider = Gtk.CssProvider()
            css_data = b"""
            window {
                background-color: #F4F5FD;
                font-family: 'Segoe UI', 'Ubuntu', 'Cantarell', 'DejaVu Sans', sans-serif;
            }
            .header-box {
                padding: 22px 20px 6px 20px;
            }
            .title-label {
                font-size: 24px;
                font-weight: 800;
                color: #1E1B4B;
            }
            .subtitle-label {
                font-size: 14px;
                color: #6366F1;
                font-weight: 500;
            }
            .card-box {
                background-color: #FFFFFF;
                border: 1px solid #E0E7FF;
                border-radius: 12px;
                padding: 16px;
                margin: 0px 22px;
            }
            .hotkey-badge {
                background: linear-gradient(135deg, #4F46E5, #7C3AED);
                color: #FFFFFF;
                font-weight: bold;
                font-size: 13px;
                padding: 5px 12px;
                border-radius: 8px;
            }
            .hotkey-desc {
                font-size: 13px;
                color: #334155;
            }
            .test-box {
                background-color: #EEF2FF;
                border: 1px solid #C7D2FE;
                border-radius: 12px;
                padding: 16px;
                margin: 10px 22px;
            }
            .test-title {
                font-size: 13px;
                font-weight: bold;
                color: #3730A3;
            }
            .test-entry {
                background-color: #FFFFFF;
                border: 1px solid #CBD5E1;
                border-radius: 8px;
                padding: 8px 12px;
                font-size: 15px;
                color: #0F172A;
            }
            .btn-test {
                background: linear-gradient(135deg, #6366F1, #8B5CF6);
                color: #FFFFFF;
                font-weight: bold;
                font-size: 13px;
                padding: 7px 16px;
                border-radius: 8px;
                border: none;
            }
            .btn-test:hover {
                background: linear-gradient(135deg, #4F46E5, #7C3AED);
            }
            .btn-kbd {
                background-color: #FFFFFF;
                border: 1px solid #C7D2FE;
                color: #4338CA;
                font-weight: 600;
                font-size: 13px;
                padding: 7px 16px;
                border-radius: 8px;
            }
            .btn-kbd:hover {
                background-color: #F8FAFC;
                border-color: #818CF8;
            }
            .btn-start {
                background: linear-gradient(135deg, #4338CA, #6D28D9);
                color: #FFFFFF;
                font-weight: 800;
                font-size: 15px;
                padding: 12px 36px;
                border-radius: 10px;
                border: none;
            }
            .btn-start:hover {
                background: linear-gradient(135deg, #3730A3, #5B21B6);
            }
            .hint-label {
                font-size: 12px;
                color: #64748B;
            }
            """
            css_provider.load_from_data(css_data)
            Gtk.StyleContext.add_provider_for_screen(
                Gdk.Screen.get_default(),
                css_provider,
                Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
            )

            vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
            self.add(vbox)

            # Header
            header = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
            header.get_style_context().add_class("header-box")

            lbl_title = Gtk.Label(label="⚡ LangSwitch מוכנה לשימוש!")
            lbl_title.get_style_context().add_class("title-label")
            lbl_sub = Gtk.Label(label="המרת טקסט מהירה כברק בין עברית לאנגלית")
            lbl_sub.get_style_context().add_class("subtitle-label")

            header.pack_start(lbl_title, False, False, 0)
            header.pack_start(lbl_sub, False, False, 0)
            vbox.pack_start(header, False, False, 0)

            # Shortcuts List Card
            card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
            card.get_style_context().add_class("card-box")

            shortcuts = [
                ("F10", "המרת טקסט + החלפת מקלדת אוטומטית (עברית ↔ אנגלית)"),
                ("Ctrl + F10", "המרת טקסט בלבד (ללא החלפת מקלדת)"),
                ("Shift + Win + G", "חיפוש מיידי במצב AI Mode של Google"),
                ("Ctrl + Shift + T", "תרגום מהיר עברית ↔ אנגלית ב-Google"),
            ]

            for key, desc in shortcuts:
                row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=14)
                badge = Gtk.Label(label=key)
                badge.get_style_context().add_class("hotkey-badge")
                badge.set_size_request(130, 28)

                desc_lbl = Gtk.Label(label=desc)
                desc_lbl.get_style_context().add_class("hotkey-desc")
                desc_lbl.set_xalign(1.0)

                row.pack_start(badge, False, False, 0)
                row.pack_start(desc_lbl, True, True, 0)
                card.pack_start(row, False, False, 0)

            vbox.pack_start(card, False, False, 0)

            # Interactive Test Playground
            test_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
            test_box.get_style_context().add_class("test-box")

            test_title = Gtk.Label(label="🧪 איזור בדיקה חיה (הקלד ולחץ F10 לבדיקת המרה ומקלדת):")
            test_title.set_xalign(1.0)
            test_title.get_style_context().add_class("test-title")
            test_box.pack_start(test_title, False, False, 0)

            entry_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            self.entry = Gtk.Entry()
            self.entry.set_text("akuo")
            self.entry.get_style_context().add_class("test-entry")
            self.entry.connect("key-press-event", self.on_entry_key)

            btn_test = Gtk.Button(label="בדוק המרה ⚡")
            btn_test.get_style_context().add_class("btn-test")
            btn_test.connect("clicked", self.on_test_clicked)

            entry_box.pack_start(self.entry, True, True, 0)
            entry_box.pack_start(btn_test, False, False, 0)
            test_box.pack_start(entry_box, False, False, 0)

            # Status feedback in test playground
            self.lbl_feedback = Gtk.Label(label="הקלד טקסט ולחץ על F10 או על הכפתור כדי לראות את ההמרה ואת החלפת שפת המקלדת.")
            self.lbl_feedback.get_style_context().add_class("hint-label")
            self.lbl_feedback.set_xalign(1.0)
            test_box.pack_start(self.lbl_feedback, False, False, 0)

            vbox.pack_start(test_box, False, False, 0)

            # System shortcut settings button
            opt_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
            opt_box.set_halign(Gtk.Align.CENTER)
            btn_kbd = Gtk.Button(label="⌨️ הגדרות מקלדת וקיצורים במערכת (Cinnamon / Mint)")
            btn_kbd.get_style_context().add_class("btn-kbd")
            btn_kbd.connect("clicked", lambda w: open_system_keyboard_settings())
            opt_box.pack_start(btn_kbd, False, False, 0)
            vbox.pack_start(opt_box, False, False, 0)

            # Start button
            btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
            btn_box.set_halign(Gtk.Align.CENTER)
            btn_start = Gtk.Button(label="🚀 בוא נתחיל! (מזער למגש המערכת)")
            btn_start.get_style_context().add_class("btn-start")
            btn_start.connect("clicked", self.on_start_clicked)
            btn_box.pack_start(btn_start, False, False, 0)
            btn_box.set_margin_bottom(14)
            btn_box.set_margin_top(4)

            vbox.pack_end(btn_box, False, False, 0)

            # Intercept window close ('X') to hide instead of terminate
            self.connect("delete-event", self.on_delete_event)

            # Setup Tray Icon
            self.setup_tray_icon()

            if not start_minimized:
                self.show_all()

        def on_test_clicked(self, widget):
            current = self.entry.get_text()
            converted, direction = convert_text(current)
            self.entry.set_text(converted)
            self.entry.set_position(-1)

            # Switch system keyboard layout!
            switch_system_layout()
            dir_name = "עברית" if direction == "he" else "אנגלית"
            self.lbl_feedback.set_text(f"✅ הטקסט הומר ל-{dir_name} ושפת המקלדת הוחלפה בהצלחה!")

        def on_entry_key(self, widget, event):
            if event.keyval == Gdk.KEY_F10:
                self.on_test_clicked(None)
                return True
            return False

        def on_delete_event(self, widget, event):
            # Hide window to tray instead of quitting
            self.hide()
            return True

        def on_start_clicked(self, widget):
            self.hide()
            try:
                subprocess.Popen([
                    "notify-send",
                    "-i", self.icon_path if os.path.exists(self.icon_path) else "langswitch",
                    "⚡ LangSwitch",
                    "התוכנה פועלת כעת במגש המערכת (System Tray). מקש F10 מוכן להמרה בכל אפליקציה!"
                ], stderr=subprocess.DEVNULL)
            except Exception:
                pass

        def create_tray_menu(self):
            menu = Gtk.Menu()

            # Title / Status
            item_title = Gtk.MenuItem(label="⚡ LangSwitch (פעיל)")
            item_title.set_sensitive(False)
            menu.append(item_title)

            menu.append(Gtk.SeparatorMenuItem())

            # Open Settings
            item_open = Gtk.MenuItem(label="🧪 מסך פתיחה ובדיקה...")
            item_open.connect("activate", lambda w: (self.show_all(), self.present()))
            menu.append(item_open)

            # Convert Now
            item_conv = Gtk.MenuItem(label="🔄 בצע המרה עכשיו (F10)")
            item_conv.connect("activate", lambda w: trigger_langswitch_convert(False))
            menu.append(item_conv)

            # Toggle Layout
            item_toggle = Gtk.MenuItem(label="🌐 החלף שפת מקלדת")
            item_toggle.connect("activate", lambda w: switch_system_layout())
            menu.append(item_toggle)

            # Keyboard settings
            item_kbd = Gtk.MenuItem(label="⌨️ הגדרות מקלדת וקיצורים במערכת")
            item_kbd.connect("activate", lambda w: open_system_keyboard_settings())
            menu.append(item_kbd)

            menu.append(Gtk.SeparatorMenuItem())

            # Exit
            item_quit = Gtk.MenuItem(label="❌ סגור את התוכנה (יציאה)")
            item_quit.connect("activate", lambda w: Gtk.main_quit())
            menu.append(item_quit)

            menu.show_all()
            return menu

        def setup_tray_icon(self):
            # 1. Try AyatanaAppIndicator3 or AppIndicator3
            self.indicator = None
            try:
                gi.require_version('AyatanaAppIndicator3', '0.1')
                from gi.repository import AyatanaAppIndicator3 as appindicator
                self.indicator = appindicator.Indicator.new(
                    "langswitch",
                    self.icon_path if os.path.exists(self.icon_path) else "langswitch",
                    appindicator.IndicatorCategory.APPLICATION_STATUS
                )
            except Exception:
                try:
                    gi.require_version('AppIndicator3', '0.1')
                    from gi.repository import AppIndicator3 as appindicator
                    self.indicator = appindicator.Indicator.new(
                        "langswitch",
                        self.icon_path if os.path.exists(self.icon_path) else "langswitch",
                        appindicator.IndicatorCategory.APPLICATION_STATUS
                    )
                except Exception:
                    pass

            if self.indicator:
                self.indicator.set_status(appindicator.IndicatorStatus.ACTIVE)
                self.indicator.set_menu(self.create_tray_menu())
                return

            # 2. Fallback to Gtk.StatusIcon (built into Gtk 3, native in Mint Cinnamon)
            try:
                self.status_icon = Gtk.StatusIcon()
                if os.path.exists(self.icon_path):
                    self.status_icon.set_from_file(self.icon_path)
                else:
                    self.status_icon.set_from_icon_name("langswitch")
                self.status_icon.set_tooltip_text("⚡ LangSwitch - פעיל (F10 להמרה)")
                self.status_icon.connect("activate", lambda icon: (self.show_all(), self.present()))

                def on_popup(icon, button, time):
                    menu = self.create_tray_menu()
                    menu.popup(None, None, Gtk.StatusIcon.position_menu, icon, button, time)

                self.status_icon.connect("popup-menu", on_popup)
                self.status_icon.set_visible(True)
            except Exception as e:
                print(f"Tray status icon fallback notice: {e}")

    start_minimized = "--tray" in sys.argv
    win = WelcomeWindow(start_minimized=start_minimized)
    Gtk.main()

def run_tkinter():
    import tkinter as tk
    root = tk.Tk()
    root.title("⚡ LangSwitch - ברוכים הבאים!")
    root.geometry("500x560")
    root.configure(bg="#F4F5FD")

    lbl_title = tk.Label(root, text="⚡ LangSwitch מוכנה לשימוש!", font=("Segoe UI", 16, "bold"), bg="#F4F5FD", fg="#1E1B4B")
    lbl_title.pack(pady=(20, 4))
    lbl_sub = tk.Label(root, text="המרת טקסט מהירה כברק בין עברית לאנגלית", font=("Segoe UI", 10), bg="#F4F5FD", fg="#6366F1")
    lbl_sub.pack(pady=(0, 16))

    btn = tk.Button(root, text="הגדרות מקלדת ב-Linux", command=open_system_keyboard_settings, bg="#4338CA", fg="white", font=("Segoe UI", 11, "bold"))
    btn.pack(pady=20)
    root.mainloop()

if __name__ == "__main__":
    try:
        run_gtk()
    except Exception as e:
        print(f"GTK3 unavailable, falling back to Tkinter: {e}")
        try:
            run_tkinter()
        except Exception:
            pass
