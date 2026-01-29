use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub target_path: PathBuf,
    pub read_only: bool,
    pub follow_symlinks: bool,
    pub show_hidden: bool,
}

impl Config {
    pub fn new(target_path: PathBuf) -> Self {
        Self {
            target_path,
            read_only: true,
            follow_symlinks: false,
            show_hidden: false,
        }
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn with_show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}
