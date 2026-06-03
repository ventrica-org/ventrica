mod handler;
mod logging;
mod server;

use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    logging::init();

    let socket_path = Arc::new(PathBuf::from(
        std::env::var(ventricad::SOCKET_ENV)
            .unwrap_or_else(|_| ventricad::DEFAULT_SOCKET.to_owned()),
    ));

    let socket_for_handler = Arc::clone(&socket_path);
    unsafe {
        let socket_ptr = Arc::into_raw(socket_for_handler) as usize;
        for &sig in &[libc::SIGINT, libc::SIGTERM] {
            libc::signal(sig, handle_signal as *const () as libc::sighandler_t);
        }
        SOCKET_PATH_PTR.store(socket_ptr, Ordering::SeqCst);
    }

    if let Err(e) = server::run(&socket_path) {
        eprintln!("ventricad: fatal: {e}");
        process::exit(1);
    }
}

static SOCKET_PATH_PTR: AtomicUsize = AtomicUsize::new(0);

extern "C" fn handle_signal(_: libc::c_int) {
    let ptr = SOCKET_PATH_PTR.load(Ordering::SeqCst);
    if ptr != 0 {
        let path = unsafe { &*(ptr as *const PathBuf) };
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }
    process::exit(0);
}
