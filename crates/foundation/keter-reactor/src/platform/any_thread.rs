// MIT/Apache2 License

//! Extension traits for reactors that can run on any thread.

use crate::Reactor;

/// Extension trait that allows for the [`Reactor`] to be run on any thread.
///
/// [`Reactor`]: crate::Reactor
pub trait ReactorExt: Sized + crate::platform::sealed::Sealed {
    /// Remove the guard that prevents this reactor from being run on the non-main thread.
    fn with_any_thread(self) -> Self;
}

impl ReactorExt for Reactor {
    #[inline]
    fn with_any_thread(mut self) -> Self {
        self.settings.any_thread = true;
        self
    }
}
