//! Terminal view — renders TermScreen cells as styled text in gpui.

use gpui::*;
use gpui_component::StyledExt;

use std::sync::{Arc, Mutex};

use crate::terminal::terminal::{TermCell, TermColor, TermScreen, TermSession};

/// Terminal view rendering a grid of cells in the content area.
pub struct TerminalView {
    session: Arc<Mutex<TermSession>>,
    cols: u16,
    rows: u16,
}

impl TerminalView {
    /// Create a new terminal view.
    pub fn new(rows: u16, cols: u16) -> Self {
        let session = Arc::new(Mutex::new(TermSession::new(rows, cols, 10000)));
        Self {
            session,
            cols,
            rows,
        }
    }

    /// Get a handle to the shared terminal session.
    pub fn session_handle(&self) -> Arc<Mutex<TermSession>> {
        self.session.clone()
    }

    /// Feed SSH channel data into the terminal.
    pub fn feed_data(&self, data: &[u8]) {
        if let Ok(mut session) = self.session.lock() {
            session.process(data);
        }
    }

    /// Render a single cell.
    fn render_cell(cell: &TermCell) -> gpui::AnyElement {
        if cell.is_wide_continuation() || !cell.has_contents() {
            return div().w(px(9.0)).h(px(18.0)).into_any_element();
        }

        let fg = Self::term_color_to_hsla(cell.fgcolor());
        let text = cell.contents().to_string();

        let mut el = div()
            .flex()
            .items_center()
            .w(px(9.0))
            .h(px(18.0))
            .text_color(fg)
            .child(text);

        match cell.bgcolor() {
            TermColor::Default => {}
            bg => {
                el = el.bg(Self::term_color_to_hsla(bg));
            }
        }

        if cell.bold() {
            el = el.font_weight(FontWeight::BOLD);
        }
        if cell.italic() {
            el = el.italic();
        }
        if cell.underline() {
            el = el.underline();
        }

        el.into_any_element()
    }

    fn term_color_to_hsla(color: TermColor) -> gpui::Hsla {
        match color {
            TermColor::Default => gpui::Hsla {
                h: 0.0, s: 0.0, l: 0.8, a: 1.0,
            },
            TermColor::Rgb(r, g, b) => {
                let rgba: gpui::Rgba = gpui::rgb((r as u32) << 16 | (g as u32) << 8 | b as u32);
                rgba.into()
            }
            TermColor::Idx(idx) => {
                let (r, g, b) = Self::ansi_color(idx);
                let rgba: gpui::Rgba = gpui::rgb((r as u32) << 16 | (g as u32) << 8 | b as u32);
                rgba.into()
            }
        }
    }

    fn ansi_color(idx: u8) -> (u8, u8, u8) {
        match idx {
            0 => (0, 0, 0),
            1 => (170, 0, 0),
            2 => (0, 170, 0),
            3 => (170, 85, 0),
            4 => (0, 0, 170),
            5 => (170, 0, 170),
            6 => (0, 170, 170),
            7 => (170, 170, 170),
            8 => (85, 85, 85),
            9 => (255, 85, 85),
            10 => (85, 255, 85),
            11 => (255, 255, 85),
            12 => (85, 85, 255),
            13 => (255, 85, 255),
            14 => (85, 255, 255),
            15 => (255, 255, 255),
            _ => (180, 180, 180),
        }
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (cols, rows) = {
            let mut session = self.session.lock().unwrap();
            let screen = session.screen();
            screen.size()
        };

        // Build row elements using the cell() accessor
        let rows_el: Vec<gpui::AnyElement> = (0..rows)
            .map(|row| {
                let cells: Vec<gpui::AnyElement> = {
                    let mut session = self.session.lock().unwrap();
                    let screen = session.screen();
                    (0..cols)
                        .filter_map(|col| screen.cell(row, col))
                        .map(|c| Self::render_cell(c))
                        .collect()
                };
                div().h_flex().children(cells).into_any_element()
            })
            .collect();

        div()
            .id("terminal-view")
            .v_flex()
            .size_full()
            .bg(gpui::rgb(0x000000))
            .overflow_hidden()
            .font_family("Consolas")
            .text_size(px(14.0))
            .children(rows_el)
    }
}
