// MIT/Apache2 License

//! Extension traits to instantiate the [`Reactor`] freely.
//! 
//! [`Reactor`]: crate::Reactor

use crate::Reactor;

/// Allows the user to instantiate the [`Reactor`] freely.
/// 
/// [`Reactor`]: crate::Reactor
pub trait ReactorExt : Sized + crate::platform::sealed::Sealed {
    /// Create a new [`Reactor`] with default settings.
    /// 
    /// [`Reactor`]: crate::Reactor
    fn new() -> Self;
}

impl ReactorExt for Reactor {
    #[inline]
    fn new() -> Self {
        Reactor {
            settings: crate::sys::Settings::empty()
        }
    }
}
