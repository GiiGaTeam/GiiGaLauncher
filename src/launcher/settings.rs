use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub engine_path: std::path::PathBuf,
    pub runtime_path: std::path::PathBuf,
    pub template_path: std::path::PathBuf,
    pub project_list_path: std::path::PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            engine_path: std::path::PathBuf::from("./Giiga/Engine.exe"),
            runtime_path: std::path::PathBuf::from("./Giiga/Runtime.exe"),
            template_path: std::path::PathBuf::from("./Giiga/TemplateProject"),
            project_list_path: std::path::PathBuf::from("ProjectList.json"),
        }
    }
}
