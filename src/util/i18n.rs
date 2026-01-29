//! Internationalization (i18n) module with embedded language strings

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum Language {
    #[default]
    English,
    Turkish,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Turkish => "tr",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Turkish => "Türkçe",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Language::English => Language::Turkish,
            Language::Turkish => Language::English,
        }
    }
}

/// Localized strings for the application
pub struct Strings {
    strings: HashMap<&'static str, &'static str>,
}

impl Strings {
    pub fn new(lang: Language) -> Self {
        let strings = match lang {
            Language::English => Self::english(),
            Language::Turkish => Self::turkish(),
        };
        Self { strings }
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).copied().unwrap_or(key)
    }

    fn english() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();

        // App title
        m.insert("app.title", "Disk Usage Analyzer");
        m.insert("app.scanning", "Scanning in Progress");

        // Computer view
        m.insert("computer.title", "Computer");
        m.insert("computer.hint", "Select a drive | Arrows: Navigate | Enter: Open | g: Refresh");
        m.insert("computer.no_drives", "No drives found. Press 'g' to refresh.");
        m.insert("computer.total_usage", "Total Disk Usage");
        m.insert("computer.drives_detected", "drives detected");
        m.insert("computer.select_drive", "Computer - Select a drive");

        // Drive info
        m.insert("drive.total", "Total:");
        m.insert("drive.used", "Used:");
        m.insert("drive.free", "Free:");

        // Scanning
        m.insert("scan.scanning", "Scanning");
        m.insert("scan.starting", "Starting scan...");
        m.insert("scan.complete", "Scan complete!");
        m.insert("scan.files", "Files:");
        m.insert("scan.dirs", "Dirs:");
        m.insert("scan.size", "Size:");
        m.insert("scan.time", "Time:");
        m.insert("scan.speed", "Speed:");
        m.insert("scan.hint", "↑↓: Navigate  Enter: Open  Tab: View  q: Cancel scan");

        // Footer
        m.insert("footer.quit", "Quit");
        m.insert("footer.help", "Help");
        m.insert("footer.about", "About");
        m.insert("footer.settings", "Settings");
        m.insert("footer.drives", "Drives");
        m.insert("footer.view", "View");
        m.insert("footer.sort", "Sort");
        m.insert("footer.select", "Select");
        m.insert("footer.delete", "Delete");
        m.insert("footer.read_only", "READ-ONLY");
        m.insert("footer.selected", "selected");

        // Help screen
        m.insert("help.title", "Keyboard Shortcuts");
        m.insert("help.navigation", "Navigation");
        m.insert("help.actions", "Actions");
        m.insert("help.views", "Views");
        m.insert("help.other", "Other");
        m.insert("help.nav_up_down", "Navigate up/down");
        m.insert("help.nav_into", "Enter directory");
        m.insert("help.nav_back", "Go back / Parent");
        m.insert("help.nav_page", "Page up/down");
        m.insert("help.nav_home_end", "First/Last item");
        m.insert("help.action_select", "Toggle selection");
        m.insert("help.action_delete", "Delete selected");
        m.insert("help.action_refresh", "Refresh / Change drive");
        m.insert("help.view_toggle", "Toggle view mode");
        m.insert("help.sort_toggle", "Cycle sort mode");
        m.insert("help.other_help", "Toggle help");
        m.insert("help.other_about", "About");
        m.insert("help.other_settings", "Settings");
        m.insert("help.other_quit", "Quit");
        m.insert("help.close", "Press any key to close");

        // About screen
        m.insert("about.title", "About");
        m.insert("about.description", "Ultra high-performance disk usage analyzer");
        m.insert("about.features", "Features");
        m.insert("about.feature1", "Lightning-fast parallel scanning");
        m.insert("about.feature2", "Interactive treemap visualization");
        m.insert("about.feature3", "Real-time statistics");
        m.insert("about.feature4", "File management capabilities");
        m.insert("about.close", "Press any key to close");

        // Settings screen
        m.insert("settings.title", "Settings");
        m.insert("settings.header", "Application Settings");
        m.insert("settings.context_menu", "Context Menu Integration");
        m.insert("settings.context_menu_desc", "Add 'Usage Analytics' to right-click menu");
        m.insert("settings.startup", "Startup Location");
        m.insert("settings.startup_desc", "Where to start when launching the app");
        m.insert("settings.path_reg", "System PATH Registration");
        m.insert("settings.path_reg_desc", "Install to Program Files and add to PATH");
        m.insert("settings.start_menu", "Start Menu Shortcut");
        m.insert("settings.start_menu_desc", "Create shortcut in Windows Start Menu");
        m.insert("settings.desktop", "Desktop Shortcut");
        m.insert("settings.desktop_desc", "Create shortcut on Desktop");
        m.insert("settings.language", "Language");
        m.insert("settings.language_desc", "Application display language");
        m.insert("settings.enabled", "Enabled");
        m.insert("settings.disabled", "Disabled");
        m.insert("settings.registered", "Registered");
        m.insert("settings.not_registered", "Not Registered");
        m.insert("settings.created", "Created");
        m.insert("settings.not_created", "Not Created");
        m.insert("settings.hint", "Navigate   ");
        m.insert("settings.toggle", "Toggle   ");
        m.insert("settings.close", "Close");
        m.insert("settings.admin_note", "Note: Some options require Administrator privileges");

        // Startup locations
        m.insert("startup.last_location", "Last Location");
        m.insert("startup.current_folder", "Current Folder");
        m.insert("startup.computer_view", "Computer View");

        // Delete confirmation
        m.insert("delete.title", "Delete Confirmation");
        m.insert("delete.confirm", "Confirm Deletion");
        m.insert("delete.items", "Items to delete:");
        m.insert("delete.total_size", "Total size:");
        m.insert("delete.warning", "This action cannot be undone!");
        m.insert("delete.yes", "Confirm");
        m.insert("delete.no", "Cancel");
        m.insert("delete.disabled", "Delete disabled in read-only mode");
        m.insert("delete.cannot_drives", "Cannot delete drives");

        // Drive selector
        m.insert("drive_select.title", "Select Drive - Press Enter to confirm, Esc to cancel");
        m.insert("drive_select.hint", "↑/↓: Navigate   Enter: Select   Esc: Cancel   g: Refresh");
        m.insert("drive_select.no_drives", "No drives found");
        m.insert("drive_select.refreshed", "Drive list refreshed");

        // Error screen
        m.insert("error.title", "ERROR");
        m.insert("error.occurred", "An error has occurred");
        m.insert("error.message", "Error Message:");
        m.insert("error.continue", "Press any key to continue...");

        // File list
        m.insert("filelist.title", "Files");
        m.insert("filelist.empty", "Empty directory");
        m.insert("filelist.items", "items");

        // Stats
        m.insert("stats.title", "Statistics");
        m.insert("stats.total_size", "Total Size:");
        m.insert("stats.files", "Files:");
        m.insert("stats.directories", "Directories:");
        m.insert("stats.largest_files", "Largest Files");
        m.insert("stats.file_types", "File Types");

        // Treemap
        m.insert("treemap.title", "Treemap");

        // Messages
        m.insert("msg.deleted", "Deleted");
        m.insert("msg.items", "items");
        m.insert("msg.failed", "failed");
        m.insert("msg.context_menu_added", "Context menu registered");
        m.insert("msg.context_menu_removed", "Context menu removed");
        m.insert("msg.path_registered", "Registered to PATH");
        m.insert("msg.path_removed", "Removed from PATH");
        m.insert("msg.shortcut_created", "shortcut created");
        m.insert("msg.shortcut_removed", "shortcut removed");
        m.insert("msg.start_menu", "Start Menu");
        m.insert("msg.desktop", "Desktop");
        m.insert("msg.language_changed", "Language changed");
        m.insert("msg.opening", "Opening:");
        m.insert("msg.switching_to", "Switching to:");
        m.insert("msg.navigating_to", "Navigating to:");
        m.insert("msg.already_at_computer", "Already at Computer view");
        m.insert("msg.error", "Error:");

        m
    }

    fn turkish() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();

        // App title
        m.insert("app.title", "Disk Kullanım Analizi");
        m.insert("app.scanning", "Tarama Devam Ediyor");

        // Computer view
        m.insert("computer.title", "Bilgisayar");
        m.insert("computer.hint", "Disk seçin | Oklar: Gezin | Enter: Aç | g: Yenile");
        m.insert("computer.no_drives", "Disk bulunamadı. Yenilemek için 'g' tuşuna basın.");
        m.insert("computer.total_usage", "Toplam Disk Kullanımı");
        m.insert("computer.drives_detected", "disk algılandı");
        m.insert("computer.select_drive", "Bilgisayar - Disk seçin");

        // Drive info
        m.insert("drive.total", "Toplam:");
        m.insert("drive.used", "Kullanılan:");
        m.insert("drive.free", "Boş:");

        // Scanning
        m.insert("scan.scanning", "Taranıyor");
        m.insert("scan.starting", "Tarama başlatılıyor...");
        m.insert("scan.complete", "Tarama tamamlandı!");
        m.insert("scan.files", "Dosya:");
        m.insert("scan.dirs", "Klasör:");
        m.insert("scan.size", "Boyut:");
        m.insert("scan.time", "Süre:");
        m.insert("scan.speed", "Hız:");
        m.insert("scan.hint", "↑↓: Gezin  Enter: Aç  Tab: Görünüm  q: İptal");

        // Footer
        m.insert("footer.quit", "Çıkış");
        m.insert("footer.help", "Yardım");
        m.insert("footer.about", "Hakkında");
        m.insert("footer.settings", "Ayarlar");
        m.insert("footer.drives", "Diskler");
        m.insert("footer.view", "Görünüm");
        m.insert("footer.sort", "Sırala");
        m.insert("footer.select", "Seç");
        m.insert("footer.delete", "Sil");
        m.insert("footer.read_only", "SALT-OKUNUR");
        m.insert("footer.selected", "seçili");

        // Help screen
        m.insert("help.title", "Klavye Kısayolları");
        m.insert("help.navigation", "Gezinme");
        m.insert("help.actions", "İşlemler");
        m.insert("help.views", "Görünümler");
        m.insert("help.other", "Diğer");
        m.insert("help.nav_up_down", "Yukarı/aşağı git");
        m.insert("help.nav_into", "Klasöre gir");
        m.insert("help.nav_back", "Geri / Üst klasör");
        m.insert("help.nav_page", "Sayfa yukarı/aşağı");
        m.insert("help.nav_home_end", "İlk/Son öğe");
        m.insert("help.action_select", "Seçimi değiştir");
        m.insert("help.action_delete", "Seçilenleri sil");
        m.insert("help.action_refresh", "Yenile / Disk değiştir");
        m.insert("help.view_toggle", "Görünümü değiştir");
        m.insert("help.sort_toggle", "Sıralama değiştir");
        m.insert("help.other_help", "Yardımı aç/kapat");
        m.insert("help.other_about", "Hakkında");
        m.insert("help.other_settings", "Ayarlar");
        m.insert("help.other_quit", "Çıkış");
        m.insert("help.close", "Kapatmak için bir tuşa basın");

        // About screen
        m.insert("about.title", "Hakkında");
        m.insert("about.description", "Ultra yüksek performanslı disk kullanım analiz aracı");
        m.insert("about.features", "Özellikler");
        m.insert("about.feature1", "Işık hızında paralel tarama");
        m.insert("about.feature2", "İnteraktif treemap görselleştirme");
        m.insert("about.feature3", "Gerçek zamanlı istatistikler");
        m.insert("about.feature4", "Dosya yönetim özellikleri");
        m.insert("about.close", "Kapatmak için bir tuşa basın");

        // Settings screen
        m.insert("settings.title", "Ayarlar");
        m.insert("settings.header", "Uygulama Ayarları");
        m.insert("settings.context_menu", "Sağ Tık Menüsü");
        m.insert("settings.context_menu_desc", "Sağ tık menüsüne 'Kullanım Analizi' ekle");
        m.insert("settings.startup", "Başlangıç Konumu");
        m.insert("settings.startup_desc", "Uygulama açıldığında başlanacak konum");
        m.insert("settings.path_reg", "Sistem PATH Kaydı");
        m.insert("settings.path_reg_desc", "Program Files'a kur ve PATH'e ekle");
        m.insert("settings.start_menu", "Başlat Menüsü Kısayolu");
        m.insert("settings.start_menu_desc", "Başlat menüsünde kısayol oluştur");
        m.insert("settings.desktop", "Masaüstü Kısayolu");
        m.insert("settings.desktop_desc", "Masaüstünde kısayol oluştur");
        m.insert("settings.language", "Dil");
        m.insert("settings.language_desc", "Uygulama görüntüleme dili");
        m.insert("settings.enabled", "Etkin");
        m.insert("settings.disabled", "Devre Dışı");
        m.insert("settings.registered", "Kayıtlı");
        m.insert("settings.not_registered", "Kayıtlı Değil");
        m.insert("settings.created", "Oluşturuldu");
        m.insert("settings.not_created", "Oluşturulmadı");
        m.insert("settings.hint", "Gezin   ");
        m.insert("settings.toggle", "Değiştir   ");
        m.insert("settings.close", "Kapat");
        m.insert("settings.admin_note", "Not: Bazı seçenekler Yönetici yetkisi gerektirir");

        // Startup locations
        m.insert("startup.last_location", "Son Konum");
        m.insert("startup.current_folder", "Mevcut Klasör");
        m.insert("startup.computer_view", "Bilgisayar Görünümü");

        // Delete confirmation
        m.insert("delete.title", "Silme Onayı");
        m.insert("delete.confirm", "Silmeyi Onayla");
        m.insert("delete.items", "Silinecek öğe:");
        m.insert("delete.total_size", "Toplam boyut:");
        m.insert("delete.warning", "Bu işlem geri alınamaz!");
        m.insert("delete.yes", "Onayla");
        m.insert("delete.no", "İptal");
        m.insert("delete.disabled", "Salt-okunur modda silme devre dışı");
        m.insert("delete.cannot_drives", "Diskler silinemez");

        // Drive selector
        m.insert("drive_select.title", "Disk Seç - Onay: Enter, İptal: Esc");
        m.insert("drive_select.hint", "↑/↓: Gezin   Enter: Seç   Esc: İptal   g: Yenile");
        m.insert("drive_select.no_drives", "Disk bulunamadı");
        m.insert("drive_select.refreshed", "Disk listesi yenilendi");

        // Error screen
        m.insert("error.title", "HATA");
        m.insert("error.occurred", "Bir hata oluştu");
        m.insert("error.message", "Hata Mesajı:");
        m.insert("error.continue", "Devam etmek için bir tuşa basın...");

        // File list
        m.insert("filelist.title", "Dosyalar");
        m.insert("filelist.empty", "Boş klasör");
        m.insert("filelist.items", "öğe");

        // Stats
        m.insert("stats.title", "İstatistikler");
        m.insert("stats.total_size", "Toplam Boyut:");
        m.insert("stats.files", "Dosyalar:");
        m.insert("stats.directories", "Klasörler:");
        m.insert("stats.largest_files", "En Büyük Dosyalar");
        m.insert("stats.file_types", "Dosya Türleri");

        // Treemap
        m.insert("treemap.title", "Treemap");

        // Messages
        m.insert("msg.deleted", "Silindi");
        m.insert("msg.items", "öğe");
        m.insert("msg.failed", "başarısız");
        m.insert("msg.context_menu_added", "Sağ tık menüsü eklendi");
        m.insert("msg.context_menu_removed", "Sağ tık menüsü kaldırıldı");
        m.insert("msg.path_registered", "PATH'e kaydedildi");
        m.insert("msg.path_removed", "PATH'ten kaldırıldı");
        m.insert("msg.shortcut_created", "kısayolu oluşturuldu");
        m.insert("msg.shortcut_removed", "kısayolu kaldırıldı");
        m.insert("msg.start_menu", "Başlat Menüsü");
        m.insert("msg.desktop", "Masaüstü");
        m.insert("msg.language_changed", "Dil değiştirildi");
        m.insert("msg.opening", "Açılıyor:");
        m.insert("msg.switching_to", "Geçiliyor:");
        m.insert("msg.navigating_to", "Gidiliyor:");
        m.insert("msg.already_at_computer", "Zaten Bilgisayar görünümündesiniz");
        m.insert("msg.error", "Hata:");

        m
    }
}

/// Global strings accessor - call with current language
pub fn t(lang: Language, key: &str) -> String {
    Strings::new(lang).get(key).to_string()
}
