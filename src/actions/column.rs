use linkme::distributed_slice;
use parse_derive::ParsableAction;

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::Frames, parser::ParsableAction,
};

#[derive(Clone,ParsableAction)]
pub struct Column;

#[distributed_slice(PARSERS)]
static PARSER: Parser = Column::parse_action;

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
}
