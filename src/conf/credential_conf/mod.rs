use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialConf {
    // pub routes: Vec<(regex::Regex, String)>
    // pub regex_set: regex::RegexSet,
    pub user: String,
    pub passwd: String,
}
