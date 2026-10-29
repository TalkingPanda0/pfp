use linkme::distributed_slice;
use parse_derive::ParsableAction;

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::Frames, parser::ParsableAction,
};

#[derive(Clone,ParsableAction)]
pub struct Row;

#[distributed_slice(PARSERS)]
static PARSER: Parser = Row::parse_action;

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
}
