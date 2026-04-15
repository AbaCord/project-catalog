use std::{
    collections::HashSet,
    process::{Command, Stdio},
};

use iced::widget::image;
use process_wrap::std::{ChildWrapper, CommandWrap};

use crate::utils::get_cache_dir;

#[derive(Clone, PartialEq)]
pub enum ProjectStatus {
    Idle,
    Installing,
    Running,
    Uninstalling,
}

pub struct Project {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub repo: String,
    pub preview: image::Handle,
    pub status: ProjectStatus,
    pub is_installed: bool,
    pub child: Option<Box<dyn ChildWrapper>>,
}

impl Project {
    pub fn new(
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

pub fn get_cached_projects() -> HashSet<String> {
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

pub async fn install_project(id: String, repo: String) {
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

pub async fn uninstall_project(id: String) {
    let target_dir = get_cache_dir().join(&id);
    assert!(target_dir.exists());

    let _ = tokio::fs::remove_dir_all(target_dir).await;
}

pub fn launch_project(project: &mut Project) {
    let pom_path = get_cache_dir().join(&project.id).join("pom.xml");
    let program = if cfg!(windows) { "mvn.cmd" } else { "mvn" };
    let mut command = CommandWrap::with_new(program, |command| {
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
        use process_wrap::std::JobObject;
        command.wrap(JobObject);
    }

    let child = CommandWrap::from(command).spawn().unwrap();
    project.child = Some(child);

    project.status = ProjectStatus::Running;
}
