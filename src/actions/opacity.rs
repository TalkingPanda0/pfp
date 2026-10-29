use anyhow::anyhow;
use linkme::distributed_slice;
use parse_derive::{Parsable, ParsableAction};

use crate::{
    action::{Action, ActionResult, PARSERS, Parser}, frames::{Frame, Frames}, parser::ParsableAction,
};

#[derive(Clone, Copy,Parsable,ParsableAction)]
pub struct Opacity(u8);


#[distributed_slice(PARSERS)]
static PARSER: Parser = Opacity::parse_action;

impl Action for Opacity {
    fn apply<'a>(&'a self, images: &'a mut Vec<Frame>, action: u32) -> ActionResult<'a> {
        Box::pin(async move {
            let frames = images.get_mut_action(-1);
            for frame in frames {
                frame.action = action;
                frame
                    .image
                    .as_mut_rgba8()
                    .ok_or(anyhow!("Failed to get image as rgba8"))?
                    .iter_mut()
                    .skip(3)
                    .step_by(4)
                    .for_each(|pixel| {
                        *pixel = (*pixel as f32 * (self.0 as f32 / 100.0)).round() as u8
                    });
            }
            Ok(())
        })
    }
}

impl Opacity {
    pub fn new(opacity: u8) -> Self {
        Self(opacity)
    }
}
