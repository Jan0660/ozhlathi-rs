use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub url: String,
    pub twitch_logins: Vec<String>,
    pub password: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub twitch_client_id: String,
    pub twitch_app_access_token: String,
}
