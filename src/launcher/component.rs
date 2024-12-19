use iced::{
    widget::{Button, Column, Container, Row, Text},
    Element, Length, Task,
};

use crate::project::Project;

pub struct Launcher {
    projects: Vec<Project>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadProjects,
    CreateProject,
    OpenProject,
    ProjectsLoaded(Vec<Project>),
}

impl Launcher {
    pub fn new() -> (Self, Task<Message>) {
        (
            Launcher {
                projects: Vec::new(),
            },
            Task::none(),
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
