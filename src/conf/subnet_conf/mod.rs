use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubnetConf {
    pub typ: String,
    pub exp: String,
}