use std::{
    io::Seek,
    path::{Path, PathBuf},
};

use chrono::Local;
use iced::{
    font::Weight,
    widget::{Button, Column, Container, Row, Text, TextInput},
    Element, Length, Task,
};

use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tracing::{error, info, warn};

use crate::project::Project;

use super::settings::Settings;

pub struct Launcher {
    settings: Settings,
    projects: Vec<Project>,
    new_project_name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadSettings(Settings),

    LoadProjects,
    LoadedProjects(Vec<Project>),

    CreateProject,
    CreatedProject(Result<PathBuf, String>),

    OpenProject(PathBuf),
    OpenedProject(Result<PathBuf, String>),

    NewProjectNameChanged(String),

    AddProject,
    TryAddProject(Option<PathBuf>),
}

impl Launcher {
    const LAUNCHER_SETTINGS_PATH: &'static str = "GiiGaLauncher.json";

    pub fn new() -> (Self, Task<Message>) {
        (
            Launcher {
                projects: Vec::new(),
                settings: Default::default(),
                new_project_name: Default::default(),
            },
            Task::perform(open_settings(Self::LAUNCHER_SETTINGS_PATH), |settings| {
                Message::LoadSettings(settings)
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadSettings(settings) => {
                self.settings = settings;
                Task::perform(async {}, |_| Message::LoadProjects)
            }
            Message::LoadProjects => Task::perform(
                load_projects(self.settings.project_list_path.clone()),
                |projects| Message::LoadedProjects(projects),
            ),
            Message::LoadedProjects(mut projects) => {
                projects.retain(|project| {
                    if project.path.exists() {
                        true
                    } else {
                        info!("Remove project from list: {:?}", project.path);
                        false
                    }
                });
                self.projects = projects;
                Task::none()
            }
            Message::CreateProject => {
                if self.new_project_name.is_empty() {
                    error!("The project name cannot be empty.");
                    return Task::none();
                }

                Task::perform(
                    create_new_project(
                        self.settings.engine_path.join(&self.settings.template_path),
                        self.new_project_name.clone(),
                    ),
                    Message::CreatedProject,
                )
            }
            Message::CreatedProject(result) => match result {
                Ok(path) => {
                    let project_name = if let Some(path) = path.file_stem() {
                        path.to_str().unwrap_or("New Project")
                    } else {
                        "New Project"
                    };

                    self.projects.push(Project {
                        title: project_name.to_string(),
                        path: path.clone(),
                        last_open_date: Local::now(),
                    });

                    Task::perform(async move { path }, move |path| Message::OpenProject(path))
                }
                Err(err) => {
                    error!("{}", err);
                    return Task::none();
                }
            },
            Message::OpenProject(path) => {
                let Some(project) = self.projects.iter_mut().find(|p| p.path == path) else {
                    return Task::none();
                };

                project.last_open_date = Local::now();

                Task::perform(
                    open_project(
                        self.settings.engine_path.clone(),
                        self.settings.editor_path.clone(),
                        project.path.clone(),
                    ),
                    Message::OpenedProject,
                )
            }
            Message::OpenedProject(result) => match result {
                Ok(path) => {
                    info!("Opened project: {:?}", path);
                    Task::none()
                }
                Err(err) => {
                    error!("{}", err);
                    Task::none()
                }
            },
            Message::NewProjectNameChanged(new_project_name) => {
                self.new_project_name = new_project_name;
                Task::none()
            }
            Message::AddProject => Task::perform(choose_path(), Message::TryAddProject),
            Message::TryAddProject(path) => {
                let Some(path) = path else {
                    error!("Folder not selected.");
                    return Task::none();
                };

                if self
                    .projects
                    .iter()
                    .find(|proj| proj.path == path)
                    .is_some()
                {
                    info!("Project already in list.");
                    return Task::none();
                }

                if !validate_project_folder(&path) {
                    error!("Folder doesn't contain a project.");
                    return Task::none();
                }

                let project_name = if let Some(path) = path.file_stem() {
                    path.to_str().unwrap_or("New Project")
                } else {
                    "New Project"
                };

                self.projects.push(Project {
                    title: project_name.to_string(),
                    path: path.clone(),
                    last_open_date: Local::now(),
                });

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let project_list =
            self.projects
                .iter()
                .fold(Column::new().spacing(20), |column, project| {
                    column.push(
                        Button::new(
                            Column::new()
                                .width(Length::Fill)
                                .push(Text::new(&project.title))
                                .push(
                                    Text::new(format!("Location: {:?}", project.path))
                                        .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
                                )
                                .push(
                                    Text::new(format!(
                                        "Last open date: {}",
                                        project.last_open_date.format("%d/%m/%Y %H:%M")
                                    ))
                                    .color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                                ),
                        )
                        .width(Length::Fill)
                        .on_press(Message::OpenProject(project.path.clone())),
                    )
                });

        let sidebar = Row::new().push(
            Column::new()
                .spacing(20)
                .push(
                    TextInput::new("Введите название проекта", &self.new_project_name)
                        .padding(10)
                        .on_input(Message::NewProjectNameChanged)
                        .width(Length::Fill),
                )
                .push(
                    Button::new(Text::new("Создать проект"))
                        .on_press(Message::CreateProject)
                        .width(iced::Length::Fill),
                )
                .push(
                    Button::new(Text::new("Добавить проект"))
                        .on_press(Message::AddProject)
                        .width(iced::Length::Fill),
                ),
        );

        let mut font = iced::Font::default();
        font.weight = Weight::Bold;

        let content = Column::new()
            .spacing(20)
            .push(
                Text::new("GiiGa Engine")
                    .size(30)
                    .width(Length::Fill)
                    .font(font)
                    .align_x(iced::alignment::Horizontal::Center),
            )
            .push(
                Row::new()
                    .spacing(20)
                    .push(
                        Column::new()
                            .width(Length::FillPortion(4))
                            .push(Text::new("Список проектов").size(20))
                            .push(project_list),
                    )
                    .push(
                        Column::new()
                            .width(Length::FillPortion(1))
                            .spacing(20)
                            .push(Text::new("Действия").size(20))
                            .push(sidebar),
                    ),
            );

        Container::new(content).padding(20).into()
    }
}

impl Drop for Launcher {
    fn drop(&mut self) {
        let Ok(mut fs) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.settings.project_list_path)
        else {
            warn!("Failed to open file with project list.");
            return;
        };

        if fs.seek(std::io::SeekFrom::Start(0)).is_err() {
            warn!("Failed to seek project list file.");
            return;
        }

        let writer = std::io::BufWriter::new(fs);

        if serde_json::to_writer_pretty(writer, &self.projects).is_err() {
            warn!("Failed to save project list file.");
        }
    }
}

async fn open_settings(path: impl AsRef<Path>) -> Settings {
    let Ok(fs) = std::fs::OpenOptions::new().open(path) else {
        return Default::default();
    };
    let reader = std::io::BufReader::new(fs);
    serde_json::from_reader::<_, Settings>(reader).unwrap_or_default()
}

async fn create_new_project(
    src: impl AsRef<Path> + Send + Sync + 'static,
    project_name: impl AsRef<Path>,
) -> Result<PathBuf, String> {
    let path = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .as_ref()
        .map(rfd::FileHandle::path)
        .map(Path::to_owned)
        .ok_or(())
        .map_err(|_| "Folder not selected".to_string())?
        .join(project_name);

    copy_dir_all(src, path.clone())
        .await
        .map_err(|err| err.to_string())?;

    Ok(path)
}

#[async_recursion::async_recursion]
async fn copy_dir_all(
    src: impl AsRef<Path> + Send + Sync + 'static,
    dst: impl AsRef<Path> + Send + Sync + 'static,
) -> Result<(), std::io::Error> {
    tokio::fs::create_dir_all(&dst).await?;
    let mut entries = tokio::fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let ty = entry.file_type().await?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        } else {
            tokio::fs::copy(entry.path(), dst.as_ref().join(entry.file_name())).await?;
        }
    }
    Ok(())
}

async fn load_projects(project_list_path: impl AsRef<Path>) -> Vec<Project> {
    let mut file = match tokio::fs::OpenOptions::new()
        .read(true)
        .open(project_list_path)
        .await
    {
        Ok(file) => file,
        Err(err) => {
            warn!("Failed to open file with project list: {}", err);
            return vec![];
        }
    };

    if file.seek(std::io::SeekFrom::Start(0)).await.is_err() {
        warn!("Failed to seek project list file.");
    }

    let mut buffer = String::new();

    if file.read_to_string(&mut buffer).await.is_err() {
        warn!("Failed to read project list file.");
    }

    serde_json::from_str(&buffer).unwrap_or_default()
}

async fn choose_path() -> Option<PathBuf> {
    let path = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .as_ref()
        .map(rfd::FileHandle::path)
        .map(Path::to_owned)?;

    Some(path)
}

async fn open_project(
    engine_path: impl AsRef<Path>,
    editor_path: impl AsRef<Path>,
    project_path: impl AsRef<Path>,
) -> Result<PathBuf, String> {
    const CREATE_NEW_CONSOLE: u32 = 0x00000010;

    tokio::process::Command::new(engine_path.as_ref().join(editor_path))
        .arg(project_path.as_ref().as_os_str())
        .current_dir(engine_path)
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok(project_path.as_ref().to_owned())
}

fn validate_project_folder(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    if !path.join("Assets").exists() {
        return false;
    }

    if !path.join("database.json").exists() {
        return false;
    }

    if !path.join("project.giga").exists() {
        return false;
    }

    true
}
