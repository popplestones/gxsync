use crate::auth::get_access_token;
use crate::error::GxsyncError;
use reqwest::Client;

pub struct GraphClient {
    pub client: Client,
    pub token: String,
}

impl GraphClient {
    pub async fn new(auth_profile: &str) -> Result<Self, GxsyncError> {
        let token = get_access_token(auth_profile).await?;
        Ok(Self {
            client: Client::new(),
            token,
        })
    }

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .bearer_auth(&self.token)
            .header("Accept", "application/json")
    }

    pub fn get_raw(&self, url: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .bearer_auth(&self.token)
            .header("Accept", "message/rfc822")
    }
}
