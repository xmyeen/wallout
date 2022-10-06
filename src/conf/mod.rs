
// use std::env;
// #![feature(once_cell)]

// use std::lazy::SyncLazy;
// use std::sync::RwLock;
use once_cell::sync::OnceCell;
use config::{Config, File as ConfigFile/*, Environment*/};
use serde_derive::{Serialize, Deserialize};

extern crate log;
extern crate log4rs;
// use log4rs;
use crate::error::{AppError};

pub mod subnet_conf;
pub mod credential_conf;
pub mod proxy_pass_location_conf;
pub mod secure_conf;
pub mod client_conf;
pub mod server_conf;
pub mod tunnal_conf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConf {
    // pub routes: Vec<(regex::Regex, String)>
    // pub regex_set: regex::RegexSet,
    pub hostnets: Vec<subnet_conf::SubnetConf>,
    pub log_cfg_file: String,
    pub client: client_conf::ClientConf,
    pub servers: Vec<server_conf::ServerConf>,
    pub secure: secure_conf::SecureConf,
    pub tunnal: tunnal_conf::TunnalConf,
    pub credentials: Vec<credential_conf::CredentialConf>,
    pub proxy_pass_locations: Vec<proxy_pass_location_conf::ProxyPassLocationConf>
}

pub struct ConfigMgr {
    app_conf: OnceCell<AppConf>
}

impl ConfigMgr {
    fn new() -> Self{
        Self {
            app_conf: OnceCell::new(),
        }
    }

    /// Start the logger (log4rs).
    fn init_logger(&self)  -> Result<(), AppError> {
        if let Err(e_) = log4rs::init_file(self.value().log_cfg_file.as_str(), Default::default()) {
            println!("[ERROR] Failed to initialize logger - {}", e_);
        } else {
            info!("Initialize logger");
        }

        Ok(())
    }

    pub fn value(&self) -> &AppConf {
        self.app_conf.get().expect("No configuration.")
    }

    pub fn init(&self, app_cfg_file: &str) -> Result<(), AppError> {
        let builder = Config::builder();
        let config = builder
            .set_default("log_cfg_file", "config/log.yaml").unwrap()
            .set_default("hostnets[0].exp", "127.0.0.1").unwrap()
            .set_default("hostnets[0].typ", "ip").unwrap()
            .set_default("client.subject_alt_name", "noname").unwrap()
            // .set_default("client.certfile", "data/client.crt").unwrap()
            // .set_default("client.keyfile", "data/client.key").unwrap()
            // .set_default("secure.whitelists", Vec::<subnet_conf::SubnetConf>::new()).unwrap()
            // .set_default("tunnal.superior,url", "http://127.0.0.1").unwrap()
            .set_default("servers[0].id", "local").unwrap()
            .set_default("servers[0].host", "127.0.0.1").unwrap()
            .set_default("servers[0].port", 8999).unwrap()
            .set_default("servers[0].realm", "wallout").unwrap()
            .set_default("servers[0].on_https", false).unwrap()
            // .set_default("servers[0].certfile", "data/server.crt").unwrap()
            // .set_default("servers[0].keyfile", "data/server.key").unwrap()
            .add_source(ConfigFile::with_name(app_cfg_file))
            .build()
            .expect("Load configuration failed");

            // Start off by merging in the "default" configuration file
            // s.merge(File::with_name("config/default"))?;

            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            // let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
            // s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

            // Add in a local configuration file
            // This file shouldn't be checked in to git
            // s.merge(File::with_name("config/local").required(false))?;

            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            // s.merge(Environment::with_prefix("app"))?;

            // You may also programmatically change settings
            // s.set("database.url", "postgres://")?;

            // Now that we're done, let's access our configuration
            // println!("debug: {:?}", s.get_bool("debug"));
            // println!("database: {:?}", s.get::<String>("database.url"));

        if let Err(_) = self.app_conf.set(config.try_deserialize().map_err(|e| AppError::RuntimeError(format!("Deserialize configuration failed: {}", e)))?) {
            error!("Can't set configuration");
        }

        self.init_logger()
    }

    // pub fn create_instance(app_cfg_file: &str) -> &'static std::sync::Arc<ConfigMgr> {
    //     INSTANCE.get_or_init(|| {
    //         let mut o = ConfigMgr::new();
    //         o.init(app_cfg_file).expect("Initialize failed");
    //         std::sync::Arc::new(o)
    //     })
    // }

    pub fn get_instance() -> &'static std::sync::Arc<ConfigMgr> {
        // static mut INSTANCE: Option<std::sync::Arc<std::sync::RwLock<ConfigMgr>>> = None;

        // unsafe {
        //     INSTANCE.get_or_insert_with(|| {
        //         std::sync::Arc::new(std::sync::RwLock::new(ConfigMgr::new()))
        //     });

        //     INSTANCE.as_ref().unwrap()
        // }

        static INSTANCE: OnceCell<std::sync::Arc<ConfigMgr>> = OnceCell::new();
        // INSTANCE.get().expect("This configuration manager is not initialized")
        INSTANCE.get_or_init(|| std::sync::Arc::new(ConfigMgr::new()))
    }
}



// lazy_static! {
//     static ref CONFIG_MGR_INSTANCE: std::sync::Arc<ConfigMgr> = ConfigMgr::new()
// }

// static APP_CONF: SyncLazy<AppConf> = SyncLazy::new(|| {
//     AppConf::new().expect("Initialize application configration failed")
// });