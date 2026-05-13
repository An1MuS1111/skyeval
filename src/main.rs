use dotenvy::dotenv;
use skyeval::{AccountCluster, Builder};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let reqwest_client = reqwest::Client::new();

    let api_key = env::var("API_KEY")?;

    let client = Builder::new(api_key)
        .set_http_client(reqwest_client)
        .build()?;
    let player = client
        .get_player_puuid(AccountCluster::Asia, "loverboy6969", "sick7")
        .await?;

    let matchlist = player.get_matchlist().await?;
    for entry in matchlist.history {
        println!("{}", entry.match_id);
    }
    Ok(())
}
