use std::{
    collections::HashSet,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use dirs::cache_dir;
use iced::{
    Border, Color, ContentFit, Element, Length, Padding, Subscription, Task,
    Theme, time,
    widget::{
        Row, button, column, container, image, row, scrollable, text,
        text_input,
    },
};
use process_wrap::std::{ChildWrapper, CommandWrap};

fn main() -> iced::Result {
    iced::application(new, update, view)
        .subscription(subscription)
        .theme(theme)
        .run()
}

fn new() -> State {
    Default::default()
}

fn theme(_state: &State) -> Theme {
    Theme::CatppuccinMocha
}

#[derive(Clone)]
enum ProjectStatus {
    Idle,
    Installing,
    Running,
}

struct Project {
    id: String,
    name: String,
    owner: String,
    repo: String,
    preview: image::Handle,
    status: ProjectStatus,
    is_installed: bool,
    child: Option<Box<dyn ChildWrapper>>,
}

impl Project {
    fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
        preview: image::Handle,
        is_cached: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            owner: owner.into(),
            repo: repo.into(),
            preview,
            status: ProjectStatus::Idle,
            is_installed: is_cached,
            child: None,
        }
    }
}

struct State {
    projects: Vec<Project>,
    search: String,
}

impl Default for State {
    fn default() -> Self {
        let cached_projects = get_cached_projects();

        let make =
            |id: &str, name: &str, owner: &str, repo: &str, path: &str| {
                Project::new(
                    id,
                    name,
                    owner,
                    repo,
                    image::Handle::from_path(path),
                    cached_projects.contains(id),
                )
            };
        Self {
            search: String::new(),
            projects: vec![make(
                "pokemon-battle-simulator",
                "Pokemon battle simulator",
                "Nikolai Ciric",
                "https://github.com/nikcir/Java-Pokemon-battle-simulator",
                "assets/previews/pokemon-battle-simulator.png",
            )],
        }
    }
}

fn get_cached_projects() -> HashSet<String> {
    let dir = get_cache_dir();

    if let Ok(iter) = dir.read_dir() {
        iter.filter_map(|res| res.ok())
            .map(|res| res.path())
            .filter(|path| path.is_dir())
            .map(|path| path.file_name().unwrap().to_string_lossy().into())
            .collect()
    } else {
        HashSet::new()
    }
}

fn get_cache_dir() -> PathBuf {
    cache_dir()
        .expect("No cache directory found. Use a better OS lamo")
        .join("project-catalog")
}

#[derive(Clone, Debug)]
enum Message {
    SearchChanged(String),
    LaunchRequested(usize),
    ProjectSynced(usize),
    StopProject(usize),
    Tick,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SearchChanged(s) => state.search = s,
        Message::LaunchRequested(i) => {
            if let Some(mut project) = state.projects.get_mut(i) {
                if project.is_installed {
                    launch_project(&mut project);
                } else {
                    project.status = ProjectStatus::Installing;
                    return Task::perform(
                        install_project(
                            project.id.clone(),
                            project.repo.clone(),
                        ),
                        move |_| Message::ProjectSynced(i),
                    );
                }
            }
        }
        Message::ProjectSynced(i) => {
            if let Some(mut project) = state.projects.get_mut(i) {
                project.is_installed = true;
                launch_project(&mut project);
            }
        }
        Message::StopProject(i) => {
            if let Some(project) = state.projects.get_mut(i) {
                if let Some(child) = &mut project.child {
                    child.kill().unwrap();
                }

                project.status = ProjectStatus::Idle
            }
        }
        Message::Tick => {
            for project in state.projects.iter_mut() {
                if let Some(child) = &mut project.child {
                    if let Ok(Some(_status)) = child.try_wait() {
                        project.child = None;
                        project.status = ProjectStatus::Idle;
                    }
                }
            }
        }
    }

    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    let header = column![
        text("Project catalog").size(22),
        text("Abacord JavaFx projects").size(14)
    ]
    .spacing(4);

    let search_bar = text_input("Search projects...", &state.search)
        .on_input(Message::SearchChanged)
        .padding(10)
        .width(Length::Fill);

    let query = state.search.to_lowercase();

    let cards: Vec<_> = state
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

fn subscription(_state: &State) -> Subscription<Message> {
    time::every(Duration::from_millis(500)).map(|_| Message::Tick)
}

fn project_card(index: usize, project: &Project) -> Element<'_, Message> {
    let preview = image(project.preview.clone())
        .width(Length::Fixed(280.0))
        .height(Length::Fixed(157.5))
        .content_fit(ContentFit::Cover);

    let status_color = match project.status {
        ProjectStatus::Idle => Color::from_rgb8(140, 140, 140),
        ProjectStatus::Installing => Color::from_rgb8(70, 130, 180),
        ProjectStatus::Running => Color::from_rgb8(59, 109, 17),
    };

    let status_label = text(match project.status {
        ProjectStatus::Idle => "● Idle",
        ProjectStatus::Installing => "● Installing",
        ProjectStatus::Running => "● Running",
    })
    .size(12)
    .color(status_color);

    let cached_color = if project.is_installed {
        Color::from_rgb8(59, 109, 17)
    } else {
        Color::from_rgb8(140, 140, 140)
    };

    let cached_label = text(if project.is_installed {
        "● Installed"
    } else {
        "● Not installed"
    })
    .size(12)
    .color(cached_color);

    let action_btn: Element<Message> = {
        let (label, on_press) = match project.status {
            ProjectStatus::Idle => {
                ("▶  Launch", Some(Message::LaunchRequested(index)))
            }
            ProjectStatus::Installing => ("⟳  Installing", None),
            ProjectStatus::Running => {
                ("■  Stop", Some(Message::StopProject(index)))
            }
        };
        let btn = button(text(label).size(13))
            .padding([7, 16])
            .width(Length::Fill);

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
        row![status_label, cached_label].spacing(3),
        action_btn
    ]
    .spacing(10)
    .padding(Padding::from(14).left(10));

    let card_body = column![preview, info].width(Length::Fixed(280.0));

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

async fn install_project(id: String, repo: String) {
    let target_dir = get_cache_dir().join(&id);
    assert!(!target_dir.exists());

    let _ = tokio::task::spawn_blocking(move || {
        let _ = Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("--no-tags")
            .arg("--quiet")
            .arg(repo)
            .arg(&target_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    })
    .await;
}

fn launch_project(project: &mut Project) {
    let pom_path = get_cache_dir().join(&project.id).join("pom.xml");
    let mut command = CommandWrap::with_new("mvn", |command| {
        command
            .arg("-f")
            .arg(pom_path)
            .arg("javafx:run")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
    });
    #[cfg(unix)]
    {
        use process_wrap::std::ProcessGroup;
        command.wrap(ProcessGroup::leader());
    }
    #[cfg(windows)]
    {
        use process_wrap::windows::JobObject;
        command.wrap(JobObject);
    }

    let child = CommandWrap::from(command).spawn().unwrap();
    project.child = Some(child);

    project.status = ProjectStatus::Running;
}
