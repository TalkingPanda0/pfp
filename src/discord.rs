use std::env;

use ::discord::Discord;
use anyhow::{Result, anyhow};
use discord::model::UserId;

pub struct DiscordPFP {
    client: Discord,
}

impl DiscordPFP {
    pub fn build() -> Self {
        let client =
            Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is empty."))
                .expect("Unable to login.");

        Self { client }
    }

    pub fn get_avatar(&self, id: u64) -> Result<String> {
        let avatar = self
            .client
            .get_user(UserId(id))
            .map(|user| user.avatar)?
            .ok_or(anyhow!("User has no avatar."))?;
        Ok(avatar)
    }

    pub async fn fetch_pfp(id: u64, avatar: &str) -> Result<Vec<u8>> {
        let url = format!("https://cdn.discordapp.com/avatars/{id}/{avatar}.webp");
        println!("Getting pfp from: {url}");
        let pfp = reqwest::get(url).await?.error_for_status()?.bytes().await?;
        Ok(pfp.to_vec())
    }
}
