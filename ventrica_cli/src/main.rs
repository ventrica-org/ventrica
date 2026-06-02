use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use serde_json::Value;
use ventricad::{DaemonClient, Message, Request};

#[derive(Parser)]
#[command(
    name = "ven",
    about,
    version,
    color = clap::ColorChoice::Never
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Install {
        #[arg(required = true)]
        names: Vec<String>,
    },
    Remove {
        #[arg(required = true)]
        names: Vec<String>,
    },
    Upgrade {
        #[arg(required = true)]
        names: Vec<String>,
    },
    Rollback {
        generation: Option<u32>,
    },
    List {
        #[arg(long, conflicts_with = "repos")]
        gens: bool,
        #[arg(long, conflicts_with = "gens")]
        repos: bool,
    },
    Gc,
    #[command(subcommand)]
    Repo(RepoCmd),
    Search {
        query: String,
    },
    BuildRepo {
        repo_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum RepoCmd {
    Add { url: String },
    Update,
}

fn main() {
    let cli = Cli::parse();

    match run(cli.cmd) {
        Ok(true) => {}
        Ok(false) => process::exit(1),
        Err(e) => {
            eprintln!("{e}");
            process::exit(2);
        }
    }
}

fn run(cmd: Cmd) -> std::io::Result<bool> {
    let req = cmd_to_request(cmd);
    let mut client = DaemonClient::connect()?;

    let mut had_error = false;

    client.send(&req, |msg| match msg {
        Message::Success(_) => {}
        Message::Error(s) => {
            eprintln!("error: {s}");
            had_error = true;
        }
        Message::Log(l) => {
            println!("{l}");
        }
        Message::Data(v) => print_data(&req, &v),
        Message::Done => {}
    })?;

    Ok(!had_error)
}

fn cmd_to_request(cmd: Cmd) -> Request {
    match cmd {
        Cmd::Install { names } => Request::Install { names },
        Cmd::Remove { names } => Request::Remove { names },
        Cmd::Upgrade { names } => Request::Upgrade { names },
        Cmd::Rollback { generation } => Request::Rollback { generation },
        Cmd::List {
            gens: false,
            repos: true,
        } => Request::ListRepos,
        Cmd::List {
            gens: false,
            repos: false,
        } => Request::ListPackages,
        Cmd::List {
            gens: true,
            repos: false,
        } => Request::ListGenerations,
        Cmd::Gc => Request::Gc,
        Cmd::Repo(RepoCmd::Add { url }) => Request::AddRepo { url },
        Cmd::Repo(RepoCmd::Update) => Request::UpdateRepos,
        Cmd::Search { query } => Request::Search { query },
        Cmd::BuildRepo { repo_dir } => Request::BuildRepo {
            repo_dir: repo_dir.display().to_string(),
        },
        _ => unreachable!(),
    }
}

fn print_data(req: &Request, v: &Value) {
    match req {
        _ => println!("{}", serde_json::to_string_pretty(v).unwrap_or_default()),
    }
}
