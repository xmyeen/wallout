use std::{convert::TryFrom, sync::Arc/*, pin::Pin, boxed::Box*/};

use log::{info, error};

/*use futures::{try_join};*/

use base64::{encode as base64encode};

use hyper::{
    Request, Response, Body,
    http,
    /*upgrade::Upgraded,*/
};

use httparse;

//use crate::io::{AsyncRead, AsyncWrite, ReadBuf};
use rustls::{self, ClientConfig as TlsClientConfig};
use tokio::{net::TcpStream, io::{AsyncRead, AsyncWrite}};
use tokio_rustls::TlsConnector;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use webpki::DNSNameRef;

use crate::util;
use crate::error::{AppError};
use crate::conf::{tunnal_conf::superior_conf::SuperiorConf};

pub struct TunnalSvc {
    tls_client_config: Arc<TlsClientConfig>,
}

impl TunnalSvc {
    pub fn new(client_config: Arc<TlsClientConfig>) -> Self {
        //cnf.load_tls_client_configuration()?
        Self {
            tls_client_config: client_config,
        }
    }

    // async fn exchange_once<T>(stream: &mut T, raw: &[u8]) -> Result<Vec<u8>, AppError> 
    //     where T: AsyncWrite + AsyncRead + Unpin
    // {
    //     stream.write_all(raw).await?;
    //     stream.readable().await?;

    //     let retdata: Vec<u8> = Vec::new();
    //     let mut buf = [0; 4096];
    //     while let Ok(n) = stream.try_read(&mut buf) {
    //         if 0 == n {
    //             break;
    //         }
    //         retdata.append(buf);
    //     }

    //     Ok(retdata);
    // }

    async fn exchange<A, B>(in_stream: A, out_stream: B )
        where 
            A: AsyncWrite + AsyncRead + Unpin,
            B: AsyncWrite + AsyncRead + Unpin,
    {
        info!("Do exchange");

        // // Proxying data
        // let amounts = {
        //     let (mut server_rd, mut server_wr) = tokio::io::split(out_stream);
        //     let (mut client_rd, mut client_wr) = tokio::io::split(in_stream);

        //     let client_to_server = tokio::io::copy(&mut client_rd, &mut server_wr);
        //     let server_to_client = tokio::io::copy(&mut server_rd, &mut client_wr);

        //     try_join!(client_to_server, server_to_client)
        // };


        // // Print message when done
        // match amounts {
        //     Ok((from_client, from_server)) => {
        //         info!("client wrote {} bytes and received {} bytes", from_client, from_server);
        //         Ok(())
        //     }
        //     Err(e) => Err(AppError::IOError(e)),
        // }
        let (mut is_mut, mut os_mut)  = (in_stream, out_stream);
        match tokio::io::copy_bidirectional(&mut is_mut, &mut os_mut).await {
            Ok((from_client, from_server)) => info!("client wrote {} bytes and received {} bytes", from_client, from_server),
            Err(e) => error!("Exchange failed: {}", e),
        };
    }

    async fn open_superior_stream<S>(&self, stream:&mut S, dest_uri: &http::uri::Uri, conf: &SuperiorConf) -> Result<(), AppError> 
    where
        S: AsyncRead + AsyncWrite + Unpin {

        let content = if conf.username.is_some() && conf.password.is_some() {
            let auth = base64encode(
                format!("{}:{}", conf.username.as_ref().unwrap().as_str(),conf.password.as_ref().unwrap().as_str())
            );
            format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Connection: Keep-Alive\r\nProxy-Authorization: Basic {}\r\n\r\n", dest_uri, dest_uri, auth)
        } else {
            format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\nProxy-Connection: Keep-Alive\r\n\r\n", dest_uri, dest_uri)
        };

        stream.write_all(content.as_bytes()).await?;

        let mut rawdata = vec![0u8; 1024];
        let size = stream.read(rawdata.as_mut_slice()).await?;

        let mut headers = [httparse::EMPTY_HEADER; 1];
        let mut response = httparse::Response::new(&mut headers[..]);
        response.parse(&rawdata[0..size]).map_err(|e| AppError::RuntimeError(format!("{}", e)))?;
        if response.code.is_none() || 200u16 != response.code.unwrap() {
            return Err(AppError::RuntimeError(response.reason.unwrap_or("Server Error").to_string()));
        }

        Ok(())
    }

