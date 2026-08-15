use std::path::PathBuf;

use clap::{Parser, Subcommand};

const VERSION_STRING: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\nGit: ",
    env!("CARGO_PKG_REPOSITORY"),
);
const HELP_TEMPLATE: &str = include_str!("help.txt");

#[derive(Parser)]
#[command(
    about = "A rather delightful package manager.",
    author,
    version = VERSION_STRING,
    color = clap::ColorChoice::Never,
    help_template = HELP_TEMPLATE,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Install packages by name
    Install {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Remove packages by name
    Remove {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Upgrade packages by name
    Upgrade {
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// Build a package from a recipe
    Build {
        path: PathBuf,
        #[arg(short = 'o', long = "output", required = true)]
        output: PathBuf,
    },
    /// Configure the package manager
    Config {
        #[arg(long, conflicts_with = "del_repo")]
        add_repo: Option<String>,
        #[arg(long, conflicts_with = "add_repo")]
        del_repo: Option<String>,
    },
}

fn main() {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("This program must be run as root.");
        std::process::exit(1);
    }

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Install { names } => {}
        Cmd::Remove { names } => {}
        Cmd::Upgrade { names } => {}
        Cmd::Build { path, output } => {}
        Cmd::Config { add_repo, del_repo } => {
            if let Some(add_repo) = add_repo {
                return;
            }
            if let Some(del_repo) = del_repo {
                return;
            }
        }
    }
}
