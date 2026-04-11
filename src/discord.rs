use std::env;

use ::discord::Discord;
use discord::model::UserId;

pub struct DiscordPFP {
    client: Discord,
}

impl DiscordPFP {
    pub fn build() -> Self {
        let client =
            Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN is empty."))
                .expect("Unable to login.");

        Self {
            client,
        }
    }

    pub fn get_avatar(&self, id: u64) -> Result<String, String> {
        let avatar = self
            .client
            .get_user(UserId(id))
            .map_err(|err| err.to_string())
            .map(|user| user.avatar)?
            .ok_or("User has no avatar.".to_string())?;
        Ok(avatar)
    }

    pub async fn fetch_pfp(id: u64, avatar: &str) -> Result<Vec<u8>, String> {
        let url = format!("https://cdn.discordapp.com/avatars/{id}/{avatar}.webp");
        println!("Getting pfp from: {url}");
        let pfp = reqwest::get(url)
            .await
            .map_err(|err| err.to_string())?
            .bytes()
            .await
            .map_err(|err| err.to_string())?;
        Ok(pfp.to_vec())
    }
}
