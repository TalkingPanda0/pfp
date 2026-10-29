use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::action::{ActionResult, PARSERS, Parser};
use crate::cache::Cache;
use crate::parser::{ParsableAction};
use crate::{action::Action, frames::Frame};

use std::env;
use std::sync::LazyLock;

use ::discord::Discord;
use anyhow::{Result, anyhow};
use discord::model::UserId;

pub struct DiscordManager {
    client: Discord,
}

impl DiscordManager {
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

static DISCORD: LazyLock<DiscordManager> = LazyLock::new(|| {
    DiscordManager::build()
});

#[derive(Clone, Debug, Parsable, ParsableAction)]
#[name(("<@",">"))]
pub struct DiscordPFPAction(u64);

#[distributed_slice(PARSERS)]
static PARSER: Parser = DiscordPFPAction::parse_action;

impl Action for DiscordPFPAction {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let avatar = DISCORD.get_avatar(self.0)?;

            let pfp_bytes = match Cache::get(&avatar).await {
                Some(pfp) => pfp,
                None => {
                    let pfp = DiscordManager::fetch_pfp(self.0, &avatar).await?;
                    let _ = Cache::save(&pfp, &avatar)
                        .await
                        .inspect_err(|err| eprintln!("Failed saving cache: {:?}", err));
                    pfp
                }
            };

            images.push(Frame::from_webp(&pfp_bytes, 1000, action)?);
            Ok(())
        })
    }
}
