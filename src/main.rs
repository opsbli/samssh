// SamSSH — A modern SSH client for Windows
// Copyright (c) 2026 Sam. All rights reserved.
//
// Binary entry point. Initializes gpui Application, gpui-component,
// loads configuration, and opens the main workspace window.

use gpui::*;
use gpui_component::Root;

use samssh::app::state::AppState;
use samssh::app::views::workspace::Workspace;
use samssh::config::store::{Config, ConfigStore};

fn main() {
    // Initialize tracing/logging (respects RUST_LOG env var)
    samssh::logger::init();

    tracing::info!("SamSSH starting...");

    // Load configuration from %APPDATA%/SamSSH/config.json
    let config_store = ConfigStore::new();
    let config = match config_store.load() {
        Ok(cfg) => {
            tracing::info!("Configuration loaded from: {:?}", config_store.path());
            cfg
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load config: {}, using defaults. Path: {:?}",
                e,
                config_store.path()
            );
            Config::default()
        }
    };

    tracing::info!("Starting gpui application...");

    // Initialize the gpui platform (Windows backend via wgpu/DX12)
    gpui_platform::application().run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        // Create the app state root entity, seeded with loaded config
        let app_state = cx.new(|_| AppState::new(config.clone()));

        // Open the main application window asynchronously
        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::new(px(100.0), px(100.0)),
                        size: Size { width: px(1024.0), height: px(768.0) },
                    })),
                    ..Default::default()
                },
                |window, cx| {
                    // Create the workspace view with app state reference
                    let workspace = cx.new(|cx| Workspace::new(app_state.clone(), cx));

                    // Wrap in gpui-component Root (required first-level element)
                    cx.new(|cx| Root::new(workspace, window, cx))
                },
            )
            .expect("Failed to open main window");

            tracing::info!("SamSSH main window opened successfully");
        })
        .detach();
    });

    // Note: control never reaches here — gpui_platform::application().run() blocks
    // until the window is closed.
}
