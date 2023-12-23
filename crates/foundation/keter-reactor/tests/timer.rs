// MIT/Apache2 License

use futures_lite::prelude::*;
use keter_reactor::{exit, main, Main, Reactor, Timer};
use macro_rules_attribute::apply;
use web_time::{Instant, Duration};

#[apply(main!)]
fn main(reactor: Reactor) -> Main {
    reactor.block_on(async {
        // Smoke test, taken from async-io tests.
        let start = Instant::now();
        Timer::after(Duration::from_secs(2)).await;
        assert!(Instant::now() - start >= Duration::from_secs(2));

        // Interval test, taken from async-io tests.
        let period = Duration::from_secs(1);
        let jitter = Duration::from_millis(500);
        let start = Instant::now();
        let mut timer = Timer::interval(period);
        timer.next().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= period && elapsed - period < jitter);
        timer.next().await;
        let elapsed = start.elapsed();
        assert!(elapsed >= period * 2 && elapsed - period * 2 < jitter);

        exit().await
    })
}
