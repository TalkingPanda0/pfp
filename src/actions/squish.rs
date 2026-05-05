use std::sync::Arc;

use anyhow::bail;

use crate::{AppState, action::{Action, ActionResult}, frames::Frame};

#[derive(Clone)]
pub struct Squish(bool);

impl Action for Squish {
    fn apply<'a>(
        &'a self,
        images: &'a mut Vec<Frame>,
        _action: u32
    ) -> ActionResult<'a> {
        Box::pin(async move {
            if images.is_empty() {
                bail!("No image to squish.");
            }
            for frame in images.iter_mut().rev() {

                let height = frame.image.height();
                let width = frame.image.width() * 2;
                Frame::can_fit(width, height)?;
                frame.image = frame.image.resize_exact(width, height, image::imageops::FilterType::Triangle);

                if !self.0 {
                    break;
                }
            }
            Ok(())
        })
    }

    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        match input.strip_prefix("squish") {
            Some("") => actions.push(Box::new(Squish(false))),
            Some("all") => actions.push(Box::new(Squish(true))),
            _ => { return false; },
        }
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
