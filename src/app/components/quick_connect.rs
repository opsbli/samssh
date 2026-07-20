//! Quick connect bar
//!
//! Inline input bar at the top of the sidebar for quick SSH connections.

use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    input::Input,
    StyledExt,
};
use gpui_component::input::InputState;

use crate::app::state::AppState;

/// Quick connect — rendered inline in the sidebar header.
pub struct QuickConnect {
    pub host_input: Option<Entity<InputState>>,
    pub user_input: Option<Entity<InputState>>,
}

impl QuickConnect {
    pub fn new() -> Self {
        Self {
            host_input: None,
            user_input: None,
        }
    }

    /// Lazily create InputStates on first render (when Window + cx are available).
    pub fn ensure_inputs(&mut self, window: &mut Window, cx: &mut App) {
        if self.host_input.is_none() {
            self.host_input = Some(
                cx.new(|cx| InputState::new(window, cx).placeholder("host:port")),
            );
        }
        if self.user_input.is_none() {
            self.user_input = Some(
                cx.new(|cx| InputState::new(window, cx).placeholder("username")),
            );
        }
    }

    pub fn parse_host(host_str: &str) -> (String, u16) {
        if let Some((host, port_str)) = host_str.rsplit_once(':') {
            let port: u16 = port_str.parse().unwrap_or(22);
            (host.to_string(), port)
        } else {
            (host_str.to_string(), 22)
        }
    }
}

/// Render the quick connect bar.
pub fn render_quick_connect(
    qc: &mut QuickConnect,
    window: &mut Window,
    cx: &mut App,
    app_state: Entity<AppState>,
) -> impl IntoElement {
    qc.ensure_inputs(window, cx);

    let host_input = qc.host_input.clone().unwrap();
    let user_input = qc.user_input.clone().unwrap();

    div()
        .id("quick-connect")
        .v_flex()
        .gap_1()
        .px_2()
        .py_2()
        .border_b_1()
        .border_color(gpui::opaque_grey(0.15, 1.0))
        .child(
            div()
                .h_flex()
                .gap_1()
                .child(div().flex_1().child(Input::new(&host_input).cleanable(true))),
        )
        .child(
            div()
                .h_flex()
                .gap_1()
                .child(div().flex_1().child(Input::new(&user_input).cleanable(true)))
                .child(
                    Button::new("quick-connect-btn")
                        .icon(gpui_component::IconName::Play)
                        .primary()
                        .compact()
                        .tooltip("Connect")
                        .on_click({
                            let host_input = host_input.clone();
                            let user_input = user_input.clone();
                            let app_state = app_state.clone();
                            move |_, _window, cx| {
                                let host_str = host_input.read(cx).value().to_string();
                                let username = user_input.read(cx).value().to_string();
                                if host_str.is_empty() || username.is_empty() {
                                    return;
                                }
                                let (host, port) = QuickConnect::parse_host(&host_str);
                                let weak = app_state.downgrade();
                                crate::session::spawn_ssh_connection(
                                    cx, weak, host, port, username, None,
                                );
                            }
                        }),
                ),
        )
}
