//! Sidebar connection tree
//!
//! Left panel showing a group→host tree for managing SSH connections.

use gpui::*;
use gpui_component::{
    list::ListItem,
    tree::{TreeItem, TreeState},
    StyledExt,
};
use gpui_component::menu::PopupMenuItem;

use std::sync::Arc;

use crate::app::state::{AppState, SidebarState};
use crate::config::profile::Profile;
use crate::config::store::ConfigStore;

/// Sidebar component rendering the connection tree.
pub struct SidebarTree {
    tree_state: Entity<TreeState>,
    app_state: Entity<AppState>,
}

impl SidebarTree {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let (sidebar, profiles) = {
            let state = app_state.read(cx);
            (state.sidebar.clone(), state.config.profiles.clone())
        };
        let items = build_tree_items(&sidebar, &profiles);
        let tree_state = cx.new(|cx| TreeState::new(cx).items(items));
        Self { tree_state, app_state }
    }

    pub fn rebuild(&mut self, cx: &mut Context<Self>) {
        let (sidebar, profiles) = {
            let state = self.app_state.read(cx);
            (state.sidebar.clone(), state.config.profiles.clone())
        };
        let items = build_tree_items(&sidebar, &profiles);
        self.tree_state.update(cx, |state, cx| {
            state.set_items(items, cx);
        });
    }
}

impl Render for SidebarTree {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.tree_state.clone();
        let app_state = self.app_state.clone();

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
                .context_menu(move |_ix, entry, menu, _window, _cx| {
                    let id = entry.item().id.to_string();
                    let is_folder = entry.is_folder();
                    let as_ = app_state.clone();

                    if is_folder {
                        let m = menu;
                        let id1 = id.clone();
                        let m = m.item(
                            PopupMenuItem::new("New Connection in Group")
                                .icon(gpui_component::IconName::Plus)
                                .on_click(move |_, _, _| {
                                    tracing::info!("New connection in group: {}", id1);
                                }),
                        );
                        let id2 = id.clone();
                        let m = m.separator().item(
                            PopupMenuItem::new("Rename Group")
                                .icon(gpui_component::IconName::Copy)
                                .on_click(move |_, _, _| {
                                    tracing::info!("Rename group: {}", id2);
                                }),
                        );
                        let id3 = id.clone();
                        let as_del = as_.clone();
                        m.separator().item(
                            PopupMenuItem::new("Delete Group")
                                .icon(gpui_component::IconName::Delete)
                                .on_click(move |_, _, cx| {
                                    as_del.update(cx, |state, _| {
                                        let mut cfg = (*state.config).clone();
                                        cfg.profiles.retain(|p| p.group.as_deref() != Some(&id3));
                                        state.config = Arc::new(cfg);
                                        state.sidebar = SidebarState::from_profiles(&state.config.profiles);
                                        let _ = ConfigStore::new().save(&state.config);
                                    });
                                }),
                        )
                    } else {
                        let m = menu;
                        let id1 = id.clone();
                        let m = m.item(
                            PopupMenuItem::new("Connect")
                                .icon(gpui_component::IconName::Play)
                                .on_click({
                                    let as_ = as_.clone();
                                    let hid = id.clone();
                                    move |_, _, cx| {
                                        let weak = as_.downgrade();
                                        if let Some(profile) = weak.upgrade().and_then(|s| {
                                            s.read(cx).config.profiles.iter()
                                                .find(|p| p.id == hid)
                                                .cloned()
                                        }) {
                                            crate::session::spawn_profile_connection(
                                                cx, weak, profile,
                                            );
                                        }
                                    }
                                }),
                        );
                        let id2 = id.clone();
                        let m = m.item(
                            PopupMenuItem::new("Connect SFTP")
                                .icon(gpui_component::IconName::Folder)
                                .on_click(move |_, _, _| {
                                    tracing::info!("SFTP to: {}", id2);
                                }),
                        );
                        let id3 = id.clone();
                        let m = m.separator().item(
                            PopupMenuItem::new("Edit Profile")
                                .icon(gpui_component::IconName::Settings)
                                .on_click(move |_, _, _| {
                                    tracing::info!("Edit profile: {}", id3);
                                }),
                        );
                        let id4 = id.clone();
                        let as_del = as_.clone();
                        m.separator().item(
                            PopupMenuItem::new("Delete")
                                .icon(gpui_component::IconName::Delete)
                                .on_click(move |_, _, cx| {
                                    as_del.update(cx, |state, _| {
                                        let mut cfg = (*state.config).clone();
                                        cfg.profiles.retain(|p| p.id != id4);
                                        state.config = Arc::new(cfg);
                                        state.sidebar = SidebarState::from_profiles(&state.config.profiles);
                                        let _ = ConfigStore::new().save(&state.config);
                                    });
                                }),
                        )
                    }
                }),
            )
    }
}

/// Build tree items using profile names as labels.
pub fn build_tree_items(sidebar: &SidebarState, profiles: &[Profile]) -> Vec<TreeItem> {
    let name_map: std::collections::HashMap<&str, &str> = profiles
        .iter()
        .map(|p| (p.id.as_str(), p.name.as_str()))
        .collect();

    let mut items = Vec::new();
    for group in &sidebar.groups {
        let mut group_item = TreeItem::new(&group.id, &group.name)
            .expanded(sidebar.is_expanded(&group.id));
        for host_id in &group.hosts {
            let display_name = name_map.get(host_id.as_str()).copied().unwrap_or(host_id);
            let host_item = TreeItem::new(host_id, display_name);
            group_item = group_item.child(host_item);
        }
        items.push(group_item);
    }
    items
}
