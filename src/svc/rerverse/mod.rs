
use std::{
    str::FromStr,
    // collections::HashMap,
    net::IpAddr,
    sync::Arc,
    convert::TryFrom,
    // pin::Pin
    /*time::Duration,*/
};
use regex;
use rustls::{self, ClientConfig as TlsClientConfig};
use tokio;
use tokio_rustls::TlsConnector;
use http::uri;
// use hyper::http::uri::InvalidUri;
use hyper::{
    self,
    Uri, Body, Client, Request, Response, StatusCode,
    header::{HOST, LOCATION, PROXY_AUTHORIZATION, HeaderValue}
};
// use hyper_tls::HttpsConnector;
// use hyper_proxy::{Proxy, ProxyConnector, Intercept};
use headers::{authorization::{Credentials, Authorization/*, Basic*/}};

use crate::error::{AppError};
use crate::util;
use crate::conf::ConfigMgr;
use crate::conf::{tunnal_conf::superior_conf::SuperiorConf};

mod header;


#[derive(Clone)]
pub struct RerverseSvc {
    tls_client_config: Arc<TlsClientConfig>,
    server_external_address: String
    // endpoint: Arc<RwLock<HashMap<String, Mutex<String>>>>,
}

impl RerverseSvc {
    pub fn new(client_config: Arc<TlsClientConfig>, server_external_address: &str) -> Self {
        Self {
            tls_client_config: client_config,
            server_external_address: server_external_address.to_string(),
            // endpoint: Arc::new(RwLock::new(HashMap::new())),
            // setting: config::Config::default(),
        }
    }

    fn parse_hostport<B>(&self, request: &Request<B>) -> Result<String, AppError> {
        let port = request.uri().port_u16();
        let host = request.uri().host();
        if host.is_none() {
            return Err(AppError::UriError("No host found".to_string()));
        } else if port.is_none() {
            return Ok(host.unwrap().to_string());
        } else {
            return Ok(format!("{}:{}", host.unwrap(), port.unwrap()));
        }
    }

    fn create_proxied_uri(&self, uri: &Uri) -> Result<Uri, AppError> {
        ConfigMgr::get_instance().value().proxy_pass_locations
            .iter()
            .filter(|c| {
                regex::Regex::from_str(c.matcher.as_str())
                    .map(|r| r.is_match(uri.path()))
                    .unwrap_or(false) 
            })
            .filter_map(|c| {
                if ! c.replace_re.is_empty() {
                    match regex::Regex::from_str(c.replace_re.as_str()) {
                        Ok(re) => {
                            Some(format!(
                                "{}://{}{}?{}",
                                uri.scheme().unwrap(),
                                c.proxy_pass.as_str(),
                                re.replace_all(uri.path(), c.replace_to.clone()).into_owned(),
                                uri.query().unwrap_or_default()
                            ))
                        }
                        Err(_e) => {
                            error!("Invalid regex expression");
                            None
                        },
                    }
                } else {
                    Some(format!(
                        "{}://{}{}?{}",
                        uri.scheme().unwrap(),
                        c.proxy_pass.as_str(),
                        uri.path(),
                        uri.query().unwrap_or_default()
                    ))
                }
            })
            .filter_map(|u| Uri::from_str(u.as_str()).ok() )
            .nth(0)
            .ok_or(AppError::UriError("No match".to_string()))
    }

    // fn parse_forward_uri<B>(forward_url: &str, req: &Request<B>) -> Result<Uri, InvalidUri> {
    //     let forward_uri = match req.uri().query() {
    //         Some(query) => format!("{}{}?{}", forward_url, req.uri().path(), query),
    //         None => format!("{}{}", forward_url, req.uri().path()),
    //     };

    //     create_proxied_uri(forward_url, req)
    // }

    fn create_proxied_request<B>(
        &self,
        client_ip: &IpAddr,
        // forward_url: &str,
        mut request: Request<B>,
        superior: &Option<&SuperiorConf>
    ) -> Result<Request<B>, AppError> {
        if let Some(superior_conf) = superior {
            request.headers_mut().insert(PROXY_AUTHORIZATION, Authorization::basic(
                superior_conf.username.as_ref().unwrap().as_str(),
                superior_conf.password.as_ref().unwrap().as_str()
            ).0.encode());
        } else {
            *request.headers_mut() = header::remove_hop_headers(request.headers());
        }

        if let Ok(uri) = self.create_proxied_uri(request.uri()) {
            *request.uri_mut() = uri;
            if let Ok(hostport) =  self.parse_hostport(&request) {
                request.headers_mut().insert(HOST, HeaderValue::from_str(hostport.as_str())?);
            }
        }

        // Add forwarding information in the headers
        match request.headers_mut().entry("x-forwarded-for") {
            hyper::header::Entry::Vacant(entry) => {
                entry.insert(format!("{}, {}", client_ip, self.server_external_address.as_str()).parse()?);
            }

            hyper::header::Entry::Occupied(mut entry) => {
                entry.insert(format!("{}, {}", entry.get().to_str()?, self.server_external_address.as_str()).parse()?);
            }
        }

        Ok(request)
    }

