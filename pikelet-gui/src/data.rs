//! Application state.

use druid::{Data, Lens};
use std::sync::Arc;

/// The top level data structure.
#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub globals: Arc<pikelet::core::Globals>,
    pub menu_count: usize,
    pub selected: usize,
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            globals: Arc::new(pikelet::core::Globals::default()),
            menu_count: 0,
            selected: 0,
        }
    }
}
