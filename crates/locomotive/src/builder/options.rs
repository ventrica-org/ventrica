use std::collections::HashMap;
use std::path::PathBuf;

use crate::store::Package;
use crate::utils::dir::make_temp_dir_name;

#[derive(Debug, Clone)]
pub struct BuildUser {
    current_uid: u32,
    current_gid: u32,
    new_uid: u32,
    new_gid: u32,
}

impl BuildUser {
    pub fn new(uid: u32, gid: u32) -> Self {
        Self {
            current_uid: unsafe { libc::getuid() },
            current_gid: unsafe { libc::getgid() },
            new_uid: uid,
            new_gid: gid,
        }
    }

    pub fn set_process_new_privileges(&self) {
        unsafe {
            libc::setgid(self.new_gid);
            libc::setuid(self.new_uid);
        }
    }

    pub fn set_process_old_privileges(&self) {
        unsafe {
            libc::setgid(self.current_gid);
            libc::setuid(self.current_uid);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageBuilderOptions {
    /// The directory where the package will be built.
    build_dir: PathBuf,
    /// The directory where the source files for the build will be stored.
    build_src_dir: PathBuf,
    /// The directory where the built package will be stored.
    build_dest_dir: PathBuf,
    /// The package to be built.
    package: Package,
    /// The prefix to be used for the build.
    prefix: Option<String>,
    /// The user to be used for the build.
    user: Option<BuildUser>,
    /// The environment variables to be used for the build.
    env: HashMap<String, String>,
    /// Whether to keep the build directory after the build.
    keep_build_dir: bool,
}

impl PackageBuilderOptions {
    pub fn new(package: Package) -> Self {
        let build_dir = make_temp_dir();
        let build_dest_dir = build_dir.join("dest");
        let mut env = HashMap::new();
        env.insert(
            "DESTDIR".into(),
            build_dest_dir.to_string_lossy().into_owned(),
        );

        Self {
            build_src_dir: build_dir.join("src"),
            build_dest_dir,
            build_dir,
            package,
            prefix: None,
            user: None,
            env,
            keep_build_dir: false,
        }
    }

    pub fn set_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        self.prefix = Some(prefix);
        self
    }

    pub fn set_user(mut self, user: BuildUser) -> Self {
        self.user = Some(user);
        self
    }

    pub fn set_env(mut self, env: HashMap<String, String>) -> Self {
        self.env.extend(env);
        self
    }

    pub fn set_keep_build_dir(mut self, keep_build_dir: bool) -> Self {
        self.keep_build_dir = keep_build_dir;
        self
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    pub fn build_src_dir(&self) -> &PathBuf {
        &self.build_src_dir
    }

    pub fn build_dest_dir(&self) -> &PathBuf {
        &self.build_dest_dir
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn prefix(&self) -> &Option<String> {
        &self.prefix
    }

    pub fn user(&self) -> &Option<BuildUser> {
        &self.user
    }

    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    pub fn keep_build_dir(&self) -> bool {
        self.keep_build_dir
    }
}

fn make_temp_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir().join(make_temp_dir_name("ventrica-build"));

    temp_dir
}
