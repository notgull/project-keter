// MIT/Apache2 License

//! Platform-specific code.

pub mod any_thread;
pub mod instantiation;
pub mod poll_io;

mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
    impl Sealed for crate::Reactor {}
}
