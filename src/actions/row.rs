use std::sync::Arc;

use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::Frames,
};

#[derive(Clone)]
pub struct Row;

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    if input.eq_ignore_ascii_case("row") {
        actions.push(Box::new(Row));
        true
    } else {
        false
    }
}

impl Action for Row {
    fn apply<'a>(
        &'a self,
        images: &'a mut Vec<crate::frames::Frame>,
        action: u32,
    ) -> ActionResult<'a> {
        Box::pin(async move {
            images.row(action)?;
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
