use std::sync::Arc;

use anyhow::anyhow;
use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::Frame,
};

#[derive(Clone, Copy)]
pub struct Delay(i32);

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    if let Some(delay) = input
        .strip_prefix("delay(")
        .and_then(|s| s.strip_suffix(")"))
        .and_then(|s| s.parse().ok())
    {
        actions.push(Box::new(Delay(delay)));
        true
    } else {
        false
    }
}

impl Action for Delay {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            images
                .last_mut()
                .ok_or(anyhow!("No image to delay."))?
                .delay = self.0;
            Ok(())
        })
    }
}
