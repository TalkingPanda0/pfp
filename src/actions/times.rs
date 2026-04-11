use std::{any::Any, sync::Arc};

use crate::{
    AppState,
    action::{Action, ActionResult},
    frames::Frame,
};

#[derive(Clone)]
pub struct Times;

impl Action for Times {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
        let Some(count) = input
            .strip_prefix("times(")
            .and_then(|s| s.strip_suffix(")"))
            .and_then(|s| s.parse::<u32>().ok())
        else {
            return false;
        };
        let Some(last_action) = actions.last() else {
            return false;
        };

        let vec = vec![last_action.clone(); count as usize];
        actions.extend(vec);

        true
    }

    fn apply<'a>(&'a self, _images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move { Ok(()) })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
