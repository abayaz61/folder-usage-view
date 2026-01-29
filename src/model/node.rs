use slotmap::new_key_type;
use smallvec::SmallVec;
use std::time::SystemTime;

new_key_type! {
    pub struct NodeId;
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub size: u64,
    pub item_count: u64,
    pub entry_type: EntryType,
    pub parent: Option<NodeId>,
    pub children: SmallVec<[NodeId; 8]>,
    pub metadata: Option<NodeMetadata>,
    pub selected: bool,
    pub depth: u16,
}

impl TreeNode {
    pub fn new_directory(name: String, parent: Option<NodeId>, depth: u16) -> Self {
        Self {
            name,
            size: 0,
            item_count: 0,
            entry_type: EntryType::Directory,
            parent,
            children: SmallVec::new(),
            metadata: None,
            selected: false,
            depth,
        }
    }

    pub fn new_file(name: String, size: u64, parent: Option<NodeId>, depth: u16, category: FileCategory) -> Self {
        Self {
            name,
            size,
            item_count: 1,
            entry_type: EntryType::File(category),
            parent,
            children: SmallVec::new(),
            metadata: None,
            selected: false,
            depth,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Directory,
    File(FileCategory),
    Symlink,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FileCategory {
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Code,
    Executable,
    Data,
    #[default]
    Other,
}

impl FileCategory {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Documents
            "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" => {
                FileCategory::Document
            }
            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "raw" | "psd" => {
                FileCategory::Image
            }
            // Video
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpeg" | "mpg" => {
                FileCategory::Video
            }
            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" => FileCategory::Audio,
            // Archives
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "dmg" => FileCategory::Archive,
            // Code
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp" | "java" | "go"
            | "rb" | "php" | "swift" | "kt" | "scala" | "cs" | "fs" | "html" | "css" | "scss"
            | "sass" | "less" | "json" | "xml" | "yaml" | "yml" | "toml" | "md" | "sql" | "sh"
            | "bash" | "ps1" | "lua" | "r" | "m" | "vue" | "svelte" => FileCategory::Code,
            // Executables
            "exe" | "msi" | "app" | "dll" | "so" | "dylib" | "bin" | "cmd" | "bat" | "com" => {
                FileCategory::Executable
            }
            // Data
            "db" | "sqlite" | "mdb" | "accdb" | "dat" | "log" | "bak" | "tmp" | "cache" => {
                FileCategory::Data
            }
            _ => FileCategory::Other,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FileCategory::Document => "Documents",
            FileCategory::Image => "Images",
            FileCategory::Video => "Videos",
            FileCategory::Audio => "Audio",
            FileCategory::Archive => "Archives",
            FileCategory::Code => "Code",
            FileCategory::Executable => "Executables",
            FileCategory::Data => "Data",
            FileCategory::Other => "Other",
        }
    }
}
