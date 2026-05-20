# Export Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Diske ait tarama sonucunu diske yazılabilen raporlara donusturmek ve sonraki fazlar icin uygulanabilir bir urun yol haritasi olusturmak.

**Architecture:** Ilk fazda mevcut tarama agacindan ve ozet istatistiklerden bagimsiz bir rapor modeli turetilir. Ardindan CLI uzerinden secilen formatta dosyaya yazan bir export katmani eklenir; bu katman TUI'den bagimsiz kalir ki sonraki fazlarda ayni model TUI export, karsilastirma ve zaman serisi ozelliklerinde tekrar kullanilabilsin.

**Tech Stack:** Rust, clap, serde, serde_json, std::fs, mevcut `FileTree` / `TreeStatistics` modeli, yerlesik unit testler

---

### Task 1: Faz Yol Haritasini Netlestir

**Files:**
- Create: `docs/superpowers/plans/2026-05-20-export-roadmap.md`
- Modify: `README.md`

- [ ] **Step 1: Fazlari tanimla**

```text
Faz 1: CLI export (JSON / CSV / Markdown)
Faz 2: Ignore pattern ve preset profilleri
Faz 3: Snapshot kaydetme ve scan karsilastirma
Faz 4: Buyuk dosya avcisi / temizlik onerileri
Faz 5: Kopya dosya bulma
```

- [ ] **Step 2: Faz 1 teslim kapsamını sabitle**

```text
Komut satirindan `--export` ile hedef dosya al.
`--export-format` ile `json`, `csv`, `md` formatlari desteklenir.
Tarama tamamlandiginda secilen dosyaya ozet + kategori dagilimi + en buyuk dosyalar yazilir.
```

- [ ] **Step 3: README kullanim metnini guncelle**

```bash
cargo run -- --path . --export report.json --export-format json
```

Expected: Dokumanda export kullanim ornegi ve desteklenen formatlar gorunur.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/plans/2026-05-20-export-roadmap.md README.md
git commit -m "docs: export yol haritasini ekle"
```

### Task 2: Rapor Modeli Ve Export Katmani

**Files:**
- Create: `src/report/mod.rs`
- Create: `src/report/export.rs`
- Create: `src/report/model.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Once test odakli API taslagini yaz**

```rust
pub enum ReportFormat {
    Json,
    Csv,
    Markdown,
}

pub struct ExportRequest {
    pub output_path: PathBuf,
    pub format: ReportFormat,
}

pub fn write_report(request: &ExportRequest, report: &ScanReport) -> anyhow::Result<()>;
```

- [ ] **Step 2: Rapor modelini tanimla**

```rust
pub struct ScanReport {
    pub scanned_path: String,
    pub generated_at: String,
    pub total_size: u64,
    pub total_files: u64,
    pub total_dirs: u64,
    pub error_count: usize,
    pub duration_secs: f64,
    pub categories: Vec<CategoryReportRow>,
    pub largest_files: Vec<LargestFileRow>,
}
```

- [ ] **Step 3: Failing unit testleri ekle**

```rust
#[test]
fn writes_json_report() { /* temp dir + assert file contains keys */ }

#[test]
fn writes_csv_report() { /* assert headings and rows */ }

#[test]
fn writes_markdown_report() { /* assert markdown headers */ }
```

- [ ] **Step 4: Minimal export implementasyonunu yaz**

```rust
match request.format {
    ReportFormat::Json => write_json(...),
    ReportFormat::Csv => write_csv(...),
    ReportFormat::Markdown => write_markdown(...),
}
```

- [ ] **Step 5: Testleri calistir**

```bash
cargo test report -- --nocapture
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/report src/lib.rs
git commit -m "feat: tarama raporu export altyapisini ekle"
```

### Task 3: Tarama Sonucundan Rapor Uret

**Files:**
- Modify: `src/model/tree.rs`
- Modify: `src/model/statistics.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Rapor icin gereken veri donusum API’sini ekle**

```rust
impl TreeStatistics {
    pub fn top_categories(&self) -> Vec<(FileCategory, f64, u64, u64)> { ... }
}
```

- [ ] **Step 2: `FileTree` icinden rapor satiri uret**

```rust
pub fn get_largest_file_rows(&self, limit: usize) -> Vec<LargestFileRow> { ... }
```

- [ ] **Step 3: `main.rs` argumanlarini ekle**

```rust
#[arg(long)]
export: Option<PathBuf>,

#[arg(long, default_value = "json")]
export_format: String,
```

- [ ] **Step 4: Tarama cikisinda export akisini bagla**

```rust
if let (Some(scan_result), Some(export_path)) = (final_scan_result.as_ref(), args.export.as_ref()) {
    let report = ScanReport::from_scan(...);
    write_report(&request, &report)?;
}
```

- [ ] **Step 5: Entegrasyon dogrulamasi**

```bash
cargo run -- --path . --export sample-report.json --export-format json
```

Expected: Dosya olusur ve icinde toplam boyut, kategori dagilimi, en buyuk dosyalar bulunur.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs src/model/tree.rs src/model/statistics.rs
git commit -m "feat: tarama sonucu export komutunu ekle"
```

### Task 4: Dokumantasyon Ve Son Dogrulama

**Files:**
- Modify: `README.md`

- [ ] **Step 1: README ozellikler kismina export maddesi ekle**

```md
- **Export Reports** - Save scan summaries as JSON, CSV, or Markdown
```

- [ ] **Step 2: Kullanim ve komut satiri seceneklerini guncelle**

```md
dua --path . --export report.md --export-format md
```

- [ ] **Step 3: Tam dogrulama kosusu**

```bash
cargo fmt
cargo check
cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: export kullanimini belgeleyi"
```
