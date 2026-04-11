use std::sync::Arc;

use crate::{AppState, action::{Action, ActionResult}, frames::Frame};


#[derive(Clone)]
pub struct Grayscale(bool);

impl Action for Grayscale {
    fn apply<'a>(
        &'a self,
        images: &'a mut Vec<Frame>,
        action: u32,
    ) -> ActionResult<'a> {
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

    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        match input.strip_prefix("grayscale") {
            Some("") => actions.push(Box::new(Grayscale(false))),
            Some("all") => actions.push(Box::new(Grayscale(true))),
            _ => {return false; },
        }
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

