use serde_derive::{Serialize, Deserialize};
use rustls::{self, client::ClientConfig as TlsClientConfig, OwnedTrustAnchor};
use rustls_pemfile;
use webpki_roots;
use webpki::{TrustAnchor};

use crate::error::{AppError};
use crate::util::cert;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientConf  {
    // pub routes: Vec<(regex::Regex, String)>
    // pub regex_set: regex::RegexSet,
    pub certfile: String,
    pub keyfile: String,
    pub trusted_cerfiles: Vec<String>,
}

impl ClientConf {
    pub fn load_tls_client_configuration(&self) -> Result<TlsClientConfig, AppError> {
        // Load public certificate.
        // let certs: Option<Vec<rustls::Certificate>> = self.servers.iter().map(|conf| cert::load_certs(conf.certfile.as_str()).ok() ).flatten().collect();
        let certs = cert::load_certs(self.certfile.as_str())?;
        if  certs.is_empty() {
            return Err(AppError::ConfigError(format!("No certificates")))?
        }

        // Load private key.
        // let mut privkeys: Option<Vec<rustls::PrivateKey> = self.servers.iter().map(|conf| cert::load_keys(conf.keyfile.as_str()).ok() ).flatten().collect::<Option<Vec<_>>>();
        let privkeys = cert::load_keys(self.keyfile.as_str())?;
        if privkeys.is_empty() {
            return Err(AppError::ConfigError(format!("No private keys")))?
        }

        let mut root_cert_store = rustls::RootCertStore::empty();

        for trusted_cerfile in &self.trusted_cerfiles {
            let mut pem = std::io::BufReader::new(std::fs::File::open(trusted_cerfile)?);
            let trust_anchors = rustls_pemfile::certs(&mut pem)?
                .into_iter()
                .map(|cert| {
                    let ta = TrustAnchor::try_from_cert_der(cert.as_slice()).unwrap();
                    OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                });
            root_cert_store.add_server_trust_anchors(trust_anchors);
        }

        root_cert_store.add_server_trust_anchors(
            webpki_roots::TLS_SERVER_ROOTS.0.iter()
                .map(|ta| {
                    OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
        );

        // let cfg = rustls::ClientConfig::builder()
        //     .with_safe_default_cipher_suites()
        //     .with_safe_default_kx_groups()
        //     // .with_safe_default_protocol_versions()
        //     // .with_kx_groups(&rustls::ALL_KX_GROUPS)
        //     //chrome不支持TLSv1.3
        //     .with_protocol_versions(&[&rustls::version::TLS12])
        //     .map_err(|e| AppError::ConfigError(format!("{}", e)))?
        //     .with_root_certificates(root_cert_store)
        //     // .with_no_client_auth()
        //     .with_single_cert(certs.clone(), privkeys.remove(0).clone())
        //     .map_err(|e| AppError::ConfigError(format!("{}", e)))?;
        let cfg = rustls::ClientConfig::builder()
            .with_safe_defaults()
            // .with_protocol_versions()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        // if !opts.cert_file.is_empty() && opts.use_signing_scheme > 0 {
        //     let scheme = lookup_scheme(opts.use_signing_scheme);
        //     cfg.client_auth_cert_resolver = Arc::new(FixedSignatureSchemeClientCertResolver {
        //         resolver: cfg.client_auth_cert_resolver.clone(),
        //         scheme,
        //     });
        // }

        // let persist = ClientCacheWithoutKxHints::new();
        // cfg.session_storage = persist;
        // cfg.enable_sni = opts.use_sni;
        // cfg.max_fragment_size = opts.max_fragment;
    
        // if !opts.protocols.is_empty() {
        //     cfg.alpn_protocols = opts
        //         .protocols
        //         .iter()
        //         .map(|proto| proto.as_bytes().to_vec())
        //         .collect();
        // }
    
        Ok(cfg)
    }
}