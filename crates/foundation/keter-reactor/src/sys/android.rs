// MIT/Apache2 License

//! Code for the Android platform.

#[path = "signal.rs"]
mod signal;

use android_activity::{AndroidApp, AndroidAppWaker, PollEvent};
use futures_lite::prelude::*;
use once_cell::sync::OnceCell;
use signal::Signal;

use std::future::Future;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

/// Settings for the Android event loop.
pub(crate) struct Settings {
    /// The android application to run.
    app: AndroidApp,
}

impl Settings {
    /// Create a new `Settings` with the android app to run with.
    #[inline]
    pub fn new(app: AndroidApp) -> Self {
        Self { app }
    }
}

/// Run the reactor.
pub(crate) fn block_on<T>(settings: Settings, f: impl Future<Output = T>) -> io::Result<Option<T>> {
    let reactor = Reactor::get();

    // We are now running the reactor make sure to set it to "not running" on our way out.
    let old_state = reactor.state.swap(NOTIFIED, Ordering::SeqCst);
    assert_eq!(old_state, NOT_RUNNING);
    let _guard = CallOnDrop(|| {
        reactor.state.store(NOT_RUNNING, Ordering::Release);
    });

    // Create the future to poll.
    let future = async move {
        // Poll the future given by the user.
        let user_future = async move { Some(f.await) };

        // Simultaneously, wait for the exit signal to be fired.
        let wait_for_end = async {
            Signal::get().wait().await;
            None
        };

        // Run these futures in parallel.
        user_future.or(wait_for_end).await
    };

    // Pin the future to the stack.
    futures_lite::pin!(future);

    // Create a waker to poll the future with.
    let waker = Waker::from(Arc::new(ReactorWaker {
        reactor,
        waker: settings.app.create_waker(),
    }));
    let mut context = Context::from_waker(&waker);

    // Begin the event processing loop.
    let mut was_notified = true;
    let mut result = None;
    loop {
        let timeout = if was_notified {
            Some(Duration::from_secs(0))
        } else {
            // TODO: Handle timers.
            None
        };

        // If we are about to go to sleep, indicate that we will be asleep.
        if timeout != Some(Duration::from_secs(0)) {
            // Make sure we aren't overwriting any NOTIFIED states; we only want to change AWAKE to ASLEEP.
            let _ =
                reactor
                    .state
                    .compare_exchange(AWAKE, ASLEEP, Ordering::SeqCst, Ordering::Relaxed);
        }

        // Go to sleep and poll for events.
        settings.app.poll_events(timeout, |event| {
            // If we were previously asleep, we are now awake.
            let _ =
                reactor
                    .state
                    .compare_exchange(ASLEEP, AWAKE, Ordering::SeqCst, Ordering::Relaxed);

            // Handle the event.
            reactor.handle_event(event);

            // Poll the future if it is notified.
            if let NOTIFIED = reactor.state.swap(AWAKE, Ordering::SeqCst) {
                // If the future is ready, we're done.
                if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
                    result = Some(value);
                }

                // If the reactor is notified immediately after polling the future, it's probably
                // yielding for a reactor event.
                was_notified = reactor.state.load(Ordering::Acquire) == NOTIFIED;
            }
        });

        // See if we are ready to exit the loop.
        if let Some(result) = result.take() {
            return Ok(result);
        }
    }
}

/// Send the signal to exit.
pub(crate) fn exit() -> io::Result<()> {
    Signal::get().stop();
    Ok(())
}

/// The timer implementation.
pub(crate) struct Timer(());

impl Unpin for Timer {}

impl Timer {
    /// Create a timer that will never fire.
    #[inline]
    pub(crate) fn never() -> Self {
        todo!()
    }

    /// Create a timer that fires at a specific deadline.
    #[inline]
    pub(crate) fn at(at: crate::Instant) -> Self {
        todo!()
    }

    /// Create a timer that fires repeatedly.
    #[inline]
    pub(crate) fn interval(at: crate::Instant, interval: crate::Duration) -> Self {
        todo!()
    }

    /// Set this timer to never fire.
    #[inline]
    pub(crate) fn set_never(&mut self) {
        todo!()
    }

    /// Set this timer to an `at()` timer.
    #[inline]
    pub(crate) fn set_at(&mut self, at: crate::Instant) {
        todo!()
    }

    /// Set this timer to an `interval()` timer.
    #[inline]
    pub(crate) fn set_interval(&mut self, at: crate::Instant, interval: crate::Duration) {
        todo!()
    }

    /// Wait for the next time this timer fires.
    #[inline]
    pub(crate) fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        todo!()
    }
}

#[inline]
pub(crate) fn is_main_thread() -> bool {
    // TODO: I think AndroidApp handles this automatically.
    true
}

/// The reactor is not running.
const NOT_RUNNING: usize = 0;
/// The reactor is about to start processing more events.
const NOTIFIED: usize = 1;
/// The reactor is currently sleeping.
const ASLEEP: usize = 2;
/// The reactor is processing events but is not notified.
const AWAKE: usize = 3;

/// The state of the current `block_on()` call.
///
/// As only one `AndroidApp` can exist at a time, this ensures that we only need one reactor.
struct Reactor {
    /// The current state of the reactor.
    state: AtomicUsize,
}

impl Reactor {
    /// Get the current instance of the reactor.
    #[inline]
    fn get() -> &'static Reactor {
        static REACTOR: OnceCell<Reactor> = OnceCell::new();

        REACTOR.get_or_init(|| Reactor {
            state: AtomicUsize::new(NOT_RUNNING),
        })
    }

    /// Handle a `PollEvent`.
    #[inline]
    fn handle_event(&self, event: PollEvent<'_>) {
        match event {
            PollEvent::Wake | PollEvent::Timeout => {
                // We just woke up the loop to poll the future or any timers. Do nothing.
            }

            PollEvent::Main(main) => {
                // TODO: Actually handle this event.
                let _ = main;
            }

            event => {
                // TODO: How do we handle this event?
                let _ = event;
            }
        }
    }
}

/// The waker to use to poll the `Reactor`.
struct ReactorWaker {
    /// Static reference to the reactor.
    reactor: &'static Reactor,

    /// The waker for the `AndroidApp`.
    waker: AndroidAppWaker,
}

impl ReactorWaker {
    #[inline]
    fn notify(&self) {
        match self.reactor.state.swap(NOTIFIED, Ordering::SeqCst) {
            NOT_RUNNING => {
                // Can't really do much if the reactor is asleep.
            }

            ASLEEP => {
                // The loop is asleep; we need to wake it up to run the future.
                self.waker.wake();
            }

            AWAKE => {
                // The loop is currently processing events and will handle our wakeup shortly.
            }

            NOTIFIED => {
                // Another waker woke the event loop up; they should poll us in the process.
            }

            state => panic!("unintelligible event loop state: {state:x}"),
        }
    }
}

impl Wake for ReactorWaker {
    #[inline]
    fn wake_by_ref(self: &Arc<Self>) {
        self.notify();
    }

    #[inline]
    fn wake(self: Arc<Self>) {
        self.notify();
    }
}

struct CallOnDrop<F: FnMut()>(F);

impl<F: FnMut()> Drop for CallOnDrop<F> {
    #[inline]
    fn drop(&mut self) {
        (self.0)()
    }
}
