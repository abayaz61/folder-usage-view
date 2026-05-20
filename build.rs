fn main() {
    #[cfg(windows)]
    {
        let compile_result = std::panic::catch_unwind(|| {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/icon.ico")
                .set_manifest_file("app.manifest")
                .set("ProductName", "Disk Usage Analyzer")
                .set("FileDescription", "Ultra high-performance disk usage analyzer")
                .set("LegalCopyright", "Copyright © 2025 Codegen")
                .set("CompanyName", "Codegen");

            res.compile()
        });

        match compile_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("Warning: Failed to compile Windows resources: {}", e),
            Err(_) => eprintln!("Warning: Windows resource compilation panicked, continuing without resources"),
        }
    }
}
