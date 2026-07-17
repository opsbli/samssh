// SamSSH — A modern SSH client for Windows
// Copyright (c) 2026 Sam. All rights reserved.

use gpui::*;

fn main() {
    let app = App::new();
    app.run(move |cx| {
        let _window = cx
            .open_window(WindowOptions {
                titlebar: TitlebarOptions {
                    title: Some("SamSSH".into()),
                    ..TitlebarOptions::default()
                },
                window_bounds: Some(WindowBounds::Fixed(
                    Bounds::centered(None, size(px(1200.0), px(800.0)), cx),
                )),
                ..WindowOptions::default()
            })
            .expect("Failed to create main window");
    });
}
