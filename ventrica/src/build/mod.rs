pub mod context;
pub mod download;
pub mod drivers;
pub mod environment;
pub mod file_types;
pub mod sandbox;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::env::{VENTRICA_LIVE_PATH, VENTRICA_STORE_PATH};
use crate::error::{Error, Result};
use crate::models::{Package, Scripts};
use crate::store::{atomic_move, seal, simple_store_path, unseal};

use context::{BuildContext, build_env, chown_scratch};

pub struct Builder<'a> {
    pkg: &'a Package,
    recipe_dir: Option<PathBuf>,
    /// UID/GID of the client that initiated the request. Build subprocesses
    /// are dropped to this identity when the daemon is running as root.
    build_user: Option<(u32, u32)>,
}

impl<'a> Builder<'a> {
    #[must_use]
    pub fn new(pkg: &'a Package) -> Self {
        Self {
            pkg,
            recipe_dir: None,
            build_user: None,
        }
    }

    #[must_use]
    pub fn with_build_user_opt(mut self, user: Option<(u32, u32)>) -> Self {
        self.build_user = user;
        self
    }

    #[must_use]
    pub fn with_recipe_dir(mut self, dir: PathBuf) -> Self {
        self.recipe_dir = Some(dir);
        self
    }

    /// `/ventrica/store/<name>-<version>`.
    #[must_use]
    pub fn store_path(&self) -> PathBuf {
        simple_store_path(&self.pkg.name, &self.pkg.version)
    }

    pub fn build_to_store(&self) -> Result<PathBuf> {
        let sp = self.store_path();

        log::info!("{} {}", self.pkg.name, self.pkg.version);
        log::info!("store: {}", sp.display());

        if self.pkg.is_disabled.unwrap_or(false) {
            return Err(Error::BuildFailed {
                name: self.pkg.name.clone(),
                reason: "recipe is marked broken".into(),
            });
        }

        if sp.exists() {
            log::info!("already in store - skipping build");
            return Ok(sp);
        }

        let result = self.run_pipeline(&sp);
        if let Err(ref e) = result {
            log::warn!("build failed: {e}");
            log::info!("cleaning up partial store entry");
            if sp.exists() {
                let _ = unseal(&sp);
                let _ = fs::remove_dir_all(&sp);
            }
        }
        result
    }

    fn run_pipeline(&self, store_dest: &Path) -> Result<PathBuf> {
        let pkg = self.pkg;
        let name = &pkg.name;

        // /tmp/ventrica-<name>-<version>-<pid>
        let scratch_raw = PathBuf::from("/tmp").join(format!(
            "ventrica-{}-{}-{}",
            name,
            pkg.version,
            std::process::id()
        ));

        fs::create_dir_all(&scratch_raw)?;
        let scratch = scratch_raw.canonicalize().unwrap_or(scratch_raw);
        let src_dir = scratch.join("src");
        let build_dir = scratch.join("build");
        let dest_dir = scratch.join("dest");
        let archive_dir = scratch.join("archives");

        for dir in [&src_dir, &build_dir, &dest_dir, &archive_dir] {
            fs::create_dir_all(dir)?;
        }

        struct ScratchGuard(PathBuf);
        impl Drop for ScratchGuard {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _guard = ScratchGuard(scratch.clone());

        fs::create_dir_all(VENTRICA_STORE_PATH)?;

        log::info!("starting build pipeline in scratch {}", scratch.display());

        let source_root = self.fetch_source(&src_dir, &archive_dir)?;

        #[cfg(unix)]
        chown_scratch(&scratch, self.build_user);

        log::info!("source ready at {}", source_root.display());

        if let Some(scripts) = &pkg.scripts {
            apply_patches(&scripts, &source_root, name, self.recipe_dir.as_deref())?;
        }

        log::info!("patches applied");

        if let Some(build_spec) = &pkg.scripts {
            if build_spec.system.as_deref().unwrap_or("shell") != "none" {
                let recipe_env = std::collections::HashMap::new();
                let env = build_env(&recipe_env);

                let sandboxed = sandbox::sandbox_exec_available();
                let ctx = BuildContext {
                    src: &source_root,
                    build: &build_dir,
                    dest: &dest_dir,
                    scratch: &scratch,
                    env: &env,
                    spec: build_spec,
                    name: &pkg.name,
                    prefix: VENTRICA_LIVE_PATH,
                    sandboxed,
                    build_user: self.build_user,
                };
                log::debug!("build context: {:#?}", ctx);
                drivers::dispatch(ctx.spec.system.as_deref(), &ctx)?;
            }
        }

        // Build tools install into DESTDIR under the live prefix, e.g.
        // dest_dir/ventrica/live/bin/...  Move that subtree into the store entry.
        let live_rel = Path::new(VENTRICA_LIVE_PATH)
            .strip_prefix("/")
            .unwrap_or(Path::new(VENTRICA_LIVE_PATH));
        let staged_prefix = dest_dir.join(live_rel);

        let promote_from = if staged_prefix.exists() {
            staged_prefix
        } else {
            dest_dir.clone()
        };

        atomic_move(&promote_from, store_dest)?;

        seal(store_dest)?;

        log::info!("committed to {}", store_dest.display());
        Ok(store_dest.to_owned())
    }

    fn fetch_source(&self, src_dir: &Path, archive_dir: &Path) -> Result<PathBuf> {
        use download::{extract, fetch_and_verify};

        match &self.pkg.source {
            None => {
                fs::create_dir_all(src_dir)?;
                Ok(src_dir.to_owned())
            }
            Some(source) => {
                let url = source.url.first().ok_or_else(|| Error::BuildFailed {
                    name: self.pkg.name.clone(),
                    reason: "source URLs are empty".into(),
                })?;
                let mirrors: Vec<String> = source.url.iter().skip(1).cloned().collect();
                let expected = source
                    .sha256
                    .strip_prefix("sha256:")
                    .unwrap_or(&source.sha256);
                let archive = fetch_and_verify(url, expected, &mirrors, archive_dir)?;
                let candidate = extract(&archive, src_dir, None)?;
                if candidate.is_dir() {
                    Ok(candidate)
                } else {
                    let url_basename = url.rsplit('/').next().unwrap_or("source");
                    let url_basename = url_basename.split('?').next().unwrap_or(url_basename);
                    let dest = src_dir.join(url_basename);
                    if candidate != dest {
                        fs::rename(&candidate, &dest)?;
                    }
                    Ok(src_dir.to_owned())
                }
            }
        }
    }
}

fn apply_patches(
    scripts: &Scripts,
    work_dir: &Path,
    name: &str,
    recipe_dir: Option<&Path>,
) -> Result<()> {
    let patches = scripts.patches.as_deref().unwrap_or(&[]);
    for patch in patches {
        let patch_path = if let Some(dir) = recipe_dir {
            let p = std::path::Path::new(patch);
            if p.is_absolute() {
                p.to_owned()
            } else {
                dir.join(p)
            }
        } else {
            PathBuf::from(patch)
        };
        log::info!("applying patch {}", patch_path.display());
        let status = Command::new("patch")
            .args(["-p1", "-i", &patch_path.display().to_string()])
            .current_dir(work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !status.success() {
            return Err(Error::BuildFailed {
                name: name.into(),
                reason: format!("patch '{}' failed with {status}", patch_path.display()),
            });
        }
    }
    Ok(())
}
