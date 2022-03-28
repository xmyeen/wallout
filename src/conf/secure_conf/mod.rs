use serde_derive::{Serialize, Deserialize};
use crate::conf::subnet_conf::{SubnetConf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecureConf {
    pub whitelists: Vec<SubnetConf>,
}