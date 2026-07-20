//! Dual-panel SFTP file manager view.
//!
//! Renders a local file panel (left) and remote SFTP panel (right)
//! with a toolbar between them for file operations.

use gpui::*;
use gpui_component::StyledExt;

use crate::app::state::{
    AppState, FileManagerState, FileOperation, SessionId,
};
use crate::sftp::SftpEntry;

/// The dual-panel file manager view.
pub struct FileManagerView {
    app_state: Entity<AppState>,
    initial_path: String,
}

impl FileManagerView {
    /// Create a new file manager view.
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        Self {
            app_state,
            initial_path: "/".to_string(),
        }
    }

    /// Resolve the active SFTP session ID from the current tab state.
    fn active_session_id(&self, cx: &Context<Self>) -> Option<SessionId> {
        let tabs = &self.app_state.read(cx).tabs;
        let active = tabs.active_tab()?;
        if active.kind != crate::app::state::SessionKind::SFTP {
            return None;
        }
        active.session_id
    }

    /// Get or create the file manager state for the active session.
    fn get_or_init_state(&mut self, cx: &mut Context<Self>) -> Option<FileManagerState> {
        let sid = self.active_session_id(cx)?;
        let state = self.app_state.read(cx).sftp_state.get(&sid).cloned();
        if state.is_none() {
            // Initialize state for this session
            self.app_state.update(cx, |state, _| {
                state
                    .sftp_state
                    .entry(sid)
                    .or_insert_with(|| FileManagerState::new(self.initial_path.clone()));
            });
        }
        self.app_state.read(cx).sftp_state.get(&sid).cloned()
    }

    /// Navigate local panel to a new path.
    pub fn navigate_local(&mut self, path: String, cx: &mut Context<Self>) {
        let sid = match self.active_session_id(cx) {
            Some(id) => id,
            None => return,
        };
        let app_state = self.app_state.clone();

        // Set loading state
        app_state.update(cx, |state, _| {
            if let Some(fm) = state.sftp_state.get_mut(&sid) {
                fm.local.current_path = path.clone();
                fm.local.loading = true;
            }
        });
        cx.notify();

        // Spawn async directory listing
        let entity_id = cx.entity_id();
        cx.spawn(move |_this, app: &mut gpui::AsyncApp| {
            let mut app = app.clone();
            async move {
                let entries = list_local_dir_task(&path).await;

                let _ = app.update(move |cx| {
                    app_state.update(cx, |state, _| {
                        if let Some(fm) = state.sftp_state.get_mut(&sid) {
                            match entries {
                                Ok(entries) => {
                                    fm.local.set_entries(entries);
                                }
                                Err(e) => {
                                    fm.local.set_error(format!("Error: {}", e));
                                }
                            }
                        }
                    });
                    cx.notify(entity_id);
                });
            }
        })
        .detach();
    }

    /// Navigate remote (SFTP) panel to a new path.
    pub fn navigate_remote(&mut self, path: String, cx: &mut Context<Self>) {
        let sid = match self.active_session_id(cx) {
            Some(id) => id,
            None => return,
        };

        self.app_state.update(cx, |state, _| {
            if let Some(fm) = state.sftp_state.get_mut(&sid) {
                fm.remote.current_path = path.clone();
                fm.remote.loading = false;
                fm.remote.error = Some("SFTP listing requires active SSH connection".to_string());
            }
        });
        cx.notify();
    }

    /// Go up one directory in the local panel.
    fn go_up_local(&mut self, cx: &mut Context<Self>) {
        let sid = match self.active_session_id(cx) {
            Some(id) => id,
            None => return,
        };
        let current = self.app_state.read(cx).sftp_state.get(&sid)
            .map(|fm| fm.local.current_path.clone())
            .unwrap_or_default();
        let parent = parent_dir(&current);
        self.navigate_local(parent, cx);
    }

    /// Go up one directory in the remote panel.
    fn go_up_remote(&mut self, cx: &mut Context<Self>) {
        let sid = match self.active_session_id(cx) {
            Some(id) => id,
            None => return,
        };
        let current = self.app_state.read(cx).sftp_state.get(&sid)
            .map(|fm| fm.remote.current_path.clone())
            .unwrap_or_default();
        let parent = parent_dir(&current);
        self.navigate_remote(parent, cx);
    }

    /// Enter a directory in the local panel.
    fn enter_dir_local(&mut self, name: String, cx: &mut Context<Self>) {
        let sid = match self.active_session_id(cx) {
            Some(id) => id,
            None => return,
        };
        let base = self.app_state.read(cx).sftp_state.get(&sid)
            .map(|fm| fm.local.current_path.clone())
            .unwrap_or_default();
        let new_path = crate::sftp::join_path(&base, &name);
        self.navigate_local(new_path, cx);
    }
}

