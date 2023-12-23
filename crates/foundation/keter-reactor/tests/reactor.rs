// MIT/Apache2 License

use keter_reactor::{exit, main, Main, Reactor};
use macro_rules_attribute::apply;

#[apply(main!)]
fn main(reactor: Reactor) -> Main {
    reactor.block_on(async { exit().await })
}
