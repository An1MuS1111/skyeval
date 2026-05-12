use serde::Deserialize;
use std::{marker::PhantomData, sync::Arc};

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
            inner: Arc::new(ClientInner {
                api_key: self.api_key.unwrap().to_owned(),
                http_client: self.request_client,
            }),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

pub struct ClientInner {
    api_key: String,
    http_client: reqwest::Client,
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
            .inner
            .http_client
            .get(&url)
            .header("X-Riot-Token", self.inner.api_key.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            return Err(anyhow::anyhow!("API Error {}: {}", status, error_body));
        }

        let data = response.json::<PlayerDto>().await?;

        Ok(Player {
            client: self.clone(),
            puuid: data.puuid,
            game_name: data.game_name,
            tag_line: data.tag_line,
            region: region.to_owned(),
        })
    }
}

pub struct Player {
    client: Client,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    region: String,
}

impl Player {
    pub async fn get_matchlist(&self) -> Result<MatchlistDto, anyhow::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/val/match/v1/matchlists/by-puuid/{}",
            self.region, self.puuid
        );

        let response = self
            .client
            .inner
            .http_client
            .get(&url)
            .header("X-Riot-Token", &self.client.inner.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            return Err(anyhow::anyhow!("API Error {}: {}", status, error_body));
        }

        let matchlist = response.json::<MatchlistDto>().await?;
        Ok(matchlist)
    }
}

#[derive(Deserialize)]
struct PlayerDto {
    puuid: String,
    #[serde(rename = "gameName")]
    game_name: String,
    #[serde(rename = "tagLine")]
    tag_line: String,
}

#[derive(Deserialize, Debug)]
pub struct MatchlistDto {
    pub puuid: String,
    pub history: Vec<MatchlistEntryDto>,
}

#[derive(Deserialize, Debug)]
pub struct MatchlistEntryDto {
    #[serde(rename = "matchId")]
    pub match_id: String,
    #[serde(rename = "gameStartTimeMillis")]
    pub game_start_time_millis: i64,
    #[serde(rename = "queueId")]
    pub queue_id: String,
}
