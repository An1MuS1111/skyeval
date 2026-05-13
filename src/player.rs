use crate::AccountCluster;
use crate::client::Client;
use serde::Deserialize;

pub struct Player {
    pub(crate) client: Client,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub region: AccountCluster,
}

impl Player {
    pub async fn get_matchlist(&self) -> Result<MatchlistDto, anyhow::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/val/match/v1/matchlists/by-puuid/{}",
            self.region.as_str(),
            self.puuid
        );

        let response = self
            .client
            .inner
            .http_client
            .get(&url)
            .header("X-Riot-Token", &self.client.inner.cfg.api_key)
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
pub(crate) struct PlayerDto {
    pub(crate) puuid: String,
    #[serde(rename = "gameName")]
    pub(crate) game_name: String,
    #[serde(rename = "tagLine")]
    pub(crate) tag_line: String,
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
