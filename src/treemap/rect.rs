use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0.0 {
            return f64::INFINITY;
        }
        let ratio = self.width / self.height;
        if ratio >= 1.0 {
            ratio
        } else {
            1.0 / ratio
        }
    }

    pub fn shorter_side(&self) -> f64 {
        self.width.min(self.height)
    }

    pub fn is_horizontal(&self) -> bool {
        self.width >= self.height
    }

    pub fn to_terminal_rect(&self) -> TerminalRect {
        TerminalRect {
            x: self.x.floor() as u16,
            y: self.y.floor() as u16,
            width: self.width.round().max(1.0) as u16,
            height: self.height.round().max(1.0) as u16,
        }
    }
}

impl From<Rect> for LayoutRect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x as f64,
            y: rect.y as f64,
            width: rect.width as f64,
            height: rect.height as f64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl TerminalRect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

impl From<TerminalRect> for Rect {
    fn from(rect: TerminalRect) -> Self {
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl From<Rect> for TerminalRect {
    fn from(rect: Rect) -> Self {
        TerminalRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}
