use std::{any::Any, sync::Arc};

use crate::{
    AppState, action::{Action, ActionResult}, frames::{Frame, Frames}
};

#[derive(Clone, Copy)]
pub struct Reverse(bool);

impl Action for Reverse {
    fn parse(
        input: &str,
        actions: &mut Vec<Box<dyn Action>>,
        _state: &Arc<AppState> 
    ) -> bool {
        match input {
            "reverse" => {
                actions.push(Box::new(Reverse(false)));
                true
            }
            "reverseesrever" => {
                actions.push(Box::new(Reverse(true)));
                true
            }
            _ => false,
        }
    }

    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>,action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames: Vec<Frame> = if self.0 {
                images.clone_action(-1,action).into_iter().rev().collect()
            }  else {
                images.extract_action(-1).into_iter().rev().collect()
            };

            images.extend(frames);

            Ok(()) 
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
