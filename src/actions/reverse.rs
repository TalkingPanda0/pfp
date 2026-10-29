use linkme::distributed_slice;

use crate::{
    action::{Action, ActionResult, PARSERS},
    frames::{Frame, Frames},
};

#[derive(Clone, Copy)]
pub struct Reverse(bool);

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>) -> bool {
    match input {
        "reverse" => {
            actions.push(Box::new(Reverse(false)));
            true
        }
        "reverseesrever" | "yoyo" => {
            actions.push(Box::new(Reverse(true)));
            true
        }
        _ => false,
    }
}

impl Action for Reverse {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames: Vec<Frame> = if self.0 {
                images.clone_action(-1, action).into_iter().rev().collect()
            } else {
                images.extract_action(-1).into_iter().rev().collect()
            };

            images.extend(frames);

            Ok(())
        })
    }

}
