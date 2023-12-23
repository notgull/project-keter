// MIT/Apache2 License

use keter_reactor::platform::poll_io::Async;
use keter_reactor::{exit, main, Main, Reactor};
use macro_rules_attribute::apply;

use std::io;
use std::net::{TcpListener, TcpStream};

#[apply(main!)]
fn main(reactor: Reactor) -> Main {
    reactor.block_on(async {
        entry().await.unwrap();
        exit().await
    })
}

async fn entry() -> io::Result<()> {
    // Connect over TCP.
    let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 0))?;
    let addr = listener.get_ref().local_addr()?;
    let task = async move { listener.accept().await };

    let stream2 = Async::<TcpStream>::connect(addr).await?;
    let stream1 = task.await?.0;

    assert_eq!(
        stream1.get_ref().peer_addr()?,
        stream2.get_ref().local_addr()?,
    );
    assert_eq!(
        stream2.get_ref().peer_addr()?,
        stream1.get_ref().local_addr()?,
    );

    // Now that the listener is closed, connect should fail.
    let err = Async::<TcpStream>::connect(addr).await.unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::ConnectionRefused);

    Ok(())
}
