use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub color: String,
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineStatus {
    pub timestamp: Option<i64>,
    pub name: String,
    pub memory: MemoryStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStatus {
    pub total_memory: u64,
    pub free_memory: u64,
    pub available_memory: u64,
    pub total_swap: u64,
    pub free_swap: u64,
}
