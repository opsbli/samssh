//! Host key verification dialog

use gpui::*;
use gpui_component::{ActiveTheme, StyledExt, WindowExt};

/// Result of the host key verification dialog.
pub enum KeyVerifyResult {
    Accept,
    Reject,
}

/// Open a host key verification dialog.
pub fn show_key_verification_dialog(
    window: &mut Window,
    cx: &mut App,
    host: &str,
    fingerprint: &str,
    key_type: &str,
) {
    let host_owned = host.to_string();
    let fp_owned = fingerprint.to_string();
    let kt_owned = key_type.to_string();

    window.open_dialog(cx, {
        move |dialog, w, cx| {
            let _ = w;
            let host = host_owned.clone();
            let fp = fp_owned.clone();
            let kt = kt_owned.clone();

            dialog
                .title("Host Key Verification")
                .w(px(420.))
                .content(move |content, w2, cx| {
                    let _ = w2;
                    let host = host.clone();
                    let fp = fp.clone();
                    let kt = kt.clone();

                    content.child(
                        div()
                            .v_flex()
                            .gap_4()
                            .px_4()
                            .py_4()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().foreground)
                                    .child("The authenticity of the host can't be established."),
                            )
                            .child(
                                div().v_flex().gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(gpui::opaque_grey(0.5, 0.7))
                                            .child(format!("{} ({})", host, kt)),
                                    )
                                    .child(
                                        div()
                                            .font_family("monospace")
                                            .text_xs()
                                            .text_color(gpui::opaque_grey(0.7, 0.9))
                                            .child(format!("Fingerprint: {}", fp)),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Are you sure you want to continue connecting?"),
                            ),
                    )
                })
        }
    });
}
