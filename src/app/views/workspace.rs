//! Main workspace layout
//!
//! Combines the title bar, sidebar, and tabbed content area into the primary
//! application layout.

use gpui::*;
use gpui_component::{StyledExt, ActiveTheme};

use crate::app::components::title_bar;

/// Main workspace view containing the entire application layout.
pub struct Workspace;

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("workspace")
            .v_flex()
            .size_full()
            .bg(cx.theme().background)
            .child(title_bar::render_title_bar(cx))
            .flex_1()
            .child(
                div()
                    .id("main-content")
                    .flex()
                    .flex_row()
                    .size_full()
                    .child(
                        div()
                            .id("sidebar-panel")
                            .w(px(240.0))
                            .h_full()
                            .bg(cx.theme().background)
                            .border_r_1()
                            .border_color(cx.theme().border)
                            .child("Sidebar"),
                    )
                    .flex_1()
                    .child(
                        div()
                            .id("content-area")
                            .size_full()
                            .bg(cx.theme().background)
                            .child(
                                div()
                                    .id("tab-placeholder")
                                    .size_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Connect to a server to get started"),
                            ),
                    ),
            )
    }
}
