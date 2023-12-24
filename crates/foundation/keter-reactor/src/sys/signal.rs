// MIT/Apache2 License

//! A flag that can be `await`ed on. Used for setting up early exits.

use event_listener::{Event, EventListener};
use std::sync::atomic::{AtomicBool, Ordering};

/// A signal to indicate that we are stopping.
pub(super) struct Signal {
    /// Whether to stop running.
    stop_running: AtomicBool,

    /// The signal.
    stop_ops: Event,
}

impl Signal {
    /// Get the global instance of `Signal`
    #[inline]
    pub(super) fn get() -> &'static Signal {
        static SIGNAL: Signal = Signal {
            stop_running: AtomicBool::new(false),
            stop_ops: Event::new(),
        };

        &SIGNAL
    }

    /// Wait for the signal to be sent.
    pub(super) async fn wait(&self) {
        let listener = EventListener::new();
        futures_lite::pin!(listener);

        loop {
            // Do we need to stop running?
            if self.stop_running.swap(false, Ordering::Relaxed) {
                return;
            }

            // Establish the listener.
            listener.as_mut().listen(&self.stop_ops);

            // Check again.
            if self.stop_running.swap(false, Ordering::SeqCst) {
                return;
            }

            // Wait on the listener.
            listener.as_mut().await;
        }
    }

    /// Send the signal.
    #[cold]
    pub(super) fn stop(&self) {
        self.stop_running.store(true, Ordering::Release);
        self.stop_ops.notify_additional(std::usize::MAX);
    }
}
