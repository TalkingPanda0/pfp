use std::sync::Arc;

use linkme::distributed_slice;
use reqwest::Client;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::{Frame, Frames},
};

#[derive(Clone)]
pub struct URLAction(String);

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    if let Some(url) = input.strip_prefix("url(").and_then(|s| s.strip_suffix(")")) {
        actions.push(Box::new(URLAction(url.to_string())));
        true
    } else {
        false
    }
}

impl Action for URLAction {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let client = Client::builder().user_agent("PFP/1.0").build()?;
            let bytes = client
                .get(&self.0)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map(|r| r.bytes())?
                .await?;

            let frames = if bytes.starts_with(b"RIFF") {
                Vec::from_webp_animation(&bytes, action)
            } else {
                Vec::from_unknown_data(&bytes, action)
            }?;

            images.extend(frames);
            Ok(())
        })
    }
}

