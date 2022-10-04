use std::{boxed::Box, ffi::{CStr/*, CString,*/}, net::{SocketAddr/*, ToSocketAddrs*/}, os::raw::c_char, pin::Pin, sync::Arc};
#[macro_use]
extern crate log;
extern crate log4rs;
// use log::{/*debug,*/ error, info};
use base64::{decode as base64decode};
// use futures_util;
use async_stream::{/*try_stream,*/ stream};
use tokio::{
    sync::{/*Mutex,*/ oneshot},
    net::{TcpStream, TcpListener}
};
use tokio::signal;
use hyper::{
    Body, Request, Response, Server, Method, StatusCode, header, /*service::Service*/
    server::conn::{AddrStream},
    service::{service_fn, make_service_fn},
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use dns_lookup;
// use futures::future::{self, Future};

mod error;
mod util;
mod conf;
mod common;
mod svc;

struct Proxier {
    // local_addrs: HashSet<std::net::IpAddr>
    id: String,
    superior_id: Option<String>,
    name: String,
    svc: Pin<Arc<svc::AppSvc>>,
    sock: SocketAddr,
}

// impl Unpin for Proxy {}

// impl Service<Request<Body>> for Proxier {
//     type Response = Response<Body>;
//     type Error = hyper::Error;
//     type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Ok(()).into()
//     }

//     fn call(&mut self, req::Request<Body>) -> Self::Future {
//         self.request(peer_addr: SocketAddr, req)
//     }
// }

impl Proxier {
    pub fn new(id:&str, superior_id:Option<String>, name:&str, sock: SocketAddr, svc: Pin<Arc<svc::AppSvc>>) -> Result<Self, error::AppError> {
        // let local_addrs: HashSet<std::net::IpAddr> = cnf.server.hostnames.iter().fold(HashSet::new(), |mut s, i|{
        //     if let Ok(ip_addr) = i.parse() {
        //         s.insert(ip_addr);
        //     }

        //     s
        // });
        // // for iface in pnet::datalink::interfaces() {
        // //     // addrs.extend(iface.ips.iter().map(|ip| ip.ip()));
        // //     iface.ips.iter().map(|ip| ip.ip()).for_each(|ip| {
        // //         addrs.insert(ip);
        // //     });
        // // }
        // // addrs.insert("127.0.0.1".parse().unwrap());

        // Ok(
        //     Self {
        //         cnf: cnf.clone(),
        //         reverse: rerverse::RerverseProxy::new(&cnf),
        //         tunnal: tunnal::TunnalProxy::new(&cnf)?,
        //         local_addrs: local_addrs,
        //     }
        // )
        Ok(
            Self {
                id: id.to_string(),
                superior_id: superior_id,
                name: name.to_string(),
                sock: sock,
                svc: svc
            }
        )
    }

    pub fn get_sock(&self) -> &SocketAddr {
        &self.sock
    }

    pub fn get_introducation(&self) -> String {
        if let Some(ref s) = self.superior_id {
            format!("'{}' server on {} and tunnal to '{}'", self.id, self.sock, s)
        } else {
            format!("'{}' server on {}", self.id, self.sock)
        }
    }

    fn is_white(&self, ip_addr: &std::net::IpAddr) -> bool {
        let ip_addr_str = format!("{}", ip_addr);
        //TODO 增加比较模式
        return conf::ConfigMgr::get_instance().value().secure.whitelists.iter().any(|w| w.typ.eq("ip") && w.exp.eq(&ip_addr_str))
    }

    fn is_reverse(&self, req: &Request<Body>) -> bool {
        let dest_addr_str = match req.headers().get(header::HOST) {
            None  => req.uri().host(),
            Some(hvd) => hvd.to_str().ok(),
        };

        let mut ips = Vec::<std::net::IpAddr>::new();
        dest_addr_str.map(|s| {
            if util::net::is_ip(&s) {
                // if let Ok(ip) = s.parse::<std::net::IpAddr>() {
                //     ips.append(ip);
                // }
                if let Err(e) = s.parse::<std::net::IpAddr>().map(|ip| ips.push(ip)) {
                    error!("{}", e);
                }
            } 
            // } else if util::network::is_ipv4_hostport(&s){
            //     ips.append(s.rsplitn(1, ":").next().unwrap().parse().unwrap());
            // } else if util::network::is_ipv6_hostport(&s) {
            //     ips.append(s.rsplitn(1, ":").next().unwrap().trim_matches(|c| c == '[' || c == ']').parse().unwrap())l
            else if util::net::is_ip_hostport(&s) {
                if let Err(e) = s.parse::<std::net::SocketAddr>().map(|ip_addr| ips.push(ip_addr.ip())) {
                    error!("{}", e);
                }
            } else {
                if let Ok(v) = dns_lookup::lookup_host(s.rsplitn(1, ":").next().unwrap()) {
                    ips.extend(v);
                }
                // if let Some(lh) = lookup_host(s.rsplitn(1, ":").next().unwrap()) {
                // }
            }
        });

        //用域名做负载均衡时，只有一个ip与当前主机匹配。
        //只要有任何一个IP与本地IP相同，则此请求非代理，否则是代理。
        !conf::ConfigMgr::get_instance().value().hostnets.iter().any(|h| {
            ips.iter().any(|ip| {
                h.typ.eq("ip") && format!("{}", ip) == h.exp
            })
        })
    }

    fn authorize(&self, req: &Request<Body>, header: &str) -> bool {
        //header::PROXY_AUTHORIZATION
        let is_successful = match req.headers().get(header) {
            None  => false,
            Some(hvd) => {
                if let Ok(hv) = hvd.to_str() {
                    let v: Vec<&str> = hv.split(" ").into_iter().collect();
                    if 2 != v.len() {
                        error!("Invalid proxy authorization");
                        return false;
                    }

                    if let Ok(Ok(auth)) = base64decode(v[1]).map(String::from_utf8) {
                        // let v = hv.split(":").collect::<Vec<&str>>();
                        // if 2 == v.len() {
                        //     // return AsRef::<Vec<PermissionConfigurer>>::as_ref(&configurer.permissions)
                        //     //     .into_iter()
                        //     //     .any(|p| v[0] == p.user && v[1] == p.passwd)
                        //     return (&configurer.permissions)
                        //         .into_iter()
                        //         .any(|p| v[0] == p.user && v[1] == p.passwd)
                        // }
                        info!("Authorization - type({}),value({})", v[0], auth);
                        
                        return conf::ConfigMgr::get_instance().value().credentials.iter().any(|p| {
                            auth == format!("{}:{}", p.user, p.passwd)
                        });
                    }

                    error!("Authorize failed - header({});value({})", header, hv);
                } else {
                    error!("Invalid http header: {}", header);
                }

                false
            },
        };

        if is_successful {
            info!("Authorize successfully");
        }

        is_successful
    }

    fn build_authenticate_response(&self, status: StatusCode, authenticate_header: header::HeaderName) -> Result<Response<Body>, error::AppError> {
        let basic_auth_string = format!("Basic realm=\"{}\"", self.name.as_str());

        Ok(
            Response::builder()
                .status(status)
                .header(authenticate_header.as_str(), basic_auth_string)
                // .header(header::ACCEPT, "*/*")
                .header("Proxy-Agent", self.name.as_str())
                .body(Body::from("No authentication"))
                .unwrap()
        )
    }

    async fn request(&self, peer_addr: SocketAddr, req: Request<Body>) -> Result<Response<Body>, error::AppError> {
        // let configurer = configurer.clone();
        // let reverse_proxy = reverse_proxy.clone();
        info!("This '{}' client  {} {} to me({})", peer_addr, req.method(), req.uri(), self.id.as_str());

        let superior = match &self.superior_id {
            Some(id) => conf::ConfigMgr::get_instance().value().tunnal.superiors.iter().find(|conf| conf.id.eq(id)),
            None => None,
        };

        if Method::CONNECT == req.method() {
            if !self.is_white(&peer_addr.ip()) && !self.authorize(&req, header::PROXY_AUTHORIZATION.as_str()) {
                return self.build_authenticate_response(StatusCode::PROXY_AUTHENTICATION_REQUIRED, header::PROXY_AUTHENTICATE)
            }

            match superior{
                Some(s) => self.svc.tunnal_to_superior(req, s).await,
                None => self.svc.tunnal_to_destination(req).await,
            }
        } else if self.is_reverse(&req) {
            if !self.is_white(&peer_addr.ip()) && !self.authorize(&req, header::PROXY_AUTHORIZATION.as_str()) {
                return self.build_authenticate_response(StatusCode::PROXY_AUTHENTICATION_REQUIRED, header::PROXY_AUTHENTICATE)
            }

            self.svc.reverse(req, &peer_addr.ip(), &superior).await
        } else {
            //TODO 考虑重定向到登录页的方式。
            if !self.is_white(&peer_addr.ip()) && !self.authorize(&req, &header::AUTHORIZATION.as_str()) {
                return self.build_authenticate_response(StatusCode::UNAUTHORIZED, header::WWW_AUTHENTICATE)
            }

            Ok(
                if "/" == req.uri().path() {
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from("Welcome!"))
                        .unwrap()
                } else {
                    Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from(req.uri().path().to_string()))
                        .unwrap()
                }
            )
        }
    }

    fn create_one(server_conf: &conf::server_conf::ServerConf) -> Result<Proxier, error::AppError> {
        let svc = Arc::pin(svc::AppSvc::new(
            server_conf.get_external_address().as_str(),
            Arc::new(conf::ConfigMgr::get_instance().value().client.load_tls_client_configuration()?)
        ));

        Proxier::new(
            server_conf.id.as_str(),
            server_conf.superior_id.clone(),
            server_conf.realm.as_str(),
            server_conf.get_address().expect("Can't load server address"),
            svc
        )
    }

    async fn run_on_http(server_conf: &conf::server_conf::ServerConf, rx: oneshot::Receiver<()>) -> Result<(), error::AppError> {
        let proxier = Arc::pin(Proxier::create_one(server_conf)?);
        let intro = proxier.get_introducation();

        // let builder = Server::bind(proxier.get_sock());
        let std_listener = std::net::TcpListener::bind(&proxier.get_sock())?;
        std_listener.set_nonblocking(true)?; //alpine版本会出现TCP DUP ACK的问题，故尝试添加nonblocking试试
        let builder = Server::from_tcp(std_listener)?;

        let builder = builder.http1_title_case_headers(true);

        let make_svc = make_service_fn(move |conn:&AddrStream|{
            let packed = (proxier.clone(), conn.remote_addr().clone());

            async move {
                Ok::<_, error::AppError>(service_fn(move |req: Request<Body>| {
                    let (proxier, peer_addr) = packed.clone();

                    async move {
                        // let (addr, proxy) = pack.clone();
                        match proxier.request(peer_addr, req).await {
                            Ok(res) => Ok(res),
                            Err(e) => {
                                error!("Error: {}", e);
                                Err(e)
                            }
                        }
                    }
                }))
            }
        });

        info!("Run http server: {}", intro.as_str());

        let server = builder.serve(make_svc);
        // server.await?;
        server.with_graceful_shutdown(async {
            rx.await.ok();
        }).await?;

        Ok(())
    }

    async fn run_on_https(server_conf: &conf::server_conf::ServerConf, rx: oneshot::Receiver<()>) -> Result<(), error::AppError>{
        let proxier = Arc::pin(Proxier::create_one(server_conf)?);
        let intro = proxier.get_introducation();
        let tls_settings = Arc::new(server_conf.load_tls_server_configuration()?);
        let tls_acceptor = TlsAcceptor::from(tls_settings);

        // let tcp_listener = TcpListener::bind(&proxier.get_sock()).await?;
        let std_listener = std::net::TcpListener::bind(&proxier.get_sock())?;
        std_listener.set_nonblocking(true)?; //alpine版本会出现TCP DUP ACK的问题，故尝试添加nonblocking试试
        let tcp_listener = TcpListener::from_std(std_listener)?;

        let tls_incoming_stream = Box::pin(stream! {
            loop {
                match tcp_listener.accept().await {
                    Ok((socket, _)) => {
                        match tls_acceptor.accept(socket).await {
                            Err(e) => { error!("{}", e); },
                            i => { yield i; },
                        };
                    },
                    Err(e) => { error!("TLS error: {}", e); },
                }
            }
        });
    
        let builder = Server::builder(
            common::tls::HyperTlsAcceptor {
                acceptor: tls_incoming_stream,
            }
        );
 
        let builder = builder.http1_title_case_headers(true);

        let make_svc = make_service_fn(move |conn: &TlsStream<TcpStream>| {
            let (tcp_stream_io, _) = conn.get_ref(); 
            let packed = (proxier.clone(), tcp_stream_io.peer_addr().unwrap());
            // let pack = (tcp_stream_io.peer_addr().unwrap(), proxier.clone());

            async move {
                let packed = packed.clone();

                Ok::<_, error::AppError>(service_fn(move |req: Request<Body>| {
                    let (proxier, peer_addr) = packed.clone();

                    async move {
                        match proxier.request(peer_addr, req).await {
                            Ok(res) => Ok(res),
                            Err(e) => {
                                error!("Error: {}", e);
                                Err(e)
                            }
                        }
                    }
                }))
            }
        });

        info!("Run https server: {}", intro.as_str());

        let server = builder.serve(make_svc);
        // server.await?;
        server.with_graceful_shutdown(async move {
            rx.await.ok();
        }).await?;

        Ok(())
    }

    // #[tokio::main]
    pub async fn setup() -> Result<(), error::AppError> {
        let (txs, rxs): (Vec<oneshot::Sender<()>>, Vec<oneshot::Receiver<()>>) = 
            (0..conf::ConfigMgr::get_instance().value().servers.len()).map(|_| oneshot::channel::<()>()).unzip();

        // tokio::spawn(futures::stream::iter(rxs.into_iter().zip(conf::ConfigMgr::get_instance().value().servers.iter())).for_each(|pack| async move {
        //     let (rx, server_con) = pack;

        //     if server_con.on_https {
        //         Proxier::run_on_https(&server_con, rx);
        //     } else {
        //         Proxier::run_on_http(&server_con, rx);
        //     }
        // }));
        rxs.into_iter().zip(conf::ConfigMgr::get_instance().value().servers.iter()).for_each(|pack| {
            let (rx, server_con) = pack;

            tokio::spawn(async move {
                let rv = if server_con.on_https {
                    Proxier::run_on_https(&server_con, rx).await
                } else {
                    Proxier::run_on_http(&server_con, rx).await
                };

                match rv {
                    Ok(()) => {},
                    Err(e) => { error!("{}", e); },
                };
            });
        });

        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Catch ctrl_c signal");
                txs.into_iter().for_each(|tx| {
                    if let Err(_) = tx.send(()) {
                        error!("ctrl_c failed");
                    }
                });
            },
            Err(err) => {
                // eprintln!("Unable to listen for shutdown signal: {}", err);
                error!("Unable to listen for shutdown signal: {}", err);
                // we also shut down in case of error
            },
        };

        // rx.await.expect("Wait rx");
        // signal_thread.join().expect("The thread being joined has panicked");

        info!("Server exited");
        Ok(())
    }
}

pub fn start_server(configuration_file_str: &str) {
    conf::ConfigMgr::get_instance().init(configuration_file_str).expect("Initialize failed");

    tokio::runtime::Builder::new_current_thread()
        .thread_name("main")
        .enable_all()
        .build()
        .unwrap()
        // .name("main")
        .block_on(async move {
            Proxier::setup().await.unwrap_or_else(|e| error!("Server Error: {}", e));
        });
}

#[no_mangle]
pub extern "C" fn c_start_server(configuration_file_cstr: *const c_char) {
    let configuration_file_str = unsafe {
        CStr::from_ptr(configuration_file_cstr).to_string_lossy().into_owned()
    };

    start_server(&configuration_file_str);
    // std::process::abort();
}

// #[no_mangle]
// pub extern "C" fn add(a: i32, b: i32) -> i32 {
//     println!("Add");
//     a + b
// }

// #[no_mangle]
// pub extern "C" fn cb(f: extern "C" fn() -> i32) -> u32 {
//     println!("CB");
//     f() as u32
// }