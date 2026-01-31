fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set_manifest_file("app.manifest")
            .set("ProductName", "Disk Usage Analyzer")
            .set("FileDescription", "Ultra high-performance disk usage analyzer")
            .set("LegalCopyright", "Copyright © 2025 Codegen")
            .set("CompanyName", "Codegen");

        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
