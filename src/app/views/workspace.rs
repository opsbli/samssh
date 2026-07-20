//! Main workspace layout
//!
//! Combines the title bar, sidebar (with quick connect), tab bar, and
//! content area. Holds the root Entity<AppState> for state access.

use gpui::*;
use gpui_component::{
    tab::{Tab, TabBar},
    ActiveTheme, StyledExt, WindowExt,
};

use crate::app::state::{AppState, SessionKind};
use crate::app::components::{
    dialog,
    quick_connect::{QuickConnect, render_quick_connect},
    sidebar_tree::SidebarTree,
    title_bar,
};
use crate::app::views::terminal_view::TerminalView;

/// Main workspace view.
pub struct Workspace {
    pub app_state: Entity<AppState>,
    quick_connect: QuickConnect,
    sidebar_tree: Entity<SidebarTree>,
    terminal: Entity<TerminalView>,
}

impl Workspace {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let sidebar_tree = cx.new(|cx| SidebarTree::new(app_state.clone(), cx));
        let terminal = cx.new(|_| TerminalView::new(24, 80));
        Self {
            app_state,
            quick_connect: QuickConnect::new(),
            sidebar_tree,
            terminal,
        }
    }

    /// Build gpui-component Tab items from TabState.
    fn build_tabs(tabs: &crate::app::state::TabState) -> Vec<Tab> {
        let mut tab_vec = Vec::new();
        for tab in &tabs.tabs {
            let icon = match tab.kind {
                SessionKind::Terminal => gpui_component::IconName::SquareTerminal,
                SessionKind::SFTP => gpui_component::IconName::Folder,
            };
            tab_vec.push(
                Tab::new()
                    .icon(icon)
                    .label(tab.title.clone()),
            );
        }
        tab_vec
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = self.app_state.read(cx).tabs.clone();
        let active_tab = tabs.active;

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
                    // ── Sidebar panel ──
                    .child(
                        div()
                            .id("sidebar-panel")
                            .w(px(280.0))
                            .h_full()
                            .bg(cx.theme().background)
                            .border_r_1()
                            .border_color(cx.theme().border)
                            .v_flex()
                            .child(render_quick_connect(
                                &mut self.quick_connect,
                                window,
                                cx,
                                self.app_state.clone(),
                            ))
                            .child(
                                div()
                                    .id("saved-connections-header")
                                    .h(px(28.0))
                                    .px_2()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(gpui::opaque_grey(0.5, 0.7))
                                            .child("Saved Connections"),
                                    )
                                    .child({
                                        dialog::new_connection_button(cx, self.app_state.clone())
                                    }),
                            )
                            .child(
                                div()
                                    .id("sidebar-content")
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .child(self.sidebar_tree.clone()),
                            ),
                    )
                    // ── Content area with tab bar ──
                    .flex_1()
                    .child(
                        div()
                            .id("content-panel")
                            .v_flex()
                            .size_full()
                            .bg(cx.theme().background)
                            // Tab bar
                            .child({
                                let tab_bar_content: gpui::AnyElement = if tabs.tabs.is_empty() {
                                    div()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .px_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(gpui::opaque_grey(0.4, 0.7))
                                                .child("No active sessions"),
                                        )
                                        .into_any_element()
                                } else {
                                    TabBar::new("session-tabs")
                                        .underline()
                                        .children(Self::build_tabs(&tabs))
                                        .selected_index(
                                            active_tab.and_then(|id| {
                                                tabs.tabs.iter().position(|t| t.id == id)
                                            }).unwrap_or(0),
                                        )
                                        .on_click({
                                            let app_state = self.app_state.clone();
                                            move |ix, _window, cx| {
                                                let tab_id = app_state.read(cx).tabs.tabs.get(*ix).map(|t| t.id);
                                                if let Some(tid) = tab_id {
                                                    app_state.update(cx, |state, _| {
                                                        state.tabs.switch_to(tid);
                                                    });
                                                }
                                            }
                                        })
                                        .into_any_element()
                                };
                                div()
                                    .id("tab-bar-area")
                                    .h(px(36.0))
                                    .bg(cx.theme().background)
                                    .border_b_1()
                                    .border_color(cx.theme().border)
                                    .child(tab_bar_content)
                            })
                            // Content area — show terminal when tabs exist
                            .child(
                                div()
                                    .id("content-area")
                                    .flex_1()
                                    .size_full()
                                    .child(if tabs.tabs.is_empty() {
                                        div()
                                            .id("tab-placeholder")
                                            .size_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Connect to a server to get started")
                                            .into_any_element()
                                    } else {
                                        self.terminal.clone().into_any_element()
                                    }),
                            ),
                    ),
            )
    }
}
