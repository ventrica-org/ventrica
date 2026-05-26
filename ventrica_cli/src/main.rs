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
            eprintln!("vnt: {e}");
            process::exit(2);
        }
    }
}

fn run(cmd: Cmd) -> std::io::Result<bool> {
    let req = cmd_to_request(cmd);
    let mut client = DaemonClient::connect().map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("cannot connect to ventricad ({e}) - is the daemon running?"),
        )
    })?;

    let mut had_error = false;

    client.send(&req, |msg| match msg {
        Message::Log(s) => println!("{s}"),
        Message::Warn(s) => eprintln!("{s}"),
        Message::Success(_) => {}
        Message::Error(s) => {
            eprintln!("error: {s}");
            had_error = true;
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
            gens: true,
            repos: true,
        } => Request::ListGenerations,
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
    }
}

fn print_data(req: &Request, v: &Value) {
    match req {
        Request::ListPackages => print_packages(v),
        Request::ListRepos => print_repos(v),
        Request::Search { .. } => print_search(v),
        _ => println!("{}", serde_json::to_string_pretty(v).unwrap_or_default()),
    }
}

fn print_packages(v: &Value) {
    let Some(arr) = v.as_array() else { return };

    if arr.is_empty() {
        println!("no packages installed");
        return;
    }

    for p in arr {
        let name = p["name"].as_str().unwrap_or("");
        let version = p["version"].as_str().unwrap_or("");
        println!("{name} {version}");
    }
}

fn print_search(v: &Value) {
    let Some(arr) = v.as_array() else { return };
    if arr.is_empty() {
        println!("no results");
        return;
    }
    for r in arr {
        let repo = r["repo"].as_str().unwrap_or("?");
        let name = r["name"].as_str().unwrap_or("");
        let ver = r["version"].as_str().unwrap_or("");
        let inst = if r["installed"].as_bool().unwrap_or(false) {
            " [installed]"
        } else {
            ""
        };
        println!("{repo}/{name} {ver}{inst}");
        if let Some(desc) = r["description"].as_str() {
            println!("    {desc}");
        }
        if let Some(deps) = r["run_deps"].as_array() {
            let ds: Vec<_> = deps.iter().filter_map(|d| d.as_str()).collect();
            if !ds.is_empty() {
                println!("    deps: {}", ds.join(", "));
            }
        }
    }
}

fn print_repos(v: &Value) {
    let Some(arr) = v.as_array() else { return };
    if arr.is_empty() {
        println!("no repos added");
        return;
    }
    for r in arr {
        let url = r["url"].as_str().unwrap_or("");
        let name = r["name"].as_str().unwrap_or("");
        println!("{name}: {url}");
    }
}
