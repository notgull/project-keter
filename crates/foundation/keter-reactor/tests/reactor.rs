// MIT/Apache2 License

use keter_reactor::{Reactor, main, Main, exit};
use macro_rules_attribute::apply;

#[apply(main!)]
fn main(reactor: Reactor) -> Main {
    reactor.block_on(async {
        exit().await
    })
}

