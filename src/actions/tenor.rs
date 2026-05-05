use std::{any::Any, sync::Arc};

use anyhow::{anyhow, bail};
use reqwest::StatusCode;

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

const PATTERN: &str = "<meta class=\"dynamic\" name=\"twitter:image\" content=\"";

#[derive(Clone)]
pub struct Tenor(String);

impl Tenor {
    pub fn new(url: String) -> Self {
        Self(url)
    }
}

impl Action for Tenor {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        if let Some(id) = input
            .strip_prefix("tenor(")
            .and_then(|s| s.strip_suffix(")"))
            .map(|s| s.trim_prefix("https://").trim_prefix("tenor.com/view"))
        {
            actions.push(Box::new(Self(id.to_string())));
            true
        } else {
            false
        }
    }

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
            let result = reqwest::get(url)
                .await?
                .error_for_status();
            let response = match result {
                Ok(res) => res,
                Err(err) if err.status() == Some(StatusCode::NOT_FOUND) => {
                    let url = content.replace("AAC/", "AA1/");
                    reqwest::get(url)
                        .await?
                        .error_for_status()?
                }
                Err(err) => bail!("Failed to get webp from tenor. {err}"),
            };
            let bytes = response.bytes().await?;
            images.extend(Vec::from_webp_animation(&bytes, action)?);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
