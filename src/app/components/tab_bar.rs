//! Tab bar component
//!
//! Manages multiple terminal/SFTP session tabs in the main workspace area.
//! Uses gpui-component's TabBar for tab management.

use gpui::*;
use gpui_component::tab::Tab;

use crate::app::state::{SessionKind, TabState};

/// Build tabs from TabState.
pub fn build_tabs(state: &TabState) -> Vec<Tab> {
    let mut tabs = Vec::new();
    for tab in &state.tabs {
        let label = SharedString::from(format!(
            "{} {}",
            match tab.kind {
                SessionKind::Terminal => "\u{1f4bb}",
                SessionKind::SFTP => "\u{1f4c1}",
            },
            tab.title
        ));
        tabs.push(Tab::new().label(label));
    }
    tabs
}

/// Get the active tab index from TabState.
pub fn active_index(state: &TabState) -> Option<usize> {
    state.active.and_then(|active_id| {
        state.tabs.iter().position(|t| t.id == active_id)
    })
}
