use std::time::Duration;

use iced::{
    Border, Color, ContentFit, Element, Length, Padding, Subscription, Task,
    Theme, time,
    widget::{
        Row, button, column, container, image, row, scrollable, text,
        text_input,
    },
};

use crate::{
    projects::{
        Project, ProjectStatus, get_cached_projects, install_project,
        launch_project, uninstall_project,
    },
    utils::ToKebabCase,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    Launch,
    Ignore,
}

#[derive(Clone, Debug)]
pub enum Message {
    SearchChanged(String),
    LaunchRequested(usize),
    ProjectSynced(usize, SyncAction),
    StopProject(usize),
    Tick,
    InstallProject(usize),
    UninstallProject(usize),
    ProjectUninstalled(usize),
}

pub struct App {
    projects: Vec<Project>,
    search: String,
}

impl App {
    pub fn new() -> Self {
        let cached_projects = get_cached_projects();

        let make = |name: &str, owner: &str, repo: &str, image: &[u8]| {
            let id = name.to_kebab_case();
            Project::new(
                &id,
                name,
                owner,
                repo,
                image::Handle::from_bytes(image.to_vec()),
                cached_projects.contains(&id),
            )
        };
        Self {
            search: String::new(),
            projects: vec![
                make(
                    "Pokemon battle simulator",
                    "Nikolai Ciric",
                    "https://github.com/nikcir/Java-Pokemon-battle-simulator",
                    include_bytes!(
                        "../assets/previews/pokemon-battle-simulator.png"
                    ),
                ),
                make(
                    "PacMan",
                    "Kristoffer Nergaard",
                    "https://github.com/Superkriss0911/PacMan",
                    include_bytes!("../assets/previews/pacman.png"),
                ),
                make(
                    "Piano Quiz",
                    "Sven Elden",
                    "https://github.com/Svela002/Piano-Quiz",
                    include_bytes!("../assets/previews/piano-quiz.png"),
                ),
                make(
                    "Attendance",
                    "Angelica Yen Skarsaune",
                    "https://github.com/ayskarsaune/tdt4100-prosjekt",
                    include_bytes!("../assets/previews/attendance.png"),
                ),
                make(
                    "Chess",
                    "Sander Kjeøy",
                    "https://github.com/SnadderCode/tdt4100-prosjekt-sjakk",
                    include_bytes!("../assets/previews/chess.png"),
                ),
            ],
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(s) => self.search = s,
            Message::LaunchRequested(i) => {
                if let Some(mut project) = self.projects.get_mut(i) {
                    if project.is_installed {
                        launch_project(&mut project);
                    } else {
                        project.status = ProjectStatus::Installing;
                        return Task::perform(
                            install_project(
                                project.id.clone(),
                                project.repo.clone(),
                            ),
                            move |_| {
                                Message::ProjectSynced(i, SyncAction::Launch)
                            },
                        );
                    }
                }
            }
            Message::ProjectSynced(i, action) => {
                if let Some(mut project) = self.projects.get_mut(i) {
                    project.is_installed = true;

                    if action == SyncAction::Launch {
                        launch_project(&mut project);
                    } else {
                        project.status = ProjectStatus::Idle;
                    }
                }
            }
            Message::StopProject(i) => {
                if let Some(project) = self.projects.get_mut(i) {
                    if let Some(child) = &mut project.child {
                        child.kill().unwrap();
                    }

                    project.status = ProjectStatus::Idle
                }
            }
            Message::Tick => {
                for project in self.projects.iter_mut() {
                    if let Some(child) = &mut project.child {
                        if let Ok(Some(_status)) = child.try_wait() {
                            project.child = None;
                            project.status = ProjectStatus::Idle;
                        }
                    }
                }
            }
            Message::InstallProject(i) => {
                if let Some(project) = self.projects.get_mut(i) {
                    assert!(!project.is_installed);

                    project.status = ProjectStatus::Installing;
                    return Task::perform(
                        install_project(
                            project.id.clone(),
                            project.repo.clone(),
                        ),
                        move |_| Message::ProjectSynced(i, SyncAction::Ignore),
                    );
                }
            }
            Message::UninstallProject(i) => {
                if let Some(project) = self.projects.get_mut(i) {
                    assert!(project.is_installed);

                    project.status = ProjectStatus::Uninstalling;
                    return Task::perform(
                        uninstall_project(project.id.clone()),
                        move |_| Message::ProjectUninstalled(i),
                    );
                }
            }
            Message::ProjectUninstalled(i) => {
                if let Some(project) = self.projects.get_mut(i) {
                    assert!(project.status == ProjectStatus::Uninstalling);

                    project.is_installed = false;
                    project.status = ProjectStatus::Idle;
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = column![
            text("Project catalog").size(22),
            text("Abacord JavaFx projects").size(14)
        ]
        .spacing(4);

        let search_bar = text_input("Search projects...", &self.search)
            .on_input(Message::SearchChanged)
            .padding(10)
            .width(Length::Fill);

        let query = self.search.to_lowercase();

        let cards: Vec<_> = self
            .projects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name.to_lowercase().contains(&query))
            .map(|(i, p)| project_card(i, p))
            .collect();

        let body: Element<Message> = if cards.is_empty() {
            container(text("No projects match your search.").size(14))
                .padding(32)
                .center_x(Length::Fill)
                .into()
        } else {
            Row::with_children(cards).spacing(14).wrap().into()
        };

        let content = column![header, search_bar, body]
            .spacing(20)
            .padding(24)
            .width(Length::Fill);

        scrollable(content).height(Length::Fill).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(500)).map(|_| Message::Tick)
    }

    pub fn theme(&self) -> Theme {
        Theme::CatppuccinMocha
    }
}

fn project_card(index: usize, project: &Project) -> Element<'_, Message> {
    const CARD_WIDTH: f32 = 400.0;
    let preview = image(project.preview.clone())
        .width(Length::Fixed(CARD_WIDTH))
        .height(Length::Fixed(CARD_WIDTH / 16.0 * 9.0))
        .content_fit(ContentFit::Cover);

