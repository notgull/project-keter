// MIT/Apache2 License

//! Platform-specific code.

#[cfg(target_os = "android")]
#[doc(hidden)]
pub mod android;
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
pub mod any_thread;
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
pub mod instantiation;
#[cfg(all(unix, not(target_vendor = "apple"), not(target_os = "android")))]
pub mod poll_io;

mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
    impl Sealed for crate::Reactor {}
}