impl Render for FileManagerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.get_or_init_state(cx).unwrap_or_else(|| {
            FileManagerState::new("/".to_string())
        });

        div()
            .id("file-manager-view")
            .v_flex()
            .size_full()
            .bg(gpui::rgb(0x1e1e1e))
            // Top toolbar
            .child(render_toolbar())
            // Main content: dual panels
            .child(
                div()
                    .id("dual-panel")
                    .flex()
                    .flex_row()
                    .size_full()
                    .overflow_hidden()
                    // Local panel
                    .child(
                        div()
                            .id("local-panel")
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(crate::app::components::file_browser::render_file_browser(
                                &state.local,
                                "local",
                            )),
                    )
                    // Divider
                    .child(
                        div()
                            .id("panel-divider")
                            .w(px(2.0))
                            .h_full()
                            .bg(gpui::rgb(0x3a3a3a)),
                    )
                    // Remote panel
                    .child(
                        div()
                            .id("remote-panel")
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(crate::app::components::file_browser::render_file_browser(
                                &state.remote,
                                "remote",
                            )),
                    ),
            )
    }
}

/// Render the operation toolbar with action buttons.
fn render_toolbar() -> impl IntoElement {
    div()
        .id("fm-toolbar")
        .h(px(32.0))
        .flex()
        .items_center()
        .px(px(8.0))
        .gap(px(4.0))
        .bg(gpui::rgb(0x252525))
        .border_b_1()
        .border_color(gpui::rgb(0x3a3a3a))
        .child(
            div()
                .id("btn-upload")
                .px(px(8.0))
                .py(px(4.0))
                .bg(gpui::rgb(0x2d4a2d))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xaaccaa))
                .rounded(px(3.0))
                .cursor_pointer()
                .child("Upload →"),
        )
        .child(
            div()
                .id("btn-download")
                .px(px(8.0))
                .py(px(4.0))
                .bg(gpui::rgb(0x3a4a6a))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xaabbdd))
                .rounded(px(3.0))
                .cursor_pointer()
                .child("← Download"),
        )
        .child(
            div()
                .id("toolbar-sep")
                .w(px(1.0))
                .h(px(16.0))
                .mx(px(4.0))
                .bg(gpui::rgb(0x3a3a3a)),
        )
        .child(
            div()
                .id("btn-newdir")
                .px(px(8.0))
                .py(px(4.0))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xcccccc))
                .rounded(px(3.0))
                .cursor_pointer()
                .child("📁 New Dir"),
        )
        .child(
            div()
                .id("btn-rename")
                .px(px(8.0))
                .py(px(4.0))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xcccccc))
                .rounded(px(3.0))
                .cursor_pointer()
                .child("✏️ Rename"),
        )
        .child(
            div()
                .id("btn-delete")
                .px(px(8.0))
                .py(px(4.0))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xcc7777))
                .rounded(px(3.0))
                .cursor_pointer()
                .child("🗑 Delete"),
        )
        .child(
            div()
                .id("toolbar-sep2")
                .w(px(1.0))
                .h(px(16.0))
                .mx(px(4.0))
                .bg(gpui::rgb(0x3a3a3a)),
        )
        .child(
            div()
                .id("btn-refresh")
                .px(px(8.0))
                .py(px(4.0))
                .text_size(px(12.0))
                .text_color(gpui::rgb(0xcccccc))
                .rounded(px(3.0))
                .cursor_pointer()
                .child("↻ Refresh"),
        )
}

/// Get the parent directory of a path.
fn parent_dir(path: &str) -> String {
    if path == "/" || path.is_empty() {
        return "/".to_string();
    }
    let trimmed = path.trim_end_matches('/');
    if let Some(parent) = trimmed.rsplitn(2, '/').nth(1) {
        if parent.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parent.trim_start_matches('/'))
        }
    } else {
        "/".to_string()
    }
}

/// List a local directory and convert entries to FileEntry.
async fn list_local_dir_task(path: &str) -> Result<Vec<crate::app::state::FileEntry>, std::io::Error> {
    let mut raw_entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(path).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let file_type = entry.file_type().await?;
        let metadata = entry.metadata().await?;

        let modified = metadata
            .modified()
            .ok()
            .map(|t| {
                let duration = t
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                format_datetime(duration.as_secs())
            })
            .unwrap_or_default();

        let permissions = if file_type.is_dir() {
            "drwx------".to_string()
        } else if metadata.permissions().readonly() {
            "-r--------".to_string()
        } else {
            "-rw-------".to_string()
        };

        raw_entries.push(crate::app::state::FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: file_type.is_dir(),
            size: metadata.len(),
            modified,
            permissions,
        });
    }

    // Sort: directories first, then by name
    raw_entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(raw_entries)
}

/// Format a Unix timestamp as a readable date-time string.
fn format_datetime(unix_secs: u64) -> String {
    let days = unix_secs / 86400;
    let years = 1970 + (days as f64 / 365.25) as u64;

    let secs_in_day = unix_secs % 86400;
    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;

    // Month/day approximation
    let month = ((days as f64 / 30.4375) % 12.0 + 1.0) as u64;
    let day = (days as f64 % 30.4375 + 1.0) as u64;

    format!("{:04}-{:02}-{:02} {:02}:{:02}", years, month.min(12), day.min(31), hours, minutes)
}
