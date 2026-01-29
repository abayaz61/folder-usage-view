fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("ProductName", "Disk Usage Analyzer")
            .set("FileDescription", "Ultra high-performance disk usage analyzer")
            .set("LegalCopyright", "Copyright © 2024 Codegen")
            .set("CompanyName", "Codegen");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
