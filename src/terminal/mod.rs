//! Terminal emulation
//!
//! Wraps wezterm-term (via tattoy-wezterm-term fork) to provide
//! VT100/xterm compatible terminal emulation for SSH sessions.
//! Processes raw byte data from SSH channels and produces
//! a screen buffer that can be rendered by gpui.

pub mod terminal;
pub use terminal::*;
