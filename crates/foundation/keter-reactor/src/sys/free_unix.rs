// MIT/Apache2 License

//! Implementation for free-Unix systems.

#[path = "signal.rs"]
mod signal;

use futures_lite::prelude::*;
use signal::Signal;

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Run the reactor.
pub(crate) fn block_on<T>(settings: Settings, f: impl Future<Output = T>) -> io::Result<Option<T>> {
    if !settings.any_thread {
        crate::check_main_thread()?;
    }

    // Use async_io to block on this future.
    let result = async_io::block_on(async move {
        // Poll the future given by the user.
        let user_future = async move { Some(f.await) };

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

/// The timer implementation.
pub(crate) struct Timer(async_io::Timer);

impl Unpin for Timer {}

impl Timer {
    /// Create a timer that will never fire.
    #[inline]
    pub(crate) fn never() -> Self {
        Self(async_io::Timer::never())
    }

    /// Create a timer that fires at a specific deadline.
    #[inline]
    pub(crate) fn at(at: crate::Instant) -> Self {
        Self(async_io::Timer::at(at))
    }

    /// Create a timer that fires repeatedly.
    #[inline]
    pub(crate) fn interval(at: crate::Instant, interval: crate::Duration) -> Self {
        Self(async_io::Timer::interval_at(at, interval))
    }

    /// Set this timer to never fire.
    #[inline]
    pub(crate) fn set_never(&mut self) {
        self.0 = async_io::Timer::never();
    }

    /// Set this timer to an `at()` timer.
    #[inline]
    pub(crate) fn set_at(&mut self, at: crate::Instant) {
        self.0.set_at(at);
    }

    /// Set this timer to an `interval()` timer.
    #[inline]
    pub(crate) fn set_interval(&mut self, at: crate::Instant, interval: crate::Duration) {
        self.0.set_interval_at(at, interval);
    }

    /// Wait for the next time this timer fires.
    #[inline]
    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        futures_lite::ready!(Pin::new(&mut self.0).poll_next(cx));
        Poll::Ready(())
    }
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
