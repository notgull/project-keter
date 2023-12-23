// MIT/Apache2 License

//! Implementation for free-Unix systems.

use event_listener::{Event, EventListener};
use futures_lite::prelude::*;

use std::future::Future;
use std::io;
use std::sync::atomic::{Ordering, AtomicBool};

/// A signal to indicate that we are stopping.
struct Signal {
    /// Whether to stop running.
    stop_running: AtomicBool,

    /// The signal.
    stop_ops: Event, 
}

impl Signal {
    /// Get the global instance of `Signal`
    #[inline]
    fn get() -> &'static Signal {
        static SIGNAL: Signal = Signal {
            stop_running: AtomicBool::new(false),
            stop_ops: Event::new()
        };

        &SIGNAL
    }

    /// Wait for the signal to be sent.
    async fn wait(&self) {
        let listener = EventListener::new();
        futures_lite::pin!(listener);

        loop {
            // Do we need to stop running?
            if self.stop_running.swap(false, Ordering::Relaxed) {
                return;
            }

            // Establish the listener.
            listener.as_mut().listen(&self.stop_ops);

            // Check again.
            if self.stop_running.swap(false, Ordering::SeqCst) {
                return;
            }

            // Wait on the listener.
            listener.as_mut().await;
        }
    } 

    /// Send the signal.
    #[cold]
    fn stop(&self) {
        self.stop_running.store(true, Ordering::Release);
        self.stop_ops.notify_additional(std::usize::MAX);
    }
}

/// Run the reactor.
pub(crate) fn block_on<T>(settings: Settings, f: impl Future<Output = T>) -> io::Result<Option<T>> {
    if !settings.any_thread {
        crate::check_main_thread()?;
    }

    // Use async_io to block on this future.
    let result = async_io::block_on(async move {
        // Poll the future given by the user.
        let user_future = async move {
            Some(f.await)
        };

        // Simultaneously, wait for the exit signal to be fired.
        let wait_for_end = async {
            Signal::get().wait().await;
            None
        };

        // Run these futures in parallel.
        user_future.or(wait_for_end).await
    });

    Ok(result)
}

/// Send the signal to exit.
pub(crate) fn exit() -> io::Result<()> {
    Signal::get().stop();
    Ok(())
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
