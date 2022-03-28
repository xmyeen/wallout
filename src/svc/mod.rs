use std::{net::IpAddr, sync::Arc};
use rustls::ClientConfig as TlsClientConfig;
use hyper::{Request, Response, Body};

use crate::error;
use crate::conf::tunnal_conf::superior_conf::SuperiorConf;

pub mod rerverse;
pub use rerverse::RerverseSvc;

pub mod tunnal;
pub use tunnal::TunnalSvc;

pub struct AppSvc {
    reverse_svc: RerverseSvc,
    tunnal_svc: TunnalSvc,
}

impl AppSvc {
    pub fn new(server_external_address: &str, client_config: Arc<TlsClientConfig>) -> Self {
        Self {
            reverse_svc: RerverseSvc::new(client_config.clone(), server_external_address),
            tunnal_svc: TunnalSvc::new(client_config.clone()),
        }
    }

    pub async fn tunnal_to_destination(&self, req: Request<Body>) -> Result<Response<Body>, error::AppError> {
        self.tunnal_svc.tunnal_to_destination(req).await
    }

    pub async fn tunnal_to_superior(&self, req: Request<Body>, superior: &SuperiorConf) -> Result<Response<Body>, error::AppError> {
        self.tunnal_svc.tunnal_to_superior(req, &superior).await
    }

    pub async fn reverse(&self, req: Request<Body>, client_ip:&IpAddr, superior: &Option<&SuperiorConf>) -> Result<Response<Body>, error::AppError> {
        self.reverse_svc.request(req, client_ip, superior).await
    }
}