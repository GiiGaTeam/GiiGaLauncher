use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub engine_path: std::path::PathBuf,
    pub editor_path: std::path::PathBuf,
    pub runtime_path: std::path::PathBuf,
    pub template_path: std::path::PathBuf,
    pub project_list_path: std::path::PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            engine_path: std::path::PathBuf::from("./Giiga"),
            editor_path: std::path::PathBuf::from("./Engine.exe"),
            runtime_path: std::path::PathBuf::from("./Runtime.exe"),
            template_path: std::path::PathBuf::from("./TemplateProject"),
            project_list_path: std::path::PathBuf::from("ProjectList.json"),
        }
    }
}
