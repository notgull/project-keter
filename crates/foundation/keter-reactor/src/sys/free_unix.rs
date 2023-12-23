// MIT/Apache2 License

//! Implementation for free-Unix systems.

use std::future::Future;
use std::io;

/// Run the reactor.
pub(crate) fn block_on<T>(settings: Settings, f: impl Future<Output = T>) -> io::Result<T> {
    if !settings.any_thread {
        crate::check_main_thread()?;
    }

    // Use async_io to block on this future.
    Ok(async_io::block_on(f))
}

/// Settings for running the reactor.
#[derive(Debug)]
pub(crate) struct Settings {
    /// Run on any thread.
    pub(crate) any_thread: bool,
}

impl Settings {
    #[inline]
    pub(crate) fn empty() -> Self {
        Self { any_thread: false }
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn is_main_thread() -> bool {
    rustix::thread::gettid() == rustix::process::getpid()
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]
pub(crate) fn is_main_thread() -> bool {
    todo!()
}

#[cfg(target_os = "netbsd")]
pub(crate) fn is_main_thread() -> bool {
    std::thread::current().name() == Some("main")
}
