use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyPassLocationConf {
    // pub routes: Vec<(regex::Regex, String)>
    // pub regex_set: regex::RegexSet,
    pub matcher: String,
    pub proxy_pass: String,
    pub replace_re: String,
    pub replace_to: String,
}