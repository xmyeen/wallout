use std::{io, fs};
use rustls;
use rustls_pemfile;
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