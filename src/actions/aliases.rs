use std::{any::Any, sync::Arc};

use crate::{AppState, action::{Action, ActionResult}, actions::{combine::{Combine}, tenor::Tenor}, frames::Frame};

#[derive(Clone,Copy)]
pub struct Aliases;

impl Action for Aliases {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool
    {
        match input {
            "explode" => {
                actions.push(Box::new(Tenor::new("explosion-deltarune-deltarune-explosion-boom-explode-gif-16548223447993760048".to_string())));
                actions.push(Box::new(Combine::new(true)));
                true
            },
            _ => false,
        }
    }

    fn apply<'a>(&'a self, _images: &'a mut Vec<Frame>,_action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            Ok(())
        })  
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
