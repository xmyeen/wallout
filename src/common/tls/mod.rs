use std::pin::Pin;
use core::task::{Context, Poll};
use futures::{Stream};
use tokio_rustls::server::TlsStream;
use tokio::{net::TcpStream};
/*time::Duration,*/

// type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

pub struct HyperTlsAcceptor<'a> {
    pub acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, std::io::Error>> + Send + 'a>>,
}

impl hyper::server::accept::Accept for HyperTlsAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = std::io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match Pin::new(&mut self.acceptor).poll_next(cx) {
            Poll::Ready(Some(Err(_e))) => Poll::from(None),
            p => p,
        }
    }
}
