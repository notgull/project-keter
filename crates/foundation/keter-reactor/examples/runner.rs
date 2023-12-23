// MIT/Apache2 License

//! Run the reactor.

keter_reactor::main! {
    fn main(reactor: keter_reactor::Reactor) -> keter_reactor::End {
        reactor.block_on(async { entry().await })
    }
}

async fn entry() -> ! {
    std::future::pending().await
}
