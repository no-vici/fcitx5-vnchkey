fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        // ID 1: Default App Icon (vncharkey.ico)
        res.set_icon("assets/vncharkey.ico");
        // ID 2: 'V' Badge Icon for Vietnamese mode
        res.set_icon_with_id("assets/vncharkey_v.ico", "2");
        // ID 3: 'E' Badge Icon for English mode
        res.set_icon_with_id("assets/vncharkey_e.ico", "3");
        let _ = res.compile();
    }
}

