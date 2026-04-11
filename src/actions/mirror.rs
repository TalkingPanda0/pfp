use std::{any::Any, sync::Arc};

use image::metadata::Orientation;

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub struct Mirror(Orientation);

impl Action for Mirror {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _discord: &Arc<AppState>) -> bool {
        if input.eq_ignore_ascii_case("mirror") {
            actions.push(Box::new(Self(Orientation::FlipHorizontal)));
            true
        } else if input.eq_ignore_ascii_case("flip") {
            actions.push(Box::new(Self(Orientation::FlipVertical)));
            true
        } else {
            false
        }
    }
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let mut images_to_mirror = images.get_from_action(-1, action);
            images.truncate(images.len() - images_to_mirror.len());
            images_to_mirror
                .iter_mut()
                .for_each(|image| image.image.apply_orientation(self.0));
            images.extend(images_to_mirror);

            Ok(())
        })
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
