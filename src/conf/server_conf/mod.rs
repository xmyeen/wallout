use std::{net};
use serde_derive::{Serialize, Deserialize};
use rustls::{self,  server::ServerConfig as TlsServerConfig};

use crate::error::{AppError};
use crate::util::cert;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConf  {
    // pub routes: Vec<(regex::Regex, String)>
    // pub regex_set: regex::RegexSet,
    pub id: String,
    pub host: String,
    pub port: u32,
    pub realm: String,
    pub on_https: bool,
    pub certfile: String,
    pub keyfile: String,
    pub superior_id: Option<String>,
    pub external_host_name: Option<String>,
}


impl ServerConf {
    pub fn get_address(&self) -> Result<net::SocketAddr, AppError> {
        if !self.host.is_empty() && 0 < self.port && 65536 > self.port {
            match format!("{}:{}", self.host, self.port).parse() {
                Ok(r) => Ok(r),
                Err(e) => Err(AppError::ConfigError(format!("{:?}", e))),
            }
        } else {
            Err(AppError::ConfigError("Invalid server configuration".to_string()))
        }
    }

    pub fn get_external_address(&self) -> String {
        format!("{}:{}", self.external_host_name.as_ref().unwrap_or(&String::from("localhost")), self.port)
    }

    pub fn load_tls_server_configuration(&self) -> Result<TlsServerConfig, AppError> {
        // let cert = {
        //     let file = File::open(cert_filename).unwrap_or_else(|e| {
        //         panic!(
        //             "Can't open tls certificate file: '{}'. {}.",
        //             cert_filename, e
        //         )
        //     });
        //     let mut rdr = BufReader::new(file);
        //     rustls::internal::pemfile::certs(&mut rdr)
        //         .unwrap_or_else(|()| panic!("Can't parse tls certificate file: {}", cert_filename))
        // };

        // let key = {
        //     let mut pkcs8 = {
        //         let file = File::open(&key_filename)
        //             .unwrap_or_else(|e| panic!("Can't open tls key file: '{}'. {}.", key_filename, e));
        //         let mut rdr = BufReader::new(file);
        //         rustls::internal::pemfile::pkcs8_private_keys(&mut rdr)
        //             .unwrap_or_else(|()| panic!("Can't parse pkcs8 tls key file: {}", key_filename))
        //     };

        //     // No pkcs8 key, try RSA PRIVATE PEM
        //     if !pkcs8.is_empty() {
        //         pkcs8.remove(0)
        //     } else {
        //         let file = File::open(key_filename)
        //             .unwrap_or_else(|e| panic!("Can't open tls key file: '{}'. {}.", key_filename, e));
        //         let mut rdr = BufReader::new(file);
        //         let mut rsa =
        //             rustls::internal::pemfile::rsa_private_keys(&mut rdr).unwrap_or_else(|()| {
        //                 panic!("Can't parse rsa_private tls key file: {}", key_filename)
        //             });

        //         if !rsa.is_empty() {
        //             rsa.remove(0)
        //         } else {
        //             panic!(
        //                 "TLS key path contains no private key. Check '{}' and '{}'.",
        //                 cert_filename, key_filename
        //             );
        //         }
        //     }
        // };

        // let mut tls = ServerConfig::new(rustls::NoClientAuth::new());
        // tls.set_single_cert(cert, key).unwrap_or_else(|e| {
        //     panic!(
        //         "Can't set_single_cert: '{}',  '{}', {}.",
        //         cert_filename, key_filename, e
        //     )
        // });
        // tls.set_protocols(&["h2".into(), "http/1.1".into()]);
        // tls

        // Load public certificate.
        let certs = cert::load_certs(self.certfile.as_str())?;
        if certs.is_empty() {
            return Err(AppError::ConfigError(format!("No certificates")))?
        }

        // Load private key.
        let mut privkeys = cert::load_keys(self.keyfile.as_str())?;
        if privkeys.is_empty() {
            return Err(AppError::ConfigError(format!("No private keys")))?
        }

        // Do not use client certificate authentication.
        let client_auth = rustls::server::NoClientAuth::new();

        let mut cfg = rustls::ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_kx_groups(&rustls::ALL_KX_GROUPS)
            //chrome不支持TLSv1.3
            .with_protocol_versions(&rustls::ALL_VERSIONS)
            .map_err(|e| AppError::ConfigError(format!("{}", e)))?
            .with_client_cert_verifier(client_auth)
            // Select a certificate to use.
            //OCSP（Online Certificate Status Protocol ）
            //SCT (Signed Certificate Timestamp)
            // .with_single_cert_with_ocsp_and_sct(
            //     certs.clone(),
            //     privkeys.remove(0).clone(),
            //     opts.server_ocsp_response.clone(),
            //     opts.server_sct_list.clone()
            // );
            .with_single_cert(certs.clone(), privkeys.remove(0).clone())
            .map_err(|e| AppError::ConfigError(format!("{}", e)))?;

        // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order
        cfg.alpn_protocols = ["h2", "http/1.1"].iter().map(|protocol| protocol.as_bytes().to_vec()).collect::<Vec<_>>();

        Ok(cfg)
    }
}