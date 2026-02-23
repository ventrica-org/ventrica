use log::{LevelFilter, Log, Metadata, Record};

thread_local! {
    static SINK: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0, 0]) };
}

pub struct SinkGuard;

impl Drop for SinkGuard {
    fn drop(&mut self) {
        SINK.with(|s| s.set([0, 0]));
    }
}

#[allow(unsafe_code)]
pub fn set_thread_sink(f: &mut dyn FnMut(&str)) -> SinkGuard {
    let raw: *mut dyn FnMut(&str) = f;
    // SAFETY: transmuting a fat pointer to [usize; 2] is sound both are
    // two pointer-sized words. The pointer is only dereferenced while the
    // SinkGuard is alive, which is within the lifetime of `f`.
    let parts: [usize; 2] = unsafe { std::mem::transmute(raw) };
    SINK.with(|s| s.set(parts));
    SinkGuard
}

fn has_sink() -> bool {
    SINK.with(|s| s.get()[0] != 0)
}

#[allow(unsafe_code)]
fn forward_to_sink(msg: &str) {
    let parts = SINK.with(|s| s.get());
    if parts[0] == 0 {
        return;
    }
    // SAFETY: the pointer was stored by `set_thread_sink` and is only
    // accessed here, on the same thread, while the SinkGuard is alive
    let f: *mut dyn FnMut(&str) = unsafe { std::mem::transmute(parts) };
    unsafe { (*f)(msg) };
}

struct VentricadLogger;

impl Log for VentricadLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let msg = format!("{}", record.args());

        forward_to_sink(&msg);

        // When no client is connected
        // fall back to stderr so the
        // daemon can know whats happning
        if !has_sink() {
            eprintln!("ventricad: {msg}");
        }
    }

    fn flush(&self) {}
}

static LOGGER: VentricadLogger = VentricadLogger;

pub fn init() {
    log::set_logger(&LOGGER).expect("logger already set");
    log::set_max_level(LevelFilter::Info);
}
