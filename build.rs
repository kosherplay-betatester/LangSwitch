fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "LangSwitch");
        res.set("FileDescription", "LangSwitch - Hebrew/English Text Layout Switcher");
        res.set("LegalCopyright", "Copyright (C) 2026 kosherplay-betatester");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resource: {}", e);
        }
    }
}
