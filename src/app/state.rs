//! Application state
//!
//! Core application state managed by gpui's Entity<T> model.
//! Contains sidebar, tab, and session state for the main window.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::config::profile::Profile;
use crate::ssh::ConnectionStatus;

// ── Type aliases ──

pub type GroupId = String;
pub type HostId = String;
pub type TabId = usize;
pub type SessionId = u64;

// ── Sidebar State ──

/// A group node in the sidebar tree, containing profiles.
#[derive(Debug, Clone)]
pub struct GroupNode {
    pub id: GroupId,
    pub name: String,
    pub hosts: Vec<HostId>,
    pub expanded: bool,
}

/// State for the sidebar connection tree.
#[derive(Debug, Clone)]
pub struct SidebarState {
    /// Groups in the sidebar tree.
    pub groups: Vec<GroupNode>,
    /// Set of expanded group IDs.
    pub expanded: HashSet<GroupId>,
    /// Currently selected host ID.
    pub selected: Option<HostId>,
    /// Quick connect input text.
    pub quick_connect: String,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            expanded: HashSet::new(),
            selected: None,
            quick_connect: String::new(),
        }
    }

    /// Build sidebar tree from a list of profiles.
    pub fn from_profiles(profiles: &[Profile]) -> Self {
        let mut groups: HashMap<String, Vec<HostId>> = HashMap::new();
        let mut ungrouped = Vec::new();

        for profile in profiles {
            match &profile.group {
                Some(g) => groups.entry(g.clone()).or_default().push(profile.id.clone()),
                None => ungrouped.push(profile.id.clone()),
            }
        }

        let mut group_nodes: Vec<GroupNode> = groups
            .into_iter()
            .map(|(id, hosts)| GroupNode {
                id: id.clone(),
                name: id,
                hosts,
                expanded: true,
            })
            .collect();
        group_nodes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let expanded: HashSet<GroupId> = group_nodes.iter().map(|g| g.id.clone()).collect();

        if !ungrouped.is_empty() {
            group_nodes.insert(
                0,
                GroupNode {
                    id: "__ungrouped__".into(),
                    name: "Ungrouped".into(),
                    hosts: ungrouped,
                    expanded: true,
                },
            );
        }

        Self {
            groups: group_nodes,
            expanded,
            selected: None,
            quick_connect: String::new(),
        }
    }

    /// Toggle a group's expanded state.
    pub fn toggle_group(&mut self, group_id: &str) {
        if self.expanded.contains(group_id) {
            self.expanded.remove(group_id);
        } else {
            self.expanded.insert(group_id.to_string());
        }
    }

    /// Select a host.
    pub fn select_host(&mut self, host_id: HostId) {
        self.selected = Some(host_id);
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// Check if a group is expanded.
    pub fn is_expanded(&self, group_id: &str) -> bool {
        self.expanded.contains(group_id)
    }
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tab State ──

/// Type of a session tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Terminal,
    SFTP,
}

/// A single tab item in the tab bar.
#[derive(Debug, Clone)]
pub struct TabItem {
    pub id: TabId,
    pub title: String,
    pub kind: SessionKind,
    pub host_id: HostId,
    pub session_id: Option<SessionId>,
}

/// State for the tab bar and session management.
#[derive(Debug, Clone)]
pub struct TabState {
    pub tabs: Vec<TabItem>,
    pub active: Option<TabId>,
    pub next_id: TabId,
}

impl TabState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: None,
            next_id: 1,
        }
    }

    /// Add a new tab and make it active.
    pub fn add_tab(&mut self, title: String, kind: SessionKind, host_id: HostId) -> TabId {
        let id = self.next_id;
        self.next_id += 1;

        self.tabs.push(TabItem {
            id,
            title,
            kind,
            host_id,
            session_id: None,
        });
        self.active = Some(id);
        id
    }

    /// Remove a tab by ID.
    pub fn remove_tab(&mut self, id: TabId) {
        let idx = self.tabs.iter().position(|t| t.id == id);
        if let Some(i) = idx {
            self.tabs.remove(i);

            // If we removed the active tab, pick a new one
            if self.active == Some(id) {
                self.active = self.tabs.last().map(|t| t.id);
            }
        }
    }

    /// Switch to a tab by ID.
    pub fn switch_to(&mut self, id: TabId) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active = Some(id);
        }
    }

    /// Get the active tab item.
    pub fn active_tab(&self) -> Option<&TabItem> {
        self.active.and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }

    /// Associate a session with a tab.
    pub fn set_session(&mut self, tab_id: TabId, session_id: SessionId) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.session_id = Some(session_id);
        }
    }
}

