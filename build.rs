fn main() {
    #[cfg(windows)]
    {
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
