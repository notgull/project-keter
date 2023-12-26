// MIT/Apache2 License

mod async_io;
mod timer;

use futures_lite::future;
use keter_reactor::{exit, main, Main, Reactor};
use macro_rules_attribute::apply;

#[apply(main!)]
fn main(reactor: Reactor) -> Main {
    keter_test::run_tests(|harness| {
        let result = reactor.block_on(async {
            // Successful launch.
            harness.test("starts_up", async {}).await;

            // Yield now.
            harness.test("yield_now", future::yield_now()).await;

            // Group of tests.
            harness
                .group("functionality", 2, async {
                    // Handle async I/O.
                    harness
                        .test("async_io", async {
                            async_io::test().await;
                        })
                        .await;

                    // Handle timers.
                    harness
                        .test("timer", async {
                            timer::test().await;
                        })
                        .await;
                })
                .await;

            exit().await
        });

        futures_lite::future::block_on(harness.test("finishes_properly", async {
            assert!(result.is_ok());
        }));

        result
    })
}
