use std::sync::Arc;

use crate::{
    AppState,
    action::Action,
    frames::{Frame, Frames},
};

#[derive(Clone)]
pub struct Copy(i32);

impl Action for Copy {
    fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _: &Arc<AppState>) -> bool
    where
        Self: Sized,
    {
        if let Some(rest) = input.strip_prefix("copy") {
            if rest.is_empty() {
                actions.push(Box::new(Copy(-1)));
                true
            } else {
                if let Some(action) = rest
                    .strip_prefix("(")
                    .and_then(|s| s.strip_suffix(")"))
                    .and_then(|s| Some(Box::new(Copy(s.parse().ok()?)) as Box<dyn Action>))
                {
                    actions.push(action);
                    true
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    fn apply<'a>(
        &'a self,
        images: &'a mut Vec<Frame>,
        action: u32,
    ) -> crate::action::ActionResult<'a> {
        Box::pin(async move {
            let copied = images.clone_action(self.0, action);
            images.extend(copied);
            Ok(())
        })
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