    let status_color = match project.status {
        ProjectStatus::Idle => Color::from_rgb8(140, 140, 140),
        ProjectStatus::Installing => Color::from_rgb8(70, 130, 180),
        ProjectStatus::Running => Color::from_rgb8(59, 109, 17),
        ProjectStatus::Uninstalling => Color::from_rgb8(192, 57, 43),
    };

    let status_label = text(match project.status {
        ProjectStatus::Idle => "● Idle",
        ProjectStatus::Installing => "● Installing",
        ProjectStatus::Running => "● Running",
        ProjectStatus::Uninstalling => "● Uninstalling",
    })
    .size(12)
    .color(status_color);

    let installed_color = if project.is_installed {
        Color::from_rgb8(59, 109, 17)
    } else {
        Color::from_rgb8(140, 140, 140)
    };

    let installed_label = text(if project.is_installed {
        "● Installed"
    } else {
        "● Not installed"
    })
    .size(12)
    .color(installed_color);

    let action_btn: Element<Message> = {
        let (label, on_press) = match project.status {
            ProjectStatus::Idle => {
                ("▶  Launch", Some(Message::LaunchRequested(index)))
            }
            ProjectStatus::Installing => ("⟳  Installing", None),
            ProjectStatus::Running => {
                ("■  Stop", Some(Message::StopProject(index)))
            }
            ProjectStatus::Uninstalling => ("↺  Uninstalling", None),
        };
        let btn = button(text(label).size(13))
            .padding([7, 16])
            .width(Length::FillPortion(2));

        if let Some(on_press) = on_press {
            btn.on_press(on_press)
        } else {
            btn
        }
        .into()
    };

    let installation_btn: Element<Message> = {
        let (label, on_press, is_destructive) =
            match (project.is_installed, project.status == ProjectStatus::Idle)
            {
                (true, true) => (
                    "✖  Uninstall",
                    Some(Message::UninstallProject(index)),
                    true,
                ),
                (true, false) => ("✖  Uninstall", None, true),
                (false, true) => {
                    ("↓  Install", Some(Message::InstallProject(index)), false)
                }
                (false, false) => ("↓  Install", None, false),
            };

        let btn = button(text(label).size(13))
            .style(if is_destructive {
                button::danger
            } else {
                button::primary
            })
            .padding([7, 16])
            .width(Length::FillPortion(1));

        if let Some(on_press) = on_press {
            btn.on_press(on_press)
        } else {
            btn
        }
        .into()
    };

    let info = column![
        column![text(&project.name).size(14), text(&project.owner).size(11)]
            .spacing(3),
        row![status_label, installed_label].spacing(3),
        row![action_btn, installation_btn].spacing(8)
    ]
    .spacing(10)
    .padding(Padding::from(14).left(10));

    let card_body = column![preview, info].width(Length::Fixed(CARD_WIDTH));

    container(card_body)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                background: Some(palette.background.base.color.into()),
                ..Default::default()
            }
        })
        .into()
}
