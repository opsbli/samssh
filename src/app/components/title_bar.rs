//! Custom title bar
//!
//! Wraps gpui-component's TitleBar to provide window title and controls.

use gpui::*;
use gpui_component::{ActiveTheme, h_flex};

/// Create the custom title bar content.
pub fn render_title_bar(cx: &App) -> impl IntoElement {
    gpui_component::TitleBar::new()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child("SamSSH"),
                ),
        )
}
