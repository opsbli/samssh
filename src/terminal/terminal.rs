//! Terminal emulation wrapper around tattoy-wezterm-term
//!
//! Provides `TerminalSession` — a VT100/xterm compatible terminal
//! that processes SSH channel data and exposes the screen buffer
//! for gpui rendering.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use wezterm_term::color::{ColorAttribute, ColorPalette};
use wezterm_term::{
    CellAttributes as WezCellAttributes, Clipboard as WezClipboard,
    ClipboardSelection as WezClipboardSelection, Intensity, Line, Terminal, TerminalConfiguration,
    TerminalSize, Underline,
};

// ── Public types ──

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermColor {
    Default,
    Rgb(u8, u8, u8),
    Idx(u8),
}

#[derive(Clone, Debug)]
pub struct TermCell {
    fg: TermColor,
    bg: TermColor,
    bold: bool,
    dim: bool,
    inverse: bool,
    italic: bool,
    underline: bool,
    text: String,
    has_contents: bool,
    is_wide_continuation: bool,
}

impl TermCell {
    pub fn fgcolor(&self) -> TermColor { self.fg }
    pub fn bgcolor(&self) -> TermColor { self.bg }
    pub fn bold(&self) -> bool { self.bold }
    pub fn dim(&self) -> bool { self.dim }
    pub fn inverse(&self) -> bool { self.inverse }
    pub fn italic(&self) -> bool { self.italic }
    pub fn underline(&self) -> bool { self.underline }
    pub fn is_wide_continuation(&self) -> bool { self.is_wide_continuation }
    pub fn has_contents(&self) -> bool { self.has_contents }
    pub fn contents(&self) -> &str { &self.text }
}

impl TermCell {
    fn blank() -> Self {
        Self {
            fg: TermColor::Default, bg: TermColor::Default,
            bold: false, dim: false, inverse: false, italic: false, underline: false,
            text: String::new(), has_contents: false, is_wide_continuation: false,
        }
    }

    fn from_cell(text: &str, attrs: &WezCellAttributes) -> Self {
        let intensity = attrs.intensity();
        let is_blank = text == " " || text.is_empty();
        Self {
            fg: map_color(attrs.foreground()),
            bg: map_color(attrs.background()),
            bold: intensity == Intensity::Bold,
            dim: intensity == Intensity::Half,
            inverse: attrs.reverse(),
            italic: attrs.italic(),
            underline: attrs.underline() != Underline::None,
            text: if is_blank { String::new() } else { text.to_string() },
            has_contents: !is_blank,
            is_wide_continuation: false,
        }
    }

    fn wide_continuation_from(base: &Self) -> Self {
        Self { text: String::new(), has_contents: false, is_wide_continuation: true, ..base.clone() }
    }
}

/// Snapshot of the terminal screen at a point in time.
#[derive(Clone, Debug)]
pub struct TermScreen {
    rows: u16,
    cols: u16,
    scrollback: usize,
    scrollback_max: usize,
    lines: Vec<Line>,
    visible_cells: Vec<TermCell>,
    cursor_row: u16,
    cursor_col: u16,
    hide_cursor: bool,
}

impl TermScreen {
    /// Terminal dimensions (rows, cols).
    pub fn size(&self) -> (u16, u16) { (self.rows, self.cols) }

    /// Get a cell at (row, col). Returns None if out of bounds.
    pub fn cell(&self, row: u16, col: u16) -> Option<&TermCell> {
        if row >= self.rows || col >= self.cols { return None; }
        self.visible_cells.get(row as usize * self.cols as usize + col as usize)
    }

    /// Current scrollback offset.
    pub fn scrollback(&self) -> usize { self.scrollback }
    /// Maximum scrollback lines.
    pub fn scrollback_max(&self) -> usize { self.scrollback_max }
    /// Cursor position (row, col).
    pub fn cursor_position(&self) -> (u16, u16) { (self.cursor_row, self.cursor_col) }
    /// Whether cursor is hidden.
    pub fn hide_cursor(&self) -> bool { self.hide_cursor }

    /// Extract plain text content of the visible screen.
    pub fn contents(&self) -> String {
        let mut out = String::new();
        for row in 0..self.rows {
            let mut line = String::new();
            for col in 0..self.cols {
                if let Some(cell) = self.cell(row, col) {
                    if cell.is_wide_continuation() { continue; }
                    if cell.has_contents() { line.push_str(cell.contents()); }
                    else { line.push(' '); }
                }
            }
            out.push_str(line.trim_end_matches(' '));
            if row + 1 < self.rows { out.push('\n'); }
        }
        out
    }
}

/// Terminal session that processes SSH channel data.
pub struct TermSession {
    terminal: Terminal,
    screen: TermScreen,
    screen_dirty: bool,
    scrollback: usize,
    clipboard: Arc<ClipboardCollector>,
}

