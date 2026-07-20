//! SSH session management — bridges gpui UI with async SSH connections.

use gpui::{App, AsyncApp, WeakEntity};

use crate::app::state::{AppState, SessionKind, SessionState};
use crate::ssh::client::SshClient;
use crate::ssh::{ConnectionStatus, SshConnectConfig, SshEvent};

/// Spawn an SSH connection from a UI context (button click etc.).
///
/// Creates a tab immediately, then connects in background.
pub fn spawn_ssh_connection(
    cx: &mut App,
    app_state: WeakEntity<AppState>,
    host: String,
    port: u16,
    username: String,
    profile_id: Option<String>,
) {
    let tab_id = create_tab(cx, &app_state, &host, port, &username, &profile_id);

    if let Some(tid) = tab_id {
        let as_ = app_state.clone();
        let h = host.clone();
        let u = username.clone();
        let p = port;

        cx.spawn(move |app: &mut AsyncApp| {
            let app = app.clone();
            async move {
                do_connect(&app, &as_, h, p, u, tid).await;
            }
        })
        .detach();
    }
}

/// Spawn an SSH connection using a saved profile.
pub fn spawn_profile_connection(
    cx: &mut App,
    app_state: WeakEntity<AppState>,
    profile: crate::config::profile::Profile,
) {
    let host = profile.host.clone();
    let port = profile.port;
    let username = profile.username.clone();
    let profile_id = Some(profile.id.clone());
    spawn_ssh_connection(cx, app_state, host, port, username, profile_id);
}

// ── Internal helpers ──

fn create_tab(
    cx: &mut App,
    app_state: &WeakEntity<AppState>,
    host: &str,
    port: u16,
    username: &str,
    profile_id: &Option<String>,
) -> Option<usize> {
    app_state.update(cx, |state, _| {
        let host_id = profile_id.clone().unwrap_or_else(|| format!("quick-{host}"));
        let label = format!("{}:{}", host, port);

        let tid = state.tabs.add_tab(label, SessionKind::Terminal, host_id.clone());
        let sid = next_session_id();

        state.sessions.insert(
            sid,
            SessionState {
                id: sid,
                host_id,
                kind: SessionKind::Terminal,
                status: ConnectionStatus::Connecting,
            },
        );
        state.tabs.set_session(tid, sid);
        tid
    })
    .ok()
}

async fn do_connect(
    app: &AsyncApp,
    app_state: &WeakEntity<AppState>,
    host: String,
    port: u16,
    username: String,
    tab_id: usize,
) {
    let (event_tx, mut event_rx) = crate::ssh::ssh_event_channel(64);

    set_status(app, app_state, tab_id, ConnectionStatus::Connecting).await;

    use crate::ssh::AuthMethod;

    let config = SshConnectConfig {
        host: host.clone(),
        port,
        username: username.clone(),
        auth_method: AuthMethod::KeyboardInteractive,
    };

    let client = SshClient::new(config);
    let _sid = client.session_id();

    match client.connect().await {
        Ok(handle) => {
            set_status(app, app_state, tab_id, ConnectionStatus::Connected).await;
            tracing::info!("SSH connected to {}:{}", host, port);

            match SshClient::open_session(&handle).await {
                Ok(_channel) => {
                    tracing::info!("Session opened for {}:{}", host, port);
                    // Monitor events
                    while let Some(event) = event_rx.recv().await {
                        match event {
                            SshEvent::Disconnected { reason, .. } => {
                                tracing::info!("Disconnected: {}", reason);
                                set_status(app, app_state, tab_id, ConnectionStatus::Disconnected)
                                    .await;
                                break;
                            }
                            SshEvent::Error { message, .. } => {
                                tracing::error!("SSH error: {}", message);
                                set_status(
                                    app,
                                    app_state,
                                    tab_id,
                                    ConnectionStatus::Error(message),
                                )
                                .await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    set_status(
                        app,
                        app_state,
                        tab_id,
                        ConnectionStatus::Error(e.to_string()),
                    )
                    .await;
                }
            }
        }
        Err(e) => {
            set_status(
                app,
                app_state,
                tab_id,
                ConnectionStatus::Error(e.to_string()),
            )
            .await;
        }
    }
}

async fn set_status(
    app: &AsyncApp,
    app_state: &WeakEntity<AppState>,
    tab_id: usize,
    status: ConnectionStatus,
) {
    let _ = app.update(|cx| {
        if let Some(state) = app_state.upgrade() {
            state.update(cx, |state, _| {
                if let Some(tab) = state.tabs.tabs.iter().find(|t| t.id == tab_id) {
                    if let Some(sid) = tab.session_id {
                        state.update_session_status(sid, status);
                    }
                }
            });
        }
    });
}

fn next_session_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
