fn main() {
    #[cfg(windows)]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let icon_path = std::path::Path::new(&manifest_dir).join("assets").join("vncharkey.ico");
        let mut res = winres::WindowsResource::new();
        if icon_path.exists() {
            res.set_icon(icon_path.to_str().unwrap());
        } else {
            res.set_icon("assets/vncharkey.ico");
        }
        let _ = res.compile();
    }
}

