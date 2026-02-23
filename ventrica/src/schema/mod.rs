pub mod package;
pub mod repo;

pub use package::{Build, BuildSystem, Deps, FetchSource, GitSource, Meta, Package, Source};

pub trait FromYaml: Sized {
    fn from_file(path: &std::path::Path) -> crate::error::Result<Self>;
    fn from_str_content(text: &str) -> crate::error::Result<Self>;
}
