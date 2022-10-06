use std::{io, fs};
use rustls;
use rustls_pemfile;
use rcgen::{PKCS_RSA_SHA512, Certificate as RcgenCertificate, CertificateParams as RcgenCertificateParams, SanType as RsgenSanType};
use crate::error;

pub fn load_certs(certfile: &str) -> Result<Vec<rustls::Certificate>, error::AppError> {
    let mut reader = io::BufReader::new(
        fs::File::open(certfile).or_else(|e| Err(error::AppError::ConfigError(format!("{}. certfile: {}", e, certfile))))?
    );

    // let certfile = fs::File::open(filename).expect("cannot open certificate file");
    // let mut reader = BufReader::new(certfile);
    Ok(rustls_pemfile::certs(&mut reader).unwrap().iter().map(|v| rustls::Certificate(v.clone())).collect())
}

pub fn load_keys(keyfile: &str) -> Result<Vec<rustls::PrivateKey>, error::AppError> {
    let mut reader = io::BufReader::new(
        fs::File::open(keyfile).or_else(|e| Err(error::AppError::ConfigError(format!("{}. keyfile: {}", e, keyfile))))?
    );

    Ok(rustls_pemfile::pkcs8_private_keys(&mut reader).unwrap().iter().map(|v| rustls::PrivateKey(v.clone())).collect())
}

pub fn generate_simple_self_signed_cert_and_key(hostnames: &str, certfile_str: &str, keyfile_str: &str) -> Result<(), error::AppError> {
    let subject_alt_names = hostnames.split(",").into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
    // let key_and_cert = generate_simple_self_signed(subject_alt_names).unwrap();
    let mut cert_param = RcgenCertificateParams::default();
    // cert_param.alg = &PKCS_RSA_SHA512;
    cert_param.subject_alt_names = subject_alt_names.iter().map(|s| RsgenSanType::DnsName(s.clone())).collect();

    let key_and_cert = RcgenCertificate::from_params(cert_param).expect("Generate certs failed");

    for path_str in [certfile_str, keyfile_str] {
        if let Some(parent_dir_path) = std::path::Path::new(path_str).parent() {
            if !parent_dir_path.exists() {
                std::fs::create_dir_all(parent_dir_path).map_err(|e| error::AppError::ConfigError(format!("{}", e)))?;
            }
        }
    }

    let cert_content = key_and_cert.serialize_pem().map_err(|e| error::AppError::ConfigError(format!("{}", e)))?;
    std::fs::write(certfile_str, cert_content)?;

    let key_content = key_and_cert.serialize_private_key_pem();
    std::fs::write(keyfile_str, key_content)?;

    Ok(())
}