impl Default for TabState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Session State ──

/// Runtime state for a single SSH session.
#[derive(Debug, Clone)]
pub struct SessionState {
    pub id: SessionId,
    pub host_id: HostId,
    pub kind: SessionKind,
    pub status: ConnectionStatus,
}

// ── File Browser State ──

/// A file/directory entry for file browser display.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

/// State for a single file browser panel (used by both local and remote).
#[derive(Debug, Clone)]
pub struct FileBrowserState {
    pub current_path: String,
    pub entries: Vec<FileEntry>,
    pub loading: bool,
    pub error: Option<String>,
    pub selected: Option<usize>,
}

impl FileBrowserState {
    pub fn new(path: String) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
            loading: false,
            error: None,
            selected: None,
        }
    }

    /// Update entries and clear loading/error state.
    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries = entries;
        self.loading = false;
        self.error = None;
    }

    /// Set error state.
    pub fn set_error(&mut self, err: String) {
        self.loading = false;
        self.error = Some(err);
    }
}

/// Current file operation being performed.
#[derive(Debug, Clone)]
pub enum FileOperation {
    None,
    Deleting { path: String },
    Renaming { old_path: String, new_name: String },
    CreatingDir { parent_path: String },
    Uploading { local_path: String, remote_path: String },
    Downloading { remote_path: String, local_path: String },
}

/// State for the SFTP dual-panel file manager, keyed by session ID.
#[derive(Debug, Clone)]
pub struct FileManagerState {
    pub local: FileBrowserState,
    pub remote: FileBrowserState,
    pub operation: FileOperation,
}

impl FileManagerState {
    pub fn new(remote_path: String) -> Self {
        // Local initial path: user's home directory or Documents
        let local_path = dirs_next::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\".to_string());

        Self {
            local: FileBrowserState::new(local_path),
            remote: FileBrowserState::new(remote_path),
            operation: FileOperation::None,
        }
    }
}

// ── AppState (top-level) ──

/// Top-level application state root entity.
pub struct AppState {
    pub sidebar: SidebarState,
    pub tabs: TabState,
    pub sessions: HashMap<SessionId, SessionState>,
    pub sftp_state: HashMap<SessionId, FileManagerState>,
    pub config: Arc<crate::config::store::Config>,
}

impl AppState {
    pub fn new(config: crate::config::store::Config) -> Self {
        let sidebar = SidebarState::from_profiles(&config.profiles);
        Self {
            sidebar,
            tabs: TabState::new(),
            sessions: HashMap::new(),
            sftp_state: HashMap::new(),
            config: Arc::new(config),
        }
    }

    /// Get or create a session state.
    pub fn get_or_create_session(&mut self, id: SessionId, host_id: HostId, kind: SessionKind) -> &mut SessionState {
        self.sessions.entry(id).or_insert(SessionState {
            id,
            host_id,
            kind,
            status: ConnectionStatus::Disconnected,
        })
    }

    /// Update connection status for a session.
    pub fn update_session_status(&mut self, session_id: SessionId, status: ConnectionStatus) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.status = status;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_from_profiles() {
        let profiles = vec![
            Profile::new("s1", "Server 1", "10.0.0.1", "admin"),
            Profile::new("s2", "Server 2", "10.0.0.2", "root"),
        ];
        let sidebar = SidebarState::from_profiles(&profiles);
        assert_eq!(sidebar.groups.len(), 1); // "Ungrouped" group
        assert_eq!(sidebar.groups[0].hosts.len(), 2);
    }

    #[test]
    fn test_sidebar_grouped_profiles() {
        // Add group info via profile fields
        let mut p1 = Profile::new("s1", "Web", "web.example.com", "admin");
        p1.group = Some("Production".into());
        let mut p2 = Profile::new("s2", "DB", "db.example.com", "admin");
        p2.group = Some("Production".into());

        let sidebar = SidebarState::from_profiles(&[p1, p2]);
        assert_eq!(sidebar.groups.len(), 1);
        assert_eq!(sidebar.groups[0].name, "Production");
        assert_eq!(sidebar.groups[0].hosts.len(), 2);
    }

    #[test]
    fn test_sidebar_toggle_group() {
        let mut sidebar = SidebarState::new();
        sidebar.groups.push(GroupNode {
            id: "g1".into(),
            name: "Group 1".into(),
            hosts: vec![],
            expanded: true,
        });
        sidebar.expanded.insert("g1".into());

        assert!(sidebar.is_expanded("g1"));
        sidebar.toggle_group("g1");
        assert!(!sidebar.is_expanded("g1"));
        sidebar.toggle_group("g1");
        assert!(sidebar.is_expanded("g1"));
    }

