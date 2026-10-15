use std::sync::Arc;

use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::Frame,
};

#[derive(Clone)]
pub struct Grayscale(bool);

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    match input.strip_prefix("grayscale") {
        Some("") => actions.push(Box::new(Grayscale(false))),
        Some("all") => actions.push(Box::new(Grayscale(true))),
        _ => {
            return false;
        }
    }
    true
}

impl Action for Grayscale {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            for frame in images.iter_mut() {
                frame.image = frame.image.grayscale();
                frame.action = action;

                if !self.0 {
                    break;
                }
            }
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
