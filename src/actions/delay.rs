use std::sync::Arc;


use crate::{
    AppState, action::{Action, ActionResult}, frames::Frame
};

#[derive(Clone, Copy)]
pub struct Delay(i32);

impl Action for Delay {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        if let Some(delay) = input
            .strip_prefix("delay(")
            .and_then(|s| s.strip_suffix(")"))
            .and_then(|s| s.parse().ok())
        {
            actions.push(Box::new(Delay(delay)));
            true
        } else {
            false
        }
    }
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>,_action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            images.last_mut().ok_or("No image to delay.".to_string())?.delay = self.0;
            Ok(())
        })
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
