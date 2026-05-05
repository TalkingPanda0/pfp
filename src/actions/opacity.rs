use std::{any::Any, sync::Arc};

use anyhow::anyhow;

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub struct Opacity(u8);

impl Action for Opacity {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        let Some(opacity) = input
            .strip_prefix("opacity(")
            .and_then(|s| s.strip_suffix(")"))
            .and_then(|s| s.parse().ok())
        else {
            return false;
        };

        actions.push(Box::new(Self(opacity)));

        true
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            for frame in frames {
                frame.action = action;
                frame.image
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Opacity {
    pub fn new(opacity: u8) -> Self {
        Self(opacity)
    }
}
