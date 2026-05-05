use std::{any::Any, sync::Arc};

use anyhow::bail;
use image::{GenericImageView, imageops::FilterType};

use crate::{AppState, action::{Action, ActionResult}, frames::{Frame, Frames}};

#[derive(Clone, Copy)]
pub struct Scale(u32, u32);

impl Action for Scale {
    
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        let Some((x,y)) = input.strip_prefix("scale(").and_then(|s| s.strip_suffix(")")).and_then(|s| s.split_once(",")).and_then(|s| {
            if let Ok(x) = s.0.parse() && let Ok(y) = s.1.parse() {
                Some((x,y))
            } else {
                None
            }
        }) else {
            return false;
        };
        actions.push(Box::new(Self(x,y)));
        true
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            if frames.is_empty() {
                 bail!("No images to scale.");
            }
            for frame in frames {
                let (width,height) = frame.image.dimensions();
                let (width,height) = ((width as f32 * (self.0 as f32 / 100.0)) as u32, ((height as f32 * (self.1 as f32 / 100.0)) as u32 ));
                Frame::can_fit(width, height)?;
                frame.image = frame.image.resize_exact(width,height , FilterType::Triangle);
            }


            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Scale {
    pub fn new(width: u32, height:u32) -> Self {
        Self(width,height)
    }
}

