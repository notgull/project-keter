// MIT/Apache2 License

//! A reactor for driving GUI systems.

#![forbid(unsafe_code)]

pub mod platform;
mod sys;

use std::convert::Infallible;
use std::future::Future;
use std::io;

/// Macro for creating the main function.
#[macro_export]
macro_rules! main {
    (
        $(#[$attr:meta])*
        fn $name:ident ($sett:ident : $sty:ty) -> $res:ty $bl:block
    ) => {
        fn main() {
            use $crate::platform::instantiation::ReactorExt as _;

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
    _private: ()
}

/// Settings for the reactor to drive the system.
pub struct Reactor {
    settings: sys::Settings
}

impl Reactor {
    /// Block on a future for as long as possible.
    #[inline]
    pub fn block_on(self, future: impl Future<Output = Infallible>) -> Result<Finished> {
        sys::block_on(self.settings, future)?;
        Ok(Finished {
            _private: ()
        })
    }
}

/// Check if a thread is the main thread.
#[inline]
fn check_main_thread() -> io::Result<()> {
    if !sys::is_main_thread() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "keter-reactor must be run on the same thread that called main()"
        ))
    } else {
        Ok(())
    }
}
