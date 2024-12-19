use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub engine_path: std::path::PathBuf,
    pub project_list_path: std::path::PathBuf,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            engine_path: std::path::PathBuf::from("./Giiga/Engine.exe"),
            project_list_path: std::path::PathBuf::from("GiigaLauncher.json"),
        }
    }
}
