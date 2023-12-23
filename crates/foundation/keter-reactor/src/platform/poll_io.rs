// MIT/Apache2 License

//! Poll I/O using the reactor.
//!
//! This is available on open-source Unixes, and can maybe be added to Apple Unixes.

use futures_lite::prelude::*;

use std::fmt;
use std::future::Future;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::os::unix::io::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A wrapper around an I/O source that allows itself to be polled on the reactor.
pub struct Async<T>(async_io::Async<T>);

impl<T: fmt::Debug> fmt::Debug for Async<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Async")
            .field("inner", self.0.get_ref())
            .finish_non_exhaustive()
    }
}

impl<T: AsFd> Async<T> {
    /// Create a new `Async<T>` wrapping around an I/O source.
    #[inline]
    pub fn new(io: T) -> io::Result<Self> {
        async_io::Async::new(io).map(Self)
    }

    /// Create a new `Async<T>` without setting the I/O source into non-blocking mode.
    #[inline]
    pub fn with_nonblocking(io: T) -> io::Result<Self> {
        async_io::Async::new_nonblocking(io).map(Self)
    }
}

impl<T> Async<T> {
    /// Get a reference to the underlying type.
    #[inline]
    pub fn get_ref(&self) -> &T {
        self.0.get_ref()
    }

    /// Convert this back into a `T`.
    #[inline]
    pub fn into_inner(self) -> io::Result<T> {
        self.0.into_inner()
    }

    /// Polls the I/O handle for readability.
    #[inline]
    pub fn poll_readable(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.poll_readable(cx)
    }

    /// Polls the I/O handle for writability.
    #[inline]
    pub fn poll_writable(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.poll_writable(cx)
    }

    /// Waits for this I/O handle to become readable.
    #[inline]
    pub fn readable(&self) -> Readable<'_, T> {
        Readable(self.0.readable())
    }

    /// Waits for this I/O handle to become writable.
    #[inline]
    pub fn writable(&self) -> Writable<'_, T> {
        Writable(self.0.writable())
    }
}

impl Async<TcpListener> {
    /// Bind to a specific TCP socket.
    #[inline]
    pub fn bind(address: impl Into<SocketAddr>) -> io::Result<Self> {
        async_io::Async::<TcpListener>::bind(address.into()).map(Self)
    }

    /// Wait for a new TCP connection.
    #[inline]
    pub async fn accept(&self) -> io::Result<(Async<TcpStream>, SocketAddr)> {
        self.0
            .accept()
            .await
            .map(|(socket, addr)| (Async(socket), addr))
    }

    /// Wait for a stream of incoming TCP connections.
    #[inline]
    pub fn incoming(&self) -> impl Stream<Item = io::Result<Async<TcpStream>>> + Send + '_ {
        self.0.incoming().map(|res| res.map(Async))
    }
}

impl Async<TcpStream> {
    /// Connect to a specific TCP socket.
    #[inline]
    pub async fn connect(address: impl Into<SocketAddr>) -> io::Result<Self> {
        async_io::Async::<TcpStream>::connect(address.into())
            .await
            .map(Self)
    }
}

impl Async<UnixListener> {
    /// Bind this listener to a specific path.
    #[inline]
    pub fn bind(path: impl AsRef<Path>) -> io::Result<Self> {
        async_io::Async::<UnixListener>::bind(path.as_ref()).map(Self)
    }
}

impl Async<UnixStream> {
    /// Connect over UDS to the specific path.
    #[inline]
    pub async fn connect(path: impl AsRef<Path>) -> io::Result<Self> {
        async_io::Async::<UnixStream>::connect(path.as_ref())
            .await
            .map(Self)
    }

    /// Create a pair of joined sockets.
    #[inline]
    pub fn pair() -> io::Result<(Self, Self)> {
        async_io::Async::<UnixStream>::pair().map(|(left, right)| (Self(left), Self(right)))
    }
}

/// The future to wait for this I/O source to be readable.
pub struct Readable<'a, T>(async_io::Readable<'a, T>);

impl<T> fmt::Debug for Readable<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Readable").finish_non_exhaustive()
    }
}

impl<T> Unpin for Readable<'_, T> {}

impl<T> Future for Readable<'_, T> {
    type Output = io::Result<()>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

/// The future to wait for this I/O source to be writable.
pub struct Writable<'a, T>(async_io::Writable<'a, T>);

impl<T> fmt::Debug for Writable<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Writable").finish_non_exhaustive()
    }
}

impl<T> Unpin for Writable<'_, T> {}

impl<T> Future for Writable<'_, T> {
    type Output = io::Result<()>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}
