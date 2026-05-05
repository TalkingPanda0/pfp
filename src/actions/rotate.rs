use std::{any::Any, f32::consts::PI, sync::Arc};

use image::{Rgba};
use imageproc::geometric_transformations::{Interpolation, rotate_about_center};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub struct Rotate(i32);

impl Action for Rotate {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        let Some(rotation) = input
            .strip_prefix("rotate(")
            .and_then(|s| s.strip_suffix(")"))
            .and_then(|s| s.parse().ok())
        else {
            return false;
        };

        actions.push(Box::new(Self(rotation)));

        true
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            for frame in frames {
                let rotated = rotate_about_center(&frame.image.to_rgba8(), self.0 as f32 * (PI / 180.0), Interpolation::Bilinear, Rgba([0,0,0,0]));
                frame.image = rotated.into();
                frame.action = action;
            }
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Rotate {
    pub fn new(rotation: i32) -> Self {
        Self(rotation)
    }
}
