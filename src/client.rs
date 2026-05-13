use crate::player::{Player, PlayerDto};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Config {
    pub(crate) api_key: String,
}

impl Config {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub pool_max_idle_per_host: usize,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            pool_max_idle_per_host: 10,
        }
    }
}

pub struct Builder {
    cfg: Config,
    http_cfg: Option<HttpConfig>,
    http_client: Option<reqwest::Client>,
}

impl Builder {
    pub fn from_config(cfg: Config) -> Self {
        Self {
            cfg,
            http_cfg: None,
            http_client: None,
        }
    }

    pub fn new(api_key: impl Into<String>) -> Self {
        Self::from_config(Config::new(api_key))
    }

    pub fn with_config<F>(&mut self, func: F) -> &mut Self
    where
        F: FnOnce(&mut Config),
    {
        func(&mut self.cfg);
        self
    }

    pub fn set_http_client(&mut self, http_client: reqwest::Client) -> &mut Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn build(&mut self) -> Result<Client, anyhow::Error> {
        let http_client = match self.http_client.take() {
            Some(client) => client,

            None => {
                let http_cfg = self.http_cfg.clone().unwrap_or_default();
                reqwest::Client::builder()
                    .timeout(http_cfg.timeout)
                    .connect_timeout(http_cfg.connect_timeout)
                    .pool_max_idle_per_host(http_cfg.pool_max_idle_per_host)
                    .build()?
            }
        };

        Ok(Client {
            inner: Arc::new(ClientInner {
                cfg: self.cfg.clone(),
                http_client,
            }),
        })
    }
}

pub enum AccountCluster {
    Americas,
    Europe,
    Asia,
}

impl AccountCluster {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountCluster::Americas => "americas",
            AccountCluster::Europe => "europe",
            AccountCluster::Asia => "asia",
        }
    }
}

#[derive(Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

pub struct ClientInner {
    pub(crate) cfg: Config,
    pub(crate) http_client: reqwest::Client,
}

impl Client {
    pub fn builder(api_key: impl Into<String>) -> Builder {
        Builder::new(api_key)
    }

    pub async fn get_player_puuid(
        &self,
        region: AccountCluster,
        name: &str,
        tag: &str,
    ) -> Result<Player, anyhow::Error> {
        let url = format!(
            "https://{}.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
            region.as_str(),
            name,
            tag
        );

        let response = self
            .inner
            .http_client
            .get(&url)
            .header("X-Riot-Token", self.inner.cfg.api_key.clone())
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
            region,
        })
    }
}
