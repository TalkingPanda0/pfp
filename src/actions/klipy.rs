use std::{env, sync::Arc};

use anyhow::anyhow;
use linkme::distributed_slice;
use serde::Deserialize;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::{Frame, Frames},
};

#[derive(Clone)]
pub struct Klipy {
    slug: String,
}

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    let Some(slug) = input
        .strip_prefix("klipy(")
        .and_then(|s| s.strip_suffix(")"))
        .map(|s| s.to_string())
    else {
        return false;
    };

    actions.push(Box::new(Klipy { slug }));

    true
}

impl Action for Klipy {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let api_key = env::var("KLIPY_TOKEN")?;
            let url = format!(
                "https://api.klipy.com/api/v1/{}/gifs/items?slugs={}",
                api_key, self.slug
            );
            let result = reqwest::get(url).await?.error_for_status()?.text().await?;
            let result: ItemResult = serde_json::from_str(&result)?;

            let url = &result
                .data
                .data
                .first()
                .ok_or(anyhow!("No images from klipy"))?
                .file
                .hd
                .webp
                .url;
            let image_bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;

            images.extend(Vec::from_webp_animation(&image_bytes, action)?);

            Ok(())
        })
    }
}

#[derive(Deserialize)]
struct ItemResult {
    pub data: ItemData,
}

#[derive(Deserialize)]
struct ItemData {
    pub data: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    pub file: File,
}

#[derive(Deserialize)]
struct File {
    pub hd: ImageQuality,
}

#[derive(Deserialize)]
struct ImageQuality {
    pub webp: ImageType,
}

#[derive(Deserialize)]
struct ImageType {
    pub url: String,
}