    pub async fn tunnal_to_destination(&self, req: Request<Body>) -> Result<Response<Body>, AppError> {
        let sock_addr = util::uri::parse_uri_socket_addr(req.uri())
            .ok_or_else(|| AppError::RuntimeError(format!("Can't parse destination addr: {}", req.uri())))?;

        let stream = TcpStream::connect(&sock_addr).await?;

        tokio::task::spawn(async move {
            let uri = req.uri().clone();
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    info!("Tunnel to destination '{}'", uri);
                    TunnalSvc::exchange(upgraded, stream).await;
                },
                Err(e) => {
                    error!("Upgrade error: {}", e);
                },
            }
        });

        Ok(Response::new(Body::empty()))
    }

    pub async fn tunnal_to_superior(&self, req: Request<Body>,  superior: &SuperiorConf) -> Result<Response<Body>, AppError> {
        // Received an HTTP request like:
        // ```
        // CONNECT www.domain.com:443 HTTP/1.1
        // Host: www.domain.com:443
        // Proxy-Connection: Keep-Alive
        // ```
        //
        // When HTTP method is CONNECT we should return an empty body
        // then we can eventually upgrade the connection and talk a new protocol.
        //
        // Note: only after client received an empty body with STATUS_OK can the
        // connection be upgraded, so we can't return a response inside
        // `on_upgrade` future.
        // match ConfigMgr::get_instance().value().tunnal.superior {
        let uri = req.uri().clone();
        let host = superior.uri.host()
            .ok_or_else(|| AppError::RuntimeError(format!("Can't parse superior host: {}", &superior.uri)))?;
        let sock_addr = util::uri::parse_uri_socket_addr(&superior.uri)
            .ok_or_else(|| AppError::RuntimeError(format!("Can't parse superior addr: {}", &superior.uri)))?;

        let scheme = superior.uri.scheme()
            .ok_or_else(|| AppError::RuntimeError(format!("Can't parse superior schem: {}", &superior.uri)))?;

        if "https" == scheme.as_ref() {
            let stream = TcpStream::connect(&sock_addr).await?;
            let tls_connector = TlsConnector::from(self.tls_client_config.clone());
            let domain = rustls::ServerName::try_from(host)
                .map_err(|e| AppError::RuntimeError(format!("{}", e)))?;
            let mut tls_stream = tls_connector.connect(domain, stream).await?;
            self.open_superior_stream(&mut tls_stream, &uri, &superior).await?;
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        info!("Tunnel to superior on tls");
                        TunnalSvc::exchange(upgraded, tls_stream).await;
                    },
                    Err(e) => {
                        error!("Upgrade error: {}", e);
                    },
                }
            });
            Ok(Response::new(Body::empty()))
        } else if "http" == scheme.as_ref() {
            // info!("55555555555555555 {}", &sock_addr);
            let mut stream = TcpStream::connect(&sock_addr).await?;
            self.open_superior_stream(&mut stream, &uri, &superior).await?;
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        info!("Tunnel to superior");
                        TunnalSvc::exchange(upgraded, stream).await;
                    },
                    Err(e) => {
                        error!("Upgrade error: {}", e);
                    },
                }
            });
            Ok(Response::new(Body::empty()))
        } else  {
            error!("Unsupport scheme: {}", superior.uri);
            let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
            *resp.status_mut() = http::StatusCode::BAD_REQUEST;
            Ok(resp)
        }
    }
}