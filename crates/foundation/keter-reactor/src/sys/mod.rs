// MIT/Apache2 License

//! System-specific code.

#[cfg_attr(
    all(unix, not(target_vendor = "apple"), not(target_os = "android")),
    path = "free_unix.rs"
)]
#[cfg_attr(target_os = "android", path = "android.rs")]
mod inner;

pub(crate) use inner::*;
