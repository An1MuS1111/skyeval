use serde::Deserialize;
use std::marker::PhantomData;

pub struct NoApiKey;
pub struct HasApiKey;

pub struct ClientBuilder<'a, State> {
    api_key: Option<&'a str>,
    request_client: reqwest::Client,
    _state: PhantomData<State>,
}

impl<'a> Default for ClientBuilder<'a, NoApiKey> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ClientBuilder<'a, NoApiKey> {
    pub fn new() -> Self {
        Self {
            api_key: None,
            request_client: reqwest::Client::new(),
            _state: PhantomData,
        }
    }

    pub fn api_key(self, key: &'a str) -> ClientBuilder<'a, HasApiKey> {
        ClientBuilder {
            api_key: Some(key),
            request_client: self.request_client,
            _state: PhantomData,
        }
    }
}

impl<'a, State> ClientBuilder<'a, State> {
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.request_client = client;
        self
    }
}

impl<'a> ClientBuilder<'a, HasApiKey> {
    pub fn build(self) -> Client {
        Client {
            api_key: self.api_key.unwrap().to_owned(),
            http_client: self.request_client,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Player {
    pub puuid: String,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
}

pub struct Client {
    pub(crate) api_key: String,
    pub(crate) http_client: reqwest::Client,
}

impl Client {
    pub async fn get_player_puuid(
        &self,
        region: &str,
        name: &str,
        tag: &str,
    ) -> Result<Player, anyhow::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
            region, name, tag
        );

        let response = self
            .http_client
            .get(&url)
            .header("X-Riot-Token", self.api_key.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            return Err(anyhow::anyhow!("API Error {}: {}", status, error_body));
        }

        let account_data = response.json::<Player>().await?;
        Ok(account_data)
    }
}
