//! Generic file browser component for dual-panel SFTP file manager.
//!
//! Renders a file/directory listing as a table with path bar navigation.
//! Used by both the local and remote panels via FileManagerView.

use gpui::*;
use gpui_component::StyledExt;

use crate::app::state::{FileBrowserState, FileEntry};

/// Row height in pixels.
const ROW_HEIGHT: f32 = 24.0;
/// Path bar height in pixels.
const PATH_BAR_HEIGHT: f32 = 32.0;

/// Renders the full file browser panel.
pub fn render_file_browser(
    state: &FileBrowserState,
    panel_id: &str,
) -> gpui::AnyElement {
    let path_bar = render_path_bar(state, panel_id);
    let header = render_table_header(panel_id);
    let rows = render_rows(state, panel_id);

    div()
        .id(SharedString::from(format!("file-browser-{panel_id}")))
        .v_flex()
        .size_full()
        .overflow_hidden()
        .child(path_bar)
        .child(header)
        .child(
            div()
                .id(SharedString::from(format!("file-rows-{panel_id}")))
                .flex_1()
                .overflow_y_scroll()
                .v_flex()
                .children(rows),
        )
        .into_any_element()
}

/// Render the path bar with refresh, up, and path display.
fn render_path_bar(state: &FileBrowserState, panel_id: &str) -> impl IntoElement {
    div()
        .id(SharedString::from(format!("path-bar-{panel_id}")))
        .h(px(PATH_BAR_HEIGHT))
        .flex()
        .items_center()
        .px(px(8.0))
        .gap(px(4.0))
        .bg(gpui::rgb(0x252525))
        .border_b_1()
        .border_color(gpui::rgb(0x3a3a3a))
        .child(
            div()
                .id(SharedString::from(format!("refresh-btn-{panel_id}")))
                .cursor_pointer()
                .text_color(gpui::rgb(0xcccccc))
                .child("↻"),
        )
        .child(
            div()
                .id(SharedString::from(format!("up-btn-{panel_id}")))
                .cursor_pointer()
                .text_color(gpui::rgb(0xcccccc))
                .child("↑"),
        )
        .child(
            div()
                .id(SharedString::from(format!("path-display-{panel_id}")))
                .flex_1()
                .px(px(4.0))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xaaaaaa))
                .child(state.current_path.clone()),
        )
}

/// Render the table header row.
fn render_table_header(panel_id: &str) -> impl IntoElement {
    div()
        .id(SharedString::from(format!("table-header-{panel_id}")))
        .h(px(ROW_HEIGHT))
        .flex()
        .items_center()
        .px(px(8.0))
        .bg(gpui::rgb(0x2a2a2a))
        .border_b_1()
        .border_color(gpui::rgb(0x3a3a3a))
        .child(
            div()
                .flex_1()
                .text_size(px(11.0))
                .text_color(gpui::rgb(0x888888))
                .child("Name"),
        )
        .child(
            div()
                .w(px(80.0))
                .text_size(px(11.0))
                .text_color(gpui::rgb(0x888888))
                .child("Size"),
        )
        .child(
            div()
                .w(px(130.0))
                .text_size(px(11.0))
                .text_color(gpui::rgb(0x888888))
                .child("Modified"),
        )
        .child(
            div()
                .w(px(90.0))
                .text_size(px(11.0))
                .text_color(gpui::rgb(0x888888))
                .child("Permissions"),
        )
}

/// Render the file/directory rows.
fn render_rows(state: &FileBrowserState, panel_id: &str) -> Vec<gpui::AnyElement> {
    if state.loading {
        return vec![div()
            .id(SharedString::from(format!("loading-{panel_id}")))
            .flex()
            .items_center()
            .justify_center()
            .h(px(40.0))
            .text_color(gpui::rgb(0x888888))
            .text_size(px(12.0))
            .child("Loading...")
            .into_any_element()];
    }

    if let Some(ref err) = state.error {
        return vec![div()
            .id(SharedString::from(format!("error-{panel_id}")))
            .flex()
            .items_center()
            .justify_center()
            .h(px(40.0))
            .text_color(gpui::rgb(0xff5555))
            .text_size(px(12.0))
            .child(err.clone())
            .into_any_element()];
    }

    if state.entries.is_empty() {
        return vec![div()
            .id(SharedString::from(format!("empty-{panel_id}")))
            .flex()
            .items_center()
            .justify_center()
            .h(px(40.0))
            .text_color(gpui::rgb(0x888888))
            .text_size(px(12.0))
            .child("(empty)")
            .into_any_element()];
    }

    state
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            render_file_row(entry, idx, state.selected == Some(idx), panel_id)
        })
        .collect()
}

/// Render a single file/directory row.
fn render_file_row(
    entry: &FileEntry,
    idx: usize,
    selected: bool,
    panel_id: &str,
) -> gpui::AnyElement {
    let bg = if selected {
        gpui::rgb(0x3a5a9a)
    } else {
        gpui::rgb(0x1e1e1e)
    };

    let icon = if entry.is_dir { "📁" } else { "📄" };
    let name_color = if entry.is_dir {
        gpui::rgb(0x66bbff)
    } else {
        gpui::rgb(0xcccccc)
    };

    div()
        .id(SharedString::from(format!("row-{panel_id}-{idx}")))
        .h(px(ROW_HEIGHT))
        .flex()
        .items_center()
        .px(px(8.0))
        .bg(bg)
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap(px(4.0))
                .overflow_x_hidden()
                .child(div().text_size(px(12.0)).child(icon))
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(name_color)
                        .overflow_x_hidden()
                        .child(entry.name.clone()),
                ),
        )
        .child(
            div()
                .w(px(80.0))
                .text_size(px(11.0))
                .text_color(gpui::rgb(0xaaaaaa))
                .child(format_size(entry.size)),
        )
        .child(
            div()
                .w(px(130.0))
                .text_size(px(11.0))
                .text_color(gpui::rgb(0xaaaaaa))
                .child(entry.modified.clone()),
        )
        .child(
            div()
                .w(px(90.0))
                .text_size(px(11.0))
                .text_color(gpui::rgb(0xaaaaaa))
                .child(entry.permissions.clone()),
        )
        .into_any_element()
}

/// Format a file size as a human-readable string.
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}
