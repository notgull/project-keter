// MIT/Apache2 License

//! Initialize the Android application.
//!
//! This is a semver-exempt module used in the `main!` function.

use crate::sys::Settings;
use crate::Reactor;

/// Re-export of the `android_activity` crate.
pub use android_activity;

impl Reactor {
    /// Create a new `Reactor` from an `AndroidApp`.
    ///
    /// This function is not meant to be used in the public API.
    #[doc(hidden)]
    #[inline]
    pub fn __new(app: android_activity::AndroidApp) -> Self {
        Reactor {
            settings: Settings::new(app),
        }
    }
}
