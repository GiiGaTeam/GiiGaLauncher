use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub engine_path: std::path::PathBuf,
    pub project_list_path: std::path::PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            engine_path: std::path::PathBuf::from("./Giiga/Engine.exe"),
            project_list_path: std::path::PathBuf::from("ProjectList.json"),
        }
    }
}
