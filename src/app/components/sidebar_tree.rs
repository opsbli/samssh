//! Sidebar connection tree
//!
//! Left panel showing a group→host tree for managing SSH connections.
//! Uses gpui-component's Tree component for hierarchical display.

use gpui::*;
use gpui_component::{
    list::ListItem,
    tree::{TreeItem, TreeState},
    StyledExt,
};

use crate::app::state::{AppState, SidebarState};

/// Type of selection in the sidebar.
#[derive(Debug, Clone, PartialEq)]
pub enum SidebarSelection {
    Host(String),
    Group(String),
    None,
}

/// Sidebar component rendering the connection tree.
pub struct SidebarTree {
    tree_state: Entity<TreeState>,
}

impl SidebarTree {
    /// Create a new sidebar tree from the app state.
    pub fn new(cx: &mut Context<AppState>) -> Self {
        let sidebar = cx.entity().read(cx).sidebar.clone();
        let items = build_tree_items(&sidebar);

        let tree_state = cx.new(|cx| TreeState::new(cx).items(items));

        Self { tree_state }
    }

    /// Get the currently selected item.
    pub fn selected(&self, cx: &App) -> SidebarSelection {
        self.tree_state.read(cx).selected_entry().map_or(SidebarSelection::None, |entry| {
            let id = entry.item().id.to_string();
            if entry.is_folder() {
                SidebarSelection::Group(id)
            } else {
                SidebarSelection::Host(id)
            }
        })
    }

    /// Get tree state entity (for use in rendering).
    pub fn tree_state(&self) -> Entity<TreeState> {
        self.tree_state.clone()
    }
}

impl Render for SidebarTree {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.tree_state.clone();

        div()
            .id("sidebar-tree")
            .v_flex()
            .size_full()
            .px_1()
            .child(
                gpui_component::tree::tree(&state, |_ix, entry, selected, _window, _cx| {
                    let item = entry.item();
                    ListItem::new(item.id.clone())
                        .child(item.label.clone())
                        .selected(selected)
                })
                .context_menu(|_ix, _entry, menu, _window, _cx| menu),
            )
    }
}

/// Build gpui-component TreeItems from sidebar state.
fn build_tree_items(sidebar: &SidebarState) -> Vec<TreeItem> {
    let mut items = Vec::new();

    for group in &sidebar.groups {
        let mut group_item = TreeItem::new(&group.id, &group.name)
            .expanded(sidebar.is_expanded(&group.id));

        for host_id in &group.hosts {
            let host_item = TreeItem::new(host_id, host_id);
            group_item = group_item.child(host_item);
        }

        items.push(group_item);
    }

    items
}
