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
        Self {
            _private: ()
        }
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
