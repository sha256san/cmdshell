use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabItem {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabBarView {
    pub tabs: Vec<TabItem>,
    pub active_index: usize,
}

impl TabBarView {
    pub fn new(tabs: Vec<TabItem>, active_index: usize) -> Self {
        Self { tabs, active_index }
    }
}
