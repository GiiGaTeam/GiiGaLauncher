use std::path::{Path, PathBuf};

use iced::{
    widget::{Button, Column, Container, Row, Text},
    Element, Length, Task,
};

use crate::project::Project;

use super::settings::Settings;

pub struct Launcher {
    settings: Settings,
    projects: Vec<Project>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadSettings(Settings),
    LoadProjects,
    CreateProject,
    CreatedProject,
    OpenProject,
    ProjectsLoaded(Vec<Project>),
}

impl Launcher {
    const LAUNCHER_SETTINGS_PATH: &'static str = "GiiGaLauncher.json";

    pub fn new() -> (Self, Task<Message>) {
        (
            Launcher {
                projects: Vec::new(),
                settings: Default::default(),
            },
            Task::perform(open_settings(Self::LAUNCHER_SETTINGS_PATH), |settings| {
                Message::LoadSettings(settings)
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadProjects => Task::none(),
            Message::CreateProject => {
                println!("Create Project button clicked");
                Task::perform(
                    create_new_project(self.settings.template_path.clone(), None),
                    |_| Message::CreatedProject,
                )
            }
            Message::OpenProject => {
                println!("Open Project button clicked");
                Task::none()
            }
            Message::ProjectsLoaded(projects) => {
                self.projects = projects;
                Task::none()
            }
            Message::LoadSettings(settings) => {
                self.settings = settings;
                Task::none()
            }
            Message::CreatedProject => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let project_list = self.projects.iter().fold(Column::new(), |column, project| {
            column.push(Text::new(&project.title))
        });

        let buttons = Row::new().push(
            Column::new()
                .spacing(20)
                .push(
                    Button::new(Text::new("Создать проект"))
                        .on_press(Message::CreateProject)
                        .width(iced::Length::Fill),
                )
                .push(
                    Button::new(Text::new("Открыть проект"))
                        .on_press(Message::OpenProject)
                        .width(iced::Length::Fill),
                ),
        );

        let content = Row::new()
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
                    .push(buttons),
            );

        Container::new(content).padding(20).into()
    }
}

impl Drop for Launcher {
    fn drop(&mut self) {
        let Ok(fs) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.settings.project_list_path)
        else {
            // TODO: Log
            return;
        };

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
    dst: Option<PathBuf>,
) -> Result<PathBuf, ()> {
    let path = if let Some(path) = dst {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .pick_folder()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(())?
    };

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
