use std::{any::Any, sync::Arc};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

#[derive(Clone)]
pub struct URLAction(String);

impl Action for URLAction {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        if let Some(url) = input.strip_prefix("url(").and_then(|s| s.strip_suffix(")")) {
            actions.push(Box::new(Self(url.to_string())));
            true
        } else {
            false
        }
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let bytes = reqwest::get(&self.0)
                .await
                .and_then(|r| r.error_for_status())
                .map(|r| r.bytes())?
                .await?;

            let frames = Vec::from_webp_animation(&bytes, action)?;

            images.extend(frames);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