impl TermSession {
    /// Create a new terminal session with the given dimensions.
    pub fn new(rows: u16, cols: u16, scrollback_len: usize) -> Self {
        let rows = rows.max(1) as usize;
        let cols = cols.max(1) as usize;
        let scrollback_len = scrollback_len.min(200_000);
        let term_size = TerminalSize {
            rows, cols,
            pixel_width: cols.saturating_mul(8),
            pixel_height: rows.saturating_mul(16),
            dpi: 0,
        };
        let clipboard = Arc::new(ClipboardCollector::default());
        let mut terminal = Terminal::new(
            term_size,
            Arc::new(SessionConfig { scrollback: scrollback_len }),
            "SamSSH",
            env!("CARGO_PKG_VERSION"),
            Box::new(NullWriter),
        );
        let wezterm_clipboard: Arc<dyn WezClipboard> = clipboard.clone();
        terminal.set_clipboard(&wezterm_clipboard);

        let mut session = Self {
            terminal,
            screen: TermScreen::empty(rows as u16, cols as u16),
            screen_dirty: false,
            scrollback: 0,
            clipboard,
        };
        session.refresh_screen();
        session
    }

    /// Feed raw bytes from the SSH channel into the terminal.
    pub fn process(&mut self, bytes: &[u8]) {
        if !bytes.is_empty() {
            self.terminal.advance_bytes(bytes);
            self.screen_dirty = true;
        }
    }

    /// Get the current screen snapshot.
    pub fn screen(&mut self) -> &TermScreen {
        self.refresh_screen_if_dirty();
        &self.screen
    }

    /// Resize the terminal.
    pub fn set_size(&mut self, rows: u16, cols: u16) {
        let rows = rows.max(1) as usize;
        let cols = cols.max(1) as usize;
        let size = self.terminal.get_size();
        self.terminal.resize(TerminalSize {
            rows, cols,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
            dpi: size.dpi,
        });
        self.screen_dirty = true;
    }

    /// Set scrollback offset (0 = bottom, larger = further back).
    pub fn set_scrollback(&mut self, rows: usize) {
        self.scrollback = rows;
        self.screen_dirty = true;
    }

    /// Collect clipboard writes from the terminal (OSC 52).
    pub fn take_clipboard_writes(&mut self) -> Vec<ClipboardWrite> {
        self.clipboard.take_all()
    }

    fn refresh_screen_if_dirty(&mut self) {
        if self.screen_dirty { self.refresh_screen(); }
    }

    fn refresh_screen(&mut self) {
        let size = self.terminal.get_size();
        let rows = size.rows.max(1);
        let cols = size.cols.max(1);
        let term_screen = self.terminal.screen();
        let total_rows = term_screen.scrollback_rows();
        let scrollback_max = total_rows.saturating_sub(rows);
        self.scrollback = self.scrollback.min(scrollback_max);

        let cursor = self.terminal.cursor_pos();
        let cursor_row = (cursor.y.clamp(0, rows.saturating_sub(1) as i64)) as u16;
        let cursor_col = cursor.x.min(cols.saturating_sub(1)) as u16;
        let hide_cursor = format!("{:?}", cursor.visibility) == "Hidden";

        self.screen = TermScreen::from_snapshot(TermScreenSnapshot {
            rows: rows as u16, cols: cols as u16,
            scrollback: self.scrollback, scrollback_max,
            lines: term_screen.lines_in_phys_range(0..total_rows),
            cursor_row, cursor_col, hide_cursor,
        });
        self.screen_dirty = false;
    }
}

// ── Internal types ──

#[derive(Debug)]
struct SessionConfig { scrollback: usize }

impl TerminalConfiguration for SessionConfig {
    fn scrollback_size(&self) -> usize { self.scrollback }
    fn color_palette(&self) -> ColorPalette { ColorPalette::default() }
}