    fn create_proxied_response<B>(
        &self,
        mut response: Response<B>
    ) -> Result<Response<B>, AppError> {
        *response.headers_mut() = header::remove_hop_headers(response.headers());

        if let hyper::header::Entry::Occupied(mut entry) = response.headers_mut().entry(LOCATION) {
            if let Ok(u) = Uri::from_str(entry.get().to_str().unwrap()) {
                if let Ok(u1) = self.create_proxied_uri(&u) {
                    entry.insert(format!("{}", u1).parse()?);
                }
            }
        }

        Ok(response)
    }

    async fn proxy(&self, request: Request<Body>, client_ip: &IpAddr, superior: &Option<&SuperiorConf>) -> Result<Response<Body>, AppError> {
        // let forward_uri = "https://www.baidu.com";
        let proxies_request = self.create_proxied_request(client_ip, request, superior)?;

        let response = if let Some(superior_conf) = superior {
            let sock_addr = util::uri::parse_uri_socket_addr(&superior_conf.uri).ok_or_else(|| AppError::RuntimeError("Can't parse superior address".to_string()))?;
            let host = superior_conf.uri.host().ok_or_else(|| AppError::RuntimeError(format!("Can't parse superior host: {}", &superior_conf.uri)))?;
            let tcp_stream = tokio::net::TcpStream::connect(sock_addr).await?;
            if "https" == superior_conf.uri.scheme().unwrap_or(&uri::Scheme::HTTPS) {
                let tls_connector = TlsConnector::from(self.tls_client_config.clone());
                let domain = rustls::ServerName::try_from(host)
                    .map_err(|e| AppError::RuntimeError(format!("{}", e)))?;
                let tls_stream = tls_connector.connect(domain, tcp_stream).await?;
                let (mut request_sender, connection) = hyper::client::conn::handshake(tls_stream).await?;
                connection.await?;
                request_sender.send_request(proxies_request).await?
            } else {
                let (mut request_sender, connection) = hyper::client::conn::handshake(tcp_stream).await?;
                connection.await?;
                request_sender.send_request(proxies_request).await?
            }
        } else {
            Client::new().request(proxies_request).await?
        };

        // let response = if let Some(superior) = superior {
        //     let scheme_str = superior.uri.scheme_str().unwrap_or("http");

        //     // let proxy_uri ="http://<your proxy>:port".parse().unwrap();
        //     let mut proxy = Proxy::new(Intercept::All, superior.uri.clone());

        //     if superior.username.is_some() && superior.password.is_some() {
        //         proxy.set_authorization(Authorization::basic(
        //             superior.username.as_ref().unwrap().as_str(),
        //             superior.password.as_ref().unwrap().as_str()
        //         ));
        //     }

        //     if "https".eq_ignore_ascii_case(scheme_str) {
        //         Client::builder().build(
        //             ProxyConnector::from_proxy(HttpsConnector::new(), proxy).unwrap()
        //         ).request(proxies_request).await?
        //     } else {
        //         Client::builder().build(
        //             ProxyConnector::from_proxy(HttpConnector::new(), proxy).unwrap()
        //         ).request(proxies_request).await?
        //     }
        // } else {
        //     let scheme_str = proxies_request.uri().scheme_str().unwrap_or("http");

        //     if "https".eq_ignore_ascii_case(scheme_str) {
        //         let https = HttpsConnector::new();
        //         Client::builder().build(https).request(proxies_request).await?
        //     } else {
        //         Client::new().request(proxies_request).await?
        //     }
        // };

        self.create_proxied_response(response)
    }

    pub async fn request(&self, request: Request<Body>, client_ip: &IpAddr, superior: &Option<&SuperiorConf>) -> Result<Response<Body>, AppError> {
        match self.proxy(request, client_ip, superior).await {
            Err(e_) =>  {
                let (status, body) = match e_ {
                    AppError::ConfigError(msg) => (StatusCode::BAD_REQUEST, msg),
                    AppError::JoinError(err) => (StatusCode::BAD_REQUEST, format!("Join Error:  {}", err)),
                    AppError::IOError(err) => (StatusCode::BAD_REQUEST, format!("IO Error:  {}", err)),
                    AppError::UriError(msg) => (StatusCode::BAD_REQUEST, msg),
                    AppError::InvalidUri(err) =>  (StatusCode::BAD_REQUEST, format!("{}", err)),
                    AppError::HttpError(err) =>  (StatusCode::BAD_REQUEST, format!("{}", err)),
                    AppError::HyperError(err) =>   (StatusCode::BAD_REQUEST, format!("{}", err)),
                    AppError::RuntimeError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    AppError::ForwardHeaderError => (StatusCode::BAD_REQUEST, format!("Unknown Error")),
                    // UnknownPath(msg) => (StatusCode::NOT_FOUND, msg),
                    // ClientError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    // AuthMissingHeader(msg) => (StatusCode::UNAUTHORIZED, msg),
                    // AuthCannotParseHeader(msg) => (StatusCode::UNAUTHORIZED, msg),
                    // AuthTokenError(err) => (StatusCode::UNAUTHORIZED, format!("{:?}", err)),
                    // AuthCannotCreateHeader(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                };
        
                Response::builder()
                    .status(status)
                    .body(Body::from(body))
                    .map_err(|e| AppError::HttpError(e))
            },

            Ok(r) => Ok(r),
        }
    }
}
