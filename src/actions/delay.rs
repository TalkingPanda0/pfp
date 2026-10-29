use anyhow::anyhow;
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::Frame, parser::ParsableAction,
};

#[derive(Clone, Copy,Parsable,ParsableAction)]
pub struct Delay(i32);

#[distributed_slice(PARSERS)]
static PARSER: Parser = Delay::parse_action;

impl Action for Delay {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, _action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            images
                .last_mut()
                .ok_or(anyhow!("No image to delay."))?
                .delay = self.0;
            Ok(())
        })
    }
}
