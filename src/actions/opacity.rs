use std::sync::Arc;

use anyhow::anyhow;
use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub struct Opacity(u8);

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    let Some(opacity) = input
        .strip_prefix("opacity(")
        .and_then(|s| s.strip_suffix(")"))
        .and_then(|s| s.parse().ok())
    else {
        return false;
    };

    actions.push(Box::new(Opacity(opacity)));

    true
}

impl Action for Opacity {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            for frame in frames {
                frame.action = action;
                frame
                    .image
                    .as_mut_rgba8()
                    .ok_or(anyhow!("Failed to get image as rgba8"))?
                    .iter_mut()
                    .skip(3)
                    .step_by(4)
                    .for_each(|pixel| {
                        *pixel = (*pixel as f32 * (self.0 as f32 / 100.0)).round() as u8
                    });
            }
            Ok(())
        })
    }
}

impl Opacity {
    pub fn new(opacity: u8) -> Self {
        Self(opacity)
    }
}
