use serde_derive::{Serialize, Deserialize};
pub mod superior_conf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnalConf {
    pub superiors: Vec<superior_conf::SuperiorConf>
}
