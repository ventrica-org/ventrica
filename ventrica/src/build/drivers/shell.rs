use super::BuildDriver;
use crate::build::context::{BuildContext, run_cmd_in};
use crate::build::environment::{Environment, expand};
use crate::error::{Error, Result};

pub struct ShellDriver;

impl BuildDriver for ShellDriver {
    fn run(&self, ctx: &BuildContext<'_>) -> Result<()> {
        let script_tmpl = if ctx.spec.build.trim().is_empty() {
            Err(Error::BuildFailed {
                name: ctx.name.into(),
                reason: "shell build system requires a non-empty build script in the recipe".into(),
            })
        } else {
            Ok(ctx.spec.build.as_str())
        }?;

        let vars = Environment {
            destdir: ctx.dest.to_str().unwrap_or_default(),
            prefix: ctx.prefix,
            srcdir: ctx.src.to_str().unwrap_or_default(),
            builddir: ctx.build.to_str().unwrap_or_default(),
            name: ctx.name,
        };
        let script = expand(script_tmpl, &vars);

        run_cmd_in(ctx, ctx.src, "sh", &["-e".into(), "-c".into(), script])
    }
}
