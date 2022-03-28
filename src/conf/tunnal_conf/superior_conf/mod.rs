use serde_derive::{Serialize, Deserialize};
use serde_with::{self, DisplayFromStr, serde_as};
use http;

// mod uri_serde {
//     use http::uri::{Uri, InvalidUri};
//     use serde::{de::Error as _, Deserialize, Deserializer, Serializer};

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Uri, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//         let uri = s.parse::<Uri>().map_err(|err: InvalidUri| D::Error::custom(err.to_string()))?;

//         Ok(uri)
//     }

//     pub fn serialize<S>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(&uri.to_string())
//     }
// }

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuperiorConf {
    // #[serde(with = "uri_serde")]
    pub id: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default)]
    pub uri: http::uri::Uri,
    pub username: Option<String>,
    pub password: Option<String>,
}