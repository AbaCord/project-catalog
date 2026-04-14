use iced::{
    Border, Color, ContentFit, Element, Length, Padding, Theme,
    widget::{
        Column, button, column, container, image, row, scrollable, text,
        text_input,
    },
};

fn main() -> iced::Result {
    iced::application(new, update, view).theme(theme).run()
}

fn new() -> State {
    Default::default()
}

fn theme(_state: &State) -> Theme {
    Theme::CatppuccinMocha
}

struct Project {
    name: String,
    owner: String,
    repo: String,
    preview: image::Handle,
    running: bool,
}

impl Project {
    fn new(
        name: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
        preview: image::Handle,
    ) -> Self {
        Self {
            name: name.into(),
            owner: owner.into(),
            repo: repo.into(),
            preview,
            running: false,
        }
    }
}

struct State {
    projects: Vec<Project>,
    search: String,
}

impl Default for State {
    fn default() -> Self {
        let make = |name: &str, owner: &str, repo: &str, path: &str| {
            Project::new(name, owner, repo, image::Handle::from_path(path))
        };
        Self {
            search: String::new(),
            projects: vec![make(
                "Pokemon battle simulator",
                "Nikolai Ciric",
                "https://github.com/nikcir/Java-Pokemon-battle-simulator",
                "assets/previews/pokemon-battle-simulator.png",
            )],
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    SearchChanged(String),
    LaunchProject(usize),
    StopProject(usize),
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::SearchChanged(s) => state.search = s,
        Message::LaunchProject(i) => {
            if let Some(p) = state.projects.get_mut(i) {
                p.running = true
            }
        }
        Message::StopProject(i) => {
            if let Some(p) = state.projects.get_mut(i) {
                p.running = false
            }
        }
    }
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
        let rows: Vec<_> = IntoChunks::into_chunks(cards, 3)
            .map(|chunk| {
                chunk
                    .into_iter()
                    .fold(row![].spacing(14), |r, card| r.push(card))
                    .width(Length::Fill)
                    .into()
            })
            .collect();
        Column::with_children(rows).spacing(14).into()
    };

    let content = column![header, search_bar, body]
        .spacing(20)
        .padding(24)
        .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

fn project_card(index: usize, project: &Project) -> Element<'_, Message> {
    let preview = image(project.preview.clone())
        .width(Length::Fixed(280.0))
        .height(Length::Fixed(157.5))
        .content_fit(ContentFit::Cover);

    let status_color = if project.running {
        Color::from_rgb8(59, 109, 17)
    } else {
        Color::from_rgb8(140, 140, 140)
    };

    let status_label = text(if project.running {
        "● Running"
    } else {
        "● Idle"
    })
    .size(12)
    .color(status_color);

    let action_btn: Element<Message> = if project.running {
        button(text("■  Stop").size(13))
            .on_press(Message::StopProject(index))
            .padding([7, 16])
            .width(Length::Fill)
            .into()
    } else {
        button(text("▶  Launch").size(13))
            .on_press(Message::LaunchProject(index))
            .padding([7, 16])
            .width(Length::Fill)
            .into()
    };

    let info = column![
        column![text(&project.name).size(14), text(&project.owner).size(11)]
            .spacing(3),
        status_label,
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

trait IntoChunks<T> {
    fn into_chunks(self, n: usize) -> std::vec::IntoIter<Vec<T>>;
}

impl<T> IntoChunks<T> for Vec<T> {
    fn into_chunks(self, n: usize) -> std::vec::IntoIter<Vec<T>> {
        let mut result = Vec::new();
        let mut chunk = Vec::with_capacity(n);
        for item in self {
            chunk.push(item);
            if chunk.len() == n {
                result
                    .push(std::mem::replace(&mut chunk, Vec::with_capacity(n)));
            }
        }
        if !chunk.is_empty() {
            result.push(chunk);
        }

        result.into_iter()
    }
}
