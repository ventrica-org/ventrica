use ventricad::ops;
use ventricad::protocol::{Message, Request};

fn emit_data<T: serde::Serialize>(val: T, send: &mut dyn FnMut(&Message)) -> ventrica::Result<()> {
    send(&Message::Data(
        serde_json::to_value(&val).unwrap_or_default(),
    ));
    Ok(())
}

pub fn dispatch(req: &Request, build_user: Option<(u32, u32)>, send: &mut dyn FnMut(&Message)) {
    let send = std::cell::RefCell::new(send);
    let mut sink = |s: &str| (send.borrow_mut())(&Message::Log(s.to_owned()));
    let _log_guard = crate::logging::set_thread_sink(&mut sink);
    let mut log = |s: &str| log::info!("{s}");
    let mut fwd = |m: &Message| (send.borrow_mut())(m);

    let result: ventrica::Result<()> = match req {
        Request::Install { names } => ops::install::install(names, &mut log),

        Request::Remove { names } => ops::remove::remove(names, &mut log),

        Request::Upgrade { names } => ops::upgrade::upgrade(names, &mut log),

        Request::Rollback { generation } => ops::rollback::rollback(*generation, &mut log),

        Request::ListPackages => ops::list::list_packages().and_then(|v| emit_data(v, &mut fwd)),
        Request::ListGenerations => {
            ops::list::list_generations().and_then(|v| emit_data(v, &mut fwd))
        }
        Request::ListRepos => ops::list::list_repos().and_then(|v| emit_data(v, &mut fwd)),

        Request::Gc => ops::gc::gc(&mut log),

        Request::AddRepo { url } => ops::add_repo::add_repo(url, &mut log).map(|_| ()),
        Request::RemoveRepo { url } => ops::repos::remove_repo(url, &mut log),
        Request::UpdateRepos => ops::update::update_repos(&mut log),

        Request::Search { query } => {
            let r = ops::search::search(query);
            match r {
                Ok(ref v) if v.is_empty() => {
                    log::info!("no packages found matching '{query}'");
                    Ok(())
                }
                Ok(v) => emit_data(v, &mut fwd),
                Err(e) => Err(e),
            }
        }

        Request::BuildRepo { repo_dir } => {
            ops::build_repo::build_repo(std::path::Path::new(repo_dir), &mut log, build_user)
        }

        Request::ListRepoPackages { url } => {
            ops::repos::list_repo_packages(url).and_then(|v| emit_data(v, &mut fwd))
        }
    };

    drop(_log_guard);

    match result {
        Ok(()) => (send.borrow_mut())(&Message::Success("ok".into())),
        Err(e) => (send.borrow_mut())(&Message::Error(e.to_string())),
    }
}
