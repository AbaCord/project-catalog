use std::path::PathBuf;

use dirs::cache_dir;

pub trait ToKebabCase {
    fn to_kebab_case(&self) -> String;
}

impl ToKebabCase for str {
    fn to_kebab_case(&self) -> String {
        self.to_lowercase().replace(" ", "-")
    }
}

pub fn get_cache_dir() -> PathBuf {
    cache_dir()
        .expect("No cache directory found. Use a better OS lamo")
        .join("project-catalog")
}
