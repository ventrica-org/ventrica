use std::path::Path;

pub fn build_repo(
    repo_dir: &Path,
    _log: &mut dyn FnMut(&str),
    build_user: Option<(u32, u32)>,
) -> ventrica::Result<()> {
    ventrica::repo::build_repo(repo_dir, build_user)
}
