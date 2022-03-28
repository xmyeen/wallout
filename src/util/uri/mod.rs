
use std::net::{SocketAddr, ToSocketAddrs};
use hyper::http::uri::Uri;

pub fn parse_uri_socket_addr(uri: &Uri) -> Option<SocketAddr> {
    uri.authority()
        .and_then(|auth| {
            if let Some(port) = auth.port_u16() {
                Some((auth.host(), port))
            } else {
                match uri.scheme_str() {
                    Some("http") => Some((auth.host(), 80)),
                    Some("https") => Some((auth.host(), 443)),
                    _ => None
                }
            }
        })
        .and_then(|hostport| match hostport.to_socket_addrs() {
            Ok(mut opt) => opt.next(),
            Err(_) => None,
        })
}