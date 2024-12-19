use std::path::Path;

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
            Task::perform(async {
                open_settings(Self::LAUNCHER_SETTINGS_PATH).await
            }, |settings| {
                Message::LoadSettings(settings)
            })
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadProjects => Task::none(),
            Message::CreateProject => {
                println!("Create Project button clicked");
                Task::none()
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
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        let project_list = self.projects.iter().fold(Column::new(), |column, project| {
            column.push(Text::new(&project.title))
        });

        let buttons = Row::new().push(
            Column::new()
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
    let Ok(fs) = std::fs::OpenOptions::new()
        .open(path) else {
            return Default::default();
        };
    let reader = std::io::BufReader::new(fs);
    serde_json::from_reader::<_, Settings>(reader).unwrap_or_default()
}
