use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    pub path: std::path::PathBuf,
    pub last_open_date: chrono::DateTime<Local>,
}
