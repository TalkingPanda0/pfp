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
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            images.get_mut_action(-1).iter_mut().for_each(|f| f.image.apply_orientation(self.0));

            Ok(())
        })
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
