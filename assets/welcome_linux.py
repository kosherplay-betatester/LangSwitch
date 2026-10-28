#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
LangSwitch - Linux Welcome Screen & Interactive Playground
Compatible with Python 3.6+ and GTK 3.0 (Linux Mint, Ubuntu, Fedora, Arch, Debian)
with automatic fallbacks to Tkinter and Zenity.
"""

import os
import sys
import subprocess

EN_KEYS = "qwertyuiop[]asdfghjkl;'zxcvbnm,./`"
HE_KEYS = "/'קראטוןםפ][שדגכעיחלךף,זסבהנמצתץ.;"

def convert_text(input_str: str) -> str:
    """Fast in-memory conversion matching LangSwitch core engine"""
    res = []
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
    return "".join(res)

def run_gtk():
    import gi
    gi.require_version('Gtk', '3.0')
    from gi.repository import Gtk, Gdk

    class WelcomeWindow(Gtk.Window):
        def __init__(self):
            super().__init__(title="⚡ LangSwitch - ברוכים הבאים!")
            self.set_default_size(530, 650)
            self.set_position(Gtk.WindowPosition.CENTER)
            self.set_resizable(False)

            # Try loading embedded app icon
            icon_path = os.path.expanduser("~/.local/share/icons/hicolor/scalable/apps/langswitch.svg")
            if os.path.exists(icon_path):
                try:
                    self.set_icon_from_file(icon_path)
                except Exception:
                    pass

            # Modern stylesheet (Soft slate, lavender/periwinkle and purple gradients)
            css_provider = Gtk.CssProvider()
            css_data = b"""
            window {
                background-color: #F4F5FD;
                font-family: 'Segoe UI', 'Ubuntu', 'Cantarell', 'DejaVu Sans', sans-serif;
            }
            .header-box {
                padding: 24px 20px 8px 20px;
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
                padding: 18px;
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
                margin: 14px 22px;
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
            """
            css_provider.load_from_data(css_data)
            Gtk.StyleContext.add_provider_for_screen(
                Gdk.Screen.get_default(),
                css_provider,
                Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
            )

            vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
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
            card = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=11)
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
                badge.set_size_request(130, 30)

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

            test_title = Gtk.Label(label="🧪 איזור בדיקה חיה (נסה עכשיו!):")
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

            vbox.pack_start(test_box, False, False, 0)

            # Start button
            btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
            btn_box.set_halign(Gtk.Align.CENTER)
            btn_start = Gtk.Button(label="🚀 בוא נתחיל!")
            btn_start.get_style_context().add_class("btn-start")
            btn_start.connect("clicked", self.on_start_clicked)
            btn_box.pack_start(btn_start, False, False, 0)
            btn_box.set_margin_bottom(16)

            vbox.pack_end(btn_box, False, False, 0)

            self.connect("destroy", Gtk.main_quit)
            self.show_all()

        def on_test_clicked(self, widget):
            current = self.entry.get_text()
            converted = convert_text(current)
            self.entry.set_text(converted)
            self.entry.set_position(-1)

        def on_entry_key(self, widget, event):
            if event.keyval == Gdk.KEY_F10:
                self.on_test_clicked(None)
                return True
            return False

        def on_start_clicked(self, widget):
            Gtk.main_quit()

    win = WelcomeWindow()
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
    lbl_sub.pack(pady=(0, 14))

    frame = tk.Frame(root, bg="white", padx=16, pady=16, relief="solid", bd=1)
    frame.pack(fill="x", padx=20, pady=5)

    shortcuts = [
        ("F10", "המרת טקסט + החלפת מקלדת אוטומטית"),
        ("Ctrl + F10", "המרת טקסט בלבד"),
        ("Shift + Win + G", "חיפוש Google AI Mode"),
        ("Ctrl + Shift + T", "תרגום מהיר עברית ↔ אנגלית ב-Google"),
    ]
    for key, desc in shortcuts:
        row = tk.Frame(frame, bg="white")
        row.pack(fill="x", pady=4)
        tk.Label(row, text=key, font=("Segoe UI", 10, "bold"), bg="#6366F1", fg="white", width=14, padx=5, pady=3).pack(side="left")
        tk.Label(row, text=desc, font=("Segoe UI", 9), bg="white", fg="#334155").pack(side="left", padx=8)

    test_frame = tk.Frame(root, bg="#EEF2FF", padx=12, pady=12)
    test_frame.pack(fill="x", padx=20, pady=15)
    tk.Label(test_frame, text="🧪 נסה להקליד מילה (akuo) וללחוץ F10:", font=("Segoe UI", 9, "bold"), bg="#EEF2FF", fg="#3730A3").pack(anchor="w")
    entry = tk.Entry(test_frame, font=("Segoe UI", 11))
    entry.insert(0, "akuo")
    entry.pack(fill="x", pady=6)

    def do_convert(ev=None):
        t = entry.get()
        entry.delete(0, tk.END)
        entry.insert(0, convert_text(t))
        return "break"

    entry.bind("<F10>", do_convert)
    btn_conv = tk.Button(test_frame, text="בדוק המרה ⚡", command=do_convert, bg="#8B5CF6", fg="white", font=("Segoe UI", 9, "bold"), padx=10, pady=3)
    btn_conv.pack(anchor="e")

    btn_start = tk.Button(root, text="🚀 בוא נתחיל!", command=root.destroy, bg="#4338CA", fg="white", font=("Segoe UI", 12, "bold"), padx=24, pady=8)
    btn_start.pack(pady=12)

    root.mainloop()

def run_zenity():
    msg = (
        "⚡ LangSwitch מוכנה לשימוש!\n\n"
        "• F10: המרת טקסט והחלפת מקלדת אוטומטית\n"
        "• Ctrl + F10: המרת טקסט בלבד\n"
        "• Shift + Win + G: חיפוש Google AI Mode\n"
        "• Ctrl + Shift + T: תרגום מהיר עברית ↔ אנגלית\n\n"
        "התוכנה פועלת במערכת ללא צריכת משאבים כלל!"
    )
    subprocess.run(["zenity", "--info", "--title=LangSwitch", "--text=" + msg, "--width=420"])

def main():
    try:
        run_gtk()
    except Exception:
        try:
            run_tkinter()
        except Exception:
            try:
                run_zenity()
            except Exception:
                pass

if __name__ == "__main__":
    main()