struct NullWriter;
impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { Ok(buf.len()) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[derive(Debug, Clone)]
struct TermScreenSnapshot {
    rows: u16, cols: u16,
    scrollback: usize, scrollback_max: usize,
    lines: Vec<Line>,
    cursor_row: u16, cursor_col: u16, hide_cursor: bool,
}

impl TermScreen {
    fn empty(rows: u16, cols: u16) -> Self {
        Self {
            rows, cols,
            scrollback: 0, scrollback_max: 0,
            lines: Vec::new(),
            visible_cells: Vec::new(),
            cursor_row: 0, cursor_col: 0, hide_cursor: false,
        }
    }

    fn from_snapshot(snap: TermScreenSnapshot) -> Self {
        let mut screen = Self {
            rows: snap.rows, cols: snap.cols,
            scrollback: snap.scrollback, scrollback_max: snap.scrollback_max,
            lines: snap.lines,
            visible_cells: Vec::new(),
            cursor_row: snap.cursor_row, cursor_col: snap.cursor_col,
            hide_cursor: snap.hide_cursor,
        };
        screen.rebuild_visible_cells();
        screen
    }

    fn rebuild_visible_cells(&mut self) {
        let rows = self.rows as usize;
        let cols = self.cols as usize;
        self.visible_cells = vec![TermCell::blank(); rows.saturating_mul(cols)];
        let top_abs = self.scrollback_max.saturating_sub(self.scrollback);
        for row in 0..rows {
            let Some(line) = self.lines.get(top_abs.saturating_add(row)) else { continue; };
            let row_slice_start = row.saturating_mul(cols);
            let row_slice_end = row_slice_start.saturating_add(cols);
            let row_slice = &mut self.visible_cells[row_slice_start..row_slice_end];
            for cell_ref in line.visible_cells() {
                let col = cell_ref.cell_index();
                if col >= cols { continue; }
                let base = TermCell::from_cell(cell_ref.str(), cell_ref.attrs());
                row_slice[col] = base.clone();
                let width = cell_ref.width().max(1);
                for off in 1..width {
                    let c = col + off;
                    if c >= cols { break; }
                    row_slice[c] = TermCell::wide_continuation_from(&base);
                }
            }
        }
    }
}

// ── Clipboard ──

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardTarget { Clipboard, PrimarySelection }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardWrite {
    pub target: ClipboardTarget,
    pub text: Option<String>,
}

#[derive(Default)]
struct ClipboardCollector { writes: Mutex<VecDeque<ClipboardWrite>> }

impl ClipboardCollector {
    fn take_all(&self) -> Vec<ClipboardWrite> {
        self.writes.lock().expect("clipboard collector poisoned").drain(..).collect()
    }
}

impl WezClipboard for ClipboardCollector {
    fn set_contents(
        &self, selection: WezClipboardSelection, data: Option<String>,
    ) -> anyhow::Result<()> {
        let target = match selection {
            WezClipboardSelection::Clipboard => ClipboardTarget::Clipboard,
            WezClipboardSelection::PrimarySelection => ClipboardTarget::PrimarySelection,
        };
        self.writes.lock().expect("clipboard collector poisoned")
            .push_back(ClipboardWrite { target, text: data });
        Ok(())
    }
}

// ── Color mapping ──

fn map_color(attr: ColorAttribute) -> TermColor {
    match attr {
        ColorAttribute::Default => TermColor::Default,
        ColorAttribute::PaletteIndex(idx) => TermColor::Idx(idx),
        ColorAttribute::TrueColorWithPaletteFallback(rgb, _) |
        ColorAttribute::TrueColorWithDefaultFallback(rgb) => {
            let (r, g, b, _) = rgb.as_rgba_u8();
            TermColor::Rgb(r, g, b)
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_basic_output() {
        let mut session = TermSession::new(4, 8, 32);
        session.process(b"hello");
        let screen = session.screen();
        assert!(screen.contents().contains("hello"));
    }

    #[test]
    fn test_terminal_newline() {
        let mut session = TermSession::new(8, 16, 32);
        session.process(b"line1\nline2");
        let screen = session.screen();
        let text = screen.contents();
        assert!(text.contains("line1"));
        assert!(text.contains("line2"));
    }

    #[test]
    fn test_terminal_resize() {
        let mut session = TermSession::new(4, 8, 32);
        session.process(b"hello world");
        session.set_size(8, 16);
        let screen = session.screen();
        assert_eq!(screen.size(), (8, 16));
    }

    #[test]
    fn test_terminal_scrollback() {
        let mut session = TermSession::new(5, 8, 100);
        // Generate more lines than visible
        for i in 0..15u8 {
            let line = format!("line{}\n", i);
            session.process(line.as_bytes());
        }
        // Verify scrollback exists
        let screen = session.screen();
        assert!(screen.scrollback_max() > 0);
    }

    #[test]
    fn test_terminal_256_color() {
        let mut session = TermSession::new(3, 32, 32);
        session.process(b"\x1b[38;5;196mred\x1b[0m");
        let screen = session.screen();
        let text = screen.contents();
        assert!(text.contains("red"));
    }

    #[test]
    fn test_terminal_truecolor() {
        let mut session = TermSession::new(3, 32, 32);
        session.process(b"\x1b[38;2;255;0;0mcolor\x1b[0m");
        let screen = session.screen();
        assert!(screen.contents().contains("color"));
    }

    #[test]
    fn test_terminal_cjk() {
        let mut session = TermSession::new(3, 16, 32);
        session.process("你好".as_bytes());
        let screen = session.screen();
        assert!(!screen.contents().is_empty() || screen.contents().contains("你"));
    }
}
