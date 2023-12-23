// MIT/Apache2 License

//! A reactor for driving GUI systems.

#![forbid(unsafe_code)]

pub mod platform;
mod sys;

use std::convert::Infallible;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::stream::Stream;
use web_time::{Duration, Instant};

pub use web_time;

/// Macro for creating the main function.
#[macro_export]
macro_rules! main {
    (
        $(#[$attr:meta])*
        fn $name:ident ($sett:ident : $sty:ty) -> $res:ty $bl:block
    ) => {
        fn main() {
            use $crate::platform::instantiation::ReactorExt as _;

            // Eat the $sty and $res items to prevent unused warnings.
            fn __eat_types(_: $sty) -> $res {
                panic!()
            }

            // Create the reactor and put it where we need it.
            let reactor = <$crate::Reactor as $crate::platform::instantiation::ReactorExt>::new();
            let $sett = reactor;

            // Run the block to get a result.
            let result = $bl;
        }
    };
}

/// The type intended to be returned from `main`.
pub type Main = Result<Finished>;

/// Result type used by this crate.
#[doc(no_inline)]
pub use io::Result;

/// The type produced by a finished application.
pub struct Finished {
    _private: (),
}

impl Finished {
    fn new() -> Self {
        Self { _private: () }
    }
}

/// Settings for the reactor to drive the system.
pub struct Reactor {
    settings: sys::Settings,
}

impl Reactor {
    /// Block on a future for as long as possible.
    #[inline]
    pub fn block_on(self, future: impl Future<Output = Infallible>) -> Result<Finished> {
        if let Some(infall) = sys::block_on(self.settings, future)? {
            match infall {}
        }
        Ok(Finished::new())
    }
}

/// Indicate to the reactor that we want to exit as soon as possible.
#[cold]
pub async fn exit() -> ! {
    sys::exit().expect("failed to exit the program");
    std::future::pending().await
}

/// A timer that waits for a specific amount of time in the run loop.
pub struct Timer(sys::Timer);

impl Unpin for Timer {}

impl Timer {
    /// Create a new timer that never fires.
    #[inline]
    pub fn never() -> Self {
        Self(sys::Timer::never())
    }

    /// Create a new timer that fires after a specific interval.
    #[inline]
    pub fn after(duration: Duration) -> Self {
        Instant::now()
            .checked_add(duration)
            .map_or_else(Timer::never, Timer::at)
    }

    /// Create a new timer that fires at a specific deadline.
    #[inline]
    pub fn at(deadline: Instant) -> Self {
        Self(sys::Timer::at(deadline))
    }

    /// Create a new timer that fires on an interval, starting now.
    #[inline]
    pub fn interval(period: Duration) -> Self {
        Instant::now()
            .checked_add(period)
            .map_or_else(Timer::never, |start| Timer::interval_at(start, period))
    }

    /// Create a new timer that fires on an interval starting at a deadline.
    #[inline]
    pub fn interval_at(start: Instant, period: Duration) -> Self {
        Self(sys::Timer::interval(start, period))
    }

    /// Set this timer to never fire.
    #[inline]
    pub fn set_never(&mut self) {
        self.0.set_never();
    }

    /// Set this timer to fire after a specific duration, clearing any prior timer.
    #[inline]
    pub fn set_after(&mut self, after: Duration) {
        match Instant::now().checked_add(after) {
            None => self.set_never(),
            Some(deadline) => self.set_at(deadline),
        }
    }

    /// Set this timer to fire at a specific deadline, clearing any prior timer.
    #[inline]
    pub fn set_at(&mut self, deadline: Instant) {
        self.0.set_at(deadline);
    }

    /// Set this timer to fire on an interval, clearing any prior timer.
    #[inline]
    pub fn set_interval(&mut self, period: Duration) {
        match Instant::now().checked_add(period) {
            None => self.set_never(),
            Some(start) => self.set_interval_at(start, period),
        }
    }

    /// Set this timer to fire on an interval starting at a specific deadline, clearing any
    /// prior timer.
    #[inline]
    pub fn set_interval_at(&mut self, start: Instant, period: Duration) {
        self.0.set_interval(start, period);
    }
}

impl Future for Timer {
    type Output = ();

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll(cx)
    }
}

impl Stream for Timer {
    type Item = ();

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll(cx).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // This stream runs forever.
        (usize::MAX, None)
    }
}

/// Check if a thread is the main thread.
#[inline]
fn check_main_thread() -> io::Result<()> {
    if !sys::is_main_thread() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "keter-reactor must be run on the same thread that called main()",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "android"))]
    #[test]
    fn not_allowed_on_any_thread() {
        use crate::platform::instantiation::ReactorExt;

        std::thread::spawn(|| {
            assert!(Reactor::new().block_on(async { panic!() }).is_err());
        })
        .join()
        .unwrap();
    }
}