    #[test]
    fn test_sidebar_select_host() {
        let mut sidebar = SidebarState::new();
        assert!(sidebar.selected.is_none());
        sidebar.select_host("host-1".into());
        assert_eq!(sidebar.selected, Some("host-1".into()));
        sidebar.clear_selection();
        assert!(sidebar.selected.is_none());
    }

    #[test]
    fn test_tab_add_and_switch() {
        let mut tabs = TabState::new();
        let id1 = tabs.add_tab("Server 1".into(), SessionKind::Terminal, "s1".into());
        let id2 = tabs.add_tab("File Browser".into(), SessionKind::SFTP, "s2".into());
        assert_eq!(tabs.tabs.len(), 2);
        assert_eq!(tabs.active, Some(id2));

        tabs.switch_to(id1);
        assert_eq!(tabs.active, Some(id1));
    }

    #[test]
    fn test_tab_remove_active() {
        let mut tabs = TabState::new();
        let id1 = tabs.add_tab("Tab 1".into(), SessionKind::Terminal, "s1".into());
        let id2 = tabs.add_tab("Tab 2".into(), SessionKind::Terminal, "s2".into());
        tabs.switch_to(id1);

        tabs.remove_tab(id1);
        assert_eq!(tabs.active, Some(id2));
        assert_eq!(tabs.tabs.len(), 1);
    }

    #[test]
    fn test_tab_session_association() {
        let mut tabs = TabState::new();
        let id = tabs.add_tab("Test".into(), SessionKind::Terminal, "s1".into());
        assert!(tabs.active_tab().unwrap().session_id.is_none());
        tabs.set_session(id, 42);
        assert_eq!(tabs.active_tab().unwrap().session_id, Some(42));
    }

    #[test]
    fn test_app_state_creation() {
        let config = crate::config::store::Config::default();
        let state = AppState::new(config);
        assert!(state.tabs.tabs.is_empty());
        assert!(state.sessions.is_empty());
    }

    #[test]
    fn test_app_state_session_management() {
        let config = crate::config::store::Config::default();
        let mut state = AppState::new(config);
        let session = state.get_or_create_session(1, "s1".into(), SessionKind::Terminal);
        assert_eq!(session.status, ConnectionStatus::Disconnected);

        state.update_session_status(1, ConnectionStatus::Connected);
        assert_eq!(state.sessions[&1].status, ConnectionStatus::Connected);
    }

    // ── File Browser State Tests ──

    #[test]
    fn test_file_browser_state_new() {
        let state = FileBrowserState::new("/home/user".to_string());
        assert_eq!(state.current_path, "/home/user");
        assert!(state.entries.is_empty());
        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(state.selected.is_none());
    }

    #[test]
    fn test_file_browser_state_set_entries() {
        let mut state = FileBrowserState::new("/".to_string());
        state.loading = true;
        let entries = vec![
            FileEntry {
                name: "file.txt".into(),
                is_dir: false,
                size: 100,
                modified: "12:00".into(),
                permissions: "-rw-------".into(),
            },
            FileEntry {
                name: "docs".into(),
                is_dir: true,
                size: 0,
                modified: "10:00".into(),
                permissions: "drwx------".into(),
            },
        ];
        state.set_entries(entries.clone());
        assert_eq!(state.entries.len(), 2);
        assert!(!state.loading);
        assert!(state.error.is_none());
        assert_eq!(state.entries[0].name, "file.txt");
        assert!(state.entries[1].is_dir);
    }

    #[test]
    fn test_file_browser_state_error() {
        let mut state = FileBrowserState::new("/".to_string());
        state.loading = true;
        state.set_error("Permission denied".to_string());
        assert!(!state.loading);
        assert_eq!(state.error, Some("Permission denied".to_string()));
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry {
            name: "test.bin".into(),
            is_dir: false,
            size: 2048,
            modified: "14:30".into(),
            permissions: "-rw-r--r--".into(),
        };
        assert_eq!(entry.name, "test.bin");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 2048);
    }

    #[test]
    fn test_file_manager_state_new() {
        let state = FileManagerState::new("/remote/path".to_string());
        assert_eq!(state.remote.current_path, "/remote/path");
        assert_eq!(state.local.current_path, dirs_next::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\".to_string()));
        assert!(matches!(state.operation, FileOperation::None));
    }
}
