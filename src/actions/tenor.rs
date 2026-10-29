use anyhow::{anyhow, bail};
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};
use reqwest::StatusCode;

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::ParsableAction,
};

const PATTERN: &str = "<meta class=\"dynamic\" name=\"twitter:image\" content=\"";

#[derive(Clone,Parsable,ParsableAction)]
pub struct Tenor(String);

impl Tenor {
    pub fn new(url: String) -> Self {
        Self(url)
    }
}

#[distributed_slice(PARSERS)]
static PARSER: Parser = Tenor::parse_action;

impl Action for Tenor {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let mut content = reqwest::get(format!("https://tenor.com/view/{}", self.0))
                .await?
                .error_for_status()?
                .text()
                .await?;

            let start = content
                .find(PATTERN)
                .ok_or(anyhow!("Failed to find image."))?;
            let content = content.split_off(start + PATTERN.len());
            let length = content
                .find("\">")
                .ok_or(anyhow!("Failed to find image."))?;
            let content = &content[0..length];

            // Request webp instead of gif
            let url = content.replace("AAC/", "AAm/");
            let result = reqwest::get(url).await?.error_for_status();
            let response = match result {
                Ok(res) => res,
                Err(err) if err.status() == Some(StatusCode::NOT_FOUND) => {
                    let url = content.replace("AAC/", "AA1/");
                    reqwest::get(url).await?.error_for_status()?
                }
                Err(err) => bail!("Failed to get webp from tenor. {err}"),
            };
            let bytes = response.bytes().await?;

            images.extend(Vec::from_webp_animation(&bytes, action)?);
            Ok(())
        })
    }
}
