use std::{
    io::Seek, os::windows::process::CommandExt, path::{Path, PathBuf}, process::Stdio
};

use chrono::Local;
use iced::{
    font::Weight,
    widget::{Button, Column, Container, Row, Text, TextInput},
    Element, Length, Task,
};

use tokio::io::{AsyncReadExt, AsyncSeekExt};

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
    ProjectsLoaded(Vec<Project>),

    CreateProject,
    CreatedProject(Result<PathBuf, ()>),

    OpenProject(PathBuf),

    NewProjectNameChanged(String),

    AddProject,
    TryAddProject(PathBuf),
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
                |projects| Message::ProjectsLoaded(projects),
            ),
            Message::ProjectsLoaded(projects) => {
                self.projects = projects;
                Task::none()
            }
            Message::CreateProject => {
                if self.new_project_name.is_empty() {
                    // TODO: Log
                    return Task::none();
                }

                Task::perform(
                    create_new_project(
                        self.settings.template_path.clone(),
                        self.new_project_name.clone(),
                    ),
                    Message::CreatedProject,
                )
            }
            Message::CreatedProject(result) => {
                let Ok(path) = result else {
                    // TODO: Log
                    return Task::none();
                };

                self.projects.push(Project {
                    title: path.file_stem().unwrap().to_string_lossy().to_string(),
                    path: path.clone(),
                    last_open_date: Local::now(),
                });

                Task::perform(async move { path }, move |path| Message::OpenProject(path))
            }
            Message::OpenProject(path) => {
                let Some(project) = self.projects.iter_mut().find(|p| p.path == path) else {
                    return Task::none();
                };

                project.last_open_date = Local::now();

                // TODO: Do crossplatform
                const CREATE_NEW_CONSOLE: u32 = 0x00000010;
                std::process::Command::new(&self.settings.engine_path)
                    .arg(path)
                    .current_dir(&self.settings.engine_path.parent().unwrap())
                    .creation_flags(CREATE_NEW_CONSOLE)
                    .spawn()
                    .unwrap();

                Task::none()
            }
            Message::NewProjectNameChanged(new_project_name) => {
                self.new_project_name = new_project_name;
                Task::none()
            }
            Message::AddProject => {
                Task::perform(choose_path(), |path| Message::TryAddProject(path.unwrap()))
            }
            Message::TryAddProject(path) => {
                if !path.join("Assets").exists() {
                    return Task::none();
                }

                if !path.join("database.json").exists() {
                    return Task::none();
                }

                if !path.join("project.giga").exists() {
                    return Task::none();
                }

                // TODO: Check already added project

                self.projects.push(Project {
                    title: path.file_stem().unwrap().to_string_lossy().to_string(),
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
            .open(&self.settings.project_list_path)
        else {
            // TODO: Log
            return;
        };

        if fs.seek(std::io::SeekFrom::Start(0)).is_err() {
            // TODO: Log
        }

        let writer = std::io::BufWriter::new(fs);

        if serde_json::to_writer_pretty(writer, &self.projects).is_err() {
            // TODO: Log
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
) -> Result<PathBuf, ()> {
    let path = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .as_ref()
        .map(rfd::FileHandle::path)
        .map(Path::to_owned)
        .ok_or(())?
        .join(project_name);

    copy_dir_all(src, path.clone()).await.map_err(|_| ())?;

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
    let Ok(mut file) = tokio::fs::File::open(project_list_path).await else {
        return vec![];
    };

    file.seek(std::io::SeekFrom::Start(0)).await.unwrap();

    let mut buffer = String::new();

    file.read_to_string(&mut buffer).await.unwrap();

    serde_json::from_str(&buffer).unwrap_or_default()
}

async fn choose_path() -> Result<PathBuf, ()> {
    let path = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .as_ref()
        .map(rfd::FileHandle::path)
        .map(Path::to_owned)
        .ok_or(())?;

    Ok(path)
}
