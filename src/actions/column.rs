use std::sync::Arc;

use linkme::distributed_slice;

use crate::{
    AppState,
    action::{Action, ActionResult, PARSERS},
    frames::Frames,
};

#[derive(Clone)]
pub struct Column;

#[distributed_slice(PARSERS)]
fn parse(input: &str, actions: &mut Vec<Box<dyn Action>>, _state: &Arc<AppState>) -> bool {
    if input.eq_ignore_ascii_case("Column") {
        actions.push(Box::new(Column));
        true
    } else {
        false
    }
}

impl Action for Column {
    fn apply<'a>(
        &'a self,
        images: &'a mut Vec<crate::frames::Frame>,
        action: u32,
    ) -> ActionResult<'a> {
        Box::pin(async move {
            images.column(action)?;
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
