use std::path::Path;

pub fn build_repo(repo_dir: &Path, build_user: Option<(u32, u32)>) -> ventrica::Result<()> {
    ventrica::repo::build_repo(repo_dir, build_user)
}
