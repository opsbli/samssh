//! Connection profile dialogs
//!
//! Dialog for creating and editing SSH connection profiles.

use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    dialog::{Dialog, DialogFooter, DialogHeader, DialogTitle},
    input::Input,
    StyledExt, WindowExt,
};
use gpui_component::input::InputState;

use std::sync::Arc;
use std::time::SystemTime;

use crate::app::state::{AppState, SidebarState};
use crate::config::profile::{Profile, StoredAuthMethod};
use crate::config::store::ConfigStore;

/// Render a "New Connection" button that opens a dialog when clicked.
pub fn new_connection_button(
    cx: &mut App,
    app_state: Entity<AppState>,
) -> impl IntoElement {
    Dialog::new(cx)
        .trigger(
            Button::new("new-connection-btn")
                .icon(gpui_component::IconName::Plus)
                .ghost()
                .compact()
                .tooltip("New Connection"),
        )
        .content(move |content, window, cx| {
            let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("My Server"));
            let host_input = cx.new(|cx| InputState::new(window, cx).placeholder("hostname or IP"));
            let port_input = cx.new(|cx| InputState::new(window, cx).placeholder("22"));
            let user_input = cx.new(|cx| InputState::new(window, cx).placeholder("username"));
            let group_input = cx.new(|cx| InputState::new(window, cx).placeholder("optional group"));
            let pass_input = cx.new(|cx| InputState::new(window, cx).placeholder("password (optional)"));
            let key_input = cx.new(|cx| InputState::new(window, cx).placeholder("key path (optional)"));

            let as_save = app_state.clone();

            content
                .child(
                    DialogHeader::new()
                        .child(DialogTitle::new().child("New SSH Connection")),
                )
                .child(
                    div()
                        .v_flex()
                        .gap_3()
                        .px_4()
                        .py_4()
                        .child(form_field("Display Name", Input::new(&name_input).cleanable(true)))
                        .child(form_field("Host", Input::new(&host_input).cleanable(true)))
                        .child(
                            div().h_flex().gap_3()
                                .child(div().flex_1().child(form_field("Port", Input::new(&port_input))))
                                .child(div().flex_1().child(form_field("Username", Input::new(&user_input).cleanable(true)))),
                        )
                        .child(form_field("Password", Input::new(&pass_input).cleanable(true)))
                        .child(form_field("Key File", Input::new(&key_input).cleanable(true)))
                        .child(form_field("Group", Input::new(&group_input))),
                )
                .child(
                    DialogFooter::new()
                        .child(
                            Button::new("cancel-btn")
                                .label("Cancel")
                                .on_click(|_, window, cx| {
                                    window.close_dialog(cx);
                                }),
                        )
                        .child(
                            Button::new("save-btn")
                                .primary()
                                .label("Save")
                                .on_click(move |_, window, cx| {
                                    let name = name_input.read(cx).value().to_string();
                                    let host = host_input.read(cx).value().to_string();
                                    let port_str = port_input.read(cx).value().to_string();
                                    let username = user_input.read(cx).value().to_string();
                                    let group_str = group_input.read(cx).value().to_string();
                                    let password = pass_input.read(cx).value().to_string();
                                    let key_path = key_input.read(cx).value().to_string();

                                    if host.is_empty() || username.is_empty() {
                                        return;
                                    }

                                    let port: u16 = port_str.parse().unwrap_or(22);
                                    let id = format!("conn-{}",
                                        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap_or_default().as_secs());
                                    let display_name = if name.is_empty() { host.clone() } else { name };

                                    // Determine auth method: Key if key path given, Password if password given, else KBI
                                    let mut profile = if !key_path.is_empty() {
                                        let mut p = Profile::new(&id, &display_name, &host, &username);
                                        p.auth_method = StoredAuthMethod::Key {
                                            key_path,
                                            passphrase_encrypted: if password.is_empty() { None } else { Some(password) },
                                        };
                                        p
                                    } else if !password.is_empty() {
                                        let mut p = Profile::new(&id, &display_name, &host, &username);
                                        p.password_encrypted = Some(password);
                                        p
                                    } else {
                                        Profile::new(&id, &display_name, &host, &username)
                                    };
                                    profile.port = port;
                                    if !group_str.is_empty() {
                                        profile.group = Some(group_str);
                                    }

                                    // Save to config on disk
                                    let store = ConfigStore::new();
                                    let mut config = store.load().unwrap_or_default();
                                    config.profiles.push(profile.clone());
                                    if let Err(e) = store.save(&config) {
                                        tracing::warn!("Failed to save config: {}", e);
                                    }

                                    // Update AppState
                                    as_save.update(cx, move |state, _cx| {
                                        let mut cfg = (*state.config).clone();
                                        cfg.profiles.push(profile);
                                        state.config = Arc::new(cfg);
                                        state.sidebar = SidebarState::from_profiles(&state.config.profiles);
                                    });

                                    window.close_dialog(cx);
                                }),
                        ),
                )
        })
}

fn form_field(label_text: impl Into<SharedString>, input: impl IntoElement) -> impl IntoElement {
    let label: SharedString = label_text.into();
    div()
        .v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(gpui::opaque_grey(0.5, 0.7))
                .child(label),
        )
        .child(input)
}